//! Parity tests — verify `auto_generated::conversation` behaves identically to
//! the hand-written `conversation` module.
//!
//! Same framework as `parity_specs.rs`: exercise the same scenarios on both
//! the hand-written Rust (`musk::conversation`) and the a2r-transpiled Auto
//! output (`musk::auto_generated::conversation`).
//!
//! Scope: the transpiled module contains the data layer (Conversation / Turn /
//! ToolRecord / GateRecord / GateInfo / ConversationSummary / ConversationEvent
//! + the enums) and the pure conversion `chat_message_to_turns`. The
//! `ConversationStore` (Mutex + broadcast + jsonl IO) and `run_event_to_turns`
//! (upstream RunEvent) are hand-written boundaries and are excluded here.
//!
//! The 2026-08-04 alignment work fixed three .at gaps:
//! - `ConversationEvent` restructured to `{ conversation_id, turn, status }`
//!   (was an outdated `{ kind, conversation_id, turn_id }`).
//! - serde attrs (`default` / `skip_serializing_if` / `rename`) added to match
//!   the hand-written wire format (C4-class).
//! - `chat_message_to_turns` main-turn condition now mirrors the hand-written
//!   `!content.is_empty() || tool_calls.is_empty()` (via two nested ifs).

use musk::conversation as hw;                 // hand-written
use musk::auto_generated::conversation as ag; // a2r-transpiled Auto

// ──────────────────────────────────────────────────────────
// Enums — to_status_str + wire format parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_conversation_status_to_str() {
    for (hw_s, ag_s, expected) in [
        (hw::ConversationStatus::Active, ag::ConversationStatus::Active, "active"),
        (hw::ConversationStatus::WaitingGate, ag::ConversationStatus::WaitingGate, "waiting_gate"),
        (hw::ConversationStatus::Completed, ag::ConversationStatus::Completed, "completed"),
        (
            hw::ConversationStatus::Failed { error: "boom".into() },
            ag::ConversationStatus::Failed { error: "boom".into() },
            "failed",
        ),
        (
            hw::ConversationStatus::Paused { reason: "gate".into() },
            ag::ConversationStatus::Paused { reason: "gate".into() },
            "paused",
        ),
    ] {
        assert_eq!(hw_s.to_status_str(), expected, "hw to_status_str mismatch");
        assert_eq!(ag_s.to_status_str(), expected, "ag to_status_str mismatch");
    }
}

#[test]
fn parity_enum_wire_format() {
    assert_eq!(
        serde_json::to_string(&hw::ConversationKind::Flow).unwrap(),
        serde_json::to_string(&ag::ConversationKind::Flow).unwrap(),
    );
    assert_eq!(
        serde_json::to_string(&hw::TurnKind::ToolCall).unwrap(),
        serde_json::to_string(&ag::TurnKind::ToolCall).unwrap(),
    );
    assert_eq!(
        serde_json::to_string(&hw::Driver::Agent { agent_id: "a1".into() }).unwrap(),
        serde_json::to_string(&ag::Driver::Agent { agent_id: "a1".into() }).unwrap(),
    );
    assert_eq!(
        serde_json::to_string(&hw::Driver::Human).unwrap(),
        serde_json::to_string(&ag::Driver::Human).unwrap(),
    );
    // Status with payload serializes the same shape in both.
    assert_eq!(
        serde_json::to_string(&hw::ConversationStatus::Failed { error: "e".into() }).unwrap(),
        serde_json::to_string(&ag::ConversationStatus::Failed { error: "e".into() }).unwrap(),
    );
}

