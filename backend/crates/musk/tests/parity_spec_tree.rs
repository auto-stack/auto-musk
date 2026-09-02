//! parity_spec_tree.rs — PLAN-025 task 1.5: `/api/specs/*` HTTP 层测试。
//!
//! Tests the hand-written `spec_tree::spec_tree_routes` end-to-end via axum
//! oneshot: tree (empty workspace + nested folders/files), file read
//! (markdown body + 404), path-traversal rejection. Each test seeds an
//! isolated temp workspace under `{dir}/docs/specs/`.

use std::path::PathBuf;
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

/// Build an isolated temp workspace; returns `(state, root)` so tests can
/// seed `docs/specs/` files on disk before exercising the routes.
fn tmp_state() -> (AppState, PathBuf) {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "musk-parity-spec-tree-{}-{}",
        std::process::id(),
        n
    ));
    let _ = std::fs::create_dir_all(&dir);
    let registry =
        musk::workspace::WorkspaceRegistry::load(dir.join("workspaces.json"), dir.clone());
    let state = AppState {
        client: Arc::new(MockClient) as Arc<dyn Client>,
        auth: Arc::new(musk::auto_generated::auth::AuthStore::new(dir.join("users.json"))),
        registry: Arc::new(registry),
        chat_runs: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
    };
    (state, dir)
}

fn app(state: AppState) -> Router {
    musk::spec_tree::spec_tree_routes().with_state(state)
}

/// Send a request; return (status, body). Non-JSON body → `Value::String(raw)`
/// (mirrors `parity_plans::send`).
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
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        match serde_json::from_slice::<Value>(&bytes) {
            Ok(v) => v,
            Err(_) => Value::String(String::from_utf8_lossy(&bytes).into_owned()),
        }
    };
    (status, json)
}

/// Seed `{dir}/docs/specs/{rel}` with content (creates parent dirs).
fn seed_spec(dir: &std::path::Path, rel: &str, content: &str) {
    let p = dir.join("docs").join("specs").join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, content).unwrap();
}

#[tokio::test]
async fn spec_tree_empty_on_fresh_workspace() {
    let (state, _dir) = tmp_state();
    let a = app(state);
    // No docs/specs/ → empty tree (build_tree returns []), NOT 404.
    let (s, body) = send(&a, "GET", "/api/specs/tree", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn spec_tree_returns_nested_tree() {
    let (state, dir) = tmp_state();
    seed_spec(&dir, "00-overview.md", "# o\n");
    seed_spec(&dir, "01-architecture.md", "# a\n");
    seed_spec(&dir, "goals/README.md", "# goals\n");
    // dotfiles must be skipped by build_tree.
    seed_spec(&dir, ".gitkeep", "");

    let a = app(state);
    let (s, body) = send(&a, "GET", "/api/specs/tree", None).await;
    assert_eq!(s, StatusCode::OK);
    let arr = body.as_array().unwrap();
    // folders-first + alphabetical; dotfile dropped → 3 top-level nodes.
    assert_eq!(arr.len(), 3, "got {arr:?}");
    // goals folder comes first (folders before files).
    let goals = arr
        .iter()
        .find(|n| n["name"] == "goals")
        .expect("goals folder present");
    assert_eq!(goals["type"], "folder");
    let kids = goals["children"].as_array().unwrap();
    assert_eq!(kids.len(), 1);
    assert_eq!(kids[0]["name"], "README.md"); // .md NOT stripped
    // two files after the folder, alphabetical.
    assert_eq!(
        arr.iter().find(|n| n["name"] == "00-overview.md").unwrap()["type"],
        "file"
    );
    assert_eq!(
        arr.iter()
            .find(|n| n["name"] == "01-architecture.md")
            .unwrap()["type"],
        "file"
    );
}

#[tokio::test]
async fn spec_file_reads_markdown_body() {
    let (state, dir) = tmp_state();
    seed_spec(&dir, "goals/README.md", "# goals index\n");

    let a = app(state);
    let (s, body) = send(&a, "GET", "/api/specs/file/goals/README.md", None).await;
    assert_eq!(s, StatusCode::OK);
    // text/markdown body → fallback Value::String.
    assert_eq!(body.as_str().unwrap(), "# goals index\n");
}

#[tokio::test]
async fn spec_file_missing_returns_404() {
    let (state, dir) = tmp_state();
    seed_spec(&dir, "00-overview.md", "# o\n"); // dir exists, file doesn't

    let a = app(state);
    let (s, _body) = send(&a, "GET", "/api/specs/file/missing.md", None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn spec_file_rejects_traversal() {
    let (state, dir) = tmp_state();
    // place a secret OUTSIDE docs/specs/ at the workspace root.
    std::fs::write(dir.join("secret.txt"), "TOPSECRET").unwrap();
    seed_spec(&dir, "00-overview.md", "# o\n");

    let a = app(state);
    // URL-encoded ".." reaches the handler (axum percent-decodes {*path}),
    // where validate_path_pub rejects it with 400.
    let (s, body) = send(&a, "GET", "/api/specs/file/%2e%2e/secret.txt", None).await;
    assert_eq!(
        s,
        StatusCode::BAD_REQUEST,
        "traversal must be rejected with 400, got {s}: {body}"
    );
    // belt-and-suspenders: secret must never appear in the body.
    let leaked = body.as_str().unwrap_or("");
    assert!(
        !leaked.contains("TOPSECRET"),
        "secret leaked via traversal: {leaked}"
    );
}
