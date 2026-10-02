//! plan_delivery_contract.rs — PLAN-096 V04: the controlled-delivery matrix
//! (T-08/T-09) against real temp Git repos + the real SpecsStore.
//! prepare/land/refresh/archive/cleanup facts, receipt re-entry, failure
//! injection (conflict / non-equivalent rebase / corrupt ledger / unmerged
//! cleanup), old-entry preservation. Serial: shares env + global registries.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use auto_ai_agent::Client;
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};
use musk::server::AppState;
use serial_test::serial;

struct MockClient;
#[async_trait::async_trait]
impl Client for MockClient {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        Err(ClientError::DaemonUnavailable)
    }
}

fn git_args(dir: &std::path::Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

const DELTA_MODULE: &str = "docs/specs/modules/demo-module.md";

const PLAN_BODY_TPL: &str = "---\n\
plan_id: PLAN-001\n\
status: drafting\n\
feature_name: delivery-demo\n\
created_at: 2026-10-02T00:00:00Z\n\
updated_at: 2026-10-02T00:00:00Z\n\
plan_revision: 1\n\
current_step: 0\n\
total_steps: 1\n\
supersedes_spec_components: []\n\
new_spec_components:\n\
  - docs/specs/modules/demo-module.md\n\
touched_goals: []\n\
---\n\n\
# [PLAN-001] delivery-demo\n\n\
## 1. 目标\n\n- 交付演示模块\n\n\
## 7. 验收标准\n\n- [ ] AC-01 模块可验证\n\n\
## 8. 执行步骤\n\n- [ ] T-01 实现模块\n";

/// 真 Git 仓环境：workspace root = 主检出（master），一路推进到 document
/// 相位（reviewed 计划 + 批准绑定 + worktree）。
struct DeliveryEnv {
    _td: tempfile::TempDir,
    state: AppState,
    ws_id: String,
    main_root: PathBuf,
    worktree: PathBuf,
    run_id: String,
}

fn unique_tag() -> String {
    format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    )
}

fn claim(stage: &str, outcome: &str) -> musk::relay::plan_contract::StageResult {
    musk::relay::plan_contract::StageResult {
        stage: stage.into(),
        plan_id: "PLAN-001".into(),
        attempt: 0,
        plan_revision: 1,
        outcome: outcome.into(),
        commit: None,
        acceptance_results: vec![musk::relay::plan_contract::AcResult {
            id: "AC-01".into(),
            status: "pass".into(),
            evidence: "cmd:check".into(),
        }],
        findings: vec![],
        evidence: vec!["cmd:ev".into()],
        spec_delta_ref: Some(DELTA_MODULE.into()),
        timestamp: 0,
        server_facts: None,
    }
}

