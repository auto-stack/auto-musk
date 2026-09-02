//! parity_feature_dev.rs — Plan 020 Phase A: feature_dev.at 移植 parity。
//!
//! The transpiled `auto_generated::feature_dev` (drive loop + substitute /
//! eval_condition / flow / require_builtin + FeatureDevResult /
//! WorkflowStreamEvent) must behave like hw `relay::feature_dev`. These tests
//! exercise the mirror directly and assert parity:
//! - pure logic: flow() wire, require_builtin error text, substitute /
//!   eval_condition (the hw unit tests, run on BOTH sides and compared),
//!   now_secs sanity.
//! - wire format: FeatureDevResult / WorkflowStreamEvent serialize identically.
//! - end-to-end: run() drives the full architect→coder→tester→reviewer loop
//!   with a canned mock client (no daemon) — hw and ag produce byte-equal
//!   FeatureDevResult wire data (the Phase A acceptance bar).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};
use auto_ai_agent::Client;
use musk::auto_generated::feature_dev as ag;
use musk::relay::feature_dev as hw;
use musk::server::AppState;

/// Canned mock client — agent.run() returns "mock answer" with no tool calls,
/// so the drive loop runs every step to completion deterministically.
struct CannedClient;
#[async_trait]
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

fn test_state() -> AppState {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "musk-parity-feature-dev-{}-{}",
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

/// The registry seeds a default workspace (root = dir name); borrow it.
fn ws_of(state: &AppState) -> Arc<musk::workspace::WorkspaceStores> {
    let q = musk::workspace::WorkspaceQuery { workspace: None };
    let ws_id = q.id_or_default(&state.registry);
    state.registry.get(&ws_id)
}

// ── pure logic parity ───────────────────────────────────────────────────────

#[test]
fn parity_flow_is_linear_four_steps() {
    let hw_f = hw::flow();
    let ag_f = ag::flow();
    assert_eq!(ag_f.id, hw_f.id, "flow id");
    assert_eq!(ag_f.steps.len(), hw_f.steps.len(), "same step count");
    assert_eq!(ag_f.steps.len(), 4, "four linear steps");
    for (i, (ag_step, hw_step)) in ag_f.steps.iter().zip(hw_f.steps.iter()).enumerate() {
        assert_eq!(ag_step.id, hw_step.id, "step[{i}].id");
        assert_eq!(ag_step.role_id, hw_step.role_id, "step[{i}].role_id");
    }
    assert_eq!(ag_f.steps[0].id, "architect", "first step");
    assert_eq!(ag_f.steps[3].id, "reviewer", "last step");
}

#[test]
fn parity_builtin_validation_accepts_feature_dev_only() {
    // Same accept/reject decision + identical error message text.
    for input in ["feature-dev", "nope", "/some/custom.at", ""] {
        let hw_res = hw::require_builtin(input);
        let ag_res = ag::require_builtin(input);
        assert_eq!(hw_res.is_ok(), ag_res.is_ok(), "accept/reject parity for '{input}'");
        if let (Err(hw_e), Err(ag_e)) = (&hw_res, &ag_res) {
            assert_eq!(ag_e, hw_e, "error text parity for '{input}'");
        }
    }
    assert!(ag::require_builtin("feature-dev").is_ok());
    assert!(ag::require_builtin("nope").is_err());
}

#[test]
fn parity_substitute_replaces_known_vars_and_keeps_unknown() {
    let mut vars = HashMap::new();
    vars.insert("user_request".to_string(), "build x".to_string());
    vars.insert("code".to_string(), "fn main() {}".to_string());
    let template = "Task:\n$user_request\nCode:\n$code\n$missing";

    let hw_out = hw::substitute(template, &vars);
    let ag_out = ag::substitute(template, vars.clone());
    assert_eq!(ag_out, hw_out, "substitute parity");
    assert_eq!(ag_out, "Task:\nbuild x\nCode:\nfn main() {}\n$missing");
}

#[test]
fn parity_substitute_edge_cases() {
    let mut vars = HashMap::new();
    vars.insert("a".to_string(), "A".to_string());
    let cases = [
        "no dollars",
        "$",
        "$$",
        "$$a",
        "a$b$c",
        "$a$b",
        "$ ",
        "$aé",
        "$a é",
        "中文$a 文本",
        "$unknown_x",
    ];
    for t in cases {
        let hw_out = hw::substitute(t, &vars);
        let ag_out = ag::substitute(t, vars.clone());
        assert_eq!(ag_out, hw_out, "edge-case parity for {t:?} (hw={hw_out:?} ag={ag_out:?})");
    }
}

#[test]
fn parity_condition_bare_var_and_contains() {
    let mut vars = HashMap::new();
    vars.insert("test_report".to_string(), "all pass".to_string());
    vars.insert("empty".to_string(), String::new());

    let cases: &[(&str, bool)] = &[
        ("$test_report", true),
        ("$test_report.contains(pass)", true),
        ("$test_report.contains(fail)", false),
        ("$empty", false),
        ("$unknown", false),
        ("$test_report.contains(\"pass\")", true),
        ("$test_report.contains('pass')", true),
        ("no-dollar-prefix", true),
    ];
    for (expr, expect) in cases {
        let hw_v = hw::eval_condition(expr, &vars);
        let ag_v = ag::eval_condition(expr, vars.clone());
        assert_eq!(hw_v, *expect, "hw truth for {expr:?}");
        assert_eq!(ag_v, *expect, "ag parity for {expr:?}");
    }
}

// ── wire format parity ──────────────────────────────────────────────────────

#[test]
fn parity_feature_dev_result_wire() {
    let mut r = hw::FeatureDevResult::default();
    r.steps.insert("architect".into(), "plan".into());
    r.outputs.insert("design".into(), "plan".into());
    r.total_tokens = 42;
    let hw_v = serde_json::to_value(&r).unwrap();

    let mut ag_r = ag::FeatureDevResult::default();
    ag_r.steps.insert("architect".into(), "plan".into());
    ag_r.outputs.insert("design".into(), "plan".into());
    ag_r.total_tokens = 42;
    let ag_v = serde_json::to_value(&ag_r).unwrap();
    assert_eq!(ag_v, hw_v, "FeatureDevResult wire mismatch");
}

#[test]
fn parity_workflow_stream_event_wire() {
    let mut steps = HashMap::new();
    steps.insert("architect".to_string(), "plan".to_string());
    let mut outputs = HashMap::new();
    outputs.insert("design".to_string(), "plan".to_string());

    let hw_events: Vec<hw::WorkflowStreamEvent> = vec![
        hw::WorkflowStreamEvent::StepStart {
            step_id: "s1".into(),
            role: "architect".into(),
            input: "in".into(),
        },
        hw::WorkflowStreamEvent::StepDone { step_id: "s1".into(), output: "out".into() },
        hw::WorkflowStreamEvent::StepSkipped { step_id: "s1".into() },
        hw::WorkflowStreamEvent::Finished { steps: steps.clone(), outputs: outputs.clone() },
    ];
    let ag_events: Vec<ag::WorkflowStreamEvent> = vec![
        ag::WorkflowStreamEvent::StepStart {
            step_id: "s1".into(),
            role: "architect".into(),
            input: "in".into(),
        },
        ag::WorkflowStreamEvent::StepDone { step_id: "s1".into(), output: "out".into() },
        ag::WorkflowStreamEvent::StepSkipped { step_id: "s1".into() },
        ag::WorkflowStreamEvent::Finished { steps, outputs },
    ];
    for (i, (hw_ev, ag_ev)) in hw_events.iter().zip(ag_events.iter()).enumerate() {
        let hw_v = serde_json::to_value(hw_ev).unwrap();
        let ag_v = serde_json::to_value(ag_ev).unwrap();
        assert_eq!(ag_v, hw_v, "WorkflowStreamEvent[{i}] wire mismatch");
    }
    // Spot-check the wire shape (tag + rename_all snake_case).
    let start = serde_json::to_value(&hw_events[0]).unwrap();
    assert_eq!(start["type"], "step_start");
    assert_eq!(start["step_id"], "s1");
    let fin = serde_json::to_value(&hw_events[3]).unwrap();
    assert_eq!(fin["type"], "finished");
}

// ── end-to-end drive parity (Phase A acceptance) ────────────────────────────

#[tokio::test]
async fn parity_feature_dev_run_matches_hw() {
    let state = test_state();
    let ws = ws_of(&state);

    let hw_res = hw::run(&state, &ws, "build a small parser").await.unwrap();
    let ag_res = ag::run(state.clone(), ws.clone(), "build a small parser")
        .await
        .unwrap();

    assert_eq!(ag_res.steps, hw_res.steps, "steps map parity");
    assert_eq!(ag_res.outputs, hw_res.outputs, "outputs map parity");
    assert_eq!(ag_res.total_tokens, hw_res.total_tokens, "total_tokens parity");

    // Every step ran (canned client drives all four; no reviewer skip).
    assert_eq!(ag_res.steps.len(), 4, "four steps recorded");
    assert!(ag_res.steps.contains_key("architect"));
    assert!(ag_res.steps.contains_key("reviewer"));
    assert_eq!(ag_res.outputs.len(), 4, "one output var per step (user_request stays internal to vars)");
    // Every recorded step output is the canned answer.
    for v in ag_res.steps.values() {
        assert_eq!(v, "mock answer", "each step output is the canned answer");
    }

    // Full wire equality (order-independent for the JSON maps).
    assert_eq!(
        serde_json::to_value(&ag_res).unwrap(),
        serde_json::to_value(&hw_res).unwrap(),
        "FeatureDevResult wire parity after full run"
    );
}

#[tokio::test]
async fn parity_feature_dev_run_with_emit_streams_same_events() {
    use tokio::sync::mpsc;

    let state = test_state();
    let ws = ws_of(&state);
    let task = "stream me";

    // hw run_stream collects events via its closure into an unbounded channel
    // (send is non-blocking — the closure runs on the tokio test thread).
    let (hw_tx, mut hw_rx) = tokio::sync::mpsc::unbounded_channel();
    let hw_events = {
        let on_event: Arc<dyn Fn(hw::WorkflowStreamEvent) + Send + Sync> =
            Arc::new(move |ev| {
                let _ = hw_tx.send(ev);
            });
        hw::run_stream(&state, &ws, task, on_event).await.unwrap()
    }; // closure + tx dropped here → channel closes after the run.
    let mut hw_collected: Vec<serde_json::Value> = Vec::new();
    while let Ok(ev) = hw_rx.try_recv() {
        hw_collected.push(serde_json::to_value(&ev).unwrap());
    }

    // ag run_with_emit: mpsc sender handle wired through the extern side-table
    // (same wire path the stream handler uses), collected after the run. The
    // channel stays open (the side-table holds the sender), so drain with a
    // short timeout instead of waiting for close.
    let ch = musk::auto_generated::extern_impl::mpsc_channel();
    let tx = musk::auto_generated::extern_impl::mpsc_sender(&ch);
    let rx = musk::auto_generated::extern_impl::mpsc_receiver(&ch);
    let ag_res =
        ag::run_with_emit(state.clone(), ws.clone(), task, tx).await.unwrap();
    let mut ag_collected: Vec<serde_json::Value> = Vec::new();
    loop {
        match tokio::time::timeout(
            std::time::Duration::from_millis(100),
            musk::auto_generated::extern_impl::mpsc_recv(&rx),
        )
        .await
        {
            Ok(Some(v)) => ag_collected.push(v),
            Ok(None) => break, // channel closed
            Err(_) => break,   // drained (channel still open) — stop
        }
    }

    // Same final result AND the same streamed event sequence (types + order).
    assert_eq!(
        serde_json::to_value(&ag_res).unwrap(),
        serde_json::to_value(&hw_events).unwrap(),
        "streaming run result parity"
    );
    assert_eq!(ag_collected.len(), hw_collected.len(), "same number of stream events");
    for (i, (ag_v, hw_v)) in ag_collected.iter().zip(hw_collected.iter()).enumerate() {
        assert_eq!(ag_v, hw_v, "stream event[{i}] wire parity");
    }
    assert_eq!(
        ag_collected[0]["type"], "step_start",
        "first event is step_start"
    );
    assert_eq!(
        ag_collected.last().unwrap()["type"], "finished",
        "last event is finished"
    );
}
