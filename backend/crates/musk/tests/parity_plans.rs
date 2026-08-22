//! parity_plans.rs — PLAN-024 task 3: `/api/plans` HTTP 层测试。
//!
//! Tests the hand-written `plans::plans_routes` end-to-end via axum oneshot:
//! list (empty + with items + include_archived), get, create (seq assignment),
//! update (plan_id preserved), transition (legal + illegal), archive. Each
//! test uses an isolated temp workspace so plans land in a throwaway
//! `{dir}/docs/plans/`.

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
        "musk-parity-plans-{}-{}",
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
    musk::plans::plans_routes().with_state(state)
}

/// Send a JSON request; return (status, body). Non-JSON body → Value::String(raw).
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

#[tokio::test]
async fn plans_full_lifecycle() {
    let a = app(tmp_state());

    // Empty list.
    let (s, body) = send(&a, "GET", "/api/plans", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["plans"].as_array().unwrap().len(), 0);

    // Create → PLAN-001, drafting.
    let (s, body) = send(
        &a,
        "POST",
        "/api/plans",
        Some(json!({ "feature_name": "Test Plan" })),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["seq"], 1);
    assert_eq!(body["id"], "PLAN-001");
    assert_eq!(body["status"], "drafting");
    assert_eq!(body["feature_name"], "Test Plan");
    assert_eq!(body["archived"], false);

    // Get by seq.
    let (s, body) = send(&a, "GET", "/api/plans/1", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["id"], "PLAN-001");

    // List now has 1.
    let (s, body) = send(&a, "GET", "/api/plans", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["plans"].as_array().unwrap().len(), 1);

    // Legal transition: drafting → executing.
    let (s, body) = send(
        &a,
        "POST",
        "/api/plans/1/transition",
        Some(json!({ "status": "executing" })),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["status"], "executing");

    // Illegal transition: executing → archived (终态不经 transition 端点).
    let (s, _body) = send(
        &a,
        "POST",
        "/api/plans/1/transition",
        Some(json!({ "status": "archived" })),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);

    // Update body (plan_id preserved even if body omits/contradicts it).
    let (s, body) = send(
        &a,
        "PUT",
        "/api/plans/1",
        Some(json!({ "content": "---\nstatus: executing\nplan_id: HACK\n---\n\n# Updated body" })),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["id"], "PLAN-001"); // plan_id preserved
    assert!(body["content"].as_str().unwrap().contains("# Updated body"));

    // Archive → archived=true + status=archived（PLAN-033 单一终态）.
    let (s, body) = send(&a, "POST", "/api/plans/1/archive", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["archived"], true);
    assert_eq!(body["status"], "archived");

    // Default list excludes archived.
    let (s, body) = send(&a, "GET", "/api/plans", None).await;
    assert_eq!(body["plans"].as_array().unwrap().len(), 0);

    // include_archived=true brings it back.
    let (s, body) = send(&a, "GET", "/api/plans?include_archived=true", None).await;
    assert_eq!(body["plans"].as_array().unwrap().len(), 1);

    // Get unknown seq → 404.
    let (s, _body) = send(&a, "GET", "/api/plans/999", None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn plans_create_assigns_sequential_seq() {
    let a = app(tmp_state());
    send(
        &a,
        "POST",
        "/api/plans",
        Some(json!({ "feature_name": "Alpha" })),
    )
    .await;
    send(
        &a,
        "POST",
        "/api/plans",
        Some(json!({ "feature_name": "Beta" })),
    )
    .await;
    let (s, body) = send(
        &a,
        "POST",
        "/api/plans",
        Some(json!({ "feature_name": "Gamma" })),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["seq"], 3);
    assert_eq!(body["id"], "PLAN-003");

    // Archived seq still counts toward next_seq (防漏号).
    send(&a, "POST", "/api/plans/3/archive", None).await;
    let (s, body) = send(
        &a,
        "POST",
        "/api/plans",
        Some(json!({ "feature_name": "Delta" })),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["seq"], 4);
}

#[tokio::test]
async fn plans_create_with_content_injects_frontmatter() {
    let a = app(tmp_state());
    let (s, body) = send(
        &a,
        "POST",
        "/api/plans",
        Some(json!({ "feature_name": "With Content", "content": "# My Plan\n\nbody text" })),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["id"], "PLAN-001");
    // content must now start with injected frontmatter.
    let content = body["content"].as_str().unwrap();
    assert!(content.starts_with("---"), "frontmatter injected: {content}");
    assert!(content.contains("plan_id: PLAN-001"));
    assert!(content.contains("status: drafting"));
    assert!(content.contains("# My Plan"));
    assert!(content.contains("body text"));
}

/// A plan body with numbered sections that map to all 6 spec sections.
const MERGE_PLAN_BODY: &str = "\
# [PLAN-001] Merge Test

## 0. 变更摘要

测试 merge 流程。

## 1. 目标

验证 plan → spec 沉淀。

## 2. 架构方案

PlansStore + plan_merge。

## 5. 详细设计

章节映射到 6 区。

## 6. 测试设计

merge API 端到端测试。

## 7. 验收标准

- [ ] merge 后 specs 出现溯源 item
";

#[tokio::test]
async fn plans_merge_gate_and_flow() {
    let state = tmp_state();
    let a = app(state.clone());

    // Create a plan with a merge-able body.
    let (s, _) = send(
        &a,
        "POST",
        "/api/plans",
        Some(json!({ "feature_name": "Merge Test", "content": MERGE_PLAN_BODY })),
    )
    .await;
    assert_eq!(s, StatusCode::OK);

    // Gate: merge before reviewed → 400.
    let (s, body) = send(&a, "POST", "/api/plans/1/merge", None).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let err_msg = body.as_str().unwrap_or("");
    assert!(
        err_msg.contains("reviewed"),
        "gate error should mention reviewed: {err_msg}"
    );

    // Walk to reviewed (drafting → reviewed is a legal skip).
    let (s, _) = send(
        &a,
        "POST",
        "/api/plans/1/transition",
        Some(json!({ "status": "reviewed" })),
    )
    .await;
    assert_eq!(s, StatusCode::OK);

    // Merge → 200 + result.
    let (s, body) = send(&a, "POST", "/api/plans/1/merge", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["plan_id"], "PLAN-001");
    assert!(body["items_created"].as_u64().unwrap() > 0);
    assert!(body["sections_touched"].as_array().unwrap().len() > 0);

    // Plan is now archived + status=archived（沉淀即归档，单一终态）.
    let (_s, body) = send(&a, "GET", "/api/plans?include_archived=true", None).await;
    let plan = &body["plans"][0];
    assert_eq!(plan["archived"], true);
    assert_eq!(plan["status"], "archived");

    // Specs ledger now has merged items (read directly via the store).
    let default_id = musk::workspace::WorkspaceQuery::default().id_or_default(&state.registry);
    let ws = state.registry.get(&default_id);
    let doc = ws.specs.load().expect("specs load");
    let total: usize = doc.sections.iter().map(|s| s.items.len()).sum();
    assert!(total > 0, "specs ledger should contain merged items");
    // At least one item traces back to the plan.
    let has_source = doc.sections.iter().any(|s| {
        s.items
            .iter()
            .any(|i| i.file.as_deref() == Some("docs/plans/001-merge-test.md"))
    });
    assert!(has_source, "some item should reference the plan file");

    // Idempotent re-merge is rejected (plan now archived, not reviewed).
    let (s, _body) = send(&a, "POST", "/api/plans/1/merge", None).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn plans_archive_reviewed_rejected() {
    let a = app(tmp_state());

    send(
        &a,
        "POST",
        "/api/plans",
        Some(json!({ "feature_name": "Reviewed Plan" })),
    )
    .await;
    // drafting → reviewed（合法跳步）
    let (s, _) = send(
        &a,
        "POST",
        "/api/plans/1/transition",
        Some(json!({ "status": "reviewed" })),
    )
    .await;
    assert_eq!(s, StatusCode::OK);

    // 直接归档被拒：400 且提示走 merge 沉淀。
    let (s, body) = send(&a, "POST", "/api/plans/1/archive", None).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let err_msg = body.as_str().unwrap_or("");
    assert!(
        err_msg.contains("merge"),
        "archive gate should point to merge: {err_msg}"
    );

    // 计划未移动（默认列表仍可见）、状态仍为 reviewed。
    let (s, body) = send(&a, "GET", "/api/plans", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["plans"].as_array().unwrap().len(), 1);
    assert_eq!(body["plans"][0]["status"], "reviewed");
}
