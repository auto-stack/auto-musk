//! PLAN-093 T-02 合同测试：Canvas 预览身份/代次、清选、帧版本契约。
//!
//! 全程无真实进程 spawn：会话状态经 `begin_session`（spawn 前置纯状态）+
//! `publish_frame`/`publish_anchor`（唯一写点）驱动；路由臂经 tower oneshot
//! 直打 `canvas_routes()`。关联 AC-03/AC-06/AC-08/AC-11/AC-12。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use auto_ai_agent::Client;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use tower::util::BoxCloneService;

use musk::canvas::anchor::AnchorIndex;
use musk::canvas::manager::{CanvasManager, FrameConflict, GenerationConflict, StopRefused};
use musk::canvas::canvas_routes;

// ── fixture ─────────────────────────────────────────────────────────────────

/// 24 字节最小 PNG 头（manager.png_size 只读签名+IHDR 尺寸）。
fn fake_png(w: u32, h: u32) -> Vec<u8> {
    let mut b = vec![0u8; 24];
    b[..8].copy_from_slice(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);
    b[12..16].copy_from_slice(b"IHDR");
    b[16..20].copy_from_slice(&w.to_be_bytes());
    b[20..24].copy_from_slice(&h.to_be_bytes());
    b
}

const ATOM: &str = r##"col vnode_1201 { bbox: {x: 0, y: 0, w: 100, h: 50}; button vnode_1403 { label: "OK"; bbox: {x: 40, y: 10, w: 60, h: 30}; events: {press: ".Ok"} } }"##;

fn new_manager() -> CanvasManager {
    CanvasManager::new()
}

fn tmp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "musk-canvas-studio-{}-{}-{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn test_state() -> (Arc<musk::server::AppState>, PathBuf) {
    let dir = tmp_dir("route");
    let registry = musk::workspace::WorkspaceRegistry::load(dir.join("workspaces.json"), dir.clone());
    let state = musk::server::AppState {
        client: Arc::new(DeadClient) as Arc<dyn Client>,
        auth: Arc::new(musk::auto_generated::auth::AuthStore::new(dir.join("users.json"))),
        registry: Arc::new(registry),
        chat_runs: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        chat_cancels: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        run_idle_timeout: Duration::from_secs(300),
        canvas: Arc::new(CanvasManager::new()),
        chat_pending: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        run_auto_continue: false,
        chat_procs: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
    };
    (Arc::new(state), dir)
}

struct DeadClient;
#[async_trait::async_trait]
impl Client for DeadClient {
    async fn complete(
        &self,
        _req: &auto_ai_client::CompletionRequest,
    ) -> Result<auto_ai_client::CompletionResponse, auto_ai_client::ClientError> {
        Err(auto_ai_client::ClientError::DaemonUnavailable)
    }
}

/// 会话前置（无 spawn）：登记归属并取得代次。
fn begin(m: &CanvasManager, conv: Option<&str>) -> u64 {
    let dir = tmp_dir("app");
    std::fs::create_dir_all(&dir).unwrap();
    m.begin_session(&dir, &dir, "ws-1", conv, None).unwrap()
}

// ── 代次与归属（AC-11） ─────────────────────────────────────────────────────

#[tokio::test]
async fn generation_increments_per_explicit_start() {
    let m = new_manager();
    assert_eq!(m.generation(), 0);
    let g1 = begin(&m, Some("conv-A"));
    assert_eq!(g1, 1);
    let g2 = begin(&m, Some("conv-A"));
    assert_eq!(g2, 2);
    assert_eq!(m.identity(g2).owner_conversation.as_deref(), Some("conv-A"));
    assert_eq!(m.identity(g2).owner_workspace, "ws-1");
}

#[tokio::test]
async fn stale_expected_generation_rejected() {
    let m = new_manager();
    begin(&m, Some("conv-A"));
    let dir = tmp_dir("app2");
    let err = m
        .begin_session(&dir, &dir, "ws-1", Some("conv-B"), Some(99))
        .unwrap_err();
    let GenerationConflict { current } = err;
    assert_eq!(current, 1, "conflict carries current generation");
    assert_eq!(m.owner_conversation().as_deref(), Some("conv-A"), "replaced owner must not apply");
    assert_eq!(m.generation(), 1, "rejected start must not bump generation");
}

