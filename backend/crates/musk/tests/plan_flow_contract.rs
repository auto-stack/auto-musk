//! plan_flow_contract.rs — PLAN-096 V02: the plan-flow execution contract
//! matrix. T-02 initial subset: complete frontmatter reading (lists), identity
//! rejection, semantic-hash positive/negative fixtures, skill snapshot
//! hard-failure, frozen-skill template injection. T-03+ extends with
//! approval/binding, stage results, budget and routing cases.
//!
//! Serial: cases share process env (MUSK_SKILLS_DIR overrides).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};

use musk::relay::plan_contract::{
    parse_frontmatter_yaml, sha256_hex, PlanContract, SkillEntry, SkillSnapshot,
};
use musk::relay::plan_flow::phase_task;
use serial_test::serial;

fn fixture(content: &str, dir: &std::path::Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, content).unwrap();
    p
}

const BASE_PLAN: &str = "---\n\
plan_id: PLAN-042\n\
status: drafting\n\
feature_name: 测试特性\n\
created_at: 2026-10-01T10:00:00Z\n\
updated_at: 2026-10-01T10:00:00Z\n\
plan_revision: 1\n\
current_step: 0\n\
total_steps: 2\n\
supersedes_spec_components:\n\
  - docs/specs/modules/alpha.md\n\
new_spec_components:\n\
  - docs/specs/modules/beta.md\n\
touched_goals:\n\
  - goal-x\n\
---\n\n\
# [PLAN-042] 测试特性\n\n\
## 0. 变更摘要\n\n概要。\n\n\
## 1. 目标\n\n- 交付 A\n\n\
## 2. 架构方案\n\n模块化。\n\n\
## 5. 详细设计\n\n核心设计。\n\n\
## 7. 验收标准\n\n- [ ] AC-01 A 可验证\n- [ ] AC-02 B 可验证\n\n\
## 8. 执行步骤\n\n- [ ] T-01 第一步\n- [ ] T-02 第二步\n\n\
## 9. 复审记录\n\n（空）\n";

fn full_snap() -> SkillSnapshot {
    let mut s = SkillSnapshot::default();
    for (n, body) in [
        ("auto-plan-new", "# auto-plan-new discipline"),
        ("auto-plan-work", "# auto-plan-work discipline"),
        ("auto-plan-review", "# auto-plan-review discipline"),
        ("auto-plan-merge", "# auto-plan-merge discipline"),
    ] {
        s.skills.insert(
            n.to_string(),
            SkillEntry {
                name: n.to_string(),
                path: format!("/{n}/SKILL.md"),
                sha256: sha256_hex(body.as_bytes()),
                content: body.to_string(),
            },
        );
    }
    s
}

/// 完整 frontmatter（含 spec-impact 三列表）无损读取；列表在显示 parser
/// （plans.rs 标量）中丢失，在合同读取中必须完整。
#[test]
fn plan_contract_reads_full_lists_and_tasks() {
    let td = tempfile::tempdir().unwrap();
    let path = fixture(BASE_PLAN, td.path(), "docs/plans/042-x.md");
    let c = PlanContract::read(&path, Some("PLAN-042")).expect("complete contract");
    assert_eq!(c.supersedes_spec_components, vec!["docs/specs/modules/alpha.md"]);
    assert_eq!(c.new_spec_components, vec!["docs/specs/modules/beta.md"]);
    assert_eq!(c.touched_goals, vec!["goal-x"]);
    assert_eq!(c.tasks.len(), 2);
    assert_eq!(c.tasks[0].id, "T-01");
    assert_eq!(c.acceptance.len(), 2);
    assert_eq!(c.acceptance[1].id, "AC-02");
    assert!(c.validate_complete().is_ok());
    // 显示用标量 parser 依旧看不到列表（旧路径兼容、不作权威）。
    let scalar = musk::plans::parse_frontmatter(BASE_PLAN);
    assert!(!scalar.contains_key("touched_goals"));
}

/// 身份与完整性反例：文件缺失 / id 不符 / 外来同名 / 无 frontmatter /
/// 缺任务 / 缺 AC / total_steps 不符 → 全部明确拒绝。
#[test]
fn contract_rejections_are_loud() {
    let td = tempfile::tempdir().unwrap();
    // 不存在。
    assert!(PlanContract::read(&td.path().join("nope.md"), None).is_err());
    let path = fixture(BASE_PLAN, td.path(), "042-x.md");
    // id 不符。
    assert!(PlanContract::read(&path, Some("PLAN-999")).is_err());
    // 无 frontmatter。
    let bare = fixture("# bare\n\n- [ ] T-01 x\n", td.path(), "043-bare.md");
    assert!(PlanContract::read(&bare, None).is_err());
    // 缺 §8 任务。
    let no_tasks = fixture(
        &BASE_PLAN.replace("## 8. 执行步骤\n\n- [ ] T-01 第一步\n- [ ] T-02 第二步\n", "## 8. 执行步骤\n\n"),
        td.path(),
        "044-notasks.md",
    );
    let c = PlanContract::read(&no_tasks, None).unwrap();
    assert!(c.validate_complete().is_err());
    // 缺 AC。
    let no_ac = fixture(
        &BASE_PLAN.replace("- [ ] AC-01 A 可验证\n- [ ] AC-02 B 可验证\n", ""),
        td.path(),
        "045-noac.md",
    );
    let c = PlanContract::read(&no_ac, None).unwrap();
    assert!(c.validate_complete().is_err());
    // total_steps 不符。
    let mismatch = fixture(
        &BASE_PLAN.replace("total_steps: 2", "total_steps: 9"),
        td.path(),
        "046-mismatch.md",
    );
    let c = PlanContract::read(&mismatch, None).unwrap();
    let err = c.validate_complete().unwrap_err();
    assert!(err.contains("total_steps"), "{err}");
}