fn reach_document(tag: &str) -> DeliveryEnv {
    let td = tempfile::tempdir().unwrap();
    let main_root = td.path().join("repo");
    std::fs::create_dir_all(&main_root).unwrap();
    git_args(&main_root, &["init", "-b", "master"]);
    git_args(&main_root, &["config", "user.email", "t@t"]);
    git_args(&main_root, &["config", "user.name", "t"]);
    std::fs::write(main_root.join("README.md"), "seed\n").unwrap();
    git_args(&main_root, &["add", "."]);
    git_args(&main_root, &["commit", "-m", "seed"]);
    std::env::set_var("MUSK_PLAN_WORKTREE_ROOT", td.path().join(format!("wt-{tag}")));

    let registry =
        musk::workspace::WorkspaceRegistry::load(main_root.join(".autoos-ws.json"), main_root.clone());
    let state = AppState {
        client: Arc::new(MockClient) as Arc<dyn Client>,
        auth: Arc::new(musk::auto_generated::auth::AuthStore::new(
            main_root.join(".autoos/users.json"),
        )),
        registry: Arc::new(registry),
        canvas: Arc::new(musk::canvas::CanvasManager::new()),
        chat_runs: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
        chat_cancels: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        run_idle_timeout: std::time::Duration::from_secs(300),
    };
    let ws_id = {
        let q = musk::workspace::WorkspaceQuery { workspace: None };
        q.id_or_default(&state.registry)
    };
    let ws = state.registry.get(&ws_id);
    let plans = musk::plans::PlansStore::new(ws.root.join("docs/plans"));
    let pf = plans.create("delivery-demo", PLAN_BODY_TPL).unwrap();
    assert_eq!(pf.seq, 1);

    let (run_id, _) = ws.relay.start_run(
        &musk::relay::store::StartRunRequest {
            run_id: Some(format!("run-{tag}-{}", unique_tag())),
            flow_id: Some("plan".into()),
            steps: Vec::new(),
            task: Some("做交付".into()),
            authorization: Some("human".into()),
        },
        Some(ws_id.clone()),
    );
    let b = musk::relay::plan_control::bootstrap_plan_run_default(
        "plan",
        "做交付",
        &ws.plans.plans_dir,
    )
    .unwrap();
    ws.relay.set_plan_execution(&run_id, b.state).unwrap();
    ws.relay
        .set_context_var(&run_id, "plan_file", &format!("docs/plans/{}", pf.filename));
    ws.relay.set_context_var(&run_id, "plan_authorization", "human");

    use musk::relay::plan_control::{attach_binding_on_gate_approve, on_stage_end, record_stage_claim, StageRouting};
    ws.relay.advance(&run_id).unwrap();
    record_stage_claim(&ws.relay, &run_id, claim("plan", "pass")).unwrap();
    assert!(matches!(on_stage_end(&state, &ws_id, &run_id), StageRouting::Advance));
    ws.relay
        .submit_handoff(&run_id, {
            let mut h = auto_ai_agent::orchestration::HandoffDocument::new("advisor", "coder");
            h.summary = "plan".into();
            h
        })
        .unwrap();
    attach_binding_on_gate_approve(&state, &ws_id, &run_id).unwrap();
    ws.relay
        .resolve_gate(&run_id, musk::relay::GateDecision::Approve)
        .unwrap();
    let worktree = PathBuf::from(
        ws.relay
            .plan_execution(&run_id)
            .unwrap()
            .binding
            .as_ref()
            .unwrap()
            .execution_root
            .clone()
            .unwrap(),
    );
    // execute：真提交 + AC 覆盖。
    std::fs::write(worktree.join("impl.txt"), "impl\n").unwrap();
    git_args(&worktree, &["add", "."]);
    git_args(&worktree, &["commit", "-m", "impl"]);
    let head = git_args(&worktree, &["rev-parse", "HEAD"]);
    let mut c = claim("execute", "pass");
    c.commit = Some(head);
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    assert!(matches!(on_stage_end(&state, &ws_id, &run_id), StageRouting::Advance));
    // review：pass → document。
    let head = git_args(&worktree, &["rev-parse", "HEAD"]);
    let mut c = claim("review", "pass");
    c.commit = Some(head);
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    assert!(matches!(on_stage_end(&state, &ws_id, &run_id), StageRouting::Advance));
    assert_eq!(ws.relay.plan_execution(&run_id).unwrap().phase, "document");

    DeliveryEnv { _td: td, state, ws_id, main_root, worktree, run_id }
}

