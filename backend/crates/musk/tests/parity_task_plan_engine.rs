//! parity_task_plan_engine.rs — Plan 020 Phase B: task_plan_engine.at 移植 parity。
//!
//! The transpiled `auto_generated::task_plan_engine` (TaskPlanEngine drive loop
//! + topological_order + validate + build_run_request, executor via the pub
//! `TaskPlanExecutor` trait) must behave like hw `relay::task_plan_engine`.
//! These tests drive both engines with equivalent fake executors (no LLM) and
//! compare the observable state: status, phase_states, run_states, and the
//! run requests the executor receives (task assembly, run_id shape, mode).

use std::sync::Arc;

use async_trait::async_trait;
use auto_ai_agent::HandoffDocument;
use musk::auto_generated::task_plan as ag_tp;
use musk::auto_generated::task_plan_engine as ag;
use musk::relay::task_plan as hw_tp;
use musk::relay::handoff_store::HandoffStore;
use musk::relay::task_plan_engine as hw;
use tempfile::TempDir;

// ── executors ───────────────────────────────────────────────────────────────

/// Fake ag executor: succeeds every run, echoes the request task as the
/// handoff summary, captures the requests it saw.
struct AgFakeExecutor {
    captured: std::sync::Mutex<Vec<ag::RunRequest>>,
    fail_run: Option<String>,
}

#[async_trait]
impl ag::TaskPlanExecutor for AgFakeExecutor {
    async fn run(
        &self,
        req: ag::RunRequest,
    ) -> Result<ag::RunExecutionResult, String> {
        self.captured.lock().unwrap().push(req.clone());
        if self.fail_run.as_deref() == Some(req.run_ref.name.as_str()) {
            return Ok(ag::RunExecutionResult {
                run_id: req.run_id.clone(),
                status: "failed".into(),
                handoff: None,
                error: Some("boom".into()),
            });
        }
        let mut handoff = HandoffDocument::new("assistant", "next");
        handoff.summary = req.task_text.clone();
        Ok(ag::RunExecutionResult {
            run_id: req.run_id,
            status: "completed".into(),
            handoff: Some(handoff),
            error: None,
        })
    }
}

/// hw 侧等价假 executor(闭包;与 AgFakeExecutor 同语义)。execute 的
/// `F: Fn`(非 FnMut)不能改捕获,故 captured 用 Arc<Mutex>。
fn hw_success_executor(
    captured: Arc<std::sync::Mutex<Vec<hw::RunRequest>>>,
) -> impl Fn(hw::RunRequest) -> std::future::Ready<Result<hw::RunExecutionResult, String>> {
    move |req: hw::RunRequest| {
        captured.lock().unwrap().push(req.clone());
        let mut handoff = HandoffDocument::new("assistant", "next");
        handoff.summary = req.task.clone();
        std::future::ready(Ok(hw::RunExecutionResult {
            run_id: req.run_id,
            status: "completed".to_string(),
            handoff: Some(handoff),
            error: None,
        }))
    }
}

fn hw_fail_executor(
    fail_run: &'static str,
    captured: Arc<std::sync::Mutex<Vec<hw::RunRequest>>>,
) -> impl Fn(hw::RunRequest) -> std::future::Ready<Result<hw::RunExecutionResult, String>> {
    move |req: hw::RunRequest| {
        captured.lock().unwrap().push(req.clone());
        let status = if req.run_ref.name == fail_run {
            "failed".to_string()
        } else {
            "completed".to_string()
        };
        let err = if req.run_ref.name == fail_run { Some("boom".to_string()) } else { None };
        std::future::ready(Ok(hw::RunExecutionResult {
            run_id: req.run_id,
            status,
            handoff: None,
            error: err,
        }))
    }
}

// ── helpers ─────────────────────────────────────────────────────────────────

fn make_handoffs() -> (TempDir, HandoffStore) {
    let dir = TempDir::new().unwrap();
    let store = HandoffStore::new(dir.path());
    (dir, store)
}

fn ag_status_str(e: &ag::TaskPlanEngine) -> String {
    serde_json::to_string(&e.status).unwrap()
}
fn hw_status_str(e: &hw::TaskPlanEngine) -> String {
    serde_json::to_string(&e.status).unwrap()
}
fn ag_phase_status_str(e: &ag::TaskPlanEngine, name: &str) -> String {
    serde_json::to_string(&e.phase_states.get(name).unwrap().status).unwrap()
}
fn hw_phase_status_str(e: &hw::TaskPlanEngine, name: &str) -> String {
    serde_json::to_string(&e.phase_states.get(name).unwrap().status).unwrap()
}

