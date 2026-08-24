//! parity_chat_branch.rs — PLAN-043 T4-T6: `/api/chats/session/{id}/fork|navigate|tree`
//! HTTP 层测试。fork/navigate 共用 set_active_leaf 机制；tree 为节点投影。
//! 核心投影语义（active_path/history_pairs/链式追加）的单测在 chats.rs。

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use tower::ServiceExt;

use auto_ai_agent::Client;
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};
use musk::server::AppState;

struct MockClient;
#[async_trait::async_trait]
impl Client for MockClient {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        Err(ClientError::DaemonUnavailable)
    }
}

fn tmp_state() -> AppState {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "musk-parity-chat-branch-{}-{}",
        std::process::id(),
        n
    ));
    let _ = std::fs::create_dir_all(&dir);
    let registry =
        musk::workspace::WorkspaceRegistry::load(dir.join("workspaces.json"), dir.clone());
    AppState {
        client: Arc::new(MockClient) as Arc<dyn Client>,
        auth: Arc::new(musk::auto_generated::auth::AuthStore::new(dir.join("users.json"))),
        registry: Arc::new(registry),
    }
}

fn app(state: AppState) -> Router {
    musk::chat_branch::branch_routes().with_state(state)
}

async fn send(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
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
    let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20).await.unwrap();
    let v = serde_json::from_slice(&bytes).unwrap_or(Value::String(
        String::from_utf8_lossy(&bytes).to_string(),
    ));
    (status, v)
}

#[tokio::test]
async fn fork_navigate_tree_end_to_end() {
    let state = tmp_state();
    let ws = state.registry.get("");
    let session = ws.chats.create("superpowers", None).unwrap();
    let sid = session.id.clone();
    ws.chats
        .append_message(&sid, musk::chats::ChatMessage::user("q1"))
        .unwrap()
        .unwrap();
    ws.chats
        .append_message(&sid, musk::chats::ChatMessage::assistant("a1"))
        .unwrap()
        .unwrap();
    ws.chats
        .append_message(&sid, musk::chats::ChatMessage::user("q2"))
        .unwrap()
        .unwrap();
    let before = ws.chats.get(&sid).unwrap();
    let messages_before: Vec<(String, String, Option<String>)> = before
        .messages
        .iter()
        .map(|m| (m.id.clone(), m.content.clone(), m.parent_id.clone()))
        .collect();
    let a1_id = before.messages[1].id.clone();

    let router = app(state);

    // fork 自 a1。
    let (status, v) = send(
        &router,
        "POST",
        &format!("/api/chats/session/{sid}/fork"),
        Some(json!({ "message_id": a1_id })),
    )
    .await;
    assert_eq!(status, 200, "{v:?}");
    assert_eq!(v["session"]["active_leaf"], a1_id.as_str());

    // append-only:旧消息(id/内容/parent)逐字段不变。
    let after = ws.chats.get(&sid).unwrap();
    let messages_after: Vec<(String, String, Option<String>)> = after
        .messages
        .iter()
        .map(|m| (m.id.clone(), m.content.clone(), m.parent_id.clone()))
        .collect();
    assert_eq!(messages_before, messages_after, "fork 不重写任何旧消息");

    // 新消息落在新支(parent = fork 点)。
    ws.chats
        .append_message(&sid, musk::chats::ChatMessage::assistant("branch-2"))
        .unwrap()
        .unwrap();
    let s2 = ws.chats.get(&sid).unwrap();
    let last = s2.messages.last().unwrap();
    assert_eq!(last.parent_id.as_deref(), Some(a1_id.as_str()));
    assert_eq!(last.content, "branch-2");

    // tree:fork 点 children=2;旧分支 q2 不在活跃路径。
    let (status, v) = send(&router, "GET", &format!("/api/chats/session/{sid}/tree"), None).await;
    assert_eq!(status, 200, "{v:?}");
    let nodes = v["nodes"].as_array().unwrap();
    let fork_node = nodes.iter().find(|n| n["id"] == a1_id.as_str()).unwrap();
    assert_eq!(fork_node["children"], 2, "a1 之下两个子分支(q2 与 branch-2)");
    let q2 = nodes.iter().find(|n| n["preview"] == "q2").unwrap();
    assert_eq!(q2["on_active_path"], false, "旧分支只读保留");

    // navigate 切回旧分支末尾(q2)。
    let q2_id = q2["id"].as_str().unwrap().to_string();
    let (status, v) = send(
        &router,
        "POST",
        &format!("/api/chats/session/{sid}/navigate"),
        Some(json!({ "message_id": q2_id })),
    )
    .await;
    assert_eq!(status, 200, "{v:?}");
    let s3 = ws.chats.get(&sid).unwrap();
    assert_eq!(s3.active_leaf.as_deref(), Some(q2_id.as_str()));
    // 切换后历史路径 = q1/a1/q2(旧支),不含 branch-2。
    let hist = s3.history_pairs();
    let contents: Vec<&str> = hist.iter().map(|(_, c)| c.as_str()).collect();
    assert_eq!(contents, vec!["q1", "a1"], "q2 为待运行消息被排除;branch-2 不在路径");

    // 未知消息 → 404;未知会话 → 404。
    let (status, _) = send(
        &router,
        "POST",
        &format!("/api/chats/session/{sid}/fork"),
        Some(json!({ "message_id": "nope" })),
    )
    .await;
    assert_eq!(status, 404);
    let (status, _) = send(
        &router,
        "GET",
        "/api/chats/session/no-such/tree",
        None,
    )
    .await;
    assert_eq!(status, 404);
}