/// T-08：prepare 需要真实 docs/specs/ 增量；提交后收据落盘；幂等重入。
#[test]
#[serial]
fn prepare_requires_delta_and_commits_in_worktree() {
    let env = reach_document("prep");
    let ws = env.state.registry.get(&env.ws_id);
    // 无增量 → 拒。
    let err = musk::plan_delivery::prepare(&env.state, &env.ws_id, &env.run_id).unwrap_err();
    assert!(err.contains("no changes under docs/specs"), "{err}");
    // 在 worktree 内撰写增量（document 相位文件工具的作用域）。
    std::fs::create_dir_all(env.worktree.join("docs/specs/modules")).unwrap();
    std::fs::write(
        env.worktree.join(DELTA_MODULE),
        "# demo-module\n\nPLAN-096 reviewed delta body.\n",
    )
    .unwrap();
    let out = musk::plan_delivery::prepare(&env.state, &env.ws_id, &env.run_id).unwrap();
    assert_eq!(out["checkpoint"], "prepared");
    let commit = out["delivery_commit"].as_str().unwrap().to_string();
    // worktree 干净、增量已提交。
    let st = git_args(&env.worktree, &["status", "--porcelain"]);
    assert!(st.is_empty());
    // 收据落盘（workspace 相对路径）。
    let receipt =
        musk::plan_delivery::load_receipt(&env.main_root, "PLAN-001").unwrap().unwrap();
    assert_eq!(receipt.receipt_key, "PLAN-001:r1");
    assert_eq!(receipt.checkpoints["prepared"]["delivery_commit"], commit.as_str());
    // 幂等重入：同 commit → ok idempotent。
    let out2 = musk::plan_delivery::prepare(&env.state, &env.ws_id, &env.run_id).unwrap();
    assert_eq!(out2["idempotent"], true);
    assert_eq!(ws.relay.plan_execution(&env.run_id).unwrap().delivery_checkpoints.contains_key("prepared"), true);
}

/// T-08：land 全链——rebase（含上游前移后的等价映射）+ range-diff 等价 +
/// 主检出 --ff-only + canonical tip 核对；冲突 → abort + 拒绝（现场保留）。
#[test]
#[serial]
fn land_rebase_equiv_and_ff_only_with_conflict_refusal() {
    let env = reach_document("land");
    // prepare 增量。
    std::fs::create_dir_all(env.worktree.join("docs/specs/modules")).unwrap();
    std::fs::write(env.worktree.join(DELTA_MODULE), "# demo-module\n\ndelta v1.\n").unwrap();
    musk::plan_delivery::prepare(&env.state, &env.ws_id, &env.run_id).unwrap();

    // 未 prepared 的 land 会过（prepared 已有）；先测正常链。
    let out = musk::plan_delivery::land(&env.state, &env.ws_id, &env.run_id).unwrap();
    assert_eq!(out["checkpoint"], "landed");
    let delivery = out["delivery_commit"].as_str().unwrap().to_string();
    let old = out["old_commit"].as_str().unwrap().to_string();
    // 无上游移动时 rebase 不改写（old==new 合法）；映射两值都已记录。
    let tip = git_args(&env.main_root, &["rev-parse", "master"]);
    assert_eq!(tip, delivery, "main ff-only at canonical tip");
    // 增量文件已在主检出。
    assert!(env.main_root.join(DELTA_MODULE).exists());
    let receipt = musk::plan_delivery::load_receipt(&env.main_root, "PLAN-001").unwrap().unwrap();
    assert_eq!(receipt.checkpoints["landed"]["range_diff_equivalent"], true);
    // 幂等重入。
    let out2 = musk::plan_delivery::land(&env.state, &env.ws_id, &env.run_id).unwrap();
    assert_eq!(out2["idempotent"], true);

    // 冲突拒绝：新 run 走到 document，上游改同一文件 → rebase 冲突 → abort。
    let env2 = reach_document("landcf");
    std::fs::create_dir_all(env2.worktree.join("docs/specs/modules")).unwrap();
    std::fs::write(env2.worktree.join(DELTA_MODULE), "# demo-module\n\nbranch version.\n").unwrap();
    musk::plan_delivery::prepare(&env2.state, &env2.ws_id, &env2.run_id).unwrap();
    // 主检出（默认分支）前移：同一文件不同内容。
    std::fs::create_dir_all(env2.main_root.join("docs/specs/modules")).unwrap();
    std::fs::write(env2.main_root.join(DELTA_MODULE), "# demo-module\n\nmain moved first.\n").unwrap();
    git_args(&env2.main_root, &["add", "."]);
    git_args(&env2.main_root, &["commit", "-m", "main-side edit"]);
    let err = musk::plan_delivery::land(&env2.state, &env2.ws_id, &env2.run_id).unwrap_err();
    assert!(err.contains("conflicted") || err.contains("rebase"), "{err}");
    // 现场保留：worktree 仍在、rebase 已 abort（状态干净）。
    assert!(env2.worktree.is_dir());
    let st = git_args(&env2.worktree, &["status", "--porcelain"]);
    assert!(st.is_empty(), "rebase aborted cleanly: {st}");
}

