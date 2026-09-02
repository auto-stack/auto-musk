//! Plan 021 Phase C2 — config 端点 HTTP 测试(写真实 ~/.config 的 5 端点)。
//!
//! `/api/roles/{name}` PUT/DELETE、`/api/app-config` PUT、
//! `/api/app-harness/{kind}/{name}` PUT/DELETE 这 5 个端点原本"刻意不测"
//! (会污染用户真实 ~/.config/autoos)。Plan 021 C2 给 musk 的 config 路径函数
//! (app_config::musk_config_path / server::app_harness_dir) + auto-ai 的
//! RoleRegistry::roles_dir 加了 AUTOOS_HOME env 覆盖,本测试用 env 重定向到
//! temp dir 做隔离。因 env 是进程全局,5 个测试串行(#[serial_test::serial])。

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
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
        "musk-parity-cfg-{}-{}",
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
        chat_runs: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
    }
}

fn ag_app(state: AppState) -> axum::Router {
    musk::auto_generated::server::build_router().with_state(state)
}

async fn send(app: &axum::Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let resp = match body {
        Some(b) => {
            builder = builder.header("content-type", "application/json");
            app.clone().oneshot(builder.body(Body::from(b.to_string())).unwrap()).await.unwrap()
        }
        None => app.clone().oneshot(builder.body(Body::empty()).unwrap()).await.unwrap(),
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

/// A temp dir to redirect AUTOOS_HOME into (isolates writes from real config).
fn autoos_home() -> std::path::PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "musk-autoos-home-{}-{}",
        std::process::id(),
        n
    ));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Set AUTOOS_HOME + return the path; test must call this inside #[serial].
fn isolate_autoos() -> std::path::PathBuf {
    let dir = autoos_home();
    std::env::set_var("AUTOOS_HOME", &dir);
    dir
}

fn clear_autoos() {
    std::env::remove_var("AUTOOS_HOME");
}

// ── /api/roles/{name} PUT + DELETE ──────────────────────────────────────────
// (RoleRegistry::load routes through roles_dir which honors AUTOOS_HOME.)

#[tokio::test]
#[serial_test::serial]
async fn role_save_then_delete_roundtrip() {
    let home = isolate_autoos();
    let state = tmp_state();
    let app = ag_app(state);
    // Save a custom user role. RoleSaveBody is a flat DTO (no "name" — name is
    // the path param); required-ish fields are Option/default.
    let (s, _v) = send(
        &app,
        "PUT",
        "/api/roles/my-custom-role",
        Some(json!({
            "model": null,
            "tier": "mid",
            "system_prompt": "you are a test role",
            "temperature": null,
            "description": "test role",
            "inherit": null,
            "allowed_tiers": ["low", "mid"],
            "skills": [],
            "token_budget": null,
            "max_turns": null,
            "tools": [],
            "soul": null,
        })),
    ).await;
    assert_eq!(s, StatusCode::OK, "role PUT → 200");
    // The role file landed under AUTOOS_HOME/roles/.
    let role_file = home.join("roles").join("my-custom-role.at");
    assert!(role_file.exists(), "role .at file written under AUTOOS_HOME");

    // DELETE it.
    let (s2, _v2) = send(&app, "DELETE", "/api/roles/my-custom-role", None).await;
    assert_eq!(s2, StatusCode::OK, "role DELETE → 200");
    assert!(!role_file.exists(), "role file removed after DELETE");

    clear_autoos();
}

// ── /api/app-config PUT ─────────────────────────────────────────────────────
// (app_config::musk_config_path honors AUTOOS_HOME.)

#[tokio::test]
#[serial_test::serial]
async fn app_config_save_writes_under_autoos_home() {
    let home = isolate_autoos();
    let state = tmp_state();
    let app = ag_app(state);
    let (s, _v) = send(
        &app,
        "PUT",
        "/api/app-config",
        Some(json!({
            "daemon_url": "http://test:9999",
            "default_mode": "superpowers",
            "context_file": null,
            "serve_addr": null,
            "auto_start_daemon": null,
            "harness": { "roles": [], "skills": [], "modes": [] },
        })),
    ).await;
    assert_eq!(s, StatusCode::OK, "app-config PUT → 200");
    let cfg_file = home.join("apps/musk/config.at");
    assert!(cfg_file.exists(), "config.at written under AUTOOS_HOME");
    let written = std::fs::read_to_string(&cfg_file).unwrap();
    assert!(written.contains("test:9999"), "daemon_url persisted; got:\n{written}");

    clear_autoos();
}

// ── /api/app-harness/{kind}/{name} PUT + DELETE ────────────────────────────
// (server::app_harness_dir honors AUTOOS_HOME.)

#[tokio::test]
#[serial_test::serial]
async fn harness_save_then_delete_roundtrip() {
    let home = isolate_autoos();
    let state = tmp_state();
    let app = ag_app(state);
    // PUT a harness snippet. harness_save only handles kind="roles"; other kinds
    // return Null → 500. AppHarnessSaveBody shares RoleSaveBody's shape.
    let (s, _v) = send(
        &app,
        "PUT",
        "/api/app-harness/roles/my-snippet",
        Some(json!({
            "model": null,
            "tier": "mid",
            "system_prompt": "echo hello",
            "temperature": null,
            "description": "a snippet",
            "inherit": null,
            "allowed_tiers": [],
            "skills": [],
            "token_budget": null,
            "max_turns": null,
            "tools": [],
            "soul": null,
        })),
    ).await;
    assert_eq!(s, StatusCode::OK, "harness PUT → 200");
    let harness_file = home.join("apps/musk/harness/roles").join("my-snippet.at");
    assert!(harness_file.exists(), "harness file written under AUTOOS_HOME");

    // DELETE it.
    let (s2, _v2) = send(&app, "DELETE", "/api/app-harness/roles/my-snippet", None).await;
    assert_eq!(s2, StatusCode::OK, "harness DELETE → 200");
    assert!(!harness_file.exists(), "harness file removed after DELETE");

    clear_autoos();
}
