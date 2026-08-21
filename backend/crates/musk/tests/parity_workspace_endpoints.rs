//! Plan 021 Phase C1 — workspace 端点 HTTP 接线测试(经 workspace,无副作用)。
//!
//! 补齐 Plan 020 遗漏的 4 个端点的 HTTP 层覆盖(逻辑层在 parity_specs/chats 已测):
//! - `/api/chats/sessions` DELETE(chat_delete_all)—— 内联测试漏挂;
//! - `/api/specs/drift-check`、`/api/specs/rebuild-relations`、
//!   `/api/specs/related/{item_id}` —— 逻辑层已覆盖,补 HTTP 接线层。
//! 这些端点经 workspace(specs/chats store),tmp_state 已隔离文件系统副作用。

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
        "musk-parity-ws-endpoints-{}-{}",
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

/// Upsert a spec item so the workspace has data for drift/rebuild/related.
/// Upsert a spec item. `refs_in_content` embeds references in the content so
/// rebuild_relations' scan_refs picks them up (depends_on isn't a SpecsItemPayload
/// field; forward refs come from content mentions of known IDs like "G1").
async fn upsert_spec(app: &axum::Router, section: &str, id: &str, status: &str, refs_in_content: &[&str]) {
    let content = if refs_in_content.is_empty() {
        "body".to_string()
    } else {
        format!("refs: {}", refs_in_content.join(" "))
    };
    let (s, _) = send(
        app,
        "POST",
        "/api/specs/item",
        Some(json!({
            "section": section,
            "item": {
                "id": id,
                "title": id,
                "content": content,
                "status": status,
            }
        })),
    ).await;
    assert_eq!(s, StatusCode::OK, "upsert {section}/{id}");
}

// ── /api/chats/sessions DELETE (chat_delete_all) ────────────────────────────

#[tokio::test]
async fn chats_delete_all_clears_sessions() {
    let state = tmp_state();
    let app = ag_app(state);
    // Create two sessions.
    for body in [
        json!({"title": "a"}),
        json!({"title": "b"}),
    ] {
        let (s, _) = send(&app, "POST", "/api/chats/session", Some(body)).await;
        assert_eq!(s, StatusCode::OK, "create session");
    }
    let (_, list) = send(&app, "GET", "/api/chats/sessions", None).await;
    assert_eq!(list["sessions"].as_array().unwrap().len(), 2, "two sessions before delete-all");

    // DELETE /api/chats/sessions → chat_delete_all.
    let (s, _) = send(&app, "DELETE", "/api/chats/sessions", None).await;
    assert_eq!(s, StatusCode::OK, "delete-all → 200");

    let (_, list2) = send(&app, "GET", "/api/chats/sessions", None).await;
    assert_eq!(list2["sessions"].as_array().unwrap().len(), 0, "all sessions cleared");
}

// ── /api/specs/drift-check ──────────────────────────────────────────────────

#[tokio::test]
async fn specs_drift_check_returns_shape() {
    let state = tmp_state();
    let app = ag_app(state);
    upsert_spec(&app, "goals", "G1", "empty", &[]).await;
    let (s, v) = send(&app, "POST", "/api/specs/drift-check", None).await;
    assert_eq!(s, StatusCode::OK, "drift-check → 200");
    // Shape: { memory_version, disk_version, drifted }.
    assert!(v.get("memory_version").is_some(), "has memory_version");
    assert!(v.get("disk_version").is_some(), "has disk_version");
    assert!(v.get("drifted").is_some(), "has drifted");
}

// ── /api/specs/rebuild-relations ────────────────────────────────────────────

#[tokio::test]
async fn specs_rebuild_relations_succeeds() {
    let state = tmp_state();
    let app = ag_app(state);
    // G1 (goal) + D1 (design depends_on G1) → rebuild populates G1.related = [D1].
    // (PLAN-030 复审修复：原用例写 7 区时代的 "plans" 区——PLAN-024 已删该区，
    // upsert 400；改用现存 designs 区，反向链接意图不变。)
    upsert_spec(&app, "goals", "G1", "empty", &[]).await;
    upsert_spec(&app, "designs", "D1", "proposed", &["G1"]).await;
    let (s, _v) = send(&app, "POST", "/api/specs/rebuild-relations", None).await;
    assert_eq!(s, StatusCode::OK, "rebuild-relations → 200");
    // After rebuild, G1.related should contain D1 (reverse link).
    let (_, related) = send(&app, "GET", "/api/specs/related/G1", None).await;
    let related_list = related["related"].as_array();
    assert!(
        related_list.map(|a| a.iter().any(|x| x == "D1")).unwrap_or(false),
        "G1.related contains D1 after rebuild; got {related}"
    );
}

// ── /api/specs/related/{item_id} ────────────────────────────────────────────

#[tokio::test]
async fn specs_related_returns_depends_and_related() {
    let state = tmp_state();
    let app = ag_app(state);
    upsert_spec(&app, "goals", "G1", "empty", &[]).await;
    // (PLAN-030 复审修复：同上，"plans" 区已删，改用 designs 区。)
    upsert_spec(&app, "designs", "D1", "proposed", &["G1"]).await;
    // D1's content mentions G1 → after rebuild G1.related gains D1 (reverse link).
    // Query G1's related: should contain D1.
    let (s, v) = send(&app, "GET", "/api/specs/related/G1", None).await;
    assert_eq!(s, StatusCode::OK, "related → 200");
    // Shape: { item_id, depends_on, related }.
    assert_eq!(v["item_id"], "G1");
    let related_list = v["related"].as_array();
    assert!(
        related_list.map(|a| a.iter().any(|x| x == "D1")).unwrap_or(false),
        "G1.related contains D1 (reverse link from content ref); got {v}"
    );
}