#[tokio::test]
async fn matching_expected_generation_accepted_and_legacy_replaces() {
    let m = new_manager();
    begin(&m, Some("conv-A"));
    let dir = tmp_dir("app3");
    // 预期一致 → 放行（换代 2）。
    let g = m.begin_session(&dir, &dir, "ws-1", Some("conv-A"), Some(1)).unwrap();
    assert_eq!(g, 2);
    // 旧无身份调用（None）→ 保持无条件替换语义。
    let g = m.begin_session(&dir, &dir, "ws-1", None, None).unwrap();
    assert_eq!(g, 3);
    assert_eq!(m.owner_conversation(), None);
}

#[tokio::test]
async fn first_start_accepts_any_expected() {
    let m = new_manager();
    let dir = tmp_dir("app4");
    let g = m.begin_session(&dir, &dir, "ws-1", None, Some(7)).unwrap();
    assert_eq!(g, 1, "current==0 时任意预期放行（首个客户端）");
}

// ── 停止守卫（AC-03/AC-11/AC-12） ───────────────────────────────────────────

#[tokio::test]
async fn stop_with_stale_generation_refused() {
    let m = new_manager();
    begin(&m, Some("conv-A"));
    let err = m.stop_guarded(Some(42), None).await.unwrap_err();
    assert!(matches!(err, StopRefused::GenerationConflict { current: 1 }));
    // 旧 stop（None）保持无条件语义（幂等，无在途进程也安全）。
    let stopped = m.stop_guarded(None, None).await.unwrap();
    assert_eq!(stopped, 1);
}

#[tokio::test]
async fn stop_by_other_conversation_refused() {
    let m = new_manager();
    begin(&m, Some("conv-A"));
    let err = m.stop_guarded(None, Some("conv-B")).await.unwrap_err();
    assert!(matches!(err, StopRefused::OwnedByOther { owner } if owner == "conv-A"));
    let stopped = m.stop_guarded(None, Some("conv-A")).await.unwrap();
    assert_eq!(stopped, 1);
}

#[tokio::test]
async fn stop_unbound_session_allowed_from_anywhere() {
    let m = new_manager();
    begin(&m, None);
    let stopped = m.stop_guarded(None, Some("conv-B")).await.unwrap();
    assert_eq!(stopped, 1, "未绑定预览保持旧语义：任何会话可停");
}

// ── 帧版本（AC-14） ─────────────────────────────────────────────────────────

#[tokio::test]
async fn frame_stale_seq_returns_conflict_not_history() {
    let m = new_manager();
    begin(&m, None);
    m.publish_frame(fake_png(480, 680));
    m.publish_frame(fake_png(480, 680));
    assert_eq!(m.seq(), 2);
    let err = m.frame_for(None, Some(1)).unwrap_err();
    assert!(matches!(err, FrameConflict::StaleSeq { current_seq: 2 }));
    let ok = m.frame_for(None, Some(2)).unwrap();
    assert_eq!(ok.len(), 24);
}

#[tokio::test]
async fn frame_generation_mismatch_conflicts() {
    let m = new_manager();
    begin(&m, None);
    m.publish_frame(fake_png(480, 680));
    let err = m.frame_for(Some(99), None).unwrap_err();
    assert!(matches!(err, FrameConflict::Generation { current: 1 }));
    assert!(m.frame_for(Some(1), None).is_ok());
}

// ── 清选（AC-08） ───────────────────────────────────────────────────────────

#[tokio::test]
async fn pick_miss_clears_picked() {
    let m = new_manager();
    begin(&m, None);
    m.publish_frame(fake_png(480, 680));
    m.publish_anchor(AnchorIndex::parse(ATOM, 1).unwrap(), 1);
    let hit = m.pick_vnode(1403).expect("vnode_1403 in atom");
    assert_eq!(hit["vnode_id"], "vnode_1403");
    // 未命中（索引存在但无此 vnode）→ None 且 picked 归零。
    let miss = m.pick_vnode(999_999);
    assert!(miss.is_none());
    assert!(m.status_full()["picked"].is_null(), "miss must clear picked");
}