/// T-09：refresh 经 SpecsStore（store-mediated）——不相关条目与历史保留、
/// 回读一致、重复刷新零增长；坏账本响亮失败且字节不变。
#[test]
#[serial]
fn refresh_store_mediated_preserves_and_fails_loud_on_corrupt_ledger() {
    let env = reach_document("refresh");
    let ws = env.state.registry.get(&env.ws_id);
    // 预置不相关条目（经 store，测试亦守 store-mediated 纪律）。
    {
        let mut doc = ws.specs.load().unwrap();
        let mut item = musk::specs::SpecItem::new("legacy-keep", "既有条目");
        item.content = "必须保留".into();
        ws.specs.upsert_item(&mut doc, "goals", item).unwrap();
        ws.specs.save(&doc).unwrap();
    }
    // 未 land 直接 refresh → 拒（canonical 先行）。
    let err = musk::plan_delivery::refresh(&env.state, &env.ws_id, &env.run_id).unwrap_err();
    assert!(err.contains("landed checkpoint"), "{err}");

    // prepare + land。
    std::fs::create_dir_all(env.worktree.join("docs/specs/modules")).unwrap();
    std::fs::write(env.worktree.join(DELTA_MODULE), "# demo-module\n\ndelta.\n").unwrap();
    musk::plan_delivery::prepare(&env.state, &env.ws_id, &env.run_id).unwrap();
    musk::plan_delivery::land(&env.state, &env.ws_id, &env.run_id).unwrap();

    let out = musk::plan_delivery::refresh(&env.state, &env.ws_id, &env.run_id).unwrap();
    assert_eq!(out["checkpoint"], "ledger_refreshed");
    // 目标条目 + 交付收据条目 + 不相关条目共存。
    let doc = ws.specs.load().unwrap();
    let ids: Vec<&str> = doc.sections.iter().flat_map(|s| s.items.iter().map(|i| i.id.as_str())).collect();
    assert!(ids.iter().any(|i| *i == "PLAN-001-demo-module.md"), "delta item: {ids:?}");
    assert!(ids.iter().any(|i| *i == "PLAN-001-delivery"), "delivery item");
    assert!(ids.iter().any(|i| *i == "legacy-keep"), "unrelated item preserved");
    let delta_item = doc
        .sections
        .iter()
        .flat_map(|s| s.items.iter())
        .find(|i| i.id == "PLAN-001-demo-module.md")
        .unwrap();
    assert!(delta_item.tags.iter().any(|t| t.starts_with("source:")));
    assert!(delta_item.tags.iter().any(|t| t.starts_with("commit:")));
    assert_eq!(delta_item.file.as_deref(), Some(DELTA_MODULE));
    let version_after_first = doc.version;
    drop(doc);

    // 重复刷新 → 幂等（零增长）。
    let out2 = musk::plan_delivery::refresh(&env.state, &env.ws_id, &env.run_id).unwrap();
    assert_eq!(out2["idempotent"], true);
    let doc2 = ws.specs.load().unwrap();
    assert_eq!(doc2.version, version_after_first, "no duplicate items on re-refresh");
    assert_eq!(
        doc2.sections.iter().flat_map(|s| s.items.iter()).filter(|i| i.id.starts_with("PLAN-001")).count(),
        2
    );

    // 坏账本：响亮失败 + 字节不变（另一个环境）。
    let env2 = reach_document("refreshbad");
    {
        let ws2 = env2.state.registry.get(&env2.ws_id);
        std::fs::create_dir_all(env2.worktree.join("docs/specs/modules")).unwrap();
        std::fs::write(env2.worktree.join(DELTA_MODULE), "# delta\n").unwrap();
        musk::plan_delivery::prepare(&env2.state, &env2.ws_id, &env2.run_id).unwrap();
        musk::plan_delivery::land(&env2.state, &env2.ws_id, &env2.run_id).unwrap();
        // 破坏账本。
        let ledger = env2.main_root.join(".autoos/specs.json");
        std::fs::write(&ledger, "{corrupt json").unwrap();
        let before = std::fs::read(&ledger).unwrap();
        let err = musk::plan_delivery::refresh(&env2.state, &env2.ws_id, &env2.run_id).unwrap_err();
        assert!(err.contains("load failed") || err.contains("blocked"), "{err}");
        let after = std::fs::read(&ledger).unwrap();
        assert_eq!(before, after, "corrupt ledger bytes must be untouched");
        let _ = ws2;
    }
}

