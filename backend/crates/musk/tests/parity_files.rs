//! Plan 021 Phase A — /api/files/{workspace_id}/{*path} HTTP 接线测试。
//!
//! `/api/files` 服务 workspace 根内文件(display_image 工具生成的内联图片 URL
//! 依赖)。Phase A 把它从 hw server.rs:718 切到 ag server.at workspace_file
//! (经 extern workspace_file_do 委托)。本测试锚定其行为契约:
//! - 读存在的文本/图片文件 → 200 + 正确 Content-Type;
//! - 读不存在的文件 → 404;
//! - 越界路径(../ 逃逸到 workspace 根之外)→ 403 FORBIDDEN(沙箱安全锚);
//! - 子目录文件 → 200。
//!
//! 策略:用 ag build_router 的 oneshot 直接打 /api/files,验证 ag handler 行为。
//! workspace 根 = tmp dir,植入测试文件。

use std::sync::Arc;

use tower::ServiceExt;

use auto_ai_agent::Client;
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};

use musk::server::AppState;

/// Mock client — workspace_file 不调 LLM,client 仅占位。
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
        "musk-parity-files-{}-{}",
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

fn ws_id_of(state: &AppState) -> String {
    let q = musk::workspace::WorkspaceQuery { workspace: None };
    q.id_or_default(&state.registry)
}

/// Build a router wired to the ag workspace_file handler (via build_router).
fn ag_app(state: AppState) -> axum::Router {
    musk::auto_generated::server::build_router().with_state(state)
}

async fn get(app: &axum::Router, uri: &str) -> (axum::http::StatusCode, Option<String>, Vec<u8>) {
    use axum::body::to_bytes;
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(uri)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let ct = resp
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .map(|b| b.to_vec())
        .unwrap_or_default();
    (status, ct, body)
}

#[tokio::test]
async fn files_serves_text_file_with_markdown_mime() {
    let state = tmp_state();
    let ws_id = ws_id_of(&state);
    let ws = state.registry.get(&ws_id);
    std::fs::write(ws.root.join("readme.md"), "# hello\n").unwrap();

    let app = ag_app(state);
    let (status, ct, body) = get(&app, &format!("/api/files/{ws_id}/readme.md")).await;
    assert_eq!(status, axum::http::StatusCode::OK, "text file → 200");
    assert_eq!(ct.as_deref(), Some("text/markdown"), "md → text/markdown");
    assert_eq!(&body[..], b"# hello\n", "body matches file content");
}

#[tokio::test]
async fn files_serves_image_with_correct_mime() {
    let state = tmp_state();
    let ws_id = ws_id_of(&state);
    let ws = state.registry.get(&ws_id);
    // Minimal valid PNG header (8-byte signature).
    let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    std::fs::write(ws.root.join("chart.png"), png).unwrap();

    let app = ag_app(state);
    let (status, ct, body) = get(&app, &format!("/api/files/{ws_id}/chart.png")).await;
    assert_eq!(status, axum::http::StatusCode::OK, "png → 200");
    assert_eq!(ct.as_deref(), Some("image/png"), "png → image/png");
    assert_eq!(&body[..], &png[..], "binary body preserved");
}

#[tokio::test]
async fn files_serves_nested_subdir_file() {
    let state = tmp_state();
    let ws_id = ws_id_of(&state);
    let ws = state.registry.get(&ws_id);
    std::fs::create_dir_all(ws.root.join("docs/sub")).unwrap();
    std::fs::write(ws.root.join("docs/sub/notes.txt"), "deep note").unwrap();

    let app = ag_app(state);
    let (status, _ct, body) = get(&app, &format!("/api/files/{ws_id}/docs/sub/notes.txt")).await;
    assert_eq!(status, axum::http::StatusCode::OK, "nested file → 200");
    assert_eq!(&body[..], b"deep note", "nested body matches");
}

#[tokio::test]
async fn files_missing_file_returns_404() {
    let state = tmp_state();
    let ws_id = ws_id_of(&state);

    let app = ag_app(state);
    let (status, _ct, _body) = get(&app, &format!("/api/files/{ws_id}/nope.md")).await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND, "missing → 404");
}

#[tokio::test]
async fn files_escape_attempt_returns_403_forbidden() {
    // The canonicalize + starts_with confinement: a path that escapes the
    // workspace root (../../<sibling>) resolves outside ws.root → 403.
    let state = tmp_state();
    let ws_id = ws_id_of(&state);
    let ws = state.registry.get(&ws_id);
    // Plant a sibling dir outside the workspace root to escape into.
    let outside = ws.root.parent().unwrap().join("outside-target-021");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("secret.txt"), "escaped").unwrap();

    let app = ag_app(state);
    let (status, _ct, body) =
        get(&app, &format!("/api/files/{ws_id}/../outside-target-021/secret.txt")).await;
    // Either 404 (canonicalize of ../-containing path may resolve to outside →
    // starts_with fails → 403) or 403 directly. Accept FORBIDDEN as the safety
    // anchor; NOT_FOUND is also acceptable if canonicalize itself failed.
    assert!(
        status == axum::http::StatusCode::FORBIDDEN
            || status == axum::http::StatusCode::NOT_FOUND,
        "escape attempt must NOT be 200; got {status}"
    );
    // The escape must never serve the outside file's content.
    assert_ne!(&body[..], b"escaped", "escape must not leak outside content");
}