// ──────────────────────────────────────────────────────────
// Data layer — wire format parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_turn_wire_format() {
    // Full turn (to/tool/gate/tokens all present).
    let hw_turn = hw::Turn {
        id: "t1".into(),
        seq: 1,
        from: "human".into(),
        to: Some("assistant".into()),
        kind: hw::TurnKind::Message,
        content: "hi".into(),
        tool: Some(hw::ToolRecord {
            name: "read_file".into(),
            args: serde_json::json!({"path": "x"}),
            result: "ok".into(),
            tool_id: Some("tc-1".into()),
            details: None,
        }),
        gate: Some(hw::GateRecord {
            step_id: "s1".into(),
            status: "waiting".into(),
            feedback: Some("do it".into()),
        }),
        child_conversation: Some("c2".into()),
        tokens: Some(42),
        timestamp: 100,
    };
    let ag_turn = ag::Turn {
        id: "t1".into(),
        seq: 1,
        from: "human".into(),
        to_role: Some("assistant".into()),
        kind: ag::TurnKind::Message,
        content: "hi".into(),
        tool: Some(ag::ToolRecord {
            name: "read_file".into(),
            args: serde_json::json!({"path": "x"}),
            result: "ok".into(),
            tool_id: Some("tc-1".into()),
            details: None,
        }),
        gate: Some(ag::GateRecord {
            step_id: "s1".into(),
            status: "waiting".into(),
            feedback: Some("do it".into()),
        }),
        child_conversation: Some("c2".into()),
        tokens: Some(42),
        timestamp: 100,
    };
    assert_eq!(
        serde_json::to_string(&hw_turn).unwrap(),
        serde_json::to_string(&ag_turn).unwrap(),
        "full Turn wire mismatch"
    );

    // Minimal turn: all optional fields omitted on the wire.
    let hw_min = hw::Turn {
        id: "t2".into(),
        seq: 0,
        from: "system".into(),
        to: None,
        kind: hw::TurnKind::ToolCall,
        content: "".into(),
        tool: None,
        gate: None,
        child_conversation: None,
        tokens: None,
        timestamp: 0,
    };
    let ag_min = ag::Turn {
        id: "t2".into(),
        seq: 0,
        from: "system".into(),
        to_role: None,
        kind: ag::TurnKind::ToolCall,
        content: "".into(),
        tool: None,
        gate: None,
        child_conversation: None,
        tokens: None,
        timestamp: 0,
    };
    let hw_json = serde_json::to_string(&hw_min).unwrap();
    assert_eq!(hw_json, serde_json::to_string(&ag_min).unwrap(), "minimal Turn wire mismatch");
    // `to` (renamed from to_role) skipped when None; tool/gate/tokens too.
    assert!(!hw_json.contains("\"to\""), "'to':None should be skipped, got {hw_json}");
    assert!(!hw_json.contains("\"tool\""));
    assert!(!hw_json.contains("\"tokens\""));
}

#[test]
fn parity_conversation_wire_format() {
    let hw_conv = hw::Conversation {
        id: "c1".into(),
        parent_id: Some("p1".into()),
        parent_turn_id: Some("pt1".into()),
        kind: hw::ConversationKind::Flow,
        workspace_id: "ws1".into(),
        driver: hw::Driver::Agent { agent_id: "a1".into() },
        status: hw::ConversationStatus::Paused { reason: "gate".into() },
        turns: vec![hw::Turn {
            id: "t1".into(),
            seq: 0,
            from: "human".into(),
            to: Some("assistant".into()),
            kind: hw::TurnKind::Message,
            content: "hi".into(),
            tool: None,
            gate: None,
            child_conversation: None,
            tokens: None,
            timestamp: 100,
        }],
        title: Some("Title".into()),
        initial_prompt: Some("prompt".into()),
        mode: Some("superpowers".into()),
        flow_id: Some("f1".into()),
        current_step: 3,
        cumulative_tokens: 5000,
        budget_limit: Some(10_000),
        pending_gate: Some(hw::GateInfo { step_id: "s1".into(), role_id: "r1".into(), since: 50 }),
        created_at: 100,
        updated_at: 200,
    };
    let ag_conv = ag::Conversation {
        id: "c1".into(),
        parent_id: Some("p1".into()),
        parent_turn_id: Some("pt1".into()),
        kind: ag::ConversationKind::Flow,
        workspace_id: "ws1".into(),
        driver: ag::Driver::Agent { agent_id: "a1".into() },
        status: ag::ConversationStatus::Paused { reason: "gate".into() },
        turns: vec![ag::Turn {
            id: "t1".into(),
            seq: 0,
            from: "human".into(),
            to_role: Some("assistant".into()),
            kind: ag::TurnKind::Message,
            content: "hi".into(),
            tool: None,
            gate: None,
            child_conversation: None,
            tokens: None,
            timestamp: 100,
        }],
        title: Some("Title".into()),
        initial_prompt: Some("prompt".into()),
        mode: Some("superpowers".into()),
        flow_id: Some("f1".into()),
        current_step: 3,
        cumulative_tokens: 5000,
        budget_limit: Some(10_000),
        pending_gate: Some(ag::GateInfo { step_id: "s1".into(), role_id: "r1".into(), since: 50 }),
        created_at: 100,
        updated_at: 200,
    };
    assert_eq!(
        serde_json::to_string(&hw_conv).unwrap(),
        serde_json::to_string(&ag_conv).unwrap(),
        "full Conversation wire mismatch"
    );

    // Minimal conversation — optional fields all omitted.
    let hw_min = hw::Conversation {
        id: "c2".into(),
        parent_id: None,
        parent_turn_id: None,
        kind: hw::ConversationKind::Chat,
        workspace_id: "ws1".into(),
        driver: hw::Driver::Human,
        status: hw::ConversationStatus::Active,
        turns: vec![],
        title: None,
        initial_prompt: None,
        mode: None,
        flow_id: None,
        current_step: 0,
        cumulative_tokens: 0,
        budget_limit: None,
        pending_gate: None,
        created_at: 0,
        updated_at: 0,
    };
    let ag_min = ag::Conversation {
        id: "c2".into(),
        parent_id: None,
        parent_turn_id: None,
        kind: ag::ConversationKind::Chat,
        workspace_id: "ws1".into(),
        driver: ag::Driver::Human,
        status: ag::ConversationStatus::Active,
        turns: vec![],
        title: None,
        initial_prompt: None,
        mode: None,
        flow_id: None,
        current_step: 0,
        cumulative_tokens: 0,
        budget_limit: None,
        pending_gate: None,
        created_at: 0,
        updated_at: 0,
    };
    let hw_json = serde_json::to_string(&hw_min).unwrap();
    assert_eq!(hw_json, serde_json::to_string(&ag_min).unwrap(), "minimal Conversation wire mismatch");
    for key in ["parent_id", "title", "mode", "flow_id", "budget_limit", "pending_gate"] {
        assert!(!hw_json.contains(key), "optional field '{key}' should be skipped");
    }
}

