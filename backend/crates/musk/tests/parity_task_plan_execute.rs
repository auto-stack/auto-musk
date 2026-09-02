//! Plan 020 Phase H — TaskPlan single-run execution kernel parity.
//!
//! The transpiled `auto_generated::task_plan_engine::RelayTaskPlanExecutor`
//! (Phase H) replaces the hand-written `DriveTaskPlanExecutor` that transparently
//! forwarded to hw `drive_task_plan_run`. It must produce an equivalent
//! `RunExecutionResult` for the same `RunRequest`: same terminal status, same
//! error (None on completed, Some otherwise), and a handoff summary whose content
//! matches the canned agent output.
//!
//! Strategy: build a RunRequest for the "default" flow on a fresh workspace,
//! run it to completion via BOTH the ag RelayTaskPlanExecutor and the hw
//! drive_task_plan_run, and assert the RunExecutionResult parity field-by-field.

use std::sync::Arc;

use auto_ai_agent::Client;
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};

use musk::auto_generated::task_plan_engine::{self as ag, RelayTaskPlanExecutor, TaskPlanExecutor};
use musk::relay::task_plan_engine as hw;
use musk::server::AppState;

/// Canned client — every agent step returns "mock answer", so a single-run relay
/// completes deterministically (status "completed", handoff summary = the answer).
struct CannedClient;

#[async_trait::async_trait]
impl Client for CannedClient {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        Ok(CompletionResponse {
            content: "mock answer".into(),
            tool_calls: vec![],
            stop_reason: Some("end_turn".into()),
            usage: None,
            model: "mock".into(),
            error: None,
                model_meta: None,
        })
    }
}

fn make_state() -> AppState {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "musk-parity-task-plan-exec-{}-{}",
        std::process::id(),
        n
    ));
    let _ = std::fs::create_dir_all(&dir);
    let registry =
        musk::workspace::WorkspaceRegistry::load(dir.join("workspaces.json"), dir.clone());
    AppState {
        client: Arc::new(CannedClient) as Arc<dyn Client>,
        auth: Arc::new(musk::auto_generated::auth::AuthStore::new(dir.join("users.json"))),
        registry: Arc::new(registry),
        chat_runs: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
    }
}

fn ws_id_of(state: &AppState) -> String {
    let q = musk::workspace::WorkspaceQuery { workspace: None };
    q.id_or_default(&state.registry)
}

/// An ag RunRequest for one run of the default flow (id "default"), phase "p1".
fn ag_run_request(run_id: &str) -> ag::RunRequest {
    ag::RunRequest {
        task_plan_id: "test-plan".into(),
        phase_name: "p1".into(),
        phase_index: 0,
        run_ref: musk::auto_generated::task_plan::RunRef::new("r1", "simple"),
        run_id: run_id.into(),
        parent_run_id: None,
        root_run_id: run_id.into(),
        task_text: "build a small parser".into(),
        mode: musk::auto_generated::task_plan::TaskMode::Gsd,
    }
}

/// The hw equivalent RunRequest (field names differ: hw uses `task`, ag `task_text`;
/// hw phase_index is usize, ag is u32).
fn hw_run_request(run_id: &str) -> hw::RunRequest {
    hw::RunRequest {
        task_plan_id: "test-plan".into(),
        phase_name: "p1".into(),
        phase_index: 0,
        run_ref: musk::relay::task_plan::RunRef::new("r1", "simple"),
        run_id: run_id.into(),
        parent_run_id: None,
        root_run_id: run_id.into(),
        task: "build a small parser".into(),
        mode: musk::relay::task_plan::TaskMode::Gsd,
    }
}

// ── parity: single-run kernel completes identically ────────────────────────