#[tokio::test]
async fn clear_pick_api_resets_selection() {
    let m = new_manager();
    begin(&m, None);
    m.publish_frame(fake_png(480, 680));
    m.publish_anchor(AnchorIndex::parse(ATOM, 1).unwrap(), 1);
    assert!(m.pick_vnode(1403).is_some());
    m.clear_pick();
    assert!(m.status_full()["picked"].is_null());
}

// ── PLAN-102 T-06（F-06-1）：embedded 无帧态拾取 ────────────────────────────

/// embedded rq 会话帧缓存设计上恒 None（T-02 口径）——vnode 拾取以逻辑窗
/// 尺寸为基准（scale=1），不再被 PNG 帧源门挡住；bbox_pct 百分比语义不变。
#[tokio::test]
async fn pick_vnode_without_frame_uses_logical_basis() {
    let m = new_manager();
    begin(&m, None);
    // 无 publish_frame——embedded 形态。
    m.publish_anchor(AnchorIndex::parse(ATOM, 1).unwrap(), 1);
    let hit = m.pick_vnode(1403).expect("无帧态 vnode 拾取不再依赖 PNG");
    assert_eq!(hit["vnode_id"], "vnode_1403");
    // scale=1：bbox_px == bbox（逻辑）；pct 按逻辑窗（480 基准）。
    assert_eq!(hit["bbox"]["w"], hit["bbox_px"]["w"]);
    let pct_w = hit["bbox_pct"]["w"].as_f64().unwrap();
    let logical_w = hit["bbox"]["w"].as_f64().unwrap();
    assert!((pct_w - logical_w / 480.0 * 100.0).abs() < 0.01, "pct 按逻辑窗基准");
}

/// 路由面：无帧态坐标形态保持 503（帧像素换算确需 PNG）；vnode 形态不再 503。
#[tokio::test]
async fn route_pick_frame_gate_scoped_to_coord_form() {
    let (state, dir) = test_state();
    state.canvas.begin_session(&dir, &dir, "ws-1", None, None).unwrap();
    state.canvas.publish_anchor(AnchorIndex::parse(ATOM, 1).unwrap(), 1);
    // 无帧。坐标形态 → 503。
    let app: BoxCloneService<Request<Body>, axum::response::Response, std::convert::Infallible> =
        canvas_routes().with_state((*state).clone()).boxed_clone();
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/canvas/pick")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"x": 10, "y": 10}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    // vnode 形态 → 非 503（命中 200 或未命中 204——不被帧门挡）。
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/canvas/pick")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"vnode_id": "vnode_1403"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn anchor_generation_drops_stale_pick() {
    let m = new_manager();
    begin(&m, None);
    m.publish_frame(fake_png(480, 680));
    m.publish_anchor(AnchorIndex::parse(ATOM, 1).unwrap(), 1);
    assert!(m.pick_vnode(1403).is_some());
    // 换代索引不再含 vnode_1403 → picked 自动收敛清空。
    let atom2 = r##"col vnode_2001 { bbox: {x: 0, y: 0, w: 100, h: 50} }"##;
    m.publish_anchor(AnchorIndex::parse(atom2, 2).unwrap(), 2);
    assert!(m.status_full()["picked"].is_null(), "stale pick must not survive generation swap");
}

// ── status 载荷（AC-04/AC-11/AC-14） ────────────────────────────────────────

#[tokio::test]
async fn status_full_carries_identity_and_frame_validity() {
    let m = new_manager();
    let s0 = m.status_full();
    assert_eq!(s0["generation_id"], 0);
    assert!(s0["frame"].is_null());
    begin(&m, Some("conv-A"));
    m.publish_frame(fake_png(480, 680));
    let s = m.status_full();
    assert_eq!(s["generation_id"], 1);
    assert_eq!(s["owner_workspace_id"], "ws-1");
    assert_eq!(s["owner_conversation_id"], "conv-A");
    assert_eq!(s["frame"]["w"], 480);
    assert_eq!(s["frame"]["h"], 680);
    assert_eq!(s["frame"]["valid"], true);
}

