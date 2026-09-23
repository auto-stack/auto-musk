//! PLAN-087 集成测试：canvas 会话全链（spawn→端点→帧→驱动→断言→收割）。
//!
//! #[ignore] 口径沿 `vm_serve_harness.rs` 先例：spawn 真 VM 进程（重、依赖
//! auto 二进制），不进默认测试面；评审/验收显式跑：
//!   cargo test --test canvas_live -- --ignored --nocapture
//! 前置：auto 可执行可解析（AUTO_EXE 或 auto-lang 主检出 release 构建）。
//! 进程卫生判定（AC-05）：tasklist 口径数 auto.exe 前后零净增
//! （scripts/vm-mcp-census.mjs 同口径的 Rust 内联版）。

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use auto_ai_agent::{Client, Tool};
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};
use tower::ServiceExt;

use musk::canvas::manager::{CanvasManager, CanvasState};
use musk::canvas::session::resolve_auto_exe;
use musk::tool_context::ToolContext;

struct MockClient;
#[async_trait::async_trait]
impl Client for MockClient {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        Err(ClientError::DaemonUnavailable)
    }
}

fn create_canvas_tool_context(ws_dir: &Path) -> ToolContext {
    let registry = musk::workspace::WorkspaceRegistry::load(ws_dir.join("workspaces.json"), ws_dir.to_path_buf());
    let state = musk::server::AppState {
        client: Arc::new(MockClient) as Arc<dyn Client>,
        auth: Arc::new(musk::auto_generated::auth::AuthStore::new(ws_dir.join("users.json"))),
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
    ToolContext {
        state: Arc::new(state),
        workspace_id: ws_id,
        parent_conversation_id: "canvas-test-conv".to_string(),
        progress: None,
        approval_mode: None,
    }
}

/// 测试工作区根（每测试独立，避免 workspace registry 串扰）。
fn test_workspace(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "musk-canvas-live-{}-{}",
        tag,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 最小 counter app（模板同构：pac.at + src/front/app.at）。
fn write_counter_app(ws: &PathBuf, name: &str) -> PathBuf {
    let app = ws.join(name);
    std::fs::create_dir_all(app.join("src/front")).unwrap();
    std::fs::write(
        app.join("pac.at"),
        format!(
            "name: \"{name}\"\nversion: \"1.0.0\"\nscene: \"ui\"\nrender: \"vue\"\ntitle: \"Counter\"\ntitle_zh: \"计数器\"\nwindow: \"fit\"\n"
        ),
    )
    .unwrap();
    std::fs::write(
        app.join("src/front/app.at"),
        "widget App {\n    model {\n        var count int = 0\n    }\n    view {\n        col {\n            text `Counter: ${.count}`\n            row {\n                button \"-\" { onclick: () => {.count -= 1} }\n                button \"Reset\" { onclick: () => {.count = 0} }\n                button \"+\" { onclick: () => {.count += 1} }\n            }\n            style: \"items-center gap-4 p-6\"\n        }\n    }\n}\n",
    )
    .unwrap();
    app
}

fn require_auto_exe() -> bool {
    if resolve_auto_exe().is_none() {
        eprintln!("SKIP: auto executable not resolvable (set AUTO_EXE)");
        return false;
    }
    true
}

/// tasklist 口径统计 auto.exe 进程数（census 内联版）。
fn count_auto_processes() -> u32 {
    #[cfg(windows)]
    {
        let out = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq auto.exe", "/FO", "CSV", "/NH"])
            .output()
            .expect("tasklist");
        let text = String::from_utf8_lossy(&out.stdout);
        text.lines().filter(|l| l.contains("auto.exe")).count() as u32
    }
    #[cfg(not(windows))]
    {
        let out = std::process::Command::new("sh")
            .args(["-c", "pgrep -c -x auto || true"])
            .output()
            .expect("pgrep");
        String::from_utf8_lossy(&out.stdout).trim().parse().unwrap_or(0)
    }
}

/// 轮询等待谓词成立（deadline 内）。
async fn wait_for<F>(deadline: Duration, mut pred: F) -> bool
where
    F: FnMut() -> bool,
{
    let start = Instant::now();
    while start.elapsed() < deadline {
        if pred() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    false
}

/// T-02/T-03 全链：spawn→端点发现→首帧 PNG→press +1→state 断言→stop→零孤儿。
/// 串行口径（#[serial]）：与 revival 臂共用全局进程表（census）。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[serial_test::serial]
#[ignore = "spawns a real VM subprocess (auto run --render=vm); run with -- --ignored"]
async fn canvas_session_lifecycle_drive_and_census() {
    if !require_auto_exe() {
        return;
    }
    let ws = test_workspace("lifecycle");
    let app = write_counter_app(&ws, "counter-live");
    let before = count_auto_processes();

    let manager = Arc::new(CanvasManager::new());
    let status = manager.start(app.clone(), &ws).await.expect("start");
    assert_eq!(status.state, CanvasState::Starting);

    // 首帧 ≤ 30s（冷启动含 iced/wgpu 初始化）。
    let got_frame = wait_for(Duration::from_secs(30), || {
        manager.frame().map(|f| !f.is_empty()).unwrap_or(false)
    })
    .await;
    assert!(got_frame, "no frame within 30s");
    let frame0 = manager.frame().unwrap();
    assert_eq!(&frame0[..4], b"\x89PNG", "frame is not a PNG");
    println!("[lifecycle] first frame: {} bytes (seq {})", frame0.len(), manager.seq());

    // 等状态 running。
    let running = wait_for(Duration::from_secs(5), || {
        manager.status().state == CanvasState::Running
    })
    .await;
    assert!(running, "session never reached running");

    // 驱动：vtree 找 "+" 按钮 id → press → state 断言计数递增（AC-03 前置）。
    let vtree = manager
        .with_client(|c| c)
        .await
        .expect("client")
        .snapshot(false)
        .await
        .expect("snapshot");
    println!("[lifecycle] vtree head:\n{}", vtree.lines().take(12).collect::<Vec<_>>().join("\n"));
    // 按 vtree 文本定位 "+" 按钮 id（AURA 树文本含节点 id 与文本标签）。
    let plus_id = find_button_id(&vtree, "+").expect("'+' button id in vtree");
    let act = manager
        .with_client(|c| c)
        .await
        .expect("client")
        .action(&plus_id, "press", None)
        .await
        .expect("press +");
    println!("[lifecycle] press result: {act}");
    let state_text = manager
        .with_client(|c| c)
        .await
        .expect("client")
        .state(Some(vec!["count".to_string()]))
        .await
        .expect("state");
    println!("[lifecycle] state after press: {state_text}");
    assert!(state_text.contains("1"), "count should be 1 after one press: {state_text}");

    // stop → 进程树零残留（census 前后零净增）。
    manager.stop().await;
    assert_eq!(manager.status().state, CanvasState::Stopped);
    let clean = wait_for(Duration::from_secs(5), || {
        count_auto_processes() <= before
    })
    .await;
    let after = count_auto_processes();
    assert!(clean, "orphan auto processes after stop: before={before} after={after}");
    let _ = std::fs::remove_dir_all(&ws);
}

/// T-04 崩溃复活：kill 本会话子进程 → ≤15s 帧恢复 → 退避计数 ≥1。
/// 串行口径（#[serial]）：与 lifecycle 臂共用全局进程表（census），并行会
/// 互杀对方会话（首跑实证）。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[serial_test::serial]
#[ignore = "spawns a real VM subprocess; run with -- --ignored"]
async fn canvas_crash_revival_within_15s() {
    if !require_auto_exe() {
        return;
    }
    let ws = test_workspace("revival");
    let app = write_counter_app(&ws, "counter-revive");

    let manager = Arc::new(CanvasManager::new());
    manager.start(app.clone(), &ws).await.expect("start");
    let got = wait_for(Duration::from_secs(30), || {
        manager.status().state == CanvasState::Running
    })
    .await;
    assert!(got, "session never reached running");

    // kill 本会话子进程树（模拟崩溃；manager.pid() 精确制导，不误伤他臂）。
    let pid = manager.pid().expect("session pid after start");
    musk::canvas::session::reap_tree_blocking(pid);

    // ≤15s 帧恢复（复活成功 → running + 新帧 seq 增长）。
    let seq_before = manager.seq();
    let recovered = wait_for(Duration::from_secs(15), || {
        manager.status().state == CanvasState::Running
            && manager.seq() > seq_before
            && manager.frame().is_some()
    })
    .await;
    assert!(recovered, "no frame recovery within 15s");
    let restarts = manager.status().restarts;
    assert!(restarts >= 1, "restart counter not incremented: {restarts}");
    println!("[revival] recovered, restarts={restarts}");

    manager.stop().await;
    let _ = std::fs::remove_dir_all(&ws);
}

/// 从 AURA vtree 文本里找文本为 `label` 的 button 的元素 id。
/// vtree 形态（autoui_snapshot 输出，8fecfcf69 实测）：`button #vnode_<n> "label" {`。
fn find_button_id(vtree: &str, label: &str) -> Option<String> {
    for line in vtree.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("button ") && line.contains(label) {
            if let Some(pos) = line.find("#vnode_") {
                let rest = &line[pos + "#vnode_".len()..];
                let id: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if !id.is_empty() {
                    return Some(format!("vnode_{id}"));
                }
            }
        }
    }
    None
}

/// T-05 API 面：resolve_multi 越界 400（无进程，纯路由层）+ 无帧 503。
/// 真实 happy-path 走 live 臂（canvas_session_lifecycle 已覆盖 manager 侧）；
/// 路由手拼 = serve() 同构（build_router + canvas_routes merge，parity_* 族
/// 同款 oneshot）。
#[tokio::test]
async fn canvas_api_out_of_root_rejected_with_roots_listed() {
    let dir = std::env::temp_dir().join(format!("musk-canvas-api-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("inside")).unwrap();

    let registry = musk::workspace::WorkspaceRegistry::load(dir.join("workspaces.json"), dir.clone());
    let state = musk::server::AppState {
        client: std::sync::Arc::new(MockClient) as std::sync::Arc<dyn Client>,
        auth: std::sync::Arc::new(musk::auto_generated::auth::AuthStore::new(dir.join("users.json"))),
        registry: std::sync::Arc::new(registry),
        chat_runs: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
        chat_cancels: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        run_idle_timeout: Duration::from_secs(300),
        canvas: std::sync::Arc::new(CanvasManager::new()),
    };
    let ws_id = {
        let q = musk::workspace::WorkspaceQuery { workspace: None };
        q.id_or_default(&state.registry)
    };
    let app = musk::auto_generated::server::build_router()
        .merge(musk::canvas::canvas_routes())
        .with_state(state.clone());

    // 越界路径（workspace 根外）→ 400 且报文含根列表（AC-06 API 面）。
    let outside = std::env::temp_dir().join("definitely-outside-musk-canvas");
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/canvas/start?workspace={ws_id}"))
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::json!({ "app_path": outside.to_string_lossy() }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 400, "out-of-root start must be rejected");
    let body = axum::body::to_bytes(resp.into_body(), 1 << 20).await.unwrap();
    let text = String::from_utf8_lossy(&body);
    assert!(text.contains("allowed root"), "must list roots: {text}");

    // 无帧 → 503。
    let resp = app
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/api/canvas/frame?workspace={ws_id}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 503);
    let _ = std::fs::remove_dir_all(&dir);
}

/// PLAN-090 T-07: M3 e2e 生成流闭环测试（AC-04/AC-06）。
/// 预置坑样例（span+onclick）→ ui_lint 抓 L001 → 修复 → 以 note-list bp spec
/// 生成 workspace app → ui_lint 零红 + bp_check 过 → canvas 启动首帧 →
/// vtree 定位按钮 → action press → state 断言 → stop → census 零孤儿。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[serial_test::serial]
#[ignore = "e2e generation flow with VM subprocess; run with -- --ignored"]
async fn canvas_generation_flow_m3_e2e() {
    if !require_auto_exe() {
        return;
    }
    if musk::canvas::bp_tools::resolve_blueprints_dir().is_none() {
        eprintln!("SKIP: blueprints dir not resolvable");
        return;
    }

    let ws = test_workspace("m3-genflow");
    let before = count_auto_processes();

    // 1. 预置坑样例（span 带 onclick）
    let pit_code = r#"
widget BadPit {
    model {
        var count int = 0
    }
    view {
        col {
            span "Click me" {
                onclick: () => { .count += 1 }
            }
        }
    }
}
"#;
    let diags = musk::canvas::ui_lint::lint_at_source(pit_code);
    assert!(!diags.is_empty(), "pit code must trigger diagnostics");
    assert!(diags.iter().any(|d| d.rule_id == "L001"), "must detect L001 for span with onclick");
    let report = musk::canvas::ui_lint::format_lint_report("bad_pit.at", &diags);
    assert!(report.contains("[L001]"), "report should mention L001: {report}");

    // 2. 修复并生成符合 data-display/note-list 契约的 workspace app
    let app_dir = ws.join("note-app");
    std::fs::create_dir_all(app_dir.join("src/front")).unwrap();
    std::fs::write(
        app_dir.join("pac.at"),
        "name: \"note-app\"\nversion: \"1.0.0\"\nscene: \"ui\"\nrender: \"vue\"\ntitle: \"Note App\"\nwindow: \"fit\"\n",
    )
    .unwrap();

    let clean_code = r#"
widget App {
    model {
        var count int = 0
        var loading bool = false
        var error str = ""
    }
    view {
        col {
            // ── EDIT: toolbar ──
            input {
                placeholder: "Search..."
            }
            separator {}
            // ── EDIT: filter_bar ──
            badge {
                text: "Notes"
            }
            text `Count: ${.count}`
            row {
                button "+" { onclick: () => {.count += 1} }
            }
            if .loading {
                text "Loading..."
            }
            if .error != "" {
                text .error
            }
        }
    }
}
"#;
    let clean_file = app_dir.join("src/front/app.at");
    std::fs::write(&clean_file, clean_code).unwrap();

    // 3. 验证 ui_lint 零红
    let diags_clean = musk::canvas::ui_lint::lint_at_source(clean_code);
    assert!(diags_clean.is_empty(), "clean app must have 0 findings: {:?}", diags_clean);
    let clean_report = musk::canvas::ui_lint::format_lint_report("app.at", &diags_clean);
    assert!(clean_report.contains("CLEAN (0 findings)"), "report must be CLEAN: {clean_report}");

    // 4. 验证 bp_check 通过 data-display/note-list 行为契约
    let ctx = create_canvas_tool_context(&ws);
    let bp_check_tool = musk::canvas::bp_tools::BpCheck::new(ctx.clone());
    let check_res = bp_check_tool
        .execute(&serde_json::json!({
            "path": "note-app/src/front/app.at",
            "spec": "data-display/note-list"
        }))
        .await
        .expect("bp_check execute");
    assert!(check_res.content.contains("PASS: bp_check passed"), "bp_check must pass: {}", check_res.content);
    assert!(check_res.content.contains("3 passed"), "bp_check should pass 3 items: {}", check_res.content);

    // 5. 启动 canvas_run 并等待首帧与 Running 状态
    let manager = Arc::new(CanvasManager::new());
    let status = manager.start(app_dir.clone(), &ws).await.expect("start");
    assert_eq!(status.state, CanvasState::Starting);

    let got_frame = wait_for(Duration::from_secs(30), || {
        manager.frame().map(|f| !f.is_empty()).unwrap_or(false)
    })
    .await;
    assert!(got_frame, "no frame within 30s");
    let frame0 = manager.frame().unwrap();
    assert_eq!(&frame0[..4], b"\x89PNG", "frame is not a PNG");

    let running = wait_for(Duration::from_secs(5), || {
        manager.status().state == CanvasState::Running
    })
    .await;
    assert!(running, "session never reached running");

    // 6. 驱动：vtree 定位 "+" 按钮 → action press → state 断言
    let vtree = manager
        .with_client(|c| c)
        .await
        .expect("client")
        .snapshot(false)
        .await
        .expect("snapshot");
    let plus_id = find_button_id(&vtree, "+").expect("'+' button id in vtree");
    let _act = manager
        .with_client(|c| c)
        .await
        .expect("client")
        .action(&plus_id, "press", None)
        .await
        .expect("press +");

    let state_text = manager
        .with_client(|c| c)
        .await
        .expect("client")
        .state(Some(vec!["count".to_string()]))
        .await
        .expect("state");
    assert!(state_text.contains("1"), "count should be 1 after press: {state_text}");

    // 7. stop → 检查 census 零孤儿
    manager.stop().await;
    assert_eq!(manager.status().state, CanvasState::Stopped);
    let clean = wait_for(Duration::from_secs(5), || {
        count_auto_processes() <= before
    })
    .await;
    let after = count_auto_processes();
    assert!(clean, "orphan auto processes after stop: before={before} after={after}");
    let _ = std::fs::remove_dir_all(&ws);
}