/// AC-06 正反样例：仅进度（勾选/证据/状态/时间戳/§9/§10）→ 语义哈希不变；
/// 目标/AC/任务动作/规范增量 → 必变。
#[test]
fn semantic_hash_progress_positive_and_semantic_negative() {
    let td = tempfile::tempdir().unwrap();
    let p0 = fixture(BASE_PLAN, td.path(), "042-a.md");
    let c0 = PlanContract::read(&p0, None).unwrap();

    // 正例：进度-only（真实落盘形态：勾选+✅尾标+证据行+状态/时间戳/§9/§10）。
    let progressed = BASE_PLAN
        .replace(
            "- [ ] T-01 第一步",
            "- [x] T-01 第一步 [✅ 已完成]\n  证据：cargo test 绿 (commit abc)",
        )
        .replace("status: drafting", "status: execution_done")
        .replace("current_step: 0", "current_step: 2")
        .replace("updated_at: 2026-10-01T10:00:00Z", "updated_at: 2026-10-03T09:00:00Z")
        .replace("## 9. 复审记录\n\n（空）", "## 9. 复审记录\n\nstage: work | pass | 证据满");
    let p1 = fixture(&progressed, td.path(), "042-b.md");
    let c1 = PlanContract::read(&p1, None).unwrap();
    assert_eq!(c0.semantic_hash, c1.semantic_hash, "progress-only must not rotate the semantic hash");
    assert_ne!(c0.contract_hash, c1.contract_hash);

    // 反例组：任一语义面变化都要换哈希。
    let negatives: [(&str, String); 5] = [
        ("goal", BASE_PLAN.replace("交付 A", "交付 A（范围扩大）")),
        ("ac", BASE_PLAN.replace("AC-01 A 可验证", "AC-01 A2 可验证")),
        ("task", BASE_PLAN.replace("T-01 第一步", "T-01 第一步改道")),
        ("design", BASE_PLAN.replace("核心设计。", "核心设计（改）。")),
        ("delta", BASE_PLAN.replace("- goal-x", "- goal-y")),
    ];
    for (name, edited) in negatives {
        let p = fixture(&edited, td.path(), &format!("042-neg-{name}.md"));
        let c = PlanContract::read(&p, None).unwrap();
        assert_ne!(
            c0.semantic_hash, c.semantic_hash,
            "{name} edit must rotate the semantic hash"
        );
    }
}

/// 技能快照缺源/空源硬失败（产品内正式 plan 流不得默默用旧模板）。
#[test]
fn skill_snapshot_missing_source_hard_fails() {
    let td = tempfile::tempdir().unwrap();
    let err = musk::relay::plan_contract::snapshot_plan_skills(td.path()).unwrap_err();
    assert!(err.contains("auto-plan-new"), "{err}");
    // 只差一个也失败。
    for name in ["auto-plan-new", "auto-plan-work", "auto-plan-review"] {
        let d = td.path().join(name);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("SKILL.md"), format!("# {name}")).unwrap();
    }
    let err = musk::relay::plan_contract::snapshot_plan_skills(td.path()).unwrap_err();
    assert!(err.contains("auto-plan-merge"), "{err}");
    // 空内容同拒。
    let d = td.path().join("auto-plan-merge");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("SKILL.md"), " \n").unwrap();
    assert!(musk::relay::plan_contract::snapshot_plan_skills(td.path()).is_err());
}

/// 真源技能指纹（AC-01 live 证据）：从仓内 .agents/skills 快照——四份内容
/// 存在、hash 与磁盘字节一致、mid-run 二次快照稳定。
#[test]
#[serial]
fn real_skill_snapshot_matches_disk_bytes() {
    // 构建期仓库根：backend/crates/musk → 上三级 = 仓根。
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..");
    let src = repo.join(".agents").join("skills");
    assert!(src.is_dir(), "vendored skills must exist at {}", src.display());
    let snap = musk::relay::plan_contract::snapshot_plan_skills(&src).expect("snapshot ok");
    for name in musk::relay::plan_contract::PLAN_SKILLS {
        let entry = &snap.skills[name];
        let bytes = std::fs::read(src.join(name).join("SKILL.md")).unwrap();
        assert_eq!(entry.sha256, sha256_hex(&bytes), "{name} hash == disk bytes");
        assert_eq!(entry.content, String::from_utf8_lossy(&bytes));
    }
    // 快照幂等：重跑一致（中途技能不变则 hash 不变）。
    let snap2 = musk::relay::plan_contract::snapshot_plan_skills(&src).unwrap();
    assert_eq!(snap.hashes(), snap2.hashes());
}