// ── 路由臂（oneshot 直打） ──────────────────────────────────────────────────

#[tokio::test]
async fn route_stop_stale_generation_409() {
    let (state, dir) = test_state();
    state.canvas.begin_session(&dir, &dir, "ws-1", Some("conv-A"), None).unwrap();
    let app: BoxCloneService<Request<Body>, axum::response::Response, std::convert::Infallible> =
        canvas_routes().with_state((*state).clone()).boxed_clone();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/canvas/stop?generation=42")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CONFLICT);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["generation_id"], 1);
}

#[tokio::test]
async fn route_frame_stale_seq_409_no_fake_history() {
    let (state, _dir) = test_state();
    state.canvas.begin_session(&_dir, &_dir, "ws-1", None, None).unwrap();
    state.canvas.publish_frame(fake_png(480, 680));
    state.canvas.publish_frame(fake_png(480, 680));
    // 当前 seq=2；请求旧 seq=1 → 409（不伪造历史帧），载荷带当前代次。
    let app: BoxCloneService<Request<Body>, axum::response::Response, std::convert::Infallible> =
        canvas_routes().with_state((*state).clone()).boxed_clone();
    let res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/canvas/frame?seq=1&generation=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CONFLICT);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["generation_id"], 1);
}

#[tokio::test]
async fn route_frame_current_seq_ok() {
    let (state, _dir) = test_state();
    state.canvas.begin_session(&_dir, &_dir, "ws-1", None, None).unwrap();
    let seq = state.canvas.publish_frame(fake_png(480, 680));
    let app: BoxCloneService<Request<Body>, axum::response::Response, std::convert::Infallible> =
        canvas_routes().with_state((*state).clone()).boxed_clone();
    let res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/canvas/frame?seq={seq}&generation=1"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn route_pick_clear_true() {
    let (state, _dir) = test_state();
    state.canvas.begin_session(&_dir, &_dir, "ws-1", None, None).unwrap();
    state.canvas.publish_frame(fake_png(480, 680));
    state.canvas.publish_anchor(AnchorIndex::parse(ATOM, 1).unwrap(), 1);
    assert!(state.canvas.pick_vnode(1403).is_some());
    let app: BoxCloneService<Request<Body>, axum::response::Response, std::convert::Infallible> =
        canvas_routes().with_state((*state).clone()).boxed_clone();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/canvas/pick")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"clear":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(state.canvas.status_full()["picked"].is_null());
}

#[tokio::test]
async fn route_pick_clear_exclusive_with_coords() {
    let (state, _dir) = test_state();
    let app: BoxCloneService<Request<Body>, axum::response::Response, std::convert::Infallible> =
        canvas_routes().with_state((*state).clone()).boxed_clone();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/canvas/pick")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"clear":true,"x":1,"y":2}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn route_status_carries_identity() {
    let (state, dir) = test_state();
    state.canvas.begin_session(&dir, &dir, "ws-1", Some("conv-A"), None).unwrap();
    let app: BoxCloneService<Request<Body>, axum::response::Response, std::convert::Infallible> =
        canvas_routes().with_state((*state).clone()).boxed_clone();
    let res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/canvas/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["generation_id"], 1);
    assert_eq!(v["owner_conversation_id"], "conv-A");
}

// ── T-12：附件上下文归属校验（chats_message 路由臂） ────────────────────────

fn full_router() -> axum::Router<musk::server::AppState> {
    musk::auto_generated::server::build_router()
}

