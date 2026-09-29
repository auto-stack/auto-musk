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
        chat_runs: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
        chat_cancels: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        run_idle_timeout: Duration::from_secs(300),
        canvas: Arc::new(CanvasManager::new()),
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