/// 相位模板消费冻结技能：完整快照 → 技能全文+hash 注入；空快照 → 阻断
/// 条款点名缺失技能（不静默降级）。
#[test]
fn phase_task_consumes_frozen_skills() {
    use std::collections::HashMap;
    let ctx = HashMap::new();
    let t = phase_task("plan", "execute", "需求", &ctx, &full_snap()).unwrap();
    assert!(t.contains("# auto-plan-work discipline"));
    assert!(t.contains("complete_plan_stage"));
    assert!(t.contains("Canvas 生成指引"));
    let empty = SkillSnapshot {
        skills: BTreeMap::new(),
    };
    let t = phase_task("plan", "review", "需求", &ctx, &empty).unwrap();
    assert!(t.contains("阻断性缺陷"));
    assert!(t.contains("auto-plan-review"));
}

/// frontmatter YAML 解析直接给出标量/列表访问（Q-02：列表不丢）。
#[test]
fn frontmatter_yaml_parses_lists() {
    let doc = parse_frontmatter_yaml(BASE_PLAN).unwrap();
    let list = &doc["touched_goals"];
    match list {
        yaml_rust::Yaml::Array(items) => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].as_str().unwrap(), "goal-x");
        }
        other => panic!("touched_goals must be a list, got {other:?}"),
    }
    assert_eq!(doc["plan_id"].as_str().unwrap(), "PLAN-042");
}

// ── T-05: stage routing / results / owner / cancel matrix (real git) ───────

use auto_ai_agent::Client as _ClientT;
use musk::relay::plan_control::{
    attach_binding_on_gate_approve, on_stage_end, record_stage_claim, StageRouting,
};
use musk::relay::plan_contract::{AcResult, Finding, StageResult};
use musk::server::AppState;

struct RouteMockClient;
#[async_trait::async_trait]
impl _ClientT for RouteMockClient {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        Err(ClientError::DaemonUnavailable)
    }
}

