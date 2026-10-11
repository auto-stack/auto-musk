//! PLAN-105 T-06（P6/AC-06）：specs transition API 诚实化 HTTP 钉测。
//!
//! UAT 87af4d4b 实锤形态（18:14:32/18:15:25）：帕斯卡 `Approved` 得 200
//! 「成功」但状态纹丝不动（未知串兜底 Draft × 同态恒过 × 回显请求串的
//! 三连叠加），错误体恒 "transition failed" 吞因。本文件钉：
//! - 未知/帕斯卡状态串 → 400 且 body 含原因与合法值清单；
//! - 合法转移 → 200 响应含应用后状态（applied_status）且回读一致；
//! - GET 返回值可直接作 POST 入参（大小写对称：GET snake_case）；
//! - upsert 直写：合法转移路径 bypassed_transition=false、绕状态机
//!   直写 true、未知串 400。

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

use musk::auto_generated::auth::AuthStore;
use musk::server::AppState;

struct MockClient;

#[async_trait::async_trait]
impl auto_ai_agent::Client for MockClient {
    async fn complete(
        &self,
        _req: &auto_ai_client::CompletionRequest,
    ) -> Result<auto_ai_client::CompletionResponse, auto_ai_client::ClientError> {
        unreachable!("specs tests never call the LLM")
    }
}

fn tmp_state() -> AppState {
    let dir = std::env::temp_dir().join(format!(
        "musk-q105-specs-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let registry =
        musk::workspace::WorkspaceRegistry::load(dir.join("workspaces.json"), dir.clone());
    AppState {
        client: Arc::new(MockClient) as Arc<dyn auto_ai_agent::Client>,
        auth: Arc::new(AuthStore::new(dir.join("users.json"))),
        registry: Arc::new(registry),
        canvas: Arc::new(musk::canvas::CanvasManager::new()),
        chat_procs: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        chat_runs: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        chat_cancels: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        run_idle_timeout: std::time::Duration::from_secs(300),
        run_auto_continue: false,
        chat_pending: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
    }
}

fn ag_app(state: AppState) -> axum::Router {
    musk::auto_generated::server::build_router().with_state(state)
}

async fn send(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let resp = match body {
        Some(b) => {
            builder = builder.header("content-type", "application/json");
            app.clone()
                .oneshot(builder.body(Body::from(b.to_string())).unwrap())
                .await
                .unwrap()
        }
        None => app
            .clone()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap(),
    };
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let v = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|_| {
            Value::String(String::from_utf8_lossy(&bytes).into_owned())
        })
    };
    (status, v)
}

/// 建 designs 区 D1 条目（upsert，绕状态机直写为合法链起点）。
async fn seed_design_item(app: &axum::Router, id: &str, status: &str) {
    let (s, v) = send(
        app,
        "POST",
        "/api/specs/item",
        Some(json!({
            "section": "designs",
            "item": {"id": id, "title": id, "content": "c", "status": status}
        })),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "seed upsert: {v}");
}

#[tokio::test]
async fn pascal_status_rejected_with_reason_not_fake_ok() {
    let app = ag_app(tmp_state());
    seed_design_item(&app, "D1", "draft").await;
    // UAT 形态：`Approved`（帕斯卡）。旧链：未知串→Draft→同态恒过→200
    // 假成功、状态纹丝不动。新链：400 + 原串 + 合法值清单。
    let (s, v) = send(
        &app,
        "POST",
        "/api/specs/transition",
        Some(json!({"section": "designs", "item_id": "D1", "new_status": "Approved"})),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "帕斯卡串应 400: {v}");
    let msg = v["error"].as_str().unwrap_or_else(|| v["error"]["message"].as_str().unwrap_or_default());
    assert!(
        msg.contains("unknown status") && msg.contains("Approved"),
        "错误体含原串: {msg}"
    );
    assert!(
        msg.contains("under_review") && msg.contains("approved"),
        "错误体含合法值清单（snake_case）: {msg}"
    );
    // 状态未动（回读仍 draft——GET snake_case）。
    let (gs, gv) = send(&app, "GET", "/api/specs", None).await;
    assert_eq!(gs, StatusCode::OK);
    let status = gv["sections"]
        .as_array()
        .and_then(|secs| {
            secs.iter()
                .find(|sec| sec["id"] == "designs")
                .and_then(|sec| sec["items"].as_array().and_then(|items| {
                    items
                        .iter()
                        .find(|i| i["id"] == "D1")
                        .and_then(|i| i["status"].as_str().map(String::from))
                }))
        })
        .expect("D1 in GET");
    assert_eq!(status, "draft", "GET 状态应 snake_case 且未动");
}

