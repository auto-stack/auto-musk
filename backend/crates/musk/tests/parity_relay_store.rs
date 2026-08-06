//! parity_relay_store.rs — Plan 020 Phase A2: relay_store.at 数据层 parity。
//!
//! The transpiled `auto_generated::relay_store` read models (RunEvent /
//! RunSummary / StepState / GateState / RunMetadata / StartRunRequest /
//! StartRunStep / GateType) must serialize byte-identically to hw
//! `relay::store` types (the frontend consumes the same wire). These tests
//! construct equivalent values on both sides and compare wire JSON, including
//! the `skip_serializing_if` None-omission behavior on RunSummary.

use musk::auto_generated::relay_store as ag;
use musk::relay::store as hw;

#[test]
fn parity_gate_type_wire() {
    assert_eq!(
        serde_json::to_value(ag::GateType::Auto).unwrap(),
        serde_json::to_value(musk::relay::GateType::Auto).unwrap(),
        "Auto wire"
    );
    assert_eq!(
        serde_json::to_value(ag::GateType::Human).unwrap(),
        serde_json::to_value(musk::relay::GateType::Human).unwrap(),
        "Human wire"
    );
    assert_eq!(
        serde_json::to_string(&ag::GateType::Human).unwrap(),
        "\"human\"",
        "snake_case wire"
    );
}

#[test]
fn parity_run_summary_wire() {
    let hw_full = hw::RunSummary {
        run_id: "run-1".into(),
        status: "running".into(),
        current_step: 2,
        total_steps: 4,
        current_profession: Some("coder".into()),
        cumulative_tokens: 123,
        created_at: 1000,
        updated_at: 2000,
        title: Some("My run".into()),
        task: Some("do stuff".into()),
    };
    let ag_full = ag::RunSummary {
        run_id: "run-1".into(),
        status: "running".into(),
        current_step: 2,
        total_steps: 4,
        current_profession: Some("coder".into()),
        cumulative_tokens: 123,
        created_at: 1000,
        updated_at: 2000,
        title: Some("My run".into()),
        task: Some("do stuff".into()),
    };
    assert_eq!(
        serde_json::to_value(&ag_full).unwrap(),
        serde_json::to_value(&hw_full).unwrap(),
        "full RunSummary wire (both title/task Some)"
    );

    // None title/task → omitted on BOTH sides (skip_serializing_if parity).
    let hw_none = hw::RunSummary {
        title: None,
        task: None,
        ..hw_full
    };
    let ag_none = ag::RunSummary {
        title: None,
        task: None,
        ..ag_full
    };
    assert_eq!(
        serde_json::to_value(&ag_none).unwrap(),
        serde_json::to_value(&hw_none).unwrap(),
        "RunSummary with None title/task wire"
    );
    let hw_json = serde_json::to_value(&hw_none).unwrap();
    assert!(hw_json.get("title").is_none(), "hw omits None title");
    assert!(hw_json.get("task").is_none(), "hw omits None task");
}

#[test]
fn parity_step_state_wire() {
    let hw_s = hw::StepState {
        id: "design".into(),
        role_id: "architect".into(),
        status: "completed".into(),
        gate: "auto".into(),
    };
    let ag_s = ag::StepState {
        id: "design".into(),
        role_id: "architect".into(),
        status: "completed".into(),
        gate: "auto".into(),
    };
    assert_eq!(
        serde_json::to_value(&ag_s).unwrap(),
        serde_json::to_value(&hw_s).unwrap(),
        "StepState wire"
    );
}

#[test]
fn parity_gate_state_wire() {
    let hw_g = hw::GateState {
        step_id: "code".into(),
        role_id: "coder".into(),
        since: 999,
    };
    let ag_g = ag::GateState {
        step_id: "code".into(),
        role_id: "coder".into(),
        since: 999,
    };
    assert_eq!(
        serde_json::to_value(&ag_g).unwrap(),
        serde_json::to_value(&hw_g).unwrap(),
        "GateState wire"
    );
}

#[test]
fn parity_run_metadata_wire() {
    // Default() on both sides (all None) must serialize identically.
    let hw_m = hw::RunMetadata::default();
    let ag_m = ag::RunMetadata::default();
    assert_eq!(
        serde_json::to_value(&ag_m).unwrap(),
        serde_json::to_value(&hw_m).unwrap(),
        "default RunMetadata wire"
    );

    // Full 11-field metadata (TaskPlan tracing) wire parity.
    let hw_full = hw::RunMetadata {
        title: Some("t".into()),
        initial_task: Some("task".into()),
        originating_chat_session: Some("chat-1".into()),
        workspace_id: Some("ws".into()),
        task_plan_id: Some("plan-1".into()),
        task_run_name: Some("phase-a/run-1".into()),
        phase_name: Some("phase-a".into()),
        phase_index: Some(0),
        parent_run_id: Some("parent".into()),
        root_run_id: Some("root".into()),
    };
    let ag_full = ag::RunMetadata {
        title: Some("t".into()),
        initial_task: Some("task".into()),
        originating_chat_session: Some("chat-1".into()),
        workspace_id: Some("ws".into()),
        task_plan_id: Some("plan-1".into()),
        task_run_name: Some("phase-a/run-1".into()),
        phase_name: Some("phase-a".into()),
        phase_index: Some(0),
        parent_run_id: Some("parent".into()),
        root_run_id: Some("root".into()),
    };
    assert_eq!(
        serde_json::to_value(&ag_full).unwrap(),
        serde_json::to_value(&hw_full).unwrap(),
        "full RunMetadata wire (11 fields)"
    );
}