/// PLAN-093 T-12（AC-11）：design_context 携陈旧代次 → 拒收（stale 信封，
/// 用户文字不落盘）；当前代次 → 收章（ownership:"current"）。缺省无 canvas
/// 会话 → "no-canvas"（旧 API 兼容面）。
#[tokio::test]
async fn route_chat_message_stale_design_context_rejected() {
    let (state, dir) = test_state();
    state.canvas.begin_session(&dir, &dir, "ws-1", Some("conv-A"), None).unwrap();
    let sid = state.registry.get("ws-1").chats.create("basic", Some("ws-1".into())).unwrap().id;
    let app: BoxCloneService<Request<Body>, axum::response::Response, std::convert::Infallible> =
        full_router().with_state((*state).clone()).boxed_clone();

    // 陈旧代次（当前 gen=1，附件带 42）→ 200 信封内 stale:true（不落盘）。
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/chats/session/{sid}/message?workspace=ws-1"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "content": "把按钮改大一点",
                        "design_context": {
                            "version": "1", "vnode_id": "vnode_42", "kind": "button",
                            "generation_id": 42
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let st = res.status();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let dbg_body = String::from_utf8_lossy(&body).to_string();
    assert_eq!(st, StatusCode::OK, "stale rejection rides an OK envelope; body={dbg_body}");
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["stale"], true, "must be the stale envelope: {v}");
    assert!(v["error"].as_str().unwrap_or("").contains("失效"), "stale error text: {v}");

    // 当前代次（gen=1）→ 正常接收；消息落盘带 ownership:"current" 章。
    let res2 = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/chats/session/{sid}/message?workspace=ws-1"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "content": "把按钮改大一点",
                        "design_context": {
                            "version": "1", "vnode_id": "vnode_42", "kind": "button",
                            "generation_id": 1
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let st2 = res2.status();
    let body2 = axum::body::to_bytes(res2.into_body(), usize::MAX).await.unwrap();
    assert_eq!(st2, StatusCode::OK, "current-gen status");
    eprintln!("[dbg] sessions: {:?}", state.registry.get("ws-1").chats.list().iter().map(|s| s.id.clone()).collect::<Vec<_>>());
    let v2: serde_json::Value = serde_json::from_slice(&body2).unwrap();
    assert!(v2.get("stale").is_none(), "current generation must not be stale: {v2}");
    // 落盘面：会话消息里的 design_context 带章（回放诚实标注）。
    let ws = state.registry.get("ws-1");
    let msgs = ws.chats.get(&sid).map(|s| s.messages).unwrap_or_default();
    let stamped = msgs.iter().any(|m| {
        m.role == musk::chats::Role::User
            && m.design_context
                .as_ref()
                .and_then(|d| d.get("ownership"))
                .and_then(|o| o.as_str())
                == Some("current")
    });
    assert!(stamped, "accepted design_context must be persisted with ownership stamp");
}

/// T-12：缺 vnode_id/kind 的 design_context → 字段面拒收（400 语义），
/// 用户文字不落盘（T-08 契约的路由臂回归）。
#[tokio::test]
async fn route_chat_message_design_context_missing_required_fields_rejected() {
    let (state, dir) = test_state();
    state.canvas.begin_session(&dir, &dir, "ws-1", Some("conv-A"), None).unwrap();
    let sid = state.registry.get("ws-1").chats.create("basic", Some("ws-1".into())).unwrap().id;
    let app: BoxCloneService<Request<Body>, axum::response::Response, std::convert::Infallible> =
        full_router().with_state((*state).clone()).boxed_clone();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/chats/session/{sid}/message?workspace=ws-1"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "content": "正文",
                        "design_context": { "version": "1", "kind": "button" }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v["error"].as_str().unwrap_or("").contains("vnode_id"), "missing-field error: {v}");
    // 拒收不落盘。
    let ws = state.registry.get("ws-1");
    let msgs = ws.chats.get(&sid).map(|s| s.messages).unwrap_or_default();
    assert!(
        !msgs.iter().any(|m| m.role == musk::chats::Role::User && m.content == "正文"),
        "rejected message content must not be persisted"
    );
}

// ── PLAN-102 T-02：present 切换路由合同（身份守卫/幂等/window 诚实 501） ──
async fn present_call(
    state: &Arc<musk::server::AppState>,
    uri: &str,
    body: &str,
) -> (StatusCode, serde_json::Value) {
    let app: BoxCloneService<Request<Body>, axum::response::Response, std::convert::Infallible> =
        canvas_routes().with_state((**state).clone()).boxed_clone();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null))
}