#[tokio::test]
async fn parity_relay_task_plan_executor_matches_hw_kernel() {
    let ag_state = Arc::new(make_state());
    let hw_state = Arc::new(make_state());
    let ag_ws = ws_id_of(&ag_state);
    let hw_ws = ws_id_of(&hw_state);

    // ag executor:RelTaskPlanExecutor::run(完整 Auto: start_run + ag drive_run +
    // 读 status/handoff)。
    let ag_ctx = ag::TaskPlanContext { state: (*ag_state).clone(), workspace_id: ag_ws };
    let ag_exec = RelayTaskPlanExecutor { ctx: ag_ctx };
    let ag_res = ag_exec.run(ag_run_request("run-ag-kernel")).await
        .expect("ag RelayTaskPlanExecutor::run");

    // hw kernel:drive_task_plan_run(原 hw 单 run 执行内核,Phase G 接线后内部调
    // ag drive_run,但本测试对照的是它的返回值语义)。
    let hw_ctx = hw::TaskPlanContext { state: (*hw_state).clone(), workspace_id: hw_ws };
    let hw_res = hw::drive_task_plan_run(&hw_ctx, hw_run_request("run-hw-kernel")).await
        .expect("hw drive_task_plan_run");

    // RunExecutionResult field-by-field parity.
    assert_eq!(ag_res.run_id, "run-ag-kernel", "ag run_id echoed back");
    assert_eq!(hw_res.run_id, "run-hw-kernel", "hw run_id echoed back");
    assert_eq!(ag_res.status, hw_res.status, "terminal status parity");
    assert_eq!(ag_res.status, "completed", "default-flow run completes on canned client");
    assert_eq!(ag_res.error, hw_res.error, "error field parity (None on completed)");
    assert!(ag_res.error.is_none(), "completed run has no error");

    // Handoff: both sides carry the canned answer as the summary.
    let ag_summary = ag_res.handoff.as_ref().map(|h| h.summary.clone()).unwrap_or_default();
    let hw_summary = hw_res.handoff.as_ref().map(|h| h.summary.clone()).unwrap_or_default();
    assert_eq!(ag_summary, hw_summary, "handoff summary parity");
    assert_eq!(ag_summary, "mock answer", "handoff carries the canned agent output");
}

// ── parity: the ag executor wires into the engine's execute() loop ────────────

/// Full TaskPlanEngine::execute with the ag RelayTaskPlanExecutor injected (the
/// production path) completes a 2-phase serial plan, exactly as the hw path does
/// with its relay executor. This proves the executor implements the trait correctly.
#[tokio::test]
async fn parity_engine_execute_with_ag_executor_completes() {
    use musk::auto_generated::task_plan as ag_tp;
    use musk::relay::handoff_store::HandoffStore;

    let state = Arc::new(make_state());
    let ws_id = ws_id_of(&state);
    let dir = std::env::temp_dir().join(format!(
        "musk-parity-task-plan-exec-engine-{}",
        std::process::id()
    ));
    let _ = std::fs::create_dir_all(&dir);
    let handoffs = HandoffStore::new(dir.join("handoffs"));

    let plan = ag_tp::TaskPlan::new("exec-plan")
        .add_phase(ag_tp::Phase::new("p1").add_run(ag_tp::RunRef::new("r1", "simple")))
        .add_phase(
            ag_tp::Phase::new("p2")
                .depends_on(vec!["p1".to_string()])
                .add_run(ag_tp::RunRef::new("r2", "simple")),
        );
    let mut engine = ag::TaskPlanEngine::new(plan, "do work");
    assert!(engine.validate().is_ok(), "plan validates");

    let ctx = ag::TaskPlanContext { state: (*state).clone(), workspace_id: ws_id };
    let executor = Arc::new(RelayTaskPlanExecutor { ctx });
    engine.execute(handoffs, executor).await.expect("execute completes");

    // Both phases complete via the ag executor's drive_run path.
    assert_eq!(
        format!("{:?}", engine.status),
        "Completed".to_string(),
        "engine reaches Completed with the ag executor"
    );
    assert_eq!(engine.run_states.len(), 2, "two runs recorded");
    for rs in engine.run_states.values() {
        assert_eq!(rs.status, "completed", "each run completed via the ag executor");
    }
}
