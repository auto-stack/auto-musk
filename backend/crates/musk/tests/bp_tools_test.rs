//! PLAN-090 T-02 集成与活体验证：bp 通道三工具 —— bp_list, bp_show, bp_check。
//!
//! 验证往返正确性、坏样例失败、好样例通过、沙箱越界防护及降级报文（AC-02, AC-06）。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use auto_ai_agent::{Client, Tool};
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};
use serde_json::json;

use musk::canvas::bp_tools::{BpCheck, BpList, BpShow};
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
    let dir = std::env::temp_dir().join(format!("musk-bp-test-{}-{}", tag, std::process::id()));
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
    };
    (ctx, dir)
}

#[tokio::test]
#[serial_test::serial]
async fn test_bp_list_and_show_live() {
    if resolve_auto_exe().is_none() {
        eprintln!("SKIP: auto executable not resolvable");
        return;
    }

    let (ctx, dir) = create_test_context("list-show");

    // 1. bp_list 往返
    let list_tool = BpList::new(ctx.clone());
    let res = list_tool.execute(&json!({})).await.expect("bp_list execute");
    assert!(res.content.contains("Blueprint Catalog"), "must contain catalog header: {}", res.content);
    assert!(res.content.contains("data-display/note-list"), "must list note-list: {}", res.content);
    assert!(res.content.contains("form/login"), "must list login: {}", res.content);

    // 2. bp_show 往返（全名）
    let show_tool = BpShow::new(ctx.clone());
    let res = show_tool
        .execute(&json!({ "name": "data-display/note-list" }))
        .await
        .expect("bp_show execute");
    assert!(res.content.contains("kind = \"data-display\""), "frontmatter must match: {}", res.content);
    assert!(res.content.contains("name = \"note-list\""), "name must match: {}", res.content);
    assert!(res.content.contains("palette ="), "palette present: {}", res.content);
    assert!(res.content.contains("# Intent"), "intent present: {}", res.content);

    // 3. bp_show 往返（kind + name）
    let res = show_tool
        .execute(&json!({ "kind": "form", "name": "login" }))
        .await
        .expect("bp_show with kind/name");
    assert!(res.content.contains("kind = \"form\""), "form frontmatter: {}", res.content);
    assert!(res.content.contains("name = \"login\""), "login frontmatter: {}", res.content);

    // 4. bp_show 未知名友好报错（非 panic）
    let res = show_tool
        .execute(&json!({ "name": "nonexistent/bp_name_404" }))
        .await
        .expect("bp_show 404 should succeed with helpful text");
    assert!(res.content.contains("not found") || res.content.contains("failed"), "expected not found text: {}", res.content);

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
#[serial_test::serial]
async fn test_bp_check_pass_and_fail() {
    if resolve_auto_exe().is_none() {
        eprintln!("SKIP: auto executable not resolvable");
        return;
    }

    let (ctx, dir) = create_test_context("check");

    // 从 auto-lang blueprints 读取真实的 note-list default.at 作为好文件
    let bp_dir = musk::canvas::bp_tools::resolve_blueprints_dir().expect("blueprints dir");
    let ref_default = bp_dir.join("data-display/note-list/reference/default.at");
    let good_content = std::fs::read_to_string(&ref_default).expect("read reference default.at");
    let good_path = dir.join("good_note.at");
    std::fs::write(&good_path, good_content).unwrap();

    let check_tool = BpCheck::new(ctx.clone());
    let res = check_tool
        .execute(&json!({ "path": "good_note.at", "spec": "data-display/note-list" }))
        .await
        .expect("bp_check on good file");
    assert!(res.content.contains("PASS: bp_check passed"), "good file must pass: {}", res.content);
    assert!(res.content.contains("passed"), "must report passed: {}", res.content);

    // 准备一个缺少 loading/error 的坏文件
    let bad_content = "\
widget BadNote {\n\
    view {\n\
        col {\n\
            button \"Click\"\n\
        }\n\
    }\n\
}\n";
    let bad_path = dir.join("bad_note.at");
    std::fs::write(&bad_path, bad_content).unwrap();

    let res = check_tool
        .execute(&json!({ "path": "bad_note.at", "spec": "data-display/note-list" }))
        .await
        .expect("bp_check on bad file");
    assert!(res.content.contains("FAIL: bp_check failed"), "bad file must fail: {}", res.content);
    assert!(res.content.contains("failed"), "must report failed: {}", res.content);

    // 沙箱越界测试（AC-06）
    let outside = std::env::temp_dir().join("some_outside_file.at");
    std::fs::write(&outside, bad_content).unwrap();
    let err = check_tool
        .execute(&json!({ "path": outside.to_string_lossy(), "spec": "data-display/note-list" }))
        .await;
    assert!(err.is_err(), "out-of-root path must be rejected by sandbox");

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_file(outside);
}

#[tokio::test]
#[serial_test::serial]
async fn test_bp_degraded_live() {
    if resolve_auto_exe().is_none() {
        eprintln!("SKIP: auto executable not resolvable");
        return;
    }

    let (ctx, dir) = create_test_context("degraded");
    let empty_bp_dir = dir.join("empty_bp_root");
    std::fs::create_dir_all(&empty_bp_dir).unwrap();

    // 覆盖环境变量指向空目录
    std::env::set_var("AUTO_BLUEPRINTS_ROOT", &empty_bp_dir);

    let list_tool = BpList::new(ctx.clone());
    let res = list_tool.execute(&json!({})).await.expect("bp_list degraded execute");
    assert!(res.content.contains("[DEGRADED]"), "must report degraded: {}", res.content);

    let show_tool = BpShow::new(ctx.clone());
    let res = show_tool
        .execute(&json!({ "name": "form/login" }))
        .await
        .expect("bp_show degraded execute");
    assert!(
        res.content.contains("not found") || res.content.contains("[DEGRADED]"),
        "must report not found or degraded: {}",
        res.content
    );

    std::env::remove_var("AUTO_BLUEPRINTS_ROOT");
    let _ = std::fs::remove_dir_all(&dir);
}
