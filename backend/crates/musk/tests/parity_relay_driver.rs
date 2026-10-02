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
        canvas: Arc::new(musk::canvas::CanvasManager::new()),
        chat_runs: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
            chat_cancels: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            run_idle_timeout: std::time::Duration::from_secs(300),
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
        authorization: None,
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
            RunEvent::StepCompleted { timestamp: now, step_id: "s".into(), handoff_summary: "h".into(), telemetry: None },
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


// ── PLAN-098 T-03 (V05 / AC-02): relay 注入路径遥测关联与折叠 ───────────────

/// Mock serving client: responds with actual-served model metadata (PLAN-031
/// shape) and usage, so the factory-injected TelemetryClient records a real
/// attribution row per request.
struct ServedClient;

#[async_trait::async_trait]
impl Client for ServedClient {
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        Ok(CompletionResponse {
            content: "mock answer".into(),
            tool_calls: vec![],
            stop_reason: Some("end_turn".into()),
            usage: Some(auto_ai_client::Usage {
                input_tokens: 11,
                output_tokens: 22,
                ..Default::default()
            }),
            model: "tier:mid".into(),
            error: None,
            model_meta: Some(auto_ai_client::ModelMeta {
                id: "glm-5.3-test".into(),
                context_window: 200_000,
                max_output_tokens: None,
            }),
        })
    }
}

/// V05（AC-02）：factory 遥测包装按 run/step/role 关联记录；相位收束在
/// store.submit_handoff（双轨委托单点）折叠注入 StepCompleted；并发双 run
/// 零错配。hw 与 ag 两条驱动轨都经 hw MuskAgentFactory 构造 step agent，
/// 分别断言。
#[tokio::test]
async fn plan098_relay_telemetry_correlates_and_folds() {
    let state = Arc::new(make_state(Arc::new(ServedClient) as Arc<dyn Client>));
    let ws_id = ws_id_of(&state);

    // 并发双 run（同 workspace，交错驱动）——correlation 各归各 run。
    for rid in ["run-tel-hw", "run-tel-ag"] {
        start_run(state.clone(), &ws_id, "simple", rid);
    }

    hw_driver::drive_run(state.clone(), ws_id.clone(), "run-tel-hw".into()).await;
    ag_driver::drive_run(state.clone(), &ws_id, "run-tel-ag")
        .await
        .expect("ag drive_run");

    for (rid, driver_tag) in [("run-tel-hw", "hw"), ("run-tel-ag", "ag")] {
        let ws = state.registry.get(&ws_id);
        let rs = ws.relay.get(rid).expect("run gone");
        assert_eq!(rs.status, "completed", "{rid} completed");
        let completed: Vec<&RunEvent> = rs
            .events
            .iter()
            .filter(|e| e.event_type() == "step_completed")
            .collect();
        assert!(completed.len() >= 2, "{driver_tag}: simple flow has 2 steps");
        for ev in completed {
            let RunEvent::StepCompleted { step_id, telemetry, .. } = ev else {
                unreachable!()
            };
            let tel = telemetry.as_ref().unwrap_or_else(|| {
                panic!("{driver_tag}: StepCompleted {step_id} must carry telemetry")
            });
            assert_eq!(tel["correlation"]["kind"], "relay");
            assert_eq!(tel["correlation"]["run_id"], rid, "correlation 归本 run（零错配）");
            assert_eq!(tel["correlation"]["step_id"], *step_id);
            assert!(
                tel["correlation"]["role"].is_string()
                    && !tel["correlation"]["role"].as_str().unwrap().is_empty(),
                "role 非空"
            );
            // 实际服务模型（回退后真值以 model_meta 为权威）与 usage 透传。
            assert_eq!(tel["model"], "glm-5.3-test", "served model recorded");
            // 请求模型 = 最终发出的值（角色 tier 解析结果，非空即可——
            // TelemetryClient 位于最外层，记录的是实际发出的请求字段）。
            let requested = tel["requested_model"].as_str().unwrap_or_default();
            assert!(!requested.is_empty(), "requested_model recorded, got {tel}");
            assert_ne!(requested, "glm-5.3-test", "requested ≠ served（回退场景形状）");
            assert_eq!(tel["out_tokens"], 22);
            // provider 不上 wire（T-01 F9）→ 显式 null。
            assert!(tel["provider"].is_null());
        }
        // 折叠即 drain：fold 后注册表不残留该 run 的条目。
        let recs = musk::telemetry::relay_drain(rid, "nonexistent-step");
        assert!(recs.is_empty(), "{driver_tag}: sink drained by fold");
    }
}


// ── PLAN-094 T-03 (UAT K4/K5): execute-gate plan-file invariant ─────────────

