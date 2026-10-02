//! plan_flow_execution.rs — PLAN-096 V03: execution-scope tests against real
//! temp Git repos + worktrees. T-03 subset: concurrent plan allocation
//! (CreateNew, no overwrite, >999 boundary). T-04+ extends with worktree
//! scope / main-root protection / Canvas path cases.
//!
//! Serial: shares process state (global allocation mutex is per-store, but
//! the >999 case touches plan dirs inside its own tempdir).

use std::sync::Arc;

use auto_ai_agent::Tool;
use musk::plans::PlansStore;

fn tmp_store() -> (tempfile::TempDir, PlansStore) {
    let td = tempfile::tempdir().unwrap();
    let store = PlansStore::new(td.path().join("docs/plans"));
    (td, store)
}

/// AC-16：并发 create 同仓唯一分配——CreateNew 不覆盖既有计划，撞号重分配，
/// N 个并发创建得到 N 个互异序号、字节级独立文件。
#[test]
fn concurrent_create_allocates_unique_seqs_without_overwrite() {
    let (_td, store) = tmp_store();
    let store = Arc::new(store);
    const N: usize = 24;
    let mut handles = Vec::new();
    for i in 0..N {
        let s = store.clone();
        handles.push(std::thread::spawn(move || {
            s.create(&format!("Feature {i}"), "").unwrap()
        }));
    }
    let mut seqs = Vec::new();
    for h in handles {
        let pf = h.join().unwrap();
        seqs.push(pf.seq);
    }
    seqs.sort();
    let unique: std::collections::BTreeSet<u32> = seqs.iter().copied().collect();
    assert_eq!(unique.len(), N, "seqs must be unique: {seqs:?}");
    assert_eq!(seqs[0], 1, "contiguous from 1");
    assert_eq!(*seqs.last().unwrap(), N as u32);
    // 每个文件独立存在（无覆盖：总数 == N）。
    let all = store.list(true);
    assert_eq!(all.len(), N);
    // 回读 id 与文件名前缀一致。
    for pf in all {
        assert_eq!(pf.id, format!("PLAN-{:03}", pf.seq));
    }
}

/// AC-16：>999 明确阻断——不截断、不环绕、不覆盖旧计划。
#[test]
fn create_beyond_999_is_blocked_loudly() {
    let (_td, store) = tmp_store();
    // 手工放一个 999 号文件（绕过 create，模拟历史满号仓）。
    std::fs::write(
        store.plans_dir.join("999-full.md"),
        "---\nplan_id: PLAN-999\nstatus: drafting\n---\n# full\n",
    )
    .unwrap();
    let err = store.create("One Too Many", "").unwrap_err();
    assert!(err.contains("999"), "{err}");
    assert!(err.contains("blocked") || err.contains("exceeds"), "{err}");
    // 旧计划字节不变。
    let bytes = std::fs::read_to_string(store.plans_dir.join("999-full.md")).unwrap();
    assert_eq!(bytes, "---\nplan_id: PLAN-999\nstatus: drafting\n---\n# full\n");
    assert_eq!(store.list(true).len(), 1);
}

/// AC-16：撞号重分配——目标路径已被占（同号文件）时 create_new 冲突 →
/// 自动顺延下一个空号，绝不覆盖。
#[test]
fn create_on_collision_reallocates_instead_of_overwriting() {
    let (_td, store) = tmp_store();
    store.create("alpha", "").unwrap(); // 001
    // 手工占用 002 号位（模拟并发写者抢先落盘）。
    let squatter = store.plans_dir.join("002-squatter.md");
    std::fs::write(&squatter, "---\nplan_id: PLAN-002\n---\n# squatted\n").unwrap();
    let pf = store.create("beta", "").unwrap();
    assert_eq!(pf.seq, 3, "collided 002 → reallocates to 003");
    // squatter 字节不变。
    let bytes = std::fs::read_to_string(&squatter).unwrap();
    assert_eq!(bytes, "---\nplan_id: PLAN-002\n---\n# squatted\n");
}

// ── T-04: execution scope（真临时 Git 仓 + worktree） ───────────────────────

use auto_ai_agent::Client;
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};
use musk::server::AppState;

use serial_test::serial;

struct ScopeMockClient;
#[async_trait::async_trait]
impl Client for ScopeMockClient {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        Err(ClientError::DaemonUnavailable)
    }
}

/// 真临时 Git 仓 + 登记 worktree 的完整环境。返回 (守卫, 仓根, worktree,
/// AppState)。MUSK_PLAN_WORKTREE_ROOT 指向临时目录（恢复原值）。
struct ScopeEnv {
    _td: tempfile::TempDir,
    main_root: std::path::PathBuf,
    worktree: std::path::PathBuf,
    state: AppState,
}