#[tokio::test]
async fn route_present_stale_generation_409() {
    let (state, dir) = test_state();
    state.canvas.begin_session(&dir, &dir, "ws-1", Some("conv-A"), None).unwrap();
    let (status, v) = present_call(&state, "/api/canvas/present?generation=42", r#"{"mode":"embedded"}"#).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(v["generation_id"], 1);
}

#[tokio::test]
async fn route_present_no_session_400() {
    let (state, _dir) = test_state();
    let (status, _v) = present_call(&state, "/api/canvas/present", r#"{"mode":"embedded"}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn route_present_embedded_idempotent_and_window_honest_501() {
    let (state, dir) = test_state();
    state.canvas.begin_session(&dir, &dir, "ws-1", Some("conv-A"), None).unwrap();
    // embedded：幂等成功（当前唯一支持形态；presentation 面板随行）。
    let (status, v) = present_call(&state, "/api/canvas/present?generation=1", r#"{"mode":"embedded"}"#).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(v["mode"], "embedded");
    assert_eq!(v["generation_id"], 1);
    assert!(v["presentation"]["mode"].is_string());
    // window：诚实 501（上游重连 seam 未就绪——不冒充、不动会话）。
    let (status, v) = present_call(&state, "/api/canvas/present?generation=1", r#"{"mode":"window"}"#).await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert!(v["error"].as_str().unwrap().contains("reconnect seam"));
    // 会话未被动：generation 不变。
    assert_eq!(state.canvas.generation(), 1);
    // 非法 mode：400。
    let (status, _v) = present_call(&state, "/api/canvas/present?generation=1", r#"{"mode":"floating"}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ── PLAN-102 T-02 余项：present 的 VMHTTP 宿主桥镜像合同（canvas_vm.at
// 薄转发 → canvas_present_host；直调 glue + lookup_http_response 解码，
// 断言语义与 axum 路由合同逐条同律） ─────────────────────────────────────

fn present_host_call(
    state: &Arc<musk::server::AppState>,
    q: &str,
    body: &str,
) -> (u16, serde_json::Value) {
    let st = axum::extract::State((**state).clone());
    let qv: serde_json::Value = serde_json::from_str(q).unwrap_or(serde_json::Value::Null);
    let bv: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let handle = musk::auto_generated::extern_impl::canvas_present_host(&st, qv, bv).unwrap();
    let (status, _headers, body) =
        auto_lang::vm::ffi::stdlib::lookup_http_response(handle as u64).unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null))
}

#[tokio::test]
async fn vmhost_present_stale_generation_409() {
    let (state, dir) = test_state();
    state.canvas.begin_session(&dir, &dir, "ws-1", Some("conv-A"), None).unwrap();
    let q = serde_json::json!({ "generation": "42" }).to_string();
    let (status, v) = present_host_call(&state, &q, r#"{"mode":"embedded"}"#);
    assert_eq!(status, 409);
    assert_eq!(v["generation_id"], 1);
}

#[tokio::test]
async fn vmhost_present_no_session_400() {
    let (state, _dir) = test_state();
    let (status, _v) = present_host_call(&state, r#"{}"#, r#"{"mode":"embedded"}"#);
    assert_eq!(status, 400);
}

#[tokio::test]
async fn vmhost_present_embedded_idempotent_window_501_invalid_400() {
    let (state, dir) = test_state();
    state.canvas.begin_session(&dir, &dir, "ws-1", Some("conv-A"), None).unwrap();
    let q1 = serde_json::json!({ "generation": "1" }).to_string();
    let (status, v) = present_host_call(&state, &q1, r#"{"mode":"embedded"}"#);
    assert_eq!(status, 200);
    assert_eq!(v["mode"], "embedded");
    assert_eq!(v["generation_id"], 1);
    assert!(v["presentation"]["mode"].is_string());
    let (status, v) = present_host_call(&state, &q1, r#"{"mode":"window"}"#);
    assert_eq!(status, 501);
    assert!(v["error"].as_str().unwrap().contains("reconnect seam"));
    assert_eq!(state.canvas.generation(), 1);
    let (status, _v) = present_host_call(&state, &q1, r#"{"mode":"floating"}"#);
    assert_eq!(status, 400);
}

// ── PLAN-102 T-02：embedded 呈现会话合同（实机；AUTO_EXE + 夹具） ─────────
// spawn_embedded 双子进程（bridge 宿主 + rq remote app）：status 携
// presentation.mode=embedded 与 bridge WS 端点（可连）；会话 running；停止
// 收割两子进程树（tasklist census 零残留）。窗口模式重连 seam 落地前
// window 臂不冒充（决策文档 §5）。
#[tokio::test]
#[serial_test::serial]
#[ignore = "live spawn: requires AUTO_EXE + canvas-embed fixture"]
async fn embedded_spawn_contract_live() {
    let exe = std::env::var("AUTO_EXE").expect("AUTO_EXE must point at auto.exe (musk-102 dep build)");
    assert!(std::path::Path::new(&exe).exists(), "AUTO_EXE missing: {exe}");
    let manifest = env!("CARGO_MANIFEST_DIR");
    let fixture = std::path::Path::new(manifest)
        .join("../../../tests/ui-parity/probes/canvas-embed/embed-fixture")
        .canonicalize()
        .expect("fixture dir");
    let ws_root = tmp_dir("p102-embed");
    // 夹具 = canvas 产线 app 形态（pac 无 api 字段——纯前端，无后端
    // split-mode；api: "rust" 形态会经 am 机制自起脱离 /T 树的后端子
    // 进程，census 假红——canvas-run 生成 app 均无 api 字段）。
    let m = new_manager();
    // 显式 start（归属链路同产线）：embedded spawn + endpoint 登记。
    let ident = m
        .start_owned(fixture.clone(), &ws_root, "ws-p102", Some("conv-p102"), None)
        .await
        .expect("embedded spawn");
    assert!(ident.generation >= 1);
    // status：presentation embedded + WS 端点可连 + Running。
    let mut status = m.status_full();
    for _ in 0..60 {
        if status["state"] == "running" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        status = m.status_full();
    }
    assert_eq!(status["state"], "running", "status: {status}");
    assert_eq!(status["presentation"]["mode"], "embedded");
    let ws_url = status["presentation"]["bridge_ws_url"].as_str().unwrap().to_string();
    let ws_port = status["presentation"]["bridge_ws_port"].as_u64().unwrap();
    assert!(ws_url.starts_with("ws://127.0.0.1:") && ws_url.contains("token="));
    tokio::net::TcpStream::connect(("127.0.0.1", ws_port as u16))
        .await
        .expect("bridge ws port connectable");
    // vtree 面（D-2）：树到达 = 锚定通道活。
    for _ in 0..40 {
        if !status["tree"].is_null() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        status = m.status_full();
    }
    assert!(!status["tree"].is_null(), "vtree never arrived: {status}");
    // 停止：双子进程收割。census 域 = 本会话进程形态（`-q` app 臂 +
    // `--bridge-ws` 桥）——机器上常有其他会话的 legacy 形态 auto.exe
    //（`--render vm`/`-r vm`，F-02-1 定谳：曾误计为孤儿；对照实验证明
    // 本会话两臂收割零残留），全名计数在本机恒假红。
    m.stop_guarded(None, Some("conv-p102")).await.expect("stop");
    tokio::time::sleep(Duration::from_millis(1500)).await;
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command",
            "Get-CimInstance Win32_Process -Filter \"Name='auto.exe'\" | ForEach-Object { $_.CommandLine }"])
        .output()
        .unwrap()
        .stdout;
    let text = String::from_utf8_lossy(&out);
    let mine: Vec<&str> = text
        .lines()
        .filter(|l| l.contains("--render=vm -q") || l.contains("--bridge-ws"))
        .collect();
    assert!(
        mine.is_empty(),
        "embedded-session processes remain after stop:
{}",
        mine.join("
")
    );
    let _ = std::fs::remove_dir_all(ws_root);
}