/// Auto mode, plan flow, advisor never materializes a plan file: the execute
/// gate must NOT pass — the run fails loudly with the contract error instead
/// of the UAT T3 cascade (coder idling → reviewer death-loop). Plan D1: no
/// gate-redirect rerun (the engine's redraft reruns the GATED step, i.e.
/// execute, not the plan phase — a plan-phase rerun is not expressible), so
/// the first miss fails directly. Both drive loops (hw + ag, single-sourced
/// rule core in plan_flow::execute_gate_action) must behave identically.
#[tokio::test]
async fn plan094_auto_gate_without_plan_file_fails_without_passing() {
    let hw_state = Arc::new(make_state(Arc::new(CannedClient) as Arc<dyn Client>));
    let ag_state = hw_state.clone();
    let ws_id = ws_id_of(&hw_state);

    for rid in ["run-hw-k5", "run-ag-k5"] {
        start_run(hw_state.clone(), &ws_id, "plan", rid);
        hw_state.registry.get(&ws_id).relay.set_context_var(rid, "approval_mode", "auto");
    }

    hw_driver::drive_run(hw_state.clone(), ws_id.clone(), "run-hw-k5".into()).await;
    ag_driver::drive_run(ag_state.clone(), &ws_id, "run-ag-k5")
        .await
        .expect("ag drive_run");

    for (state, rid) in [(hw_state.clone(), "run-hw-k5"), (ag_state.clone(), "run-ag-k5")] {
        let ws = state.registry.get(&ws_id);
        let rs = ws.relay.get(rid).expect("run gone");
        assert_eq!(rs.status, "failed", "{rid}: plan-less gate fails the run");
        // The plan phase ran exactly once — no rerun, and no phase past the
        // gate (the UAT T3 coder idle-run is impossible).
        let advisor_runs = rs
            .step_history
            .iter()
            .filter(|r| r.role_id == "advisor" && r.handoff.is_some())
            .count();
        assert_eq!(advisor_runs, 1, "{rid}: plan phase ran once, no blind rerun");
        // No phase past the gate ever ran.
        assert!(
            !rs.step_history.iter().any(|r| r.role_id != "advisor"),
            "{rid}: coder/reviewer/assistant must not run without a plan"
        );
        // Terminal event carries the contract error string.
        assert!(
            rs.events.iter().any(
                |e| matches!(e, RunEvent::RunFailed { error, .. }
                    if error == musk::relay::plan_flow::PLAN_GATE_FAIL_ERROR)
            ),
            "{rid}: RunFailed carries the plan-file error"
        );
        // The gate was never resolved (no approve slipped through).
        assert!(
            !rs.events.iter().any(|e| matches!(e, RunEvent::GateResolved { .. })),
            "{rid}: gate must not be consumed"
        );
    }

    // hw/ag parity on terminal status + event-type sequence.
    let (hw_status, _) = run_summary(&hw_state, &ws_id, "run-hw-k5");
    let (ag_status, _) = run_summary(&ag_state, &ws_id, "run-ag-k5");
    assert_eq!(ag_status, hw_status, "status parity (K5 fail-fast)");
    assert_eq!(
        event_types(&hw_state, &ws_id, "run-hw-k5"),
        event_types(&ag_state, &ws_id, "run-ag-k5"),
        "event-type sequence parity (K5 fail-fast)"
    );
}

/// AC-04 regression: with the plan file present (binding channel), the auto
/// gate approves straight through — behavior unchanged from PLAN-067.
#[tokio::test]
async fn plan094_auto_gate_with_plan_file_approves_through() {
    let hw_state = Arc::new(make_state(Arc::new(CannedClient) as Arc<dyn Client>));
    let ag_state = hw_state.clone();
    let ws_id = ws_id_of(&hw_state);

    for rid in ["run-hw-ac04", "run-ag-ac04"] {
        start_run(hw_state.clone(), &ws_id, "plan", rid);
        let ws = hw_state.registry.get(&ws_id);
        ws.relay.set_context_var(rid, "approval_mode", "auto");
        ws.relay.set_context_var(rid, "plan_file", "docs/plans/001-x.md");
    }

    hw_driver::drive_run(hw_state.clone(), ws_id.clone(), "run-hw-ac04".into()).await;
    ag_driver::drive_run(ag_state.clone(), &ws_id, "run-ag-ac04")
        .await
        .expect("ag drive_run");

    for (state, rid) in [(hw_state.clone(), "run-hw-ac04"), (ag_state.clone(), "run-ag-ac04")] {
        let ws = state.registry.get(&ws_id);
        let rs = ws.relay.get(rid).expect("run gone");
        assert_eq!(rs.status, "completed", "{rid}: plan file present → gate approves");
        // K4 audit: the plain auto-approve still records its default note.
        assert!(
            rs.events.iter().any(|e| matches!(e, RunEvent::GateResolved { decision, note: Some(n), .. }
                if decision == "approve" && n == musk::relay::plan_flow::AUTO_APPROVE_NOTE)),
            "{rid}: auto-approve audit carries the default note"
        );
    }
    let (hw_status, _) = run_summary(&hw_state, &ws_id, "run-hw-ac04");
    let (ag_status, _) = run_summary(&ag_state, &ws_id, "run-ag-ac04");
    assert_eq!(ag_status, hw_status, "status parity (AC-04)");
}