/// Compare the captured request's observable fields (task assembly + shapes)。
fn assert_req_parity(ag_req: &ag::RunRequest, hw_req: &hw::RunRequest, ctx: &str) {
    assert_eq!(ag_req.task_plan_id, hw_req.task_plan_id, "{ctx}: task_plan_id");
    assert_eq!(ag_req.phase_name, hw_req.phase_name, "{ctx}: phase_name");
    assert_eq!(ag_req.phase_index as usize, hw_req.phase_index, "{ctx}: phase_index");
    assert_eq!(ag_req.run_ref.name, hw_req.run_ref.name, "{ctx}: run_ref.name");
    assert_eq!(ag_req.run_ref.flow_id, hw_req.run_ref.flow_id, "{ctx}: run_ref.flow_id");
    assert_eq!(ag_req.task_text, hw_req.task, "{ctx}: task assembly");
    assert_eq!(ag_req.parent_run_id, hw_req.parent_run_id, "{ctx}: parent_run_id");
    // root_run_id = instance_id = "{plan.id}-{uuidish}"——uuidish 随机,比较形状。
    assert_eq!(
        ag_req.root_run_id.split('-').next(),
        hw_req.root_run_id.split('-').next(),
        "{ctx}: root_run_id plan prefix"
    );
    assert!(ag_req.root_run_id.starts_with(&ag_req.task_plan_id), "{ctx}: root prefix");
    assert!(hw_req.root_run_id.starts_with(&hw_req.task_plan_id), "{ctx}: hw root prefix");
    assert!(
        ag_req.run_id.starts_with(&ag_req.root_run_id),
        "{ctx}: run_id rooted in instance_id"
    );
    assert_eq!(
        serde_json::to_string(&ag_req.mode).unwrap(),
        serde_json::to_string(&hw_req.mode).unwrap(),
        "{ctx}: mode"
    );
}

// ── tests ───────────────────────────────────────────────────────────────────

#[test]
fn parity_engine_new_status_and_validate() {
    let ag_plan = ag_tp::TaskPlan::new("p")
        .add_phase(ag_tp::Phase::new("p1").add_run(ag_tp::RunRef::new("r1", "default")));
    let hw_plan = hw_tp::TaskPlan::new("p")
        .add_phase(hw_tp::Phase::new("p1").add_run(hw_tp::RunRef::new("r1", "default")));

    let ag_engine = ag::TaskPlanEngine::new(ag_plan, "do work");
    let hw_engine = hw::TaskPlanEngine::new(hw_plan, "do work");

    // instance_id shape: "{plan.id}-{uuidish}"; status Pending; phase pending.
    assert!(ag_engine.instance_id.starts_with("p-"), "instance_id prefix");
    assert!(hw_engine.instance_id.starts_with("p-"), "hw instance_id prefix");
    assert_eq!(ag_status_str(&ag_engine), hw_status_str(&hw_engine), "initial status");
    assert_eq!(ag_phase_status_str(&ag_engine, "p1"), hw_phase_status_str(&hw_engine, "p1"));
    assert_eq!(ag_engine.phase_states.get("p1").unwrap().run_results.len(), 0);

    // validate: same accept/reject + same error text.
    assert_eq!(ag_engine.validate().is_ok(), hw_engine.validate().is_ok(), "valid plan");

    let ag_bad = ag::TaskPlanEngine::new(
        ag_tp::TaskPlan::new("bad").add_phase(
            ag_tp::Phase::new("p1").add_run(ag_tp::RunRef::new("r1", "nonexistent-flow")),
        ),
        "do work",
    );
    let hw_bad = hw::TaskPlanEngine::new(
        hw_tp::TaskPlan::new("bad").add_phase(
            hw_tp::Phase::new("p1").add_run(hw_tp::RunRef::new("r1", "nonexistent-flow")),
        ),
        "do work",
    );
    let ag_err = ag_bad.validate().unwrap_err();
    let hw_err = hw_bad.validate().unwrap_err();
    assert_eq!(ag_err, hw_err, "unknown-flow validation error text");
    assert!(ag_err.contains("unknown flow"));
}

