//! Plan 020 Phase G — relay_driver drive_run/drive_loop/run_step parity.
//!
//! The transpiled `auto_generated::relay_driver::drive_run` (the relay
//! orchestration loop moved here from hw `relay/driver.rs` in Phase G) must drive
//! a relay run identically to the hw original: same terminal status, same
//! StepCompleted / RunCompleted event sequence, same handoff summaries, and the
//! same error-handoff path when an agent fails.
//!
//! Strategy: a canned client that returns "mock answer" deterministically drives
//! both the hw and ag loops on freshly-started runs of the same builtin flow;
//! the resulting RunState (status / step_history handoff summaries) is compared
//! end-to-end. A failing-client variant exercises the error-handoff branch.

use std::sync::Arc;

use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};
use auto_ai_agent::Client;

use musk::auto_generated::relay_driver as ag_driver;
use musk::relay::driver as hw_driver;
use musk::relay::store::{RunEvent, StartRunRequest};
use musk::server::AppState;

/// Canned mock client — every step's agent returns "mock answer" with no tool
/// calls, so the drive loop runs every auto step to completion deterministically.
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

/// Failing mock client — every agent step errors, exercising the drive loop's
/// error-handoff path (hw driver.rs:120-127 wraps the error as a handoff with
/// summary `[agent error] {e}` and submit_handoff fails the run).
struct FailClient;

#[async_trait::async_trait]
impl Client for FailClient {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        Err(ClientError::DaemonUnavailable)
    }
}

fn make_state(client: Arc<dyn Client>) -> AppState {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "musk-parity-relay-driver-{}-{}",
        std::process::id(),
        n
    ));
    let _ = std::fs::create_dir_all(&dir);
    let registry =
        musk::workspace::WorkspaceRegistry::load(dir.join("workspaces.json"), dir.clone());
    AppState {
        client,
        auth: Arc::new(musk::auto_generated::auth::AuthStore::new(dir.join("users.json"))),
        registry: Arc::new(registry),
        chat_runs: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
    }
}

fn ws_id_of(state: &AppState) -> String {
    let q = musk::workspace::WorkspaceQuery { workspace: None };
    q.id_or_default(&state.registry)
}

/// Start a relay run for the given flow; returns (state, ws_id, run_id).
fn start_run(state: Arc<AppState>, ws_id: &str, flow_id: &str, run_id: &str) {
    let ws = state.registry.get(ws_id);
    let req = StartRunRequest {
        run_id: Some(run_id.to_string()),
        flow_id: Some(flow_id.to_string()),
        steps: Vec::new(),
        task: Some("build a small parser".into()),
    };
    ws.relay.start_run(&req, Some(ws_id.to_string()));
}

/// Extract (status, handoff summaries of each step) for comparison.
fn run_summary(state: &AppState, ws_id: &str, run_id: &str) -> (String, Vec<String>) {
    let ws = state.registry.get(ws_id);
    let rs = ws
        .relay
        .get(run_id)
        .unwrap_or_else(|| panic!("run {run_id} gone"));
    let summaries: Vec<String> = rs
        .step_history
        .iter()
        .map(|r| {
            r.handoff
                .as_ref()
                .map(|h| h.summary.clone())
                .unwrap_or_default()
        })
        .collect();
    (rs.status, summaries)
}

/// Only the event *types* are compared (timestamps vary between hw/ag runs since
/// they run at different wall-clock moments).
fn event_types(state: &AppState, ws_id: &str, run_id: &str) -> Vec<&'static str> {
    let ws = state.registry.get(ws_id);
    let rs = ws.relay.get(run_id).expect("run gone");
    rs.events.iter().map(|e| e.event_type()).collect()
}

// ── parity: simple flow (code→super-coder) drives to completion ────────────

#[tokio::test]
async fn parity_drive_run_simple_flow_matches_hw() {
    let hw_state = Arc::new(make_state(Arc::new(CannedClient) as Arc<dyn Client>));
    let ag_state = hw_state.clone();
    let ws_id = ws_id_of(&hw_state);

    // Two independent runs on the same workspace/flow.
    start_run(hw_state.clone(), &ws_id, "simple", "run-hw-simple");
    start_run(ag_state.clone(), &ws_id, "simple", "run-ag-simple");

    // hw drive_run (the original; still the production source of truth until the
    // Phase G switchover in extern_impl).
    hw_driver::drive_run(hw_state.clone(), ws_id.clone(), "run-hw-simple".into()).await;
    // ag drive_run (the transpiled loop under test).
    ag_driver::drive_run(ag_state.clone(), &ws_id, "run-ag-simple")
        .await
        .expect("ag drive_run");

    let (hw_status, hw_handoffs) = run_summary(&hw_state, &ws_id, "run-hw-simple");
    let (ag_status, ag_handoffs) = run_summary(&ag_state, &ws_id, "run-ag-simple");
    assert_eq!(ag_status, hw_status, "terminal status parity (simple flow)");
    assert_eq!(ag_status, "completed", "simple flow completes");
    assert_eq!(ag_handoffs, hw_handoffs, "per-step handoff summaries parity");
    // Every step output is the canned answer.
    for s in &ag_handoffs {
        assert_eq!(s, "mock answer", "step output is the canned answer");
    }

    // Event-type sequence parity (timestamps differ; types must match).
    let hw_events = event_types(&hw_state, &ws_id, "run-hw-simple");
    let ag_events = event_types(&ag_state, &ws_id, "run-ag-simple");
    assert_eq!(ag_events, hw_events, "event-type sequence parity (simple flow)");
    // Sanity: the sequence ends with RunCompleted.
    assert!(
        ag_events.iter().any(|t| *t == "run_completed"),
        "simple flow emits RunCompleted"
    );
}