#[test]
fn parity_run_event_wire_and_event_type() {
    // Every variant, serialized identically + event_type() string parity.
    let hw_events = vec![
        hw::RunEvent::StepStarted { timestamp: 1, step_id: "s".into(), role_id: "r".into() },
        hw::RunEvent::StepCompleted { timestamp: 2, step_id: "s".into(), handoff_summary: "h".into() },
        hw::RunEvent::GateWaiting { timestamp: 3, step_id: "s".into(), gate: "human".into() },
        hw::RunEvent::GateResolved { timestamp: 4, step_id: "s".into(), decision: "approve".into() },
        hw::RunEvent::RunCompleted { timestamp: 5 },
        hw::RunEvent::RunFailed { timestamp: 6, error: "boom".into() },
        hw::RunEvent::TokenSpend { timestamp: 7, cumulative: 10, step_tokens: 3 },
        hw::RunEvent::RelayUpdate { timestamp: 8, step_id: "s".into(), role_id: "r".into(), status: "running".into() },
        hw::RunEvent::TurnDelta { timestamp: 9, role_id: "r".into(), text: "hi".into() },
        hw::RunEvent::TurnToolCall { timestamp: 10, role_id: "r".into(), tool_id: "t".into(), tool_name: "read".into(), arguments: serde_json::json!({"path": "a"}) },
        hw::RunEvent::TurnToolResult { timestamp: 11, role_id: "r".into(), tool_id: "t".into(), result: "ok".into() },
        hw::RunEvent::TurnComplete { timestamp: 12, role_id: "r".into() },
        hw::RunEvent::TurnError { timestamp: 13, role_id: "r".into(), message: "e".into() },
        hw::RunEvent::TurnBudgetWarning { timestamp: 14, role_id: "r".into(), remaining: 5 },
        hw::RunEvent::TurnBudgetExceeded { timestamp: 15, role_id: "r".into() },
    ];
    let ag_events = vec![
        ag::RunEvent::StepStarted { timestamp: 1, step_id: "s".into(), role_id: "r".into() },
        ag::RunEvent::StepCompleted { timestamp: 2, step_id: "s".into(), handoff_summary: "h".into() },
        ag::RunEvent::GateWaiting { timestamp: 3, step_id: "s".into(), gate: "human".into() },
        ag::RunEvent::GateResolved { timestamp: 4, step_id: "s".into(), decision: "approve".into() },
        ag::RunEvent::RunCompleted { timestamp: 5 },
        ag::RunEvent::RunFailed { timestamp: 6, error: "boom".into() },
        ag::RunEvent::TokenSpend { timestamp: 7, cumulative: 10, step_tokens: 3 },
        ag::RunEvent::RelayUpdate { timestamp: 8, step_id: "s".into(), role_id: "r".into(), status: "running".into() },
        ag::RunEvent::TurnDelta { timestamp: 9, role_id: "r".into(), text: "hi".into() },
        ag::RunEvent::TurnToolCall { timestamp: 10, role_id: "r".into(), tool_id: "t".into(), tool_name: "read".into(), arguments: serde_json::json!({"path": "a"}) },
        ag::RunEvent::TurnToolResult { timestamp: 11, role_id: "r".into(), tool_id: "t".into(), result: "ok".into() },
        ag::RunEvent::TurnComplete { timestamp: 12, role_id: "r".into() },
        ag::RunEvent::TurnError { timestamp: 13, role_id: "r".into(), message: "e".into() },
        ag::RunEvent::TurnBudgetWarning { timestamp: 14, role_id: "r".into(), remaining: 5 },
        ag::RunEvent::TurnBudgetExceeded { timestamp: 15, role_id: "r".into() },
    ];
    assert_eq!(hw_events.len(), 15, "15 hw variants");
    assert_eq!(ag_events.len(), hw_events.len(), "15 ag variants");

    for (i, (hw_ev, ag_ev)) in hw_events.iter().zip(ag_events.iter()).enumerate() {
        assert_eq!(
            serde_json::to_value(ag_ev).unwrap(),
            serde_json::to_value(hw_ev).unwrap(),
            "RunEvent[{i}] wire parity"
        );
        assert_eq!(ag_ev.event_type(), hw_ev.event_type(), "RunEvent[{i}] event_type parity");
    }

    // Spot-check the tagged wire shape.
    let start = serde_json::to_value(&hw_events[0]).unwrap();
    assert_eq!(start["type"], "step_started");
    assert_eq!(start["step_id"], "s");
}

#[test]
fn parity_start_run_request_round_trip() {
    // Both Deserialize the same request wire into structurally equal values.
    let wire = serde_json::json!({
        "run_id": "run-9",
        "flow_id": "relay",
        "steps": [
            {"id": "a", "role_id": "architect", "gate": "auto"},
            {"id": "b", "role_id": "coder"}
        ],
        "task": "build it"
    });
    let hw_req: hw::StartRunRequest = serde_json::from_value(wire.clone()).unwrap();
    let ag_req: ag::StartRunRequest = serde_json::from_value(wire).unwrap();

    assert_eq!(ag_req.run_id, hw_req.run_id, "run_id");
    assert_eq!(ag_req.flow_id, hw_req.flow_id, "flow_id");
    assert_eq!(ag_req.task, hw_req.task, "task");
    assert_eq!(ag_req.steps.len(), hw_req.steps.len(), "steps len");
    for (i, (ag_s, hw_s)) in ag_req.steps.iter().zip(hw_req.steps.iter()).enumerate() {
        assert_eq!(ag_s.id, hw_s.id, "step[{i}].id");
        assert_eq!(ag_s.role_id, hw_s.role_id, "step[{i}].role_id");
        assert_eq!(
            serde_json::to_value(&ag_s.gate).unwrap(),
            serde_json::to_value(&hw_s.gate).unwrap(),
            "step[{i}].gate wire"
        );
    }
}