#[test]
fn parity_topological_order() {
    // Chain: p1 -> p2 -> p3 (plan order p3, p2, p1 → DFS post-order p1, p2, p3)。
    let ag_plan = ag_tp::TaskPlan::new("chain")
        .add_phase(ag_tp::Phase::new("p1").add_run(ag_tp::RunRef::new("r1", "default")))
        .add_phase(
            ag_tp::Phase::new("p2")
                .depends_on(vec!["p1".to_string()])
                .add_run(ag_tp::RunRef::new("r2", "default")),
        )
        .add_phase(
            ag_tp::Phase::new("p3")
                .depends_on(vec!["p2".to_string()])
                .add_run(ag_tp::RunRef::new("r3", "default")),
        );
    let hw_plan = hw_tp::TaskPlan::new("chain")
        .add_phase(hw_tp::Phase::new("p1").add_run(hw_tp::RunRef::new("r1", "default")))
        .add_phase(
            hw_tp::Phase::new("p2")
                .depends_on(vec!["p1".to_string()])
                .add_run(hw_tp::RunRef::new("r2", "default")),
        )
        .add_phase(
            hw_tp::Phase::new("p3")
                .depends_on(vec!["p2".to_string()])
                .add_run(hw_tp::RunRef::new("r3", "default")),
        );
    let ag_e = ag::TaskPlanEngine::new(ag_plan, "x");
    let hw_e = hw::TaskPlanEngine::new(hw_plan, "x");
    assert_eq!(ag_e.topological_order().unwrap(), hw_e.topological_order().unwrap());
    assert_eq!(ag_e.topological_order().unwrap(), vec!["p1", "p2", "p3"]);

    // Cycle → same error text.
    let ag_cyc = ag::TaskPlanEngine::new(
        ag_tp::TaskPlan::new("cyc")
            .add_phase(
                ag_tp::Phase::new("a")
                    .depends_on(vec!["b".to_string()])
                    .add_run(ag_tp::RunRef::new("r", "default")),
            )
            .add_phase(
                ag_tp::Phase::new("b")
                    .depends_on(vec!["a".to_string()])
                    .add_run(ag_tp::RunRef::new("r", "default")),
            ),
        "x",
    );
    let hw_cyc = hw::TaskPlanEngine::new(
        hw_tp::TaskPlan::new("cyc")
            .add_phase(
                hw_tp::Phase::new("a")
                    .depends_on(vec!["b".to_string()])
                    .add_run(hw_tp::RunRef::new("r", "default")),
            )
            .add_phase(
                hw_tp::Phase::new("b")
                    .depends_on(vec!["a".to_string()])
                    .add_run(hw_tp::RunRef::new("r", "default")),
            ),
        "x",
    );
    let ag_err = ag_cyc.topological_order().unwrap_err();
    let hw_err = hw_cyc.topological_order().unwrap_err();
    assert_eq!(ag_err, hw_err, "cycle error text");
    assert!(ag_err.contains("cycle detected at phase 'a'"));
}