fn scope_env() -> ScopeEnv {
    let td = tempfile::tempdir().unwrap();
    let main_root = td.path().join("repo");
    std::fs::create_dir_all(&main_root).unwrap();
    let out = std::process::Command::new("git")
        .args(["init", "-b", "master"])
        .current_dir(&main_root)
        .output()
        .unwrap();
    assert!(out.status.success(), "git init failed: {}", String::from_utf8_lossy(&out.stderr));
    std::process::Command::new("git").args(["config", "user.email", "t@t"]).current_dir(&main_root).output().unwrap();
    std::process::Command::new("git").args(["config", "user.name", "t"]).current_dir(&main_root).output().unwrap();
    std::fs::write(main_root.join("README.md"), "seed\n").unwrap();
    std::process::Command::new("git").args(["add", "."]).current_dir(&main_root).output().unwrap();
    std::process::Command::new("git").args(["commit", "-m", "seed"]).current_dir(&main_root).output().unwrap();

    let base = std::process::Command::new("git").args(["rev-parse", "HEAD"]).current_dir(&main_root).output().unwrap();
    let base = String::from_utf8_lossy(&base.stdout).trim().to_string();
    std::env::set_var("MUSK_PLAN_WORKTREE_ROOT", td.path().join("wt"));
    let lease = musk::plan_worktree::ensure_plan_worktree(&main_root, "PLAN-021", 21, &base).unwrap();
    let worktree = std::path::PathBuf::from(lease.worktree_root.clone());

    let registry = musk::workspace::WorkspaceRegistry::load(
        main_root.join(".autoos-workspaces.json"),
        main_root.clone(),
    );
    let state = AppState {
        client: Arc::new(ScopeMockClient) as Arc<dyn Client>,
        auth: Arc::new(musk::auto_generated::auth::AuthStore::new(
            main_root.join(".autoos/users.json"),
        )),
        registry: Arc::new(registry),
        canvas: Arc::new(musk::canvas::CanvasManager::new()),
        chat_runs: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
        chat_cancels: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        run_idle_timeout: std::time::Duration::from_secs(300),
    };
    ScopeEnv { _td: td, main_root, worktree, state }
}

/// AC-03：文件/命令工具落 worktree——相对写落 worktree；越界写拒；
/// run_command cwd = worktree（git status 看见 worktree 内的新文件）；
/// 主检出零改动。
#[test]
#[serial]
fn execution_scope_confines_tools_to_worktree() {
    let env = scope_env();
    let roots = Arc::new(vec![env.worktree.clone()]);
    // 相对写 → worktree。
    let wf = musk::tools::WriteFile::with_roots(roots.clone());
    let out = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(wf.execute(&serde_json::json!({"path": "w.txt", "content": "x"})));
    assert!(out.is_ok(), "write inside worktree must pass: {out:?}");
    assert!(env.worktree.join("w.txt").exists());
    // 越界写（主检出方向）→ 拒。
    let escape = env.worktree.parent().unwrap().join("escape.txt");
    let out = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(wf.execute(&serde_json::json!({"path": escape.display().to_string(), "content": "x"})));
    assert!(out.is_err(), "write outside scope must be rejected");
    assert!(!escape.exists());
    // run_command cwd = worktree：git status 看见 worktree 内未跟踪文件。
    let rc = musk::tools::RunCommand::with_roots(roots.clone());
    let out = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(rc.execute(&serde_json::json!({"cmd": "git status --porcelain"})));
    let text = out.expect("git status").content;
    assert!(text.contains("w.txt"), "cwd must be the worktree, got: {text}");
    // 主检出零改动（除 git 自身文件外无 w.txt/escape）。
    assert!(!env.main_root.join("w.txt").exists());
    assert!(!env.main_root.join("escape.txt").exists());
    std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
}