#[tokio::test]
async fn legal_transition_applies_and_echoes_applied_status() {
    let app = ag_app(tmp_state());
    seed_design_item(&app, "D1", "draft").await;
    // 合法转移 draft → under_review。
    let (s, v) = send(
        &app,
        "POST",
        "/api/specs/transition",
        Some(json!({"section": "designs", "item_id": "D1", "new_status": "under_review"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "合法转移应 200: {v}");
    assert_eq!(v["applied_status"], json!("under_review"), "回显应用后真值");
    assert_eq!(v["new_status"], json!("under_review"), "请求串对照保留");
    // 回读一致 + GET/POST 对称：GET 到的值直接作下一笔 POST 入参。
    let (_, gv) = send(&app, "GET", "/api/specs", None).await;
    let status = gv["sections"]
        .as_array()
        .and_then(|secs| {
            secs.iter()
                .find(|sec| sec["id"] == "designs")
                .and_then(|sec| sec["items"].as_array().and_then(|items| {
                    items
                        .iter()
                        .find(|i| i["id"] == "D1")
                        .and_then(|i| i["status"].as_str().map(String::from))
                }))
        })
        .expect("D1 in GET");
    assert_eq!(status, "under_review", "回读与应用一致");
    // GET 值写回（under_review → approved，合法链下一步）——对称闭环。
    let (s2, v2) = send(
        &app,
        "POST",
        "/api/specs/transition",
        Some(json!({"section": "designs", "item_id": "D1", "new_status": status})),
    )
    .await;
    assert_eq!(s2, StatusCode::OK, "GET 值应可直接作 POST 入参: {v2}");
    assert_eq!(v2["applied_status"], json!("under_review"), "同态幂等重放 ok");
}

#[tokio::test]
async fn illegal_transition_400_carries_real_reason() {
    let app = ag_app(tmp_state());
    seed_design_item(&app, "D1", "draft").await;
    // 设计区状态机正确拒绝 draft → approved（跳过 under_review）——但错误
    // 体必须带因（旧链恒 "transition failed"，模型无从得知原因）。
    let (s, v) = send(
        &app,
        "POST",
        "/api/specs/transition",
        Some(json!({"section": "designs", "item_id": "D1", "new_status": "approved"})),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "非法转移应 400: {v}");
    let msg = v["error"].as_str().unwrap_or_else(|| v["error"]["message"].as_str().unwrap_or_default());
    assert!(
        msg.contains("draft -> approved"),
        "错误体应含 from -> to 详情: {msg}"
    );
}

#[tokio::test]
async fn upsert_bypassed_transition_annotation() {
    let app = ag_app(tmp_state());
    seed_design_item(&app, "D1", "draft").await;
    // ① 合法转移形态的 upsert（draft → under_review）：不标 bypass。
    let (s, v) = send(
        &app,
        "POST",
        "/api/specs/item",
        Some(json!({
            "section": "designs",
            "item": {"id": "D1", "title": "D1", "content": "c", "status": "under_review"}
        })),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "合法 upsert: {v}");
    assert_eq!(
        v["bypassed_transition"], json!(false),
        "合法转移路径不标 bypass: {v}"
    );
    // ② 绕状态机直写（D2 从 draft 直写 approved——跳过 under_review
    // review 决议的非法转移）：标注 bypassed。
    seed_design_item(&app, "D2", "draft").await;
    let (s, v) = send(
        &app,
        "POST",
        "/api/specs/item",
        Some(json!({
            "section": "designs",
            "item": {"id": "D2", "title": "D2", "content": "c", "status": "approved"}
        })),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "直写能力保留: {v}");
    assert_eq!(
        v["bypassed_transition"], json!(true),
        "绕状态机直写应标注: {v}"
    );
    // ③ upsert 未知串 → 400（不再静默 Draft）。
    let (s, v) = send(
        &app,
        "POST",
        "/api/specs/item",
        Some(json!({
            "section": "designs",
            "item": {"id": "D1", "title": "D1", "content": "c", "status": "Reviewed"}
        })),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "upsert 未知串应 400: {v}");
    assert!(
        v["error"].as_str().unwrap_or_default().contains("unknown status"),
        "错误体: {v}"
    );
}