/// 真 Git 仓工作环境：workspace root = 临时 git 仓（默认分支 master）。
struct RouteEnv {
    _td: tempfile::TempDir,
    state: AppState,
    ws_id: String,
    main_root: std::path::PathBuf,
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

fn route_env(tag: &str) -> RouteEnv {
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
    let registry = musk::workspace::WorkspaceRegistry::load(
        main_root.join(".autoos-ws.json"),
        main_root.clone(),
    );
    let state = AppState {
        client: Arc::new(RouteMockClient) as Arc<dyn _ClientT>,
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
    RouteEnv { _td: td, state, ws_id, main_root }
}

const PLAN_BODY: &str = "---\n\
plan_id: PLAN-001\n\
status: drafting\n\
feature_name: route-demo\n\
created_at: 2026-10-02T00:00:00Z\n\
updated_at: 2026-10-02T00:00:00Z\n\
plan_revision: 1\n\
current_step: 0\n\
total_steps: 1\n\
supersedes_spec_components: []\n\
new_spec_components: []\n\
touched_goals: []\n\
---\n\n\
# [PLAN-001] route-demo\n\n\
## 1. 目标\n\n- 交付演示功能\n\n\
## 7. 验收标准\n\n- [ ] AC-01 演示可验证\n\n\
## 8. 执行步骤\n\n- [ ] T-01 实现演示\n";

fn start_bound_run(env: &RouteEnv, tag: &str) -> String {
    let ws = env.state.registry.get(&env.ws_id);
    let plans = musk::plans::PlansStore::new(ws.root.join("docs/plans"));
    let pf = plans.create("route-demo", PLAN_BODY).unwrap();
    assert_eq!(pf.seq, 1);
    let (run_id, _) = ws.relay.start_run(
        &musk::relay::store::StartRunRequest {
            run_id: Some(format!("run-{tag}-{}", uuid_tag())),
            flow_id: Some("plan".into()),
            steps: Vec::new(),
            task: Some("做演示".into()),
            authorization: Some("human".into()),
        },
        Some(env.ws_id.clone()),
    );
    let b = musk::relay::plan_control::bootstrap_plan_run_default(
        "plan",
        "做演示",
        &ws.plans.plans_dir,
    )
    .unwrap();
    ws.relay.set_plan_execution(&run_id, b.state).unwrap();
    ws.relay
        .set_context_var(&run_id, "plan_file", &format!("docs/plans/{}", pf.filename));
    ws.relay.set_context_var(&run_id, "plan_authorization", "human");
    run_id
}

fn claim(stage: &str, outcome: &str, plan_rev: u32) -> StageResult {
    StageResult {
        stage: stage.into(),
        plan_id: "PLAN-001".into(),
        attempt: 0,
        plan_revision: plan_rev,
        outcome: outcome.into(),
        commit: None,
        acceptance_results: vec![],
        findings: vec![],
        evidence: vec![],
        spec_delta_ref: None,
        timestamp: 0,
        server_facts: None,
    }
}

fn uuid_tag() -> String {
    format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    )
}

/// 推进到 execute 相位（plan pass → handoff → 批准绑定 → 门批准）。
fn reach_execute(env: &RouteEnv, run_id: &str) -> std::path::PathBuf {
    let ws = env.state.registry.get(&env.ws_id);
    ws.relay.advance(run_id).unwrap();
    record_stage_claim(&ws.relay, run_id, claim("plan", "pass", 1)).unwrap();
    assert!(matches!(
        on_stage_end(&env.state, &env.ws_id, run_id),
        StageRouting::Advance
    ));
    ws.relay
        .submit_handoff(run_id, {
            let mut h = auto_ai_agent::orchestration::HandoffDocument::new("advisor", "coder");
            h.summary = "plan done".into();
            h
        })
        .unwrap();
    attach_binding_on_gate_approve(&env.state, &env.ws_id, run_id).unwrap();
    let binding = ws.relay.plan_execution(run_id).unwrap().binding.unwrap();
    let wt = std::path::PathBuf::from(binding.execution_root.unwrap());
    ws.relay
        .resolve_gate(run_id, musk::relay::GateDecision::Approve)
        .unwrap();
    wt
}

/// AC-04/05/06/10 主矩阵：plan→批准绑定→execute（未提交拒/提交过）→
/// review（needs_fix 回退、无进展早停）+ 取消 + 迟到结果 + 语义漂移。
#[test]
#[serial]
fn stage_routing_full_lifecycle_matrix() {
    let env = route_env("matrix");
    let ws = env.state.registry.get(&env.ws_id);
    let run_id = start_bound_run(&env, "matrix");

    // ── plan 相位 ──
    ws.relay.advance(&run_id).unwrap();
    // Done 无结果 → stage_incomplete（AC-04）。
    match on_stage_end(&env.state, &env.ws_id, &run_id) {
        StageRouting::Fail(e) => assert!(e.contains("stage_incomplete"), "{e}"),
        other => panic!("expected stage_incomplete, got {other:?}"),
    }
    // 迟到/错相位声明被工具入口拒绝。
    let err = record_stage_claim(&ws.relay, &run_id, claim("review", "pass", 1)).unwrap_err();
    assert!(err.contains("does not match"), "{err}");
    // 正常 plan pass → Advance。
    record_stage_claim(&ws.relay, &run_id, claim("plan", "pass", 1)).unwrap();
    assert!(matches!(
        on_stage_end(&env.state, &env.ws_id, &run_id),
        StageRouting::Advance
    ));
    let pe = ws.relay.plan_execution(&run_id).unwrap();
    assert_eq!(pe.phase, "execute");
    assert_eq!(pe.plan_id, "PLAN-001");

    // ── 执行门：human 批准绑定 ──
    ws.relay
        .submit_handoff(&run_id, {
            let mut h = auto_ai_agent::orchestration::HandoffDocument::new("advisor", "coder");
            h.summary = "plan done".into();
            h
        })
        .unwrap();
    assert!(ws.relay.plan_execution(&run_id).unwrap().binding.is_none());
    attach_binding_on_gate_approve(&env.state, &env.ws_id, &run_id).unwrap();
    let binding = ws.relay
        .plan_execution(&run_id)
        .unwrap()
        .binding
        .expect("bound at approval");
    assert_eq!(binding.default_branch, "master");
    assert_eq!(binding.authorization, "human");
    assert_eq!(binding.repair_limit, 3);
    assert_eq!(binding.dev_branch.as_deref(), Some("plan-001-dev"));
    assert_eq!(binding.skills_hashes.len(), 4);
    let wt = std::path::PathBuf::from(binding.execution_root.clone().unwrap());
    assert!(wt.is_dir(), "worktree created at approval");
    ws.relay
        .resolve_gate(&run_id, musk::relay::GateDecision::Approve)
        .unwrap();

    // ── execute 相位 ──
    // 伪 pass（无提交）→ 拒（AC-04）。
    let mut c = claim("execute", "pass", 1);
    c.acceptance_results = vec![AcResult {
        id: "AC-01".into(),
        status: "pass".into(),
        evidence: "cmd:check".into(),
    }];
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    match on_stage_end(&env.state, &env.ws_id, &run_id) {
        StageRouting::Fail(e) => assert!(e.contains("no commits since base"), "{e}"),
        other => panic!("uncommitted pass must fail, got {other:?}"),
    }
    // 真提交 + AC 覆盖 → Advance（进 review，轮次 1）。
    let rnd = uuid_tag();
    std::fs::write(wt.join(format!("app-{rnd}.txt")), "implementation\n").unwrap();
    git_args(&wt, &["add", "."]);
    git_args(&wt, &["commit", "-m", &format!("feat: demo {rnd}")]);
    let head1 = git_args(&wt, &["rev-parse", "HEAD"]);
    let mut c = claim("execute", "pass", 1);
    c.commit = Some(head1.clone());
    c.acceptance_results = vec![AcResult {
        id: "AC-01".into(),
        status: "pass".into(),
        evidence: format!("cmd:cargo test -- --nocapture # verified {rnd}"),
    }];
    c.evidence = vec![format!("cmd:cargo test green ({rnd})")];
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    assert!(matches!(
        on_stage_end(&env.state, &env.ws_id, &run_id),
        StageRouting::Advance
    ));
    let pe = ws.relay.plan_execution(&run_id).unwrap();
    assert_eq!(pe.phase, "review");
    assert_eq!(pe.attempt, 1, "round 1");
    let plans = musk::plans::PlansStore::new(ws.root.join("docs/plans"));
    assert_eq!(plans.get(1).unwrap().status.as_str(), "execution_done");

    // ── review：needs_fix 有界回退 ──
    let mut c = claim("review", "needs_fix", 1);
    c.findings = vec![Finding {
        id: "F-1".into(),
        task: Some("T-01".into()),
        ac: Some("AC-01".into()),
        description: "输出不完整".into(),
    }];
    c.evidence = vec![format!("review notes round1 ({rnd})")];
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    assert!(matches!(
        on_stage_end(&env.state, &env.ws_id, &run_id),
        StageRouting::RewoundToExecute
    ));
    let pe = ws.relay.plan_execution(&run_id).unwrap();
    assert_eq!(pe.phase, "execute");
    assert_eq!(pe.attempt, 2);
    assert_eq!(pe.repair_count, 1);
    assert_eq!(plans.get(1).unwrap().status.as_str(), "executing");
    let (r, _) = ws.relay.advance(&run_id).unwrap();
    assert!(
        matches!(&r, musk::relay::AdvanceResult::ExecuteStep { role_id, .. } if role_id == "coder"),
        "engine rewound to coder: {r:?}"
    );

    // ── 修复轮 2：无进展早停（同 findings/同提交/同证据）──
    let mut c = claim("execute", "pass", 1);
    c.commit = Some(head1.clone()); // 无新提交
    c.acceptance_results = vec![AcResult {
        id: "AC-01".into(),
        status: "pass".into(),
        evidence: "cmd:cargo test verified again".into(),
    }];
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    assert!(matches!(
        on_stage_end(&env.state, &env.ws_id, &run_id),
        StageRouting::Advance
    ));
    let mut c = claim("review", "needs_fix", 1);
    c.findings = vec![Finding {
        id: "F-1".into(),
        task: None,
        ac: None,
        description: "还是不完整".into(),
    }];
    c.evidence = vec![format!("review notes round1 ({rnd})")]; // 同证据
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    match on_stage_end(&env.state, &env.ws_id, &run_id) {
        StageRouting::Fail(e) => assert!(e.contains("no_progress"), "{e}"),
        other => panic!("identical round must no_progress, got {other:?}"),
    }
    std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
}

/// 修复轮上限：needs_fix 三轮后明确停止（AC-05）。
#[test]
#[serial]
fn repair_limit_exhaustion_stops() {
    let env = route_env("limit");
    let ws = env.state.registry.get(&env.ws_id);
    let run_id = start_bound_run(&env, "limit");
    let wt = reach_execute(&env, &run_id);
    for round in 1..=4usize {
        std::fs::write(wt.join(format!("f{round}-{}.txt", uuid_tag())), "x\n").unwrap();
        git_args(&wt, &["add", "."]);
        git_args(&wt, &["commit", "-m", &format!("round {round}")]);
        let head = git_args(&wt, &["rev-parse", "HEAD"]);
        let mut c = claim("execute", "pass", 1);
        c.commit = Some(head);
        c.acceptance_results = vec![AcResult {
            id: "AC-01".into(),
            status: "pass".into(),
            evidence: format!("cmd:verify round {round}"),
        }];
        c.evidence = vec![format!("cmd:ev{round}")];
        record_stage_claim(&ws.relay, &run_id, c).unwrap();
        assert!(matches!(
            on_stage_end(&env.state, &env.ws_id, &run_id),
            StageRouting::Advance
        ));
        let mut c = claim("review", "needs_fix", 1);
        c.findings = vec![Finding {
            id: format!("F-{round}"),
            task: None,
            ac: None,
            description: "n".into(),
        }];
        c.evidence = vec![format!("notes{round}")];
        record_stage_claim(&ws.relay, &run_id, c).unwrap();
        let r = on_stage_end(&env.state, &env.ws_id, &run_id);
        if round < 4 {
            assert!(
                matches!(r, StageRouting::RewoundToExecute),
                "round {round}: {r:?}"
            );
        } else {
            match r {
                StageRouting::Fail(e) => assert!(e.contains("repair limit"), "{e}"),
                other => panic!("round 4 must exhaust, got {other:?}"),
            }
        }
    }
    std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
}

/// 取消：旗标置位 → 收束即停，现场保留（AC-10）。
#[test]
#[serial]
fn cancel_stops_routing_and_keeps_scene() {
    let env = route_env("cancel");
    let ws = env.state.registry.get(&env.ws_id);
    let run_id = start_bound_run(&env, "cancel");
    ws.relay.advance(&run_id).unwrap();
    musk::relay::plan_control::cancel_register(&run_id);
    assert!(musk::relay::plan_control::cancel_set(&run_id));
    record_stage_claim(&ws.relay, &run_id, claim("plan", "pass", 1)).unwrap();
    match on_stage_end(&env.state, &env.ws_id, &run_id) {
        StageRouting::Fail(e) => assert!(e.contains("cancelled"), "{e}"),
        other => panic!("cancel must stop, got {other:?}"),
    }
    assert!(ws.relay.plan_execution(&run_id).is_some(), "scene kept");
    musk::relay::plan_control::cancel_remove(&run_id);
    std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
}

/// needs_replan：明确停止、计划留 executing、不 document（AC-06）。
#[test]
#[serial]
fn needs_replan_stops_without_document() {
    let env = route_env("replan");
    let ws = env.state.registry.get(&env.ws_id);
    let run_id = start_bound_run(&env, "replan");
    ws.relay.advance(&run_id).unwrap();
    record_stage_claim(&ws.relay, &run_id, claim("plan", "pass", 1)).unwrap();
    assert!(matches!(
        on_stage_end(&env.state, &env.ws_id, &run_id),
        StageRouting::Advance
    ));
    ws.relay
        .submit_handoff(&run_id, {
            let mut h = auto_ai_agent::orchestration::HandoffDocument::new("advisor", "coder");
            h.summary = "x".into();
            h
        })
        .unwrap();
    attach_binding_on_gate_approve(&env.state, &env.ws_id, &run_id).unwrap();
    ws.relay
        .resolve_gate(&run_id, musk::relay::GateDecision::Approve)
        .unwrap();
    let mut c = claim("execute", "needs_replan", 1);
    c.findings = vec![Finding {
        id: "B-1".into(),
        task: None,
        ac: None,
        description: "需求缺关键约束".into(),
    }];
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    match on_stage_end(&env.state, &env.ws_id, &run_id) {
        StageRouting::Fail(e) => {
            assert!(e.contains("needs_replan") && e.contains("需求缺关键约束"), "{e}")
        }
        other => panic!("needs_replan must stop, got {other:?}"),
    }
    let pe = ws.relay.plan_execution(&run_id).unwrap();
    assert_eq!(pe.blocker.as_deref(), Some("需求缺关键约束"));
    let plans = musk::plans::PlansStore::new(ws.root.join("docs/plans"));
    assert_eq!(plans.get(1).unwrap().status.as_str(), "executing");
    std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
}

/// 语义漂移：批准后仅进度变化不失效；语义变化（目标改写）→ needs_replan 停。
#[test]
#[serial]
fn semantic_drift_rules() {
    let env = route_env("drift");
    let ws = env.state.registry.get(&env.ws_id);
    let run_id = start_bound_run(&env, "drift");
    let wt = reach_execute(&env, &run_id);
    let plan_path = ws.root.join("docs/plans/001-route-demo.md");

    // 仅进度变化：勾选+证据 → 批准保持（AC-06 正向）。
    let content = std::fs::read_to_string(&plan_path).unwrap();
    std::fs::write(
        &plan_path,
        content.replace("- [ ] T-01 实现演示", "- [x] T-01 实现演示 [✅ 已完成]\n  证据：进行中"),
    )
    .unwrap();

    std::fs::write(wt.join("impl.txt"), "x\n").unwrap();
    git_args(&wt, &["add", "."]);
    git_args(&wt, &["commit", "-m", "impl"]);
    let head = git_args(&wt, &["rev-parse", "HEAD"]);
    let mut c = claim("execute", "pass", 1);
    c.commit = Some(head.clone());
    c.acceptance_results = vec![AcResult {
        id: "AC-01".into(),
        status: "pass".into(),
        evidence: "cmd:check".into(),
    }];
    c.evidence = vec!["cmd:ev".into()];
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    assert!(matches!(
        on_stage_end(&env.state, &env.ws_id, &run_id),
        StageRouting::Advance
    ));

    // 语义变化：目标文字改写 → 旧批准失效（review pass 被拒 → needs_replan）。
    let content = std::fs::read_to_string(&plan_path).unwrap();
    std::fs::write(
        &plan_path,
        content.replace("交付演示功能", "交付演示功能（范围重定义）"),
    )
    .unwrap();
    let mut c = claim("review", "pass", 1);
    c.commit = Some(head);
    c.acceptance_results = vec![AcResult {
        id: "AC-01".into(),
        status: "pass".into(),
        evidence: "cmd:check".into(),
    }];
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    match on_stage_end(&env.state, &env.ws_id, &run_id) {
        StageRouting::Fail(e) => assert!(
            e.contains("semantic contract drift") && e.contains("needs_replan"),
            "{e}"
        ),
        other => panic!("semantic drift must stop, got {other:?}"),
    }
    std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
}

/// T-06/AC-09：截断响应不准跨阶段成功——一次有界续做（同相位重入，与
/// 修复轮分开计数），再截断 → 带工件位置的响亮失败。
#[tokio::test]
#[serial]
async fn truncation_gets_one_bounded_continuation_then_fails_loud() {
    use auto_ai_agent::Client as _C;
    use auto_ai_client::{ClientError as _CE, CompletionRequest as _CR, CompletionResponse as _CRes};
    use musk::relay::plan_runtime_client::{clear, truncated_for};

    struct TruncClient;
    #[async_trait::async_trait]
    impl _C for TruncClient {
        async fn complete(&self, _req: &_CR) -> Result<_CRes, _CE> {
            Err(_CE::DaemonUnavailable)
        }
    }

    let env = route_env("trunc");
    let ws = env.state.registry.get(&env.ws_id);
    let run_id = start_bound_run(&env, "trunc");
    ws.relay.advance(&run_id).unwrap();

    // 模拟一次截断响应（真实链路由 PlanRuntimeClient 记录；此处经同一
    // 注册表 API 登记，与包装器落点一致）。
    let wrapper = musk::relay::plan_runtime_client::PlanRuntimeClient::new(
        Arc::new(TruncClient) as Arc<dyn _C>,
        &run_id,
        1024,
    );
    // wrapper 只在成功响应后登记 stop——直接走单元级注册面：用内部可写
    // 的 records 通道不可行（私有），改为经 MockInner 成功路径登记。
    struct TruncOkClient;
    #[async_trait::async_trait]
    impl _C for TruncOkClient {
        async fn complete(&self, _req: &_CR) -> Result<_CRes, _CE> {
            Ok(_CRes {
                content: "partial…".into(),
                tool_calls: Vec::new(),
                stop_reason: Some("max_tokens".into()),
                usage: None,
                model: "mock".into(),
                error: None,
                model_meta: None,
            })
        }
    }
    drop(wrapper);
    let wrapper = musk::relay::plan_runtime_client::PlanRuntimeClient::new(
        Arc::new(TruncOkClient) as Arc<dyn _C>,
        &run_id,
        1024,
    );
    let _ = wrapper.complete(&_CR::single("m", "x")).await;
    assert!(truncated_for(&run_id));

    // 阶段收束：截断 → 一次有界续做（同相位重入；声明作废）。
    record_stage_claim(&ws.relay, &run_id, claim("plan", "pass", 1)).unwrap();
    assert!(matches!(
        on_stage_end(&env.state, &env.ws_id, &run_id),
        StageRouting::RewoundToExecute
    ));
    let pe = ws.relay.plan_execution(&run_id).unwrap();
    assert_eq!(pe.continuations.get("plan"), Some(&1));
    assert_eq!(pe.repair_count, 0, "continuation is NOT a repair round");
    assert!(
        !pe.stage_results.iter().any(|r| r.stage == "plan"),
        "stale claim invalidated for the continuation"
    );
    // 续做内再截断 → 响亮失败（带工件位置）。
    let w2 = musk::relay::plan_runtime_client::PlanRuntimeClient::new(
        Arc::new(TruncOkClient) as Arc<dyn _C>,
        &run_id,
        1024,
    );
    let _ = w2.complete(&_CR::single("m", "x")).await;
    record_stage_claim(&ws.relay, &run_id, claim("plan", "pass", 1)).unwrap();
    match on_stage_end(&env.state, &env.ws_id, &run_id) {
        StageRouting::Fail(e) => {
            assert!(e.contains("continuation budget exhausted"), "{e}");
            assert!(e.contains("docs/plans/"), "error carries artifact location: {e}");
        }
        other => panic!("second truncation must fail loud, got {other:?}"),
    }
    clear(&run_id);
    std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
}

/// T-07/AC-07：伪证据路径拒绝——evidence 指向不存在的工件 → execute pass
/// 被拒；cmd: 前缀 = 命令记录（不执行、不解析路径）。
#[test]
#[serial]
fn fake_evidence_paths_are_rejected() {
    let env = route_env("fakeev");
    let ws = env.state.registry.get(&env.ws_id);
    let run_id = start_bound_run(&env, "fakeev");
    let wt = reach_execute(&env, &run_id);
    std::fs::write(wt.join("impl.txt"), "x\n").unwrap();
    git_args(&wt, &["add", "."]);
    git_args(&wt, &["commit", "-m", "impl"]);
    let head = git_args(&wt, &["rev-parse", "HEAD"]);

    // 伪路径证据 → 拒（报文点名工件）。
    let mut c = claim("execute", "pass", 1);
    c.commit = Some(head.clone());
    c.acceptance_results = vec![AcResult {
        id: "AC-01".into(),
        status: "pass".into(),
        evidence: "cmd:check".into(),
    }];
    c.evidence = vec!["evidence/nope-自诩报告.md".into()];
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    match on_stage_end(&env.state, &env.ws_id, &run_id) {
        StageRouting::Fail(e) => {
            assert!(e.contains("does not exist") && e.contains("nope-"), "{e}")
        }
        other => panic!("fake evidence path must be rejected, got {other:?}"),
    }

    // 真实工件路径（worktree 相对）→ 过。
    std::fs::write(wt.join("evidence.txt"), "real proof\n").unwrap();
    git_args(&wt, &["add", "."]);
    git_args(&wt, &["commit", "-m", "evidence"]);
    let head2 = git_args(&wt, &["rev-parse", "HEAD"]);
    let mut c = claim("execute", "pass", 1);
    c.commit = Some(head2);
    c.acceptance_results = vec![AcResult {
        id: "AC-01".into(),
        status: "pass".into(),
        evidence: "cmd:check".into(),
    }];
    c.evidence = vec!["evidence.txt".into(), "cmd:git log --oneline -3".into()];
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    assert!(matches!(
        on_stage_end(&env.state, &env.ws_id, &run_id),
        StageRouting::Advance
    ));
    std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
}

/// T-07/AC-07：依赖漂移——批准后 MUSK_PLAN_DEP_DIRS 指向的仓 tip 前移
/// → review pass 被拒（needs_replan，重新复审）。
#[test]
#[serial]
fn dependency_drift_refuses_review_pass() {
    // 依赖仓：独立临时 git 仓。
    let dep_td = tempfile::tempdir().unwrap();
    let dep = dep_td.path().join("dep");
    std::fs::create_dir_all(&dep).unwrap();
    git_args(&dep, &["init", "-b", "master"]);
    git_args(&dep, &["config", "user.email", "t@t"]);
    git_args(&dep, &["config", "user.name", "t"]);
    std::fs::write(dep.join("lib.txt"), "v1\n").unwrap();
    git_args(&dep, &["add", "."]);
    git_args(&dep, &["commit", "-m", "v1"]);
    let tip1 = git_args(&dep, &["rev-parse", "HEAD"]);
    std::env::set_var(
        "MUSK_PLAN_DEP_DIRS",
        format!("dep={}", dep.display()),
    );

    let env = route_env("depdrift");
    let ws = env.state.registry.get(&env.ws_id);
    let run_id = start_bound_run(&env, "depdrift");
    let wt = reach_execute(&env, &run_id);
    // 绑定已冻结 dep@tip1。
    let binding = ws.relay.plan_execution(&run_id).unwrap().binding.unwrap();
    assert_eq!(binding.dependency_revisions.get("dep").map(String::as_str), Some(tip1.as_str()));

    std::fs::write(wt.join("impl.txt"), "x\n").unwrap();
    git_args(&wt, &["add", "."]);
    git_args(&wt, &["commit", "-m", "impl"]);
    let head = git_args(&wt, &["rev-parse", "HEAD"]);
    let mut c = claim("execute", "pass", 1);
    c.commit = Some(head.clone());
    c.acceptance_results = vec![AcResult {
        id: "AC-01".into(),
        status: "pass".into(),
        evidence: "cmd:check".into(),
    }];
    c.evidence = vec!["cmd:ev".into()];
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    assert!(matches!(
        on_stage_end(&env.state, &env.ws_id, &run_id),
        StageRouting::Advance
    ));

    // 依赖仓前移 → review pass 被拒。
    std::fs::write(dep.join("lib.txt"), "v2\n").unwrap();
    git_args(&dep, &["add", "."]);
    git_args(&dep, &["commit", "-m", "v2"]);
    let mut c = claim("review", "pass", 1);
    c.commit = Some(head);
    c.acceptance_results = vec![AcResult {
        id: "AC-01".into(),
        status: "pass".into(),
        evidence: "cmd:check".into(),
    }];
    c.evidence = vec!["cmd:ev".into()];
    record_stage_claim(&ws.relay, &run_id, c).unwrap();
    match on_stage_end(&env.state, &env.ws_id, &run_id) {
        StageRouting::Fail(e) => {
            assert!(e.contains("dependency drift") && e.contains("re-review"), "{e}")
        }
        other => panic!("dep drift must refuse deposition, got {other:?}"),
    }
    std::env::remove_var("MUSK_PLAN_WORKTREE_ROOT");
    std::env::remove_var("MUSK_PLAN_DEP_DIRS");
}

/// T-07（§5.6）：review 相位任务携带批准绑定事实（机械输入）——reviewer
/// 不靠 coder 自述，凭据核验面向绑定（plan_revision/base/worktree）。
#[test]
fn review_template_carries_binding_facts() {
    use std::collections::HashMap;
    let mut ctx = HashMap::new();
    ctx.insert("plan_file".to_string(), "docs/plans/001-x.md".to_string());
    ctx.insert(
        "binding_facts".to_string(),
        "plan_revision=1 contract_hash=abcdef123456 semantic_hash=123456abcdef base_commit=deadbeef1234 default_branch=master reviewed_commit=(none yet) worktree=/tmp/wt dep_revisions=(none frozen)".to_string(),
    );
    let mut snap = SkillSnapshot::default();
    snap.skills.insert(
        "auto-plan-review".into(),
        SkillEntry {
            name: "auto-plan-review".into(),
            path: "/p".into(),
            sha256: "h".into(),
            content: "# review discipline".into(),
        },
    );
    let t = phase_task("plan", "review", "需求", &ctx, &snap).unwrap();
    assert!(t.contains("批准绑定事实"), "binding facts block injected");
    assert!(t.contains("plan_revision=1"));
    assert!(t.contains("base_commit=deadbeef1234"));
    assert!(t.contains("worktree=/tmp/wt"));
    // 无绑定时该块缺省（旧 run 兼容）。
    let mut ctx2 = HashMap::new();
    ctx2.insert("plan_file".to_string(), "docs/plans/001-x.md".to_string());
    let t2 = phase_task("plan", "review", "需求", &ctx2, &snap).unwrap();
    assert!(!t2.contains("批准绑定事实"));
}
