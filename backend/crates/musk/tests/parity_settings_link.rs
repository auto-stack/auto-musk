//! parity_settings_link.rs — Plan 020 Phase E: settings_link Auto 化契约测试。
//!
//! ag `auto_generated::server_stream::settings_link` 在真实 HTTP 层上验证
//! **shape 契约**(不依赖具体 daemon —— `effective_daemon_url` 优先读用户
//! config.at 的 daemon_url,AAID_URL 只在无配置时生效;daemon 可能 up/down):
//! - 200 ↔ `{"status":"running","url":<str>}`(hw running 响应形状)
//! - 500 ↔ `{"status":"error","error":<str>}`(hw 错误包络形状)
//!
//! 两者必居其一;契约断言在两种环境下都确定性成立。running 的精确 wire
//! (url 值)由 extern 与 hw server.rs settings_link 逐行等价保证。

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::Value;
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
        "musk-parity-settings-link-{}-{}",
        std::process::id(),
        n
    ));
    let _ = std::fs::create_dir_all(&dir);
    let registry = musk::workspace::WorkspaceRegistry::load(dir.join("workspaces.json"), dir.clone());
    AppState {
        client: Arc::new(MockClient) as Arc<dyn Client>,
        auth: Arc::new(musk::auto_generated::auth::AuthStore::new(dir.join("users.json"))),
        registry: Arc::new(registry),
    }
}

fn ag_app(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/settings-link",
            axum::routing::post(musk::auto_generated::server_stream::settings_link),
        )
        .with_state(state)
}

#[tokio::test]
async fn settings_link_shape_contract() {
    let app = ag_app(tmp_state());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings-link")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let v: Value = serde_json::from_slice(&bytes).expect("settings-link returns JSON");
    match status {
        // daemon up:aaid 返回 running + url → 200,hw Json(json!({status,url})) 形状。
        StatusCode::OK => {
            assert_eq!(v["status"], "running", "200 → status running");
            assert!(v["url"].as_str().map(|s| !s.is_empty()).unwrap_or(false), "200 → url non-empty");
        }
        // daemon down / 非 running:500 + {"status":"error","error":…} 包络。
        StatusCode::INTERNAL_SERVER_ERROR => {
            assert_eq!(v["status"], "error", "500 → status error");
            assert!(v["error"].is_string(), "500 → error message");
        }
        other => panic!("unexpected status {other}; body {v:?}"),
    }
}