/// T-09：archive 门禁顺序 + finalize；cleanup 移除自有 worktree/分支并
/// 裁空组目录；清理失败 → cleanup_pending（archive 保持 delivered）。
#[test]
#[serial]
fn archive_cleanup_order_and_pending_semantics() {
    let env = reach_document("archcl");
    // 未 prepare 就 archive → 拒。
    let err = musk::plan_delivery::archive(&env.state, &env.ws_id, &env.run_id).unwrap_err();
    assert!(err.contains("prepared"), "{err}");
    std::fs::create_dir_all(env.worktree.join("docs/specs/modules")).unwrap();
    std::fs::write(env.worktree.join(DELTA_MODULE), "# delta\n").unwrap();
    musk::plan_delivery::prepare(&env.state, &env.ws_id, &env.run_id).unwrap();
    let err = musk::plan_delivery::archive(&env.state, &env.ws_id, &env.run_id).unwrap_err();
    assert!(err.contains("landed"), "{err}");
    musk::plan_delivery::land(&env.state, &env.ws_id, &env.run_id).unwrap();
    let err = musk::plan_delivery::archive(&env.state, &env.ws_id, &env.run_id).unwrap_err();
    assert!(err.contains("ledger_refreshed"), "{err}");
    musk::plan_delivery::refresh(&env.state, &env.ws_id, &env.run_id).unwrap();
    // 未完成 cleanup 前 cleanup 自身先拒（archive 未做）。
    let err = musk::plan_delivery::cleanup(&env.state, &env.ws_id, &env.run_id).unwrap_err();
    assert!(err.contains("archived checkpoint"), "{err}");

    let out = musk::plan_delivery::archive(&env.state, &env.ws_id, &env.run_id).unwrap();
    assert_eq!(out["completion_kind"], "delivered");
    let plans = musk::plans::PlansStore::new(env.main_root.join("docs/plans"));
    let pf = plans.get(1).unwrap();
    assert!(pf.archived);
    assert!(env.main_root.join("docs/plans/archived/001-delivery-demo.md").exists());

    // cleanup：worktree/分支移除，组目录裁空。
    let out = musk::plan_delivery::cleanup(&env.state, &env.ws_id, &env.run_id).unwrap();
    assert_eq!(out["checkpoint"], "cleaned");
    assert!(!env.worktree.exists(), "worktree removed");
    let gone = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/plan-001-dev"])
        .current_dir(&env.main_root)
        .output()
        .unwrap();
    assert!(!gone.status.success(), "branch deleted");
    // 幂等。
    let out2 = musk::plan_delivery::cleanup(&env.state, &env.ws_id, &env.run_id).unwrap();
    assert_eq!(out2["idempotent"], true);
}