// ── parity: design flow pauses at the human gate (advisor→architect gate) ──

#[tokio::test]
async fn parity_drive_run_design_flow_pauses_at_gate() {
    let hw_state = Arc::new(make_state(Arc::new(CannedClient) as Arc<dyn Client>));
    let ag_state = hw_state.clone();
    let ws_id = ws_id_of(&hw_state);

    start_run(hw_state.clone(), &ws_id, "design", "run-hw-design");
    start_run(ag_state.clone(), &ws_id, "design", "run-ag-design");

    hw_driver::drive_run(hw_state.clone(), ws_id.clone(), "run-hw-design".into()).await;
    ag_driver::drive_run(ag_state.clone(), &ws_id, "run-ag-design")
        .await
        .expect("ag drive_run");

    let (hw_status, _) = run_summary(&hw_state, &ws_id, "run-hw-design");
    let (ag_status, _) = run_summary(&ag_state, &ws_id, "run-ag-design");
    // The design flow's first gate (advisor→architect) pauses the driver; neither
    // side completes — both stop at waiting_for_gate.
    assert_eq!(
        ag_status, hw_status,
        "status parity at gate (design flow)"
    );
    assert_eq!(
        ag_status, "waiting_approval",
        "design flow pauses at the human gate (waiting_approval)"
    );

    let hw_events = event_types(&hw_state, &ws_id, "run-hw-design");
    let ag_events = event_types(&ag_state, &ws_id, "run-ag-design");
    assert_eq!(ag_events, hw_events, "event-type sequence parity (design/gate)");
}

// ── parity: agent failure → error handoff → run fails ───────────────────────

#[tokio::test]
async fn parity_drive_run_agent_error_submits_error_handoff() {
    let hw_state = Arc::new(make_state(Arc::new(FailClient) as Arc<dyn Client>));
    let ag_state = hw_state.clone();
    let ws_id = ws_id_of(&hw_state);

    start_run(hw_state.clone(), &ws_id, "simple", "run-hw-fail");
    start_run(ag_state.clone(), &ws_id, "simple", "run-ag-fail");

    hw_driver::drive_run(hw_state.clone(), ws_id.clone(), "run-hw-fail".into()).await;
    ag_driver::drive_run(ag_state.clone(), &ws_id, "run-ag-fail")
        .await
        .expect("ag drive_run");

    let (hw_status, hw_handoffs) = run_summary(&hw_state, &ws_id, "run-hw-fail");
    let (ag_status, ag_handoffs) = run_summary(&ag_state, &ws_id, "run-ag-fail");
    // Both sides fail the run on agent error (PLAN-030 置败停车：error →
    // fail_run 直接置败，不再级联后续相位；不再提交 error handoff）。
    assert_eq!(ag_status, hw_status, "status parity on agent error");
    assert_eq!(ag_status, "failed", "agent error fails the run");
    assert_eq!(ag_handoffs, hw_handoffs, "error-handoff summaries parity");
    // The failure is recorded as a RunFailed event carrying the marker.
    let ws = ag_state.registry.get(&ws_id);
    let rs = ws.relay.get("run-ag-fail").expect("run gone");
    let marker = rs.events.iter().any(|e| match e {
        musk::relay::store::RunEvent::RunFailed { error, .. } => {
            error.starts_with("[agent error]")
        }
        _ => false,
    });
    assert!(marker, "RunFailed event carries the [agent error] marker");
}

// ── parity: event_type() wire tags are stable across hw/ag ───────────────────

/// Regression guard: the RunEvent::event_type() tags that the ag loop emits
/// (StepStarted/StepCompleted/RunCompleted/...) must match the hw tags exactly,
/// since the relay SSE `/events` stream filters by them.
#[test]
fn parity_run_event_type_tags_match() {
    let now = 0u64;
    let cases = [
        (
            RunEvent::StepStarted { timestamp: now, step_id: "s".into(), role_id: "r".into() },
            "step_started",
        ),
        (
            RunEvent::StepCompleted { timestamp: now, step_id: "s".into(), handoff_summary: "h".into() },
            "step_completed",
        ),
        (RunEvent::RunCompleted { timestamp: now, report: Default::default() }, "run_completed"),
        (
            RunEvent::TurnDelta { timestamp: now, role_id: "r".into(), text: "t".into() },
            "turn_delta",
        ),
    ];
    for (ev, expected) in cases {
        assert_eq!(ev.event_type(), expected, "event_type tag for {expected}");
    }
}

