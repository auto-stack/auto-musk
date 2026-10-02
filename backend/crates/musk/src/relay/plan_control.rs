//! PLAN-096 T-03/T-05: PlanControl — the musk-side controller for the
//! product-internal plan flow (§5.4 single-writer stage routing).
//!
//! The controller owns:
//! - **Bootstrap** ([`bootstrap_plan_run`]): a plan/plan-merge run starts only
//!   with a frozen four-skill snapshot (missing skills = hard failure) and —
//!   for plan-merge — a resolvable target plan (id parsed from the task,
//!   contract fully readable). The resulting [`PlanExecutionState`] is the
//!   run's plan facts record (AC-13) and feeds the phase templates.
//! - **Single owner** ([`owner()`] guard key per run): stage routing runs
//!   serially; a second concurrent advance for the same run is refused
//!   (AC-10). Abnormal exits release the guard.
//! - **Stage routing** (T-05): validated [`StageResult`]s route the engine —
//!   pass advances, review/needs_fix rewinds to execute (≤ repair_limit, the
//!   work-skill bound of three), needs_replan/blocked stop loudly, and a
//!   Done without a valid result is `stage_incomplete` (AC-04).
//!
//! The engine cursor is moved through the musk-side controlled interface
//! [`engine_rewind_to_step`] (Q-01): local state mutation, single caller
//! (PlanControl), no auto-ai engine change.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::relay::plan_contract::{
    PlanContract, PlanExecutionState, SkillSnapshot, PLAN_SKILLS,
};
use crate::relay::store::RunStore;

use crate::plan_worktree::validate_main_checkout;

/// Per-run owner guard registry: PlanControl routes one stage transition at a
/// time per run. Key = run_id. Value = whether the run currently holds the
/// plan owner (stage in flight / delivery action in flight).
static PLAN_OWNERS: std::sync::Mutex<BTreeMap<String, ()>> = std::sync::Mutex::new(BTreeMap::new());

/// Try to claim the plan owner for `run_id`. False = an owner already holds
/// it (concurrent advance/delivery refused, AC-10).
pub fn owner_try_claim(run_id: &str) -> bool {
    let mut owners = PLAN_OWNERS.lock().unwrap();
    if owners.contains_key(run_id) {
        return false;
    }
    owners.insert(run_id.to_string(), ());
    true
}

/// Release the plan owner (every exit path of a routed stage must call this).
pub fn owner_release(run_id: &str) {
    PLAN_OWNERS.lock().unwrap().remove(run_id);
}

/// True when some actor currently owns the run's plan stage.
pub fn owner_held(run_id: &str) -> bool {
    PLAN_OWNERS.lock().unwrap().contains_key(run_id)
}

/// The plan-flow run's cancel flag (T-05 wiring registers per run; set by the
/// cancel endpoint to stop the controller before further side effects and
/// keep the scene intact — AC-10).
static PLAN_CANCELS: std::sync::Mutex<BTreeMap<String, Arc<AtomicBool>>> =
    std::sync::Mutex::new(BTreeMap::new());

pub fn cancel_register(run_id: &str) -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    PLAN_CANCELS
        .lock()
        .unwrap()
        .insert(run_id.to_string(), flag.clone());
    flag
}

pub fn cancel_flag(run_id: &str) -> Option<Arc<AtomicBool>> {
    PLAN_CANCELS.lock().unwrap().get(run_id).cloned()
}

pub fn cancel_set(run_id: &str) -> bool {
    match cancel_flag(run_id) {
        Some(f) => {
            f.store(true, Ordering::SeqCst);
            true
        }
        None => false,
    }
}

pub fn cancel_remove(run_id: &str) {
    PLAN_CANCELS.lock().unwrap().remove(run_id);
}

/// Parse `PLAN-NNN` out of free text (plan-merge entry / handoff task).
pub fn parse_plan_id(text: &str) -> Option<(u32, String)> {
    let re = regex::Regex::new(r"PLAN-([0-9]{1,3})").ok()?;
    let caps = re.captures(text)?;
    let seq: u32 = caps[1].parse().ok()?;
    Some((seq, format!("PLAN-{seq:03}")))
}

/// Q-01 controlled fallback: rewind the local engine cursor to `step_id`
/// (needs_fix → execute). Single authorized caller (PlanControl routing);
/// NOT a generic API. Truncates step_history at the target step (the removed
/// records live on in the PlanExecutionState attempt log / run events, which
/// are the §9-grade evidence trail) and clears gate bookkeeping so `advance`
/// re-runs the target step. Returns Err for unknown step ids.
pub fn engine_rewind_to_step(
    engine: &mut crate::relay::PipelineEngine,
    step_id: &str,
) -> Result<(), String> {
    let idx = engine
        .flow
        .steps
        .iter()
        .position(|s| s.id == step_id)
        .ok_or_else(|| format!("rewind target step '{step_id}' not in flow '{}'", engine.flow.id))?;
    engine.current_step = idx;
    engine.step_history.truncate(idx);
    engine.pending_gate = None;
    // 修复轮回退不重开已过的人审门（AC-05：无手工 nudge；AC-06：原契约内
    // 修复保留授权）——回退目标是 gated 步时保留/补写 resolved 标记。
    if matches!(engine.flow.steps[idx].gate, crate::relay::GateType::Human) {
        engine.gate_resolved_for_step = Some(step_id.to_string());
    } else {
        engine.gate_resolved_for_step = None;
    }
    engine.gate_feedback.remove(step_id);
    engine.status = crate::relay::PipelineStatus::Idle;
    engine.resumed_step_id = None;
    Ok(())
}

/// Validate a plan-merge target: contract readable under `plans_dir`, id
/// matches, and — when `require_reviewed` — status must be `reviewed`
/// (delivery never re-runs execution/review, §5.7).
pub fn read_managed_contract(
    plans_dir: &std::path::Path,
    rel_path: &str,
    expect_plan_id: &str,
) -> Result<PlanContract, String> {
    let path = plans_dir.join(rel_path.trim_start_matches("docs/plans/"));
    PlanContract::read(&path, Some(expect_plan_id))
}

