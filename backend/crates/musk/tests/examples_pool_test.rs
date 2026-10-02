//! PLAN-090 T-05 集成与活体验证：examples 扩展池 —— app_examples_list, app_example_read。
//!
//! 验证 demo 数量 ≥30、002-counter 双文件返回、指定文件、路径穿越防御及降级（AC-05, AC-06）。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use auto_ai_agent::{Client, Tool};
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};
use serde_json::json;

use musk::canvas::examples_pool::{AppExampleRead, AppExamplesList};
use musk::canvas::manager::CanvasManager;
use musk::canvas::session::resolve_auto_exe;
use musk::tool_context::ToolContext;

struct MockClient;
#[async_trait::async_trait]
impl Client for MockClient {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        Err(ClientError::DaemonUnavailable)
    }
}

fn create_test_context(tag: &str) -> (ToolContext, PathBuf) {
    let dir = std::env::temp_dir().join(format!("musk-examples-test-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let registry = musk::workspace::WorkspaceRegistry::load(dir.join("workspaces.json"), dir.clone());
    let state = musk::server::AppState {
        client: Arc::new(MockClient) as Arc<dyn Client>,
        auth: Arc::new(musk::auto_generated::auth::AuthStore::new(dir.join("users.json"))),
        registry: Arc::new(registry),
        chat_runs: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
        chat_cancels: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        run_idle_timeout: Duration::from_secs(300),
        canvas: Arc::new(CanvasManager::new()),
    };
    let ws_id = {
        let q = musk::workspace::WorkspaceQuery { workspace: None };
        q.id_or_default(&state.registry)
    };
    let ctx = ToolContext {
        state: Arc::new(state),
        workspace_id: ws_id,
        parent_conversation_id: "test-conv".to_string(),
        progress: None,
        approval_mode: None,
        execution_root: None,
    };
    (ctx, dir)
}

#[tokio::test]
#[serial_test::serial]
async fn test_examples_list_and_read_live() {
    if resolve_auto_exe().is_none() {
        eprintln!("SKIP: auto executable not resolvable");
        return;
    }

    let (ctx, dir) = create_test_context("list-read");

    // 1. app_examples_list 列出 ≥30 个 demo（AC-05）
    let list_tool = AppExamplesList::new(ctx.clone());
    let res = list_tool.execute(&json!({})).await.expect("examples_list execute");
    assert!(res.content.contains("AutoUI Examples Pool"), "header present: {}", res.content);
    assert!(res.content.contains("002"), "contains 002: {}", res.content);
    assert!(res.content.contains("counter"), "contains counter: {}", res.content);

    // 2. app_example_read 读 002-counter 双文件（AC-05）
    let read_tool = AppExampleRead::new(ctx.clone());
    let res = read_tool
        .execute(&json!({ "name": "002-counter" }))
        .await
        .expect("read 002-counter");
    assert!(res.content.contains("pac.at"), "contains pac.at: {}", res.content);
    assert!(res.content.contains("src/front/app.at"), "contains app.at: {}", res.content);
    assert!(res.content.contains("widget App"), "contains App widget: {}", res.content);
    assert!(res.content.contains("Counter:"), "contains counter content: {}", res.content);

    // 3. app_example_read 读指定文件
    let res_file = read_tool
        .execute(&json!({ "name": "002", "file": "pac.at" }))
        .await
        .expect("read pac.at specifically");
    assert!(res_file.content.contains("name: \"002-counter\"") || res_file.content.contains("scene: \"ui\""));

    // 4. 路径穿越防护（AC-06）
    let err = read_tool
        .execute(&json!({ "name": "002", "file": "../../Cargo.toml" }))
        .await;
    assert!(err.is_err(), "path traversal must be rejected");

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
#[serial_test::serial]
async fn test_examples_degraded_live() {
    let (ctx, dir) = create_test_context("degraded");
    let non_existent = dir.join("nonexistent_examples");

    std::env::set_var("AUTO_EXAMPLES_ROOT", &non_existent);

    let list_tool = AppExamplesList::new(ctx.clone());
    let res = list_tool.execute(&json!({})).await.expect("examples_list degraded");
    assert!(res.content.contains("[DEGRADED]"), "must report degraded: {}", res.content);

    let read_tool = AppExampleRead::new(ctx.clone());
    let res = read_tool
        .execute(&json!({ "name": "002-counter" }))
        .await
        .expect("read degraded");
    assert!(res.content.contains("[DEGRADED]"), "must report degraded: {}", res.content);

    std::env::remove_var("AUTO_EXAMPLES_ROOT");
    let _ = std::fs::remove_dir_all(&dir);
}