/// T-09：清理失败（分支含未合入提交）→ cleanup_pending 检查点 + archive
/// 保持 delivered；不重跑前面副作用（landed 的 canonical tip 不变）。
#[test]
#[serial]
fn cleanup_failure_keeps_delivered_and_reports_pending() {
    let env = reach_document("clfail");
    std::fs::create_dir_all(env.worktree.join("docs/specs/modules")).unwrap();
    std::fs::write(env.worktree.join(DELTA_MODULE), "# delta\n").unwrap();
    musk::plan_delivery::prepare(&env.state, &env.ws_id, &env.run_id).unwrap();
    musk::plan_delivery::land(&env.state, &env.ws_id, &env.run_id).unwrap();
    musk::plan_delivery::refresh(&env.state, &env.ws_id, &env.run_id).unwrap();
    musk::plan_delivery::archive(&env.state, &env.ws_id, &env.run_id).unwrap();
    let main_tip = git_args(&env.main_root, &["rev-parse", "master"]);
    // 制造未合入提交（worktree 内多一笔）。
    std::fs::write(env.worktree.join("late.txt"), "late work\n").unwrap();
    git_args(&env.worktree, &["add", "."]);
    git_args(&env.worktree, &["commit", "-m", "late unmerged"]);
    let err = musk::plan_delivery::cleanup(&env.state, &env.ws_id, &env.run_id).unwrap_err();
    assert!(err.contains("cleanup_pending"), "{err}");
    // archive 保持 delivered；canonical tip 不动（不重跑副作用）。
    let receipt = musk::plan_delivery::load_receipt(&env.main_root, "PLAN-001").unwrap().unwrap();
    assert_eq!(receipt.completion_kind.as_deref(), Some("delivered"));
    assert!(receipt.checkpoints.contains_key("cleanup_pending"));
    assert_eq!(git_args(&env.main_root, &["rev-parse", "master"]), main_tip);
    assert!(env.worktree.exists(), "scene kept for manual resolution");
    // 手工解决（merge）后 cleanup 只补缺项。
    git_args(&env.main_root, &["merge", "--ff-only", "plan-001-dev"]);
    let out = musk::plan_delivery::cleanup(&env.state, &env.ws_id, &env.run_id).unwrap();
    assert_eq!(out["checkpoint"], "cleaned");
    assert!(!env.worktree.exists());
}

/// document 相位收束：五检查点齐 → Complete；缺任一 → Fail 点名缺项。
#[test]
#[serial]
fn document_stage_end_gates_on_checkpoints() {
    use musk::relay::plan_control::{on_stage_end, record_stage_claim, StageRouting};
    let env = reach_document("docgate");
    let ws = env.state.registry.get(&env.ws_id);
    record_stage_claim(&ws.relay, &env.run_id, claim("document", "pass")).unwrap();
    match on_stage_end(&env.state, &env.ws_id, &env.run_id) {
        StageRouting::Fail(e) => {
            for cp in ["prepared", "landed", "ledger_refreshed", "archived", "cleaned"] {
                assert!(e.contains(cp), "missing {cp} named: {e}");
            }
        }
        other => panic!("document pass without checkpoints must fail, got {other:?}"),
    }
    // 完成五检查点 → Complete。
    std::fs::create_dir_all(env.worktree.join("docs/specs/modules")).unwrap();
    std::fs::write(env.worktree.join(DELTA_MODULE), "# delta\n").unwrap();
    musk::plan_delivery::prepare(&env.state, &env.ws_id, &env.run_id).unwrap();
    musk::plan_delivery::land(&env.state, &env.ws_id, &env.run_id).unwrap();
    musk::plan_delivery::refresh(&env.state, &env.ws_id, &env.run_id).unwrap();
    musk::plan_delivery::archive(&env.state, &env.ws_id, &env.run_id).unwrap();
    musk::plan_delivery::cleanup(&env.state, &env.ws_id, &env.run_id).unwrap();
    record_stage_claim(&ws.relay, &env.run_id, claim("document", "pass")).unwrap();
    assert!(matches!(on_stage_end(&env.state, &env.ws_id, &env.run_id), StageRouting::Complete));
    std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
}
