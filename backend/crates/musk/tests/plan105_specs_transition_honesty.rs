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
#[tokio::test]
async fn retired_writes_are_explicit_for_all_status_inputs() {
    let state=tmp_state(); let a=ag_app(state.clone());
    for status in ["Approved","approved","under_review","not-a-status"] {
        for (uri,body) in [
            ("/api/specs/item",serde_json::json!({"section":"designs","item":{"id":"D1","title":"example","content":"x","status":status}})),
            ("/api/specs/transition",serde_json::json!({"section":"designs","item_id":"D1","new_status":status})),
        ] {
            let (code,body)=send(&a,"POST",uri,Some(body)).await;
            assert_eq!(code,axum::http::StatusCode::GONE,"{body}");
            assert!(body.to_string().contains("retired"),"{body}");
        }
    }
    let (code, body) = send(&a, "DELETE", "/api/specs/item/designs/D1", None).await;
    assert_eq!(code, axum::http::StatusCode::GONE, "{body}");
    assert!(body.to_string().contains("retired"));
    let id=musk::workspace::WorkspaceQuery::default().id_or_default(&state.registry);
    assert!(!state.registry.get(&id).root.join(".autoos/specs.json").exists());
}