/// Bootstrap facts for a new plan-flow run.
pub struct Bootstrap {
    pub state: PlanExecutionState,
}

/// Bootstrap a `plan` or `plan-merge` run: freeze the four skills (hard
/// failure when any is missing/unreadable — the flow never runs on stale
/// inline templates), and for plan-merge resolve + read the target contract
/// up front. Returns the initial PlanExecutionState for the caller to attach
/// via [`RunStore::set_plan_execution`].
pub fn bootstrap_plan_run(
    flow_id: &str,
    task: &str,
    plans_dir: &std::path::Path,
    skills_source: &std::path::Path,
) -> Result<Bootstrap, String> {
    if flow_id != "plan" && flow_id != "plan-merge" {
        return Err(format!("flow '{flow_id}' is not a plan flow"));
    }
    let snap: SkillSnapshot =
        crate::relay::plan_contract::snapshot_plan_skills(skills_source)?;
    let mut state = match flow_id {
        "plan-merge" => {
            let (seq, plan_id) = parse_plan_id(task)
                .ok_or_else(|| "plan-merge task carries no PLAN-NNN target".to_string())?;
            // 目标计划必须能按 id 读出完整合同（裸 plan 号/外仓文件/伪路径拒绝）。
            let listed = std::fs::read_dir(plans_dir.join("archived"))
                .map(|rd| {
                    rd.filter_map(|e| e.ok())
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .find(|n| n.starts_with(&format!("{seq:03}-")))
                })
                .unwrap_or(None);
            let plan_path = match listed {
                Some(f) => format!("docs/plans/archived/{f}"),
                None => {
                    let active = std::fs::read_dir(plans_dir)
                        .map(|rd| {
                            rd.filter_map(|e| e.ok())
                                .map(|e| e.file_name().to_string_lossy().to_string())
                                .find(|n| n.starts_with(&format!("{seq:03}-")) && n.ends_with(".md"))
                        })
                        .unwrap_or(None);
                    match active {
                        Some(f) => format!("docs/plans/{f}"),
                        None => {
                            return Err(format!(
                                "plan {plan_id} not found under {} (active or archived)",
                                plans_dir.display()
                            ))
                        }
                    }
                }
            };
            let contract = read_managed_contract(plans_dir, &plan_path, &plan_id)?;
            let mut st = PlanExecutionState::new(&plan_id, seq, &plan_path, contract.plan_revision);
            st.phase = "document".into();
            st.skills = snap.skills;
            st
        }
        _ => {
            let mut st = PlanExecutionState::new("", 0, "", 0);
            st.skills = snap.skills;
            st
        }
    };
    // 冻结的技能集必须恰好四份（快照器保证；双保险断言在测试里钉住）。
    if state.skills.len() != PLAN_SKILLS.len() {
        return Err("skill snapshot incomplete".into());
    }
    Ok(Bootstrap { state })
}

/// Convenience: snapshot from the default skills resolution chain.
pub fn bootstrap_plan_run_default(
    flow_id: &str,
    task: &str,
    plans_dir: &std::path::Path,
) -> Result<Bootstrap, String> {
    let src = crate::relay::plan_contract::plan_skills_source_root()?;
    bootstrap_plan_run(flow_id, task, plans_dir, &src)
}

// ── T-05: stage-result validation & single-writer routing (§5.4) ───────────