#[test]
fn parity_conversation_summary_wire_format() {
    let hw_sum = hw::ConversationSummary {
        id: "c1".into(),
        kind: hw::ConversationKind::Flow,
        parent_id: Some("p1".into()),
        workspace_id: "ws1".into(),
        status: "active".into(),
        title: Some("T".into()),
        turn_count: 4,
        cumulative_tokens: 99,
        created_at: 1,
        updated_at: 2,
    };
    let ag_sum = ag::ConversationSummary {
        id: "c1".into(),
        kind: ag::ConversationKind::Flow,
        parent_id: Some("p1".into()),
        workspace_id: "ws1".into(),
        status: "active".into(),
        title: Some("T".into()),
        turn_count: 4,
        cumulative_tokens: 99,
        created_at: 1,
        updated_at: 2,
    };
    assert_eq!(
        serde_json::to_string(&hw_sum).unwrap(),
        serde_json::to_string(&ag_sum).unwrap(),
        "ConversationSummary wire mismatch"
    );
}

// ──────────────────────────────────────────────────────────
// chat_message_to_turns — conversion behavior parity
// ──────────────────────────────────────────────────────────

/// Run the conversion on both versions and assert the produced Turn lists are
/// byte-identical when serialized.
fn assert_conversion_parity(hw_msg: &musk::chats::ChatMessage, ag_msg: ag::ChatMessage, seq_base: u32) -> usize {
    let hw_turns = hw::chat_message_to_turns(hw_msg, seq_base as usize);
    let ag_turns = ag::chat_message_to_turns(ag_msg, seq_base);
    assert_eq!(hw_turns.len(), ag_turns.len(), "turn count mismatch");
    assert_eq!(
        serde_json::to_string(&hw_turns).unwrap(),
        serde_json::to_string(&ag_turns).unwrap(),
        "turns wire mismatch"
    );
    hw_turns.len()
}

fn hw_msg(id: &str, role: musk::chats::Role, content: &str, tool_calls: Vec<musk::chats::ToolCall>, created_at: u64) -> musk::chats::ChatMessage {
    musk::chats::ChatMessage {
        id: id.into(),
        role,
        content: content.into(),
        tool_calls,
        created_at,
        thinking: String::new(),
    }
}

