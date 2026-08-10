//! Plan 022 遗留 — /api/forge/mode 端点 HTTP 测试（useForgeMode 后端持久化）。
//!
//! GET 读当前 forge 执行模式（默认 "gsd"），PUT 写 "gsd"/"check" 到
//! ~/.config/autoos/apps/musk/config.at（经 AUTOOS_HOME 隔离到 temp dir）。
//! 仅测 ag build_router（hw 无独立 router 副本——mode 读写委托 hw app_config，
//! 与 settings_link 同模式：ag 端点 + hw 数据层）。

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
        "musk-parity-forge-mode-{}-{}",
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

fn isolate_autoos() -> std::path::PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "musk-autoos-forge-mode-{}-{}",
        std::process::id(),
        n
    ));
    let _ = std::fs::create_dir_all(&dir);
    std::env::set_var("AUTOOS_HOME", &dir);
    dir
}

fn clear_autoos() {
    std::env::remove_var("AUTOOS_HOME");
}

/// 默认 mode 为 "gsd"（无配置文件时 GET 返回 gsd）。
#[tokio::test]
#[serial_test::serial]
async fn forge_mode_get_defaults_to_gsd() {
    isolate_autoos();
    let state = tmp_state();
    let app = ag_app(state);
    let (s, v) = send(&app, "GET", "/api/forge/mode", None).await;
    assert_eq!(s, StatusCode::OK, "GET /api/forge/mode → 200");
    assert_eq!(v["mode"], json!("gsd"), "默认 forge mode 为 gsd; got {v}");
    clear_autoos();
}

/// PUT check 持久化到 config.at，GET 回读为 check；重启（重新 load）仍为 check。
#[tokio::test]
#[serial_test::serial]
async fn forge_mode_put_persists_and_reads_back() {
    let home = isolate_autoos();
    let state = tmp_state();
    let app = ag_app(state);

    // 初始 gsd
    let (_s, v0) = send(&app, "GET", "/api/forge/mode", None).await;
    assert_eq!(v0["mode"], json!("gsd"));

    // PUT → check
    let (s, v) = send(&app, "PUT", "/api/forge/mode", Some(json!({ "mode": "check" }))).await;
    assert_eq!(s, StatusCode::OK, "PUT /api/forge/mode → 200");
    assert_eq!(v["mode"], json!("check"), "PUT 响应回显 check; got {v}");

    // 配置文件落盘且含 forge_mode
    let cfg_file = home.join("apps/musk/config.at");
    assert!(cfg_file.exists(), "config.at written under AUTOOS_HOME");
    let written = std::fs::read_to_string(&cfg_file).unwrap();
    assert!(written.contains("forge_mode"), "forge_mode persisted; got:\n{written}");

    // 同一 router 回读
    let (s2, v2) = send(&app, "GET", "/api/forge/mode", None).await;
    assert_eq!(s2, StatusCode::OK);
    assert_eq!(v2["mode"], json!("check"), "GET 回读 check; got {v2}");

    clear_autoos();
}

/// 非法 mode 拒绝（回落 gsd / 不污染配置）。
#[tokio::test]
#[serial_test::serial]
async fn forge_mode_rejects_invalid_value() {
    let home = isolate_autoos();
    let state = tmp_state();
    let app = ag_app(state);
    let (s, _v) = send(&app, "PUT", "/api/forge/mode", Some(json!({ "mode": "bogus" }))).await;
    assert_ne!(s, StatusCode::OK, "非法 mode 不应 200");
    let cfg_file = home.join("apps/musk/config.at");
    if cfg_file.exists() {
        let written = std::fs::read_to_string(&cfg_file).unwrap();
        assert!(!written.contains("bogus"), "非法 mode 不应写入配置");
    }
    // GET 仍回落 gsd
    let (_s, v) = send(&app, "GET", "/api/forge/mode", None).await;
    assert_eq!(v["mode"], json!("gsd"));
    clear_autoos();
}