#[tokio::test]
async fn parity_execute_serial_plan() {
    let (_dir, handoffs) = make_handoffs();
    let ag_plan = ag_tp::TaskPlan::new("serial")
        .add_phase(ag_tp::Phase::new("p1").add_run(ag_tp::RunRef::new("r1", "default")))
        .add_phase(
            ag_tp::Phase::new("p2")
                .depends_on(vec!["p1".to_string()])
                .add_run(ag_tp::RunRef::new("r2", "default")),
        );
    let mut ag_engine = ag::TaskPlanEngine::new(ag_plan, "do work");
    let ag_exec = Arc::new(AgFakeExecutor { captured: std::sync::Mutex::new(vec![]), fail_run: None });
    ag_engine
        .execute(HandoffStore::new(_dir.path().join("ag")), ag_exec.clone())
        .await
        .unwrap();

    let (_dir2, handoffs) = make_handoffs();
    let hw_plan = hw_tp::TaskPlan::new("serial")
        .add_phase(hw_tp::Phase::new("p1").add_run(hw_tp::RunRef::new("r1", "default")))
        .add_phase(
            hw_tp::Phase::new("p2")
                .depends_on(vec!["p1".to_string()])
                .add_run(hw_tp::RunRef::new("r2", "default")),
        );
    let mut hw_engine = hw::TaskPlanEngine::new(hw_plan, "do work");
    let hw_captured: Arc<std::sync::Mutex<Vec<hw::RunRequest>>> = Arc::new(std::sync::Mutex::new(vec![]));
    hw_engine
        .execute(&handoffs, hw_success_executor(hw_captured.clone()))
        .await
        .unwrap();

    // Status + phase states parity.
    assert_eq!(ag_status_str(&ag_engine), hw_status_str(&hw_engine), "final status");
    assert_eq!(ag_status_str(&ag_engine), "\"completed\"");
    for p in ["p1", "p2"] {
        assert_eq!(
            ag_phase_status_str(&ag_engine, p),
            hw_phase_status_str(&hw_engine, p),
            "phase {p} status"
        );
    }

    // Run states parity (keys + fields)。run_id 含 uuidish(hw/ag 各异),
    // 按 (phase_name, run_name) 配对比较。
    assert_eq!(ag_engine.run_states.len(), hw_engine.run_states.len());
    let mut ag_pairs: Vec<(String, String, String)> = ag_engine
        .run_states
        .values()
        .map(|s| (s.phase_name.clone(), s.run_name.clone(), s.status.clone()))
        .collect();
    let mut hw_pairs: Vec<(String, String, String)> = hw_engine
        .run_states
        .values()
        .map(|s| (s.phase_name.clone(), s.run_name.clone(), s.status.clone()))
        .collect();
    ag_pairs.sort();
    hw_pairs.sort();
    assert_eq!(ag_pairs, hw_pairs, "run_states content parity");
    for (phase, run, status) in &ag_pairs {
        assert_eq!(status, "completed", "run {phase}.{run} status");
    }

    // Executor received equivalent requests (task assembly / shapes)。
    let ag_captured = ag_exec.captured.lock().unwrap();
    assert_eq!(ag_captured.len(), hw_captured.lock().unwrap().len(), "same run count");
    for (i, (ag_req, hw_req)) in ag_captured.iter().zip(hw_captured.lock().unwrap().iter()).enumerate() {
        assert_req_parity(ag_req, hw_req, &format!("run[{i}]"));
    }
    // Serial runs in dependency order: p1.r1 first, then p2.r2.
    assert_eq!(ag_captured[0].phase_name, "p1");
    assert_eq!(ag_captured[1].phase_name, "p2");
}

#[tokio::test]
async fn parity_execute_parallel_phase() {
    let (_dir, handoffs) = make_handoffs();
    let ag_plan = ag_tp::TaskPlan::new("parallel").add_phase(
        ag_tp::Phase::new("p1")
            .with_mode(ag_tp::PhaseMode::Parallel)
            .add_run(ag_tp::RunRef::new("a", "default"))
            .add_run(ag_tp::RunRef::new("b", "default")),
    );
    let mut ag_engine = ag::TaskPlanEngine::new(ag_plan, "do work");
    let ag_exec = Arc::new(AgFakeExecutor { captured: std::sync::Mutex::new(vec![]), fail_run: None });
    ag_engine
        .execute(HandoffStore::new(_dir.path().join("ag")), ag_exec.clone())
        .await
        .unwrap();

    let hw_plan = hw_tp::TaskPlan::new("parallel").add_phase(
        hw_tp::Phase::new("p1")
            .with_mode(hw_tp::PhaseMode::Parallel)
            .add_run(hw_tp::RunRef::new("a", "default"))
            .add_run(hw_tp::RunRef::new("b", "default")),
    );
    let mut hw_engine = hw::TaskPlanEngine::new(hw_plan, "do work");
    let hw_captured: Arc<std::sync::Mutex<Vec<hw::RunRequest>>> = Arc::new(std::sync::Mutex::new(vec![]));
    hw_engine
        .execute(&handoffs, hw_success_executor(hw_captured.clone()))
        .await
        .unwrap();

    assert_eq!(ag_status_str(&ag_engine), hw_status_str(&hw_engine));
    assert_eq!(ag_phase_status_str(&ag_engine, "p1"), hw_phase_status_str(&hw_engine, "p1"));
    let ag_state = ag_engine.phase_states.get("p1").unwrap();
    let hw_state = hw_engine.phase_states.get("p1").unwrap();
    assert!(ag_state.run_results.contains_key("a"));
    assert!(ag_state.run_results.contains_key("b"));
    assert_eq!(ag_state.run_results.len(), hw_state.run_results.len());
}