fn ag_msg(id: &str, role: ag::Role, content: &str, tool_calls: Vec<ag::ToolCall>, created_at: u64) -> ag::ChatMessage {
    ag::ChatMessage {
        id: id.into(),
        role,
        content: content.into(),
        tool_calls,
        created_at,
    }
}

#[test]
fn parity_chat_message_to_turns_user() {
    let hw_m = hw_msg("m1", musk::chats::Role::User, "list files", vec![], 100);
    let ag_m = ag_msg("m1", ag::Role::User, "list files", vec![], 100);
    let n = assert_conversion_parity(&hw_m, ag_m, 0);
    assert_eq!(n, 1, "user message → single main turn");
}

#[test]
fn parity_chat_message_to_turns_assistant_with_tool_calls() {
    let hw_tc = musk::chats::ToolCall {
        tool: "read_file".into(),
        args: serde_json::json!({"path": "x"}),
        result: "ok".into(),
        status: "success".into(),
        id: "".into(),
    };
    let ag_tc = ag::ToolCall {
        tool: "read_file".into(),
        args: serde_json::json!({"path": "x"}),
        result: "ok".into(),
    };
    // content + tool call → main + ToolCall + ToolResult = 3 turns.
    let hw_m = hw_msg("m2", musk::chats::Role::Assistant, "using tool", vec![hw_tc.clone()], 100);
    let ag_m = ag_msg("m2", ag::Role::Assistant, "using tool", vec![ag_tc.clone()], 100);
    assert_eq!(assert_conversion_parity(&hw_m, ag_m, 0), 3);

    // seq base offset respected identically.
    let hw_m2 = hw_msg("m2", musk::chats::Role::Assistant, "using tool", vec![hw_tc], 100);
    let ag_m2 = ag_msg("m2", ag::Role::Assistant, "using tool", vec![ag_tc], 100);
    assert_eq!(assert_conversion_parity(&hw_m2, ag_m2, 5), 3);
}

#[test]
fn parity_chat_message_to_turns_empty_content_with_tools() {
    // Empty content + tool calls → main turn skipped, only tc + tr (both sides).
    let hw_tc = musk::chats::ToolCall {
        tool: "read_file".into(),
        args: serde_json::json!({}),
        result: "ok".into(),
        status: "success".into(),
        id: "".into(),
    };
    let ag_tc = ag::ToolCall {
        tool: "read_file".into(),
        args: serde_json::json!({}),
        result: "ok".into(),
    };
    let hw_m = hw_msg("m3", musk::chats::Role::Assistant, "", vec![hw_tc], 100);
    let ag_m = ag_msg("m3", ag::Role::Assistant, "", vec![ag_tc], 100);
    assert_eq!(assert_conversion_parity(&hw_m, ag_m, 0), 2);
}

#[test]
fn parity_chat_message_to_turns_empty_message() {
    // Empty content AND no tool calls → an (empty) main turn is still pushed
    // (hand-written: `!content.is_empty() || tool_calls.is_empty()`). This is
    // the case fixed in conversation.at via two nested ifs.
    let hw_m = hw_msg("m4", musk::chats::Role::Assistant, "", vec![], 100);
    let ag_m = ag_msg("m4", ag::Role::Assistant, "", vec![], 100);
    let n = assert_conversion_parity(&hw_m, ag_m, 0);
    assert_eq!(n, 1, "empty message → single empty main turn");
}

#[test]
fn parity_tool_message_to_turns_role_mapping() {
    // Role → from mapping: User→"human", Assistant→"assistant", Tool→"system".
    for (hw_role, ag_role, expected_from) in [
        (musk::chats::Role::User, ag::Role::User, "human"),
        (musk::chats::Role::Assistant, ag::Role::Assistant, "assistant"),
        (musk::chats::Role::Tool, ag::Role::Tool, "system"),
    ] {
        let hw_m = hw_msg("m", hw_role, "x", vec![], 100);
        let ag_m = ag_msg("m", ag_role, "x", vec![], 100);
        let hw_turns = hw::chat_message_to_turns(&hw_m, 0);
        let ag_turns = ag::chat_message_to_turns(ag_m, 0);
        assert_eq!(hw_turns[0].from, expected_from);
        assert_eq!(ag_turns[0].from, expected_from);
        // to field: only user messages point to "assistant".
        assert_eq!(
            hw_turns[0].to.is_some(),
            ag_turns[0].to_role.is_some(),
            "to/to_role presence mismatch"
        );
    }
}
