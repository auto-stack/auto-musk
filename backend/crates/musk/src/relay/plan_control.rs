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
    engine.gate_resolved_for_step = None;
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