#[tokio::test]
async fn parity_execute_failure_propagates() {
    let (_dir, handoffs) = make_handoffs();
    let ag_plan = ag_tp::TaskPlan::new("fail").add_phase(
        ag_tp::Phase::new("p1")
            .add_run(ag_tp::RunRef::new("r1", "default"))
            .add_run(ag_tp::RunRef::new("r2", "default")),
    );
    let mut ag_engine = ag::TaskPlanEngine::new(ag_plan, "do work");
    let ag_exec = Arc::new(AgFakeExecutor {
        captured: std::sync::Mutex::new(vec![]),
        fail_run: Some("r2".into()),
    });
    let ag_err = ag_engine
        .execute(HandoffStore::new(_dir.path().join("ag")), ag_exec.clone())
        .await
        .unwrap_err();

    let hw_plan = hw_tp::TaskPlan::new("fail").add_phase(
        hw_tp::Phase::new("p1")
            .add_run(hw_tp::RunRef::new("r1", "default"))
            .add_run(hw_tp::RunRef::new("r2", "default")),
    );
    let mut hw_engine = hw::TaskPlanEngine::new(hw_plan, "do work");
    let hw_captured: Arc<std::sync::Mutex<Vec<hw::RunRequest>>> = Arc::new(std::sync::Mutex::new(vec![]));
    let hw_err = hw_engine
        .execute(&handoffs, hw_fail_executor("r2", hw_captured.clone()))
        .await
        .unwrap_err();

    assert_eq!(ag_err, hw_err, "failure error parity");
    assert!(ag_err.contains("boom"));
    assert_eq!(ag_status_str(&ag_engine), hw_status_str(&hw_engine), "failed status");
    assert_eq!(ag_status_str(&ag_engine), "\"failed\"");
    assert_eq!(
        ag_phase_status_str(&ag_engine, "p1"),
        hw_phase_status_str(&hw_engine, "p1"),
        "phase failed status"
    );
}

#[tokio::test]
async fn parity_execute_input_from_resolves_previous_handoff() {
    // Pre-save a handoff; the plan's run uses input_from → both executors see
    // the resolved value in the assembled task.
    let (dir, handoffs) = make_handoffs();
    let mut prev = HandoffDocument::new("assistant", "next");
    prev.summary = "previous result".to_string();
    handoffs.save("input-plan", "p1", "r1", &prev).unwrap();
    // ag execute 按值接管 store——给 ag 一份同内容的独立 store。
    let ag_handoffs = HandoffStore::new(dir.path().join("ag"));
    {
        let mut prev2 = HandoffDocument::new("assistant", "next");
        prev2.summary = "previous result".to_string();
        ag_handoffs.save("input-plan", "p1", "r1", &prev2).unwrap();
    }

    let ag_plan = ag_tp::TaskPlan::new("input-plan").add_phase(
        ag_tp::Phase::new("p2").add_run(
            ag_tp::RunRef::new("r2", "default")
                .with_input_from(vec!["p1.r1.handoff.summary".to_string()]),
        ),
    );
    let mut ag_engine = ag::TaskPlanEngine::new(ag_plan, "initial task");
    let ag_exec = Arc::new(AgFakeExecutor { captured: std::sync::Mutex::new(vec![]), fail_run: None });
    ag_engine
        .execute(ag_handoffs, ag_exec.clone())
        .await
        .unwrap();

    let hw_plan = hw_tp::TaskPlan::new("input-plan").add_phase(
        hw_tp::Phase::new("p2").add_run(
            hw_tp::RunRef::new("r2", "default")
                .with_input_from(vec!["p1.r1.handoff.summary".to_string()]),
        ),
    );
    let mut hw_engine = hw::TaskPlanEngine::new(hw_plan, "initial task");
    let hw_captured: Arc<std::sync::Mutex<Vec<hw::RunRequest>>> = Arc::new(std::sync::Mutex::new(vec![]));
    hw_engine
        .execute(&handoffs, hw_success_executor(hw_captured.clone()))
        .await
        .unwrap();

    let ag_captured = ag_exec.captured.lock().unwrap();
    assert_eq!(ag_captured.len(), hw_captured.lock().unwrap().len());
    assert_req_parity(&ag_captured[0], &hw_captured.lock().unwrap()[0], "input_from run");
    assert!(
        ag_captured[0].task_text.contains("previous result"),
        "resolved handoff text in task: {:?}",
        ag_captured[0].task_text
    );
    // hw semantics: explicit input wins over initial_input for non-first phase;
    // input_from text is appended.
    assert!(ag_captured[0].task_text.contains("## Input from p1.r1.handoff.summary"));
    assert_eq!(ag_status_str(&ag_engine), hw_status_str(&hw_engine));
}