/// What the driver must do after a plan-flow stage ends.
#[derive(Debug, Clone, PartialEq)]
pub enum StageRouting {
    /// Server facts validated — submit the handoff and let the engine route
    /// (plan → gate, execute → review, review/pass → document).
    Advance,
    /// review/needs_fix accepted within the repair bound — the engine cursor
    /// was rewound to `execute` and the attempt bumped; the driver continues
    /// its loop (new coder attempt).
    RewoundToExecute,
    /// document/pass with all delivery checkpoints settled — run completes.
    Complete,
    /// Terminal failure: stage_incomplete / validation refusal / repair limit
    /// / no-progress / needs_replan / blocked / cancellation. The error is
    /// the reason (recorded on the run + plan §10-grade facts).
    Fail(String),
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Push a controller facts event (run history + SSE bus + conversation
/// mirror — the standard RunStore path).
pub fn push_facts(store: &RunStore, run_id: &str, facts: crate::relay::plan_contract::RunPlanEvent) {
    store.push_event(
        run_id,
        crate::relay::store::RunEvent::PlanStageFacts {
            timestamp: now_secs(),
            facts,
        },
    );
}

fn facts_of(
    pe: &crate::relay::plan_contract::PlanExecutionState,
    stage: &str,
    outcome: &str,
    blocker: Option<String>,
    delivery_checkpoint: Option<String>,
) -> crate::relay::plan_contract::RunPlanEvent {
    crate::relay::plan_contract::RunPlanEvent {
        timestamp: now_secs(),
        plan_id: pe.plan_id.clone(),
        stage: stage.to_string(),
        attempt: pe.attempt,
        outcome: outcome.to_string(),
        repair_count: pe.repair_count,
        repair_limit: pe.repair_limit,
        blocker,
        reviewed_commit: pe.reviewed_commit.clone(),
        delivery_checkpoint,
        receipt_ref: pe.receipt_ref.clone(),
    }
}

/// Record the model's raw stage claim (from the `complete_plan_stage` tool).
/// Validates shape (run is a bound plan run, stage == current phase, ids and
/// revision match) and appends to the attempt log. Server-side fact
/// verification happens later in [`on_stage_end`].
pub fn record_stage_claim(
    store: &RunStore,
    run_id: &str,
    result: crate::relay::plan_contract::StageResult,
) -> Result<(), String> {
    let pe = store
        .plan_execution(run_id)
        .ok_or_else(|| "run carries no plan execution".to_string())?;
    let phase = pe.phase.clone();
    let attempt = pe.attempt;
    if result.stage != phase {
        return Err(format!(
            "stage '{}' does not match the run's current phase '{}'",
            result.stage, phase
        ));
    }
    // plan 相位（bootstrap 时身份未知，plan_id 为空）放行任意声明——身份
    // 由 plan 路由/门批准从真实文件回填；其余相位必须与绑定一致。
    if !pe.plan_id.is_empty() && result.plan_id != pe.plan_id {
        return Err(format!(
            "plan_id '{}' does not match the bound plan '{}'",
            result.plan_id, pe.plan_id
        ));
    }
    if pe.plan_revision != 0 && result.plan_revision != pe.plan_revision {
        return Err(format!(
            "plan_revision {} does not match the bound revision {}",
            result.plan_revision, pe.plan_revision
        ));
    }
    if result.outcome.is_empty() {
        return Err("outcome is required (pass|needs_fix|needs_replan|blocked)".into());
    }
    let mut result = result;
    result.attempt = attempt;
    store.mutate_plan_execution(run_id, |pe| {
        pe.stage_results.push(result);
    });
    Ok(())
}

/// The latest recorded claim for the run's current (stage, attempt).
fn latest_claim(
    pe: &crate::relay::plan_contract::PlanExecutionState,
    stage: &str,
) -> Option<crate::relay::plan_contract::StageResult> {
    pe.stage_results
        .iter()
        .rev()
        .find(|r| r.stage == stage && r.attempt == pe.attempt)
        .cloned()
}

/// Gate-approve binding (§5.2): freeze the approved contract (exact +
/// semantic hashes), git facts (detected default branch + base commit), the
/// skill hashes, and the run-scoped authorization into the binding, then
/// create / reuse the dev worktree. Called from the execute-gate approve
/// paths (hw + ag) BEFORE the gate decision is applied; a failure refuses
/// the approval (auto → run fails loud; human → 409, gate not consumed).
pub fn attach_binding_on_gate_approve(
    state: &crate::server::AppState,
    ws_id: &str,
    run_id: &str,
) -> Result<(), String> {
    use crate::relay::plan_contract::PlanExecutionBinding;
    let ws = state.registry.get(ws_id);
    let plan_file = ws
        .relay
        .context_var(run_id, "plan_file")
        .filter(|p| !p.trim().is_empty())
        .ok_or_else(|| "gate approve without a plan file — approval refused".to_string())?;
    let authorization = ws
        .relay
        .context_var(run_id, "plan_authorization")
        .unwrap_or_else(|| "human".into());
    // 完整合同读取（绑定主根 docs/plans/ 下的真实文件）。
    let plan_path = ws.root.join(&plan_file);
    let contract = crate::relay::plan_contract::PlanContract::read(&plan_path, None)?;
    contract.validate_complete()?;
    let seq = contract
        .plan_id
        .trim_start_matches("PLAN-")
        .parse::<u32>()
        .map_err(|_| format!("plan id '{}' carries no sequence number", contract.plan_id))?;
    let main_root = validate_main_checkout(&ws.root)?;
    let default_branch = crate::plan_worktree::default_branch_of(&main_root)?;
    let base_commit = {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&main_root)
            .output()
            .map_err(|e| format!("git spawn failed: {e}"))?;
        if !out.status.success() {
            return Err("base commit resolve failed (not a git checkout?)".into());
        }
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    let lease = crate::plan_worktree::ensure_plan_worktree(
        &main_root,
        &contract.plan_id,
        seq,
        &base_commit,
    )?;
    // 批准即开工：计划状态机 drafting → executing（execute 相位的起点）。
    {
        let plans = crate::plans::PlansStore::new(main_root.join("docs/plans"));
        plans
            .transition(seq, crate::plans::PlanStatus::Executing)
            .map_err(|e| format!("plan transition to executing failed: {e}"))?;
    }
    let mut pe = ws
        .relay
        .plan_execution(run_id)
        .ok_or_else(|| "run carries no plan execution".to_string())?;
    // plan 相位完成即回填身份（bootstrap 阶段 plan 流还没有 plan_id）。
    pe.plan_id = contract.plan_id.clone();
    pe.plan_seq = seq;
    pe.plan_path = plan_file.clone();
    pe.plan_revision = contract.plan_revision;
    pe.binding = Some(PlanExecutionBinding {
        contract_version: crate::relay::plan_contract::PLAN_EXECUTION_CONTRACT_VERSION,
        workspace_id: ws_id.to_string(),
        main_root: main_root.display().to_string(),
        plan_id: contract.plan_id.clone(),
        plan_path: plan_file.clone(),
        plan_revision: contract.plan_revision,
        contract_hash: contract.contract_hash.clone(),
        semantic_hash: contract.semantic_hash.clone(),
        skills_hashes: {
            let mut m = std::collections::BTreeMap::new();
            for (k, v) in &pe.skills {
                m.insert(k.clone(), v.sha256.clone());
            }
            m
        },
        default_branch,
        base_commit,
        execution_root: Some(lease.worktree_root.clone()),
        dev_branch: Some(lease.branch.clone()),
        authorization,
        repair_limit: pe.repair_limit,
        dependency_revisions: freeze_dependency_revisions(),
    });
    ws.relay.set_plan_execution(run_id, pe.clone());
    push_facts(&ws.relay, run_id, facts_of(&pe, "approve", "bound", None, None));
    Ok(())
}

/// Stage end: validate server-side facts for the latest claim and route
/// (§5.4 table). The driver calls this INSTEAD of a blind `submit_handoff`
/// for plan-flow steps. Single-writer: one routing at a time per run (AC-10).
pub fn on_stage_end(state: &crate::server::AppState, ws_id: &str, run_id: &str) -> StageRouting {
    if !owner_try_claim(run_id) {
        return StageRouting::Fail(format!(
            "run {run_id} plan stage routing already in progress (single owner)"
        ));
    }
    let routing = on_stage_end_inner(state, ws_id, run_id);
    owner_release(run_id);
    routing
}

fn on_stage_end_inner(
    state: &crate::server::AppState,
    ws_id: &str,
    run_id: &str,
) -> StageRouting {
    let ws = state.registry.get(ws_id);
    // 取消旗标：停止推进，保留现场（AC-10）。
    if let Some(flag) = cancel_flag(run_id) {
        if flag.load(std::sync::atomic::Ordering::SeqCst) {
            if let Some(pe) = ws.relay.plan_execution(run_id) {
                let f = facts_of(
                    &pe,
                    &pe.phase,
                    "cancelled",
                    Some("run cancelled — plan left active, scene kept".into()),
                    None,
                );
                push_facts(&ws.relay, run_id, f);
            }
            return StageRouting::Fail("run cancelled — plan left active (scene kept)".into());
        }
    }
    let Some(pe) = ws.relay.plan_execution(run_id) else {
        return StageRouting::Fail("stage end on a run without plan execution".into());
    };
    let stage = pe.phase.clone();
    // 1.5 输出截断（T-06/AC-09）：截断形态响应登记为不完整——不准以截断
    // 跨阶段成功；每个阶段允许一次有界续做（读取已落盘工件、不重复
    // create_plan；与修复轮分开计数），仍截断/无结果 → 带工件位置响亮失败。
    if crate::relay::plan_runtime_client::truncated_for(run_id) {
        let used = pe.continuations.get(&pe.phase).copied().unwrap_or(0);
        if used == 0 {
            ws.relay.mutate_plan_execution(run_id, |pe| {
                *pe.continuations.entry(pe.phase.clone()).or_insert(0) += 1;
                // 旧 attempt 的同相位声明作废（续做重新提交）。
                let (ph, at) = (pe.phase.clone(), pe.attempt);
                pe.stage_results
                    .retain(|r| !(r.stage == ph && r.attempt == at));
            });
            crate::relay::plan_runtime_client::clear(run_id);
            let pe2 = ws.relay.plan_execution(run_id).unwrap();
            let f = facts_of(&pe2, &pe2.phase, "continuation", None, None);
            push_facts(&ws.relay, run_id, f);
            // 游标未动（本相位未提交 handoff）——直接重入同相位。
            return StageRouting::RewoundToExecute;
        }
        let artifact = if pe.plan_path.is_empty() {
            ws.relay
                .context_var(run_id, "plan_file")
                .filter(|p| !p.trim().is_empty())
                .unwrap_or_else(|| "(plan file not yet materialized)".into())
        } else {
            pe.plan_path.clone()
        };
        return StageRouting::Fail(format!(
            "output truncated twice for phase '{stage}' — continuation budget exhausted;              on-disk artifacts at {artifact} for manual resume"
        ));
    }
    // 2. 必须有本 attempt 的有效结果（Done/handoff/绿勾不能替代，AC-04）。
    let Some(claim) = latest_claim(&pe, &stage) else {
        let f = facts_of(&pe, &stage, "stage_incomplete", None, None);
        push_facts(&ws.relay, run_id, f);
        return StageRouting::Fail(format!(
            "stage_incomplete: agent ended phase '{stage}' without a valid complete_plan_stage result"
        ));
    };
    // 2. needs_replan / blocked：停止、保留现场、计划留 active（不 document）。
    if matches!(claim.outcome.as_str(), "needs_replan" | "blocked") {
        let blocker = claim
            .findings
            .first()
            .map(|f| f.description.clone())
            .or_else(|| claim.evidence.first().cloned())
            .unwrap_or_else(|| format!("{} reported by {}", claim.outcome, stage));
        ws.relay.mutate_plan_execution(run_id, |pe| {
            pe.blocker = Some(blocker.clone());
        });
        let f = facts_of(&pe, &stage, &claim.outcome, Some(blocker.clone()), None);
        push_facts(&ws.relay, run_id, f);
        return StageRouting::Fail(format!("{}: {blocker}", claim.outcome));
    }
    match stage.as_str() {
        "plan" => route_plan_end(&ws, run_id, &pe, &claim),
        "execute" => route_execute_end(&ws, run_id, &pe, &claim),
        "review" => route_review_end(&ws, run_id, &pe, &claim),
        "document" => route_document_end(&ws, run_id, &pe, &claim),
        other => StageRouting::Fail(format!("unknown plan stage '{other}'")),
    }
}

type WsStores = Arc<crate::workspace::WorkspaceStores>;

fn route_plan_end(
    ws: &WsStores,
    run_id: &str,
    pe: &crate::relay::plan_contract::PlanExecutionState,
    claim: &crate::relay::plan_contract::StageResult,
) -> StageRouting {
    use crate::relay::plan_contract::PlanContract;
    if claim.outcome != "pass" {
        return StageRouting::Fail(format!(
            "plan phase outcome '{}' is not routable (want pass|needs_replan|blocked)",
            claim.outcome
        ));
    }
    // 服务器回读计划文件：存在、完整、身份一致（AC-02）。
    let plan_path = match pe.binding.as_ref() {
        Some(b) => std::path::Path::new(&b.main_root).join(&b.plan_path),
        None => match ws.relay.context_var(run_id, "plan_file") {
            Some(p) if !p.trim().is_empty() => ws.root.join(p.trim()),
            _ => {
                let f = facts_of(pe, "plan", "stage_incomplete", None, None);
                push_facts(&ws.relay, run_id, f);
                return StageRouting::Fail(
                    "plan phase pass without a materialized plan file (no binding, no plan_file)"
                        .into(),
                );
            }
        },
    };
    let contract = match PlanContract::read(&plan_path, None) {
        Ok(c) => c,
        Err(e) => return StageRouting::Fail(format!("plan read-back failed: {e}")),
    };
    if let Err(e) = contract.validate_complete() {
        return StageRouting::Fail(format!("plan contract incomplete: {e}"));
    }
    if let Some(b) = pe.binding.as_ref() {
        if b.plan_id != contract.plan_id {
            return StageRouting::Fail(format!(
                "plan identity drift: binding '{}' vs file '{}'",
                b.plan_id, contract.plan_id
            ));
        }
    }
    // 回填身份 + 推进相位。
    let filename = plan_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    ws.relay.mutate_plan_execution(run_id, |pe| {
        pe.plan_id = contract.plan_id.clone();
        pe.plan_seq = contract
            .plan_id
            .trim_start_matches("PLAN-")
            .parse()
            .unwrap_or(pe.plan_seq);
        pe.plan_path = format!("docs/plans/{filename}");
        pe.plan_revision = contract.plan_revision;
        pe.phase = "execute".into();
        pe.attempt = 1;
        pe.outcome = Some("pass".into());
    });
    let pe2 = ws.relay.plan_execution(run_id).unwrap();
    push_facts(&ws.relay, run_id, facts_of(&pe2, "plan", "pass", None, None));
    StageRouting::Advance
}

/// Approval-scope verification for execute/review stages: the plan file's
/// semantic hash must still match the approved binding (AC-06: in-contract
/// fixes keep authorization; semantic edits expire it → needs_replan).
fn verify_binding_intact(
    pe: &crate::relay::plan_contract::PlanExecutionState,
) -> Result<crate::relay::plan_contract::PlanContract, String> {
    use crate::relay::plan_contract::PlanContract;
    let binding = pe
        .binding
        .as_ref()
        .ok_or_else(|| "no approval binding — plan was never approved".to_string())?;
    let path = std::path::Path::new(&binding.main_root).join(&binding.plan_path);
    let contract = PlanContract::read(&path, Some(&binding.plan_id))?;
    if contract.semantic_hash != binding.semantic_hash {
        return Err(format!(
            "semantic contract drift since approval — re-approval required (old {})",
            &binding.semantic_hash[..8.min(binding.semantic_hash.len())]
        ));
    }
    Ok(contract)
}

/// Worktree facts: clean status + HEAD != base (real commits landed, AC-04).
fn verify_committed_work(
    binding: &crate::relay::plan_contract::PlanExecutionBinding,
) -> Result<String, String> {
    let wt = binding
        .execution_root
        .as_ref()
        .ok_or_else(|| "no execution worktree registered".to_string())?;
    let wt = std::path::Path::new(wt);
    if !wt.is_dir() {
        return Err(format!("execution worktree missing: {}", wt.display()));
    }
    let run = |args: &[&str]| -> Result<String, String> {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(wt)
            .output()
            .map_err(|e| format!("git spawn failed: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "git {} failed: {}",
                args.first().unwrap_or(&""),
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    };
    let status = run(&["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        return Err(format!(
            "worktree has uncommitted changes — commit before claiming pass:\n{}",
            status.trim()
        ));
    }
    let head = run(&["rev-parse", "HEAD"])?.trim().to_string();
    if head == binding.base_commit {
        return Err("no commits since base — nothing was implemented".into());
    }
    Ok(head)
}

/// PLAN-096 T-07（§5.6/AC-07）：证据完整性核验——事实/完整性面（不执行
/// 任何模型文本）。规则：`cmd:` 前缀 = 命令记录（逐字登记，语义正确性由
/// reviewer 真实测试负责）；其余条目必须能解析为存在的工件（worktree/
/// 主根相对或绝对路径）。伪路径/不存在 → 拒。
fn verify_evidence_artifacts(
    binding: &crate::relay::plan_contract::PlanExecutionBinding,
    claim: &crate::relay::plan_contract::StageResult,
) -> Result<(), String> {
    let wt = binding.execution_root.as_deref();
    let main = std::path::Path::new(&binding.main_root);
    for ev in &claim.evidence {
        if ev.starts_with("cmd:") {
            if ev.len() <= 4 {
                return Err("cmd: evidence entry carries no command record".into());
            }
            continue;
        }
        // 相对/绝对工件必须存在。
        let candidates = [
            std::path::PathBuf::from(ev),
            wt.map(|w| std::path::Path::new(w).join(ev)).unwrap_or_default(),
            main.join(ev),
        ];
        if !candidates.iter().any(|c| c.is_file() || c.is_dir()) {
            return Err(format!(
                "evidence artifact '{ev}' does not exist (worktree/main-root relative or                  absolute; 'cmd:' prefix records a command without a path)"
            ));
        }
    }
    Ok(())
}

/// PLAN-096 T-07（AC-07）：依赖冻结——绑定时记录 MUSK_PLAN_DEP_DIRS
/// （name=path;name=path）各仓 git tip；review/pass 与交付前复验漂移。
pub fn freeze_dependency_revisions() -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    let Ok(spec) = std::env::var("MUSK_PLAN_DEP_DIRS") else {
        return out;
    };
    for pair in spec.split(';') {
        let Some((name, path)) = pair.split_once('=') else {
            continue;
        };
        let p = std::path::Path::new(path);
        let out_ok = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(p)
            .output();
        if let Ok(o) = out_ok {
            if o.status.success() {
                out.insert(
                    name.trim().to_string(),
                    String::from_utf8_lossy(&o.stdout).trim().to_string(),
                );
            }
        }
    }
    out
}

/// 交付面（plan_delivery）复用同一依赖漂移核验（AC-07）。
pub fn verify_dependencies_for_delivery(
    binding: &crate::relay::plan_contract::PlanExecutionBinding,
) -> Result<(), String> {
    verify_dependencies_unchanged(binding)
}

fn verify_dependencies_unchanged(
    binding: &crate::relay::plan_contract::PlanExecutionBinding,
) -> Result<(), String> {
    for (name, frozen) in &binding.dependency_revisions {
        let Some((_, path)) = std::env::var("MUSK_PLAN_DEP_DIRS")
            .ok()
            .and_then(|spec| {
                spec.split(';')
                    .find_map(|pair| pair.split_once('=').map(|(n, p)| (n.to_string(), p.to_string())))
                    .filter(|(n, _)| n == name)
            })
        else {
            continue; // env 变了：无法复验的条目跳过（冻结值保留在绑定里）
        };
        let o = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&path)
            .output()
            .map_err(|e| format!("dep {name} probe failed: {e}"))?;
        if !o.status.success() {
            return Err(format!("dep {name} at {path} is no longer a working git checkout"));
        }
        let tip = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if tip != *frozen {
            return Err(format!(
                "dependency drift: {name} moved {}..{} since approval — deposition refused, re-review required",
                &frozen[..frozen.len().min(12)],
                &tip[..tip.len().min(12)]
            ));
        }
    }
    Ok(())
}

/// Every §7 AC must be covered by a pass record with evidence (AC-04).
fn verify_acceptance_covered(
    contract: &crate::relay::plan_contract::PlanContract,
    claim: &crate::relay::plan_contract::StageResult,
) -> Result<(), String> {
    for ac in &contract.acceptance {
        let rec = claim.acceptance_results.iter().find(|r| r.id == ac.id);
        match rec {
            None => return Err(format!("AC {} has no verification record", ac.id)),
            Some(r) => {
                if r.status != "pass" {
                    return Err(format!("AC {} is '{}', not pass", ac.id, r.status));
                }
                if r.evidence.trim().is_empty() {
                    return Err(format!("AC {} pass carries no evidence", ac.id));
                }
            }
        }
    }
    Ok(())
}

fn transition_plan_status(
    ws: &WsStores,
    pe: &crate::relay::plan_contract::PlanExecutionState,
    to: &str,
) -> Result<(), String> {
    use crate::plans::PlanStatus;
    let binding = pe
        .binding
        .as_ref()
        .ok_or_else(|| "no binding".to_string())?;
    let main_root = std::path::Path::new(&binding.main_root);
    let plans = crate::plans::PlansStore::new(main_root.join("docs/plans"));
    let status = match to {
        "executing" => PlanStatus::Executing,
        "execution_done" => PlanStatus::ExecutionDone,
        "reviewed" => PlanStatus::Reviewed,
        other => return Err(format!("unsupported plan status '{other}'")),
    };
    plans.transition(pe.plan_seq, status).map(|_| ())
}

fn route_execute_end(
    ws: &WsStores,
    run_id: &str,
    pe: &crate::relay::plan_contract::PlanExecutionState,
    claim: &crate::relay::plan_contract::StageResult,
) -> StageRouting {
    if claim.outcome != "pass" {
        return StageRouting::Fail(format!(
            "execute phase outcome '{}' is not routable (review reports needs_fix; execute stops via needs_replan|blocked)",
            claim.outcome
        ));
    }
    let binding = match pe.binding.as_ref() {
        Some(b) => b.clone(),
        None => return StageRouting::Fail("execute end without approval binding".into()),
    };
    // 语义漂移 → 停（AC-06）。
    let contract = match verify_binding_intact(pe) {
        Ok(c) => c,
        Err(e) => {
            ws.relay.mutate_plan_execution(run_id, |pe| pe.blocker = Some(e.clone()));
            let f = facts_of(pe, "execute", "needs_replan", Some(e.clone()), None);
            push_facts(&ws.relay, run_id, f);
            return StageRouting::Fail(format!("needs_replan: {e}"));
        }
    };
    // 提交事实（AC-04：未提交不能 execution_done）。
    let head = match verify_committed_work(&binding) {
        Ok(h) => h,
        Err(e) => return StageRouting::Fail(e),
    };
    // AC 覆盖。
    if let Err(e) = verify_acceptance_covered(&contract, claim) {
        return StageRouting::Fail(e);
    }
    // 证据完整性与依赖漂移（T-07/AC-07）。
    if let Err(e) = verify_evidence_artifacts(&binding, claim) {
        return StageRouting::Fail(e);
    }
    if let Err(e) = verify_dependencies_unchanged(&binding) {
        return StageRouting::Fail(format!("needs_replan: {e}"));
    }
    // 计划状态机：executing → execution_done。
    if let Err(e) = transition_plan_status(ws, pe, "execution_done") {
        return StageRouting::Fail(format!("plan transition failed: {e}"));
    }
    ws.relay.mutate_plan_execution(run_id, |pe| {
        // 轮次制：execute/review 共享同一轮次号（review/needs_fix 回退时
        // attempt+1）；跨轮比对（无进展判定）依赖该单调性。
        pe.phase = "review".into();
        pe.outcome = Some("pass".into());
    });
    let pe2 = ws.relay.plan_execution(run_id).unwrap();
    let mut f = facts_of(&pe2, "execute", "pass", None, None);
    f.reviewed_commit = Some(head);
    push_facts(&ws.relay, run_id, f);
    StageRouting::Advance
}

fn route_review_end(
    ws: &WsStores,
    run_id: &str,
    pe: &crate::relay::plan_contract::PlanExecutionState,
    claim: &crate::relay::plan_contract::StageResult,
) -> StageRouting {
    let pe = pe.clone();
    match claim.outcome.as_str() {
        "pass" => {
            let binding = match pe.binding.as_ref() {
                Some(b) => b.clone(),
                None => return StageRouting::Fail("review end without approval binding".into()),
            };
            let contract = match verify_binding_intact(&pe) {
                Ok(c) => c,
                Err(e) => return StageRouting::Fail(format!("needs_replan: {e}")),
            };
            let head = match verify_committed_work(&binding) {
                Ok(h) => h,
                Err(e) => return StageRouting::Fail(e),
            };
            if let Err(e) = verify_acceptance_covered(&contract, claim) {
                return StageRouting::Fail(format!("review pass rejected: {e}"));
            }
            if let Err(e) = verify_evidence_artifacts(&binding, claim) {
                return StageRouting::Fail(format!("review pass rejected: {e}"));
            }
            if let Err(e) = verify_dependencies_unchanged(&binding) {
                return StageRouting::Fail(format!("needs_replan: {e}"));
            }
            if let Err(e) = transition_plan_status(ws, &pe, "reviewed") {
                return StageRouting::Fail(format!("plan transition failed: {e}"));
            }
            ws.relay.mutate_plan_execution(run_id, |pe| {
                pe.phase = "document".into();
                pe.outcome = Some("pass".into());
                pe.reviewed_commit = Some(head.clone());
            });
            let pe2 = ws.relay.plan_execution(run_id).unwrap();
            let mut f = facts_of(&pe2, "review", "pass", None, None);
            f.reviewed_commit = Some(head);
            push_facts(&ws.relay, run_id, f);
            StageRouting::Advance
        }
        "needs_fix" => {
            // 有界修复（AC-05）：findings 必须带稳定 id；无进展早停。
            if claim.findings.is_empty() {
                return StageRouting::Fail(
                    "needs_fix without findings — reviewer must name stable finding/task/AC ids"
                        .into(),
                );
            }
            // no-progress 判定（§5.4）：同一 finding 的代码与验收证据均无
            // 变化 → 提前停。比对键 =（本轮 findings id 集, 本轮 execute
            // 提交, 本轮 evidence）vs 上一修复轮同键。
            let prev_review = pe
                .stage_results
                .iter()
                .rev()
                .find(|r| {
                    r.stage == "review"
                        && r.outcome == "needs_fix"
                        && r.attempt + 1 == pe.attempt
                })
                .cloned();
            if let Some(prev) = prev_review {
                let exec_of = |round: u32| {
                    pe.stage_results
                        .iter()
                        .rev()
                        .find(|r| r.stage == "execute" && r.attempt == round)
                        .and_then(|r| r.commit.clone())
                };
                let ids = |r: &crate::relay::plan_contract::StageResult| {
                    let mut v: Vec<String> = r.findings.iter().map(|f| f.id.clone()).collect();
                    v.sort();
                    v
                };
                if ids(claim) == ids(&prev)
                    && !claim.evidence.is_empty()
                    && claim.evidence == prev.evidence
                    && exec_of(pe.attempt).is_some()
                    && exec_of(pe.attempt) == exec_of(pe.attempt - 1)
                {
                    ws.relay.mutate_plan_execution(run_id, |pe| {
                        pe.blocker = Some(
                            "no progress: identical findings, commit and evidence across repair attempts"
                                .into(),
                        );
                    });
                    let pe2 = ws.relay.plan_execution(run_id).unwrap();
                    let f = facts_of(&pe2, "review", "no_progress", pe2.blocker.clone(), None);
                    push_facts(&ws.relay, run_id, f);
                    return StageRouting::Fail(
                        "no_progress: identical findings/commit/evidence across repair attempts — stopping early"
                            .into(),
                    );
                }
            }
            // 修复轮上限（默认 3，work 技能界定的产品默认）。
            if pe.repair_count >= pe.repair_limit {
                let pe2 = ws.relay.plan_execution(run_id).unwrap();
                let f = facts_of(&pe2, "review", "repair_limit", None, None);
                push_facts(&ws.relay, run_id, f);
                return StageRouting::Fail(format!(
                    "repair limit exhausted ({} rounds) — stopping",
                    pe.repair_limit
                ));
            }
            // 计划状态机回退 executing；引擎回退到 execute（Q-01 受控入口）；
            // attempt/repair 计数推进——三者在一个 runs 锁内原子完成。
            if let Err(e) = transition_plan_status(ws, &pe, "executing") {
                return StageRouting::Fail(format!("plan transition failed: {e}"));
            }
            let rewound = ws.relay.mutate_engine_and_plan(run_id, |engine, pe| {
                engine_rewind_to_step(engine, "execute")?;
                pe.phase = "execute".into();
                pe.attempt += 1;
                pe.repair_count += 1;
                pe.outcome = Some("needs_fix".into());
                Ok(())
            });
            // 修复轮的 coder 必须看到复审发现（引擎 history 已截断，模板
            // 不带 coder 自述）——findings 注入上下文，step_context 消费。
            let findings_block = claim
                .findings
                .iter()
                .map(|f| {
                    format!(
                        "- [{}] task={} ac={}：{}",
                        f.id,
                        f.task.as_deref().unwrap_or("-"),
                        f.ac.as_deref().unwrap_or("-"),
                        f.description
                    )
                })
                .collect::<Vec<_>>()
                .join("
");
            ws.relay.set_context_var(run_id, "repair_findings", &findings_block);
            match rewound {
                Some(Ok(())) => {}
                Some(Err(e)) => return StageRouting::Fail(format!("rewind failed: {e}")),
                None => return StageRouting::Fail("run vanished during rewind".into()),
            }
            let pe2 = ws.relay.plan_execution(run_id).unwrap();
            let f = facts_of(&pe2, "review", "needs_fix", None, None);
            push_facts(&ws.relay, run_id, f);
            StageRouting::RewoundToExecute
        }
        other => StageRouting::Fail(format!(
            "review outcome '{other}' is not routable (want pass|needs_fix|needs_replan|blocked)"
        )),
    }
}

fn route_document_end(
    ws: &WsStores,
    run_id: &str,
    pe: &crate::relay::plan_contract::PlanExecutionState,
    claim: &crate::relay::plan_contract::StageResult,
) -> StageRouting {
    if claim.outcome != "pass" {
        return StageRouting::Fail(format!(
            "document outcome '{}' is not routable (delivery checkpoints live in plan_delivery; needs_replan|blocked stop the run)",
            claim.outcome
        ));
    }
    // 交付检查点事实核对（AC-08）：五个检查点必须已由 plan_delivery 工具
    // 逐项登记（动作本体在 T-08/T-09 的 plan_delivery）。
    let required = ["prepared", "landed", "ledger_refreshed", "archived", "cleaned"];
    let missing: Vec<&str> = required
        .iter()
        .filter(|cp| !pe.delivery_checkpoints.contains_key(**cp))
        .copied()
        .collect();
    if !missing.is_empty() {
        return StageRouting::Fail(format!(
            "delivery checkpoints missing: {missing:?} — run plan_delivery actions first"
        ));
    }
    ws.relay.mutate_plan_execution(run_id, |pe| {
        pe.phase = "delivered".into();
        pe.outcome = Some("pass".into());
    });
    let pe2 = ws.relay.plan_execution(run_id).unwrap();
    push_facts(
        &ws.relay,
        run_id,
        facts_of(&pe2, "document", "pass", None, Some("all".into())),
    );
    StageRouting::Complete
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill_src(td: &std::path::Path) -> std::path::PathBuf {
        for name in PLAN_SKILLS {
            let d = td.join(name);
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("SKILL.md"), format!("# {name} discipline")).unwrap();
        }
        td.to_path_buf()
    }

    fn plan_dir(td: &std::path::Path) -> std::path::PathBuf {
        let p = td.join("plans");
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(
            p.join("042-demo.md"),
            "---\nplan_id: PLAN-042\nstatus: reviewed\nfeature_name: Demo\ncreated_at: 2026-10-01T00:00:00Z\nupdated_at: 2026-10-01T00:00:00Z\nplan_revision: 1\ncurrent_step: 2\ntotal_steps: 2\nsupersedes_spec_components: []\nnew_spec_components: []\ntouched_goals: []\n---\n\n# [PLAN-042] Demo\n\n## 1. 目标\n\n- A\n\n## 7. 验收标准\n\n- [x] AC-01 ok\n\n## 8. 执行步骤\n\n- [x] T-01 done\n",
        )
        .unwrap();
        p
    }

    #[test]
    fn bootstrap_plan_freezes_skills() {
        let td = tempfile::tempdir().unwrap();
        let src = skill_src(td.path());
        let b = bootstrap_plan_run("plan", "做一个功能", &plan_dir(td.path()), &src).unwrap();
        assert_eq!(b.state.phase, "plan");
        assert_eq!(b.state.attempt, 1);
        assert_eq!(b.state.repair_limit, 3);
        assert_eq!(b.state.skills.len(), 4);
        assert_eq!(b.state.contract_version, 1);
    }

    #[test]
    fn bootstrap_plan_merge_resolves_target_contract() {
        let td = tempfile::tempdir().unwrap();
        let src = skill_src(td.path());
        let pd = plan_dir(td.path());
        let b = bootstrap_plan_run("plan-merge", "沉淀 PLAN-042 到 Spec", &pd, &src).unwrap();
        assert_eq!(b.state.phase, "document");
        assert_eq!(b.state.plan_id, "PLAN-042");
        assert_eq!(b.state.plan_seq, 42);
        assert_eq!(b.state.plan_path, "docs/plans/042-demo.md");
        assert_eq!(b.state.plan_revision, 1);
    }

    #[test]
    fn bootstrap_rejects_missing_skill_or_bad_target() {
        let td = tempfile::tempdir().unwrap();
        let pd = plan_dir(td.path());
        // 缺技能源 → 硬失败。
        assert!(bootstrap_plan_run("plan", "任务", &pd, td.path()).is_err());
        // plan-merge 无 PLAN-NNN → 拒绝。
        let src = skill_src(&td.path().join("skills"));
        assert!(bootstrap_plan_run("plan-merge", "沉淀吧", &pd, &src)
            .err()
            .unwrap()
            .contains("PLAN-NNN"));
        // 指向不存在的计划 → 拒绝。
        assert!(bootstrap_plan_run("plan-merge", "沉淀 PLAN-777", &pd, &src).is_err());
        // 非 plan 流程 → 拒绝。
        assert!(bootstrap_plan_run("simple", "x", &pd, &src).is_err());
    }

    #[test]
    fn parse_plan_id_extracts_first_match() {
        assert_eq!(
            parse_plan_id("沉淀 PLAN-042 到 Spec"),
            Some((42, "PLAN-042".into()))
        );
        assert_eq!(parse_plan_id("no plan here"), None);
        assert_eq!(parse_plan_id("PLAN-abc"), None);
    }

    #[test]
    fn engine_rewind_resets_cursor_gate_and_history() {
        use crate::relay::{FlowSpec, FlowStep, PipelineEngine};
        let mut flow = FlowSpec::new("plan");
        flow.add_step(FlowStep::new("plan", "advisor"));
        flow.add_step(FlowStep::new("execute", "coder"));
        flow.add_step(FlowStep::new("review", "reviewer"));
        let mut eng = PipelineEngine::new(flow, "run-1");
        let _ = eng.advance();
        assert!(matches!(eng.status, crate::relay::PipelineStatus::Running { .. }));
        // 提交两步到 review。
        use crate::relay::HandoffDocument;
        let _ = eng.submit_handoff(HandoffDocument::new("advisor", "coder"));
        let _ = eng.advance();
        let _ = eng.submit_handoff(HandoffDocument::new("coder", "reviewer"));
        let _ = eng.advance();
        assert_eq!(eng.current_step, 2);
        assert_eq!(eng.step_history.len(), 2);
        engine_rewind_to_step(&mut eng, "execute").unwrap();
        assert_eq!(eng.current_step, 1);
        assert_eq!(eng.step_history.len(), 1);
        assert!(matches!(eng.status, crate::relay::PipelineStatus::Idle));
        // 未知 step → Err。
        assert!(engine_rewind_to_step(&mut eng, "nope").is_err());
    }

    #[test]
    fn owner_guard_is_exclusive_and_releasable() {
        owner_release("run-owner-test"); // 隔离：清掉可能的残留
        assert!(owner_try_claim("run-owner-test"));
        assert!(!owner_try_claim("run-owner-test"), "second claim refused");
        assert!(owner_held("run-owner-test"));
        owner_release("run-owner-test");
        assert!(!owner_held("run-owner-test"));
        assert!(owner_try_claim("run-owner-test"), "re-claim after release");
        owner_release("run-owner-test");
    }

    #[test]
    fn cancel_flag_round_trip() {
        cancel_remove("run-cancel-test");
        assert!(!cancel_set("run-cancel-test"), "unregistered cancel = false");
        let flag = cancel_register("run-cancel-test");
        assert!(!flag.load(Ordering::SeqCst));
        assert!(cancel_set("run-cancel-test"));
        assert!(flag.load(Ordering::SeqCst));
        cancel_remove("run-cancel-test");
        assert!(cancel_flag("run-cancel-test").is_none());
    }
}