/// AC-03：工厂作用域判定——绑定 execute/review 相位 → worktree 根；
/// plan/document 相位 → None；无绑定 → None。
#[test]
#[serial]
fn factory_execution_scope_follows_phase_and_binding() {
    let env = scope_env();
    let ws_id = {
        let q = musk::workspace::WorkspaceQuery { workspace: None };
        q.id_or_default(&env.state.registry)
    };
    let ws = env.state.registry.get(&ws_id);
    let (run_id, _) = ws.relay.start_run(
        &musk::relay::store::StartRunRequest {
            run_id: Some("run-scope-1".into()),
            flow_id: Some("plan".into()),
            steps: Vec::new(),
            task: Some("需求".into()),
            authorization: None,
        },
        Some(ws_id.clone()),
    );
    let factory = musk::relay::driver::MuskAgentFactory {
        state: Arc::new(env.state.clone()),
        workspace_id: ws_id.clone(),
        run_id: run_id.clone(),
    };
    // 相位 plan（current_step=0）：无 scope。
    assert!(factory.execution_scope_for_current_step().is_none());

    // 绑定 + 推进到 execute 门：scope = worktree。
    let binding = musk::relay::plan_contract::PlanExecutionBinding {
        contract_version: 1,
        workspace_id: ws_id.clone(),
        main_root: env.main_root.display().to_string(),
        plan_id: "PLAN-021".into(),
        plan_path: "docs/plans/021-x.md".into(),
        plan_revision: 1,
        contract_hash: "ch".into(),
        semantic_hash: "sh".into(),
        semantic_parts: Default::default(),
        approved_canonical: None,
        skills_hashes: Default::default(),
        default_branch: "master".into(),
        base_commit: "base".into(),
        execution_root: Some(env.worktree.display().to_string()),
        dev_branch: Some("plan-021-dev".into()),
        authorization: "human".into(),
        repair_limit: 3,
        dependency_revisions: Default::default(),
    };
    let mut pe = musk::relay::plan_contract::PlanExecutionState::new("PLAN-021", 21, "docs/plans/021-x.md", 1);
    pe.binding = Some(binding);
    ws.relay.set_plan_execution(&run_id, pe).unwrap();
    ws.relay.advance(&run_id).unwrap();
    ws.relay
        .submit_handoff(&run_id, {
            let mut h = auto_ai_agent::orchestration::HandoffDocument::new("advisor", "coder");
            h.summary = "plan ready".into();
            h
        })
        .unwrap();
    let factory = musk::relay::driver::MuskAgentFactory {
        state: Arc::new(env.state.clone()),
        workspace_id: ws_id.clone(),
        run_id: run_id.clone(),
    };
    let scope = factory.execution_scope_for_current_step().expect("scope at execute");
    assert_eq!(*scope, env.worktree);
    std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
}

/// AC-03：Canvas app_path 解析走 execution scope——scope 外目标在 spawn
/// 前被拒（resolve 层报文列出授权根）。
#[test]
#[serial]
fn canvas_app_path_resolves_within_execution_scope() {
    let env = scope_env();
    let ws_id = {
        let q = musk::workspace::WorkspaceQuery { workspace: None };
        q.id_or_default(&env.state.registry)
    };
    let ctx = musk::tool_context::ToolContext {
        state: Arc::new(env.state.clone()),
        workspace_id: ws_id,
        parent_conversation_id: "run-canvas-scope".into(),
        progress: None,
        approval_mode: None,
        execution_root: Some(Arc::new(env.worktree.clone())),
    };
    // 越界目标：workspace 根下的目录（有 pac.at 以排除"非 app 目录"错误，
    // 确保拒因是 scope 而非目录形状）。
    let outside = env.main_root.join("outside-app");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("pac.at"), "app {}\n").unwrap();
    let reg = musk::canvas::tools::canvas_tool_registry(&ctx);
    let canvas_run = reg
        .iter()
        .find(|(n, _)| *n == "canvas_run")
        .map(|(_, t)| t.clone())
        .unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    // 绝对路径（存在、但在 scope 外）→ 解析层拒绝并列出授权根。
    let out = rt.block_on(canvas_run.execute(&serde_json::json!({
        "app_path": outside.display().to_string(),
    })));
    let err = format!("{:?}", out.unwrap_err());
    // 报文列出的授权根为 canonical 形态（\?\ 前缀）；比对尾段 + 拒因。
    let scope_tail = env
        .worktree
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    assert!(
        err.contains("outside all of the") && err.contains(&scope_tail),
        "denial must list the execution scope root: {err}"
    );
    // scope 内目标（合法 app 目录形状）通过解析与目录校验（不 spawn——
    // manager 会启动会话，此测试只验证解析层到达 validate 之后的语义；
    // 为避免进程副作用，这里用缺 pac.at 的目录名拿到"非 app 目录"错误，
    // 证明解析已落 scope 内）。
    let _ = std::fs::create_dir_all(env.worktree.join("plain-dir"));
    let out = rt.block_on(canvas_run.execute(&serde_json::json!({
        "app_path": "plain-dir",
    })));
    let err = format!("{:?}", out.unwrap_err());
    assert!(err.contains("pac.at"), "resolution must land inside the scope: {err}");
    std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
}
