//! Parity tests — verify `auto_generated::chats` behaves identically to the
//! hand-written `chats` module.
//!
//! Same framework as `parity_specs.rs`: exercise the same scenarios on both
//! the hand-written Rust (`musk::chats`) and the a2r-transpiled Auto output
//! (`musk::auto_generated::chats`).
//!
//! Scope: the transpiled module contains the data model (Role / ToolCall /
//! ChatMessage / ChatSession / ChatSessionSummary) + the pure methods
//! (`ChatMessage::user/assistant`, `ChatSession::new/summary/append`) + a
//! self-contained `SpecChange` mirror. The 11 `ChatStore` CRUD methods and
//! `new_id`/`now_sec` are hand-written concerns — the ag `ChatStore` only
//! carries `at`/`load_map`/`save_map`, so store IO is excluded here.
//!
//! The C4 closure aligned the serde attributes (`rename`/`alias`/`default`/
//! `skip_serializing_if`) and added `ToolCall.status/id`; these tests pin that
//! the wire format and the deserialization fallbacks match exactly.

use musk::chats as hw;                 // hand-written
use musk::auto_generated::chats as ag; // a2r-transpiled Auto

// ──────────────────────────────────────────────────────────
// Role — wire format parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_role_wire_format() {
    for (hw_r, ag_r, expected) in [
        (hw::Role::User, ag::Role::User, "\"user\""),
        (hw::Role::Assistant, ag::Role::Assistant, "\"assistant\""),
        (hw::Role::Tool, ag::Role::Tool, "\"tool\""),
    ] {
        assert_eq!(serde_json::to_string(&hw_r).unwrap(), expected);
        assert_eq!(serde_json::to_string(&ag_r).unwrap(), expected);
    }
}

// ──────────────────────────────────────────────────────────
// ToolCall — wire + fallback parity (C4 targets)
// ──────────────────────────────────────────────────────────

#[test]
fn parity_tool_call_wire_format() {
    // Fully-populated: status + id present, so nothing is skipped.
    let hw_tc = hw::ToolCall {
        tool: "read_file".into(),
        args: serde_json::json!({"path": "src/main.rs"}),
        result: "ok".into(),
        status: "error".into(),
        id: "tc-1".into(),
    };
    let ag_tc = ag::ToolCall {
        tool: "read_file".into(),
        args: serde_json::json!({"path": "src/main.rs"}),
        result: "ok".into(),
        status: "error".into(),
        id: "tc-1".into(),
    };
    assert_eq!(
        serde_json::to_string(&hw_tc).unwrap(),
        serde_json::to_string(&ag_tc).unwrap(),
        "full ToolCall wire mismatch"
    );

    // Defaults: status = "success", id = "" → both skipped on the wire.
    let hw_def = hw::ToolCall {
        tool: "read_file".into(),
        args: serde_json::json!({}),
        result: "".into(),
        status: "success".into(),
        id: "".into(),
    };
    let ag_def = ag::ToolCall {
        tool: "read_file".into(),
        args: serde_json::json!({}),
        result: "".into(),
        status: "success".into(),
        id: "".into(),
    };
    let hw_json = serde_json::to_string(&hw_def).unwrap();
    let ag_json = serde_json::to_string(&ag_def).unwrap();
    assert_eq!(hw_json, ag_json, "default ToolCall wire mismatch");
    assert!(!hw_json.contains("status") && !hw_json.contains("id"), "defaults must be skipped");
}

#[test]
fn parity_tool_call_alias_deserialization() {
    // Legacy persisted rows used `tool`/`args` — the alias must map them back.
    let legacy = r#"{"tool": "read_file", "args": {"path": "x"}, "result": "ok"}"#;
    let hw_tc: hw::ToolCall = serde_json::from_str(legacy).unwrap();
    let ag_tc: ag::ToolCall = serde_json::from_str(legacy).unwrap();
    assert_eq!(hw_tc.tool, ag_tc.tool);
    assert_eq!(hw_tc.args, ag_tc.args);
    assert_eq!(hw_tc.result, ag_tc.result);
    assert_eq!(hw_tc.status, ag_tc.status, "missing status should default");
    assert_eq!(hw_tc.id, ag_tc.id, "missing id should default");

    // Current-format rows (`name`/`arguments`) work on both too.
    let current = r#"{"name": "write_file", "arguments": {"path": "y"}, "result": "saved", "status": "success", "id": "tc-2"}"#;
    let hw_c: hw::ToolCall = serde_json::from_str(current).unwrap();
    let ag_c: ag::ToolCall = serde_json::from_str(current).unwrap();
    assert_eq!(hw_c.tool, ag_c.tool);
    assert_eq!(hw_c.args, ag_c.args);
    assert_eq!(hw_c.id, ag_c.id);
}

#[test]
fn parity_tool_call_default_status_and_id() {
    // Minimal row: status defaults to "success", id to "" on both.
    let minimal = r#"{"name": "read_file", "arguments": {}, "result": ""}"#;
    let hw_tc: hw::ToolCall = serde_json::from_str(minimal).unwrap();
    let ag_tc: ag::ToolCall = serde_json::from_str(minimal).unwrap();
    assert_eq!(hw_tc.status, "success");
    assert_eq!(ag_tc.status, "success");
    assert_eq!(hw_tc.id, "");
    assert_eq!(ag_tc.id, "");
}

// ──────────────────────────────────────────────────────────
// ChatMessage / ChatSession — wire format parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_chat_message_wire_format() {
    let hw_tc = hw::ToolCall {
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
        status: "success".into(),
        id: "".into(),
    };
    let hw_msg = hw::ChatMessage {
        id: "m1".into(),
        role: hw::Role::Assistant,
        content: "using tool".into(),
        tool_calls: vec![hw_tc],
        created_at: 50,
    };
    let ag_msg = ag::ChatMessage {
        id: "m1".into(),
        role: ag::Role::Assistant,
        content: "using tool".into(),
        tool_calls: vec![ag_tc],
        created_at: 50,
    };
    assert_eq!(
        serde_json::to_string(&hw_msg).unwrap(),
        serde_json::to_string(&ag_msg).unwrap(),
        "ChatMessage with tool_calls wire mismatch"
    );

    // Empty tool_calls are skipped on the wire.
    let hw_empty = hw::ChatMessage {
        id: "m2".into(),
        role: hw::Role::User,
        content: "plain".into(),
        tool_calls: vec![],
        created_at: 51,
    };
    let ag_empty = ag::ChatMessage {
        id: "m2".into(),
        role: ag::Role::User,
        content: "plain".into(),
        tool_calls: vec![],
        created_at: 51,
    };
    let hw_json = serde_json::to_string(&hw_empty).unwrap();
    assert_eq!(hw_json, serde_json::to_string(&ag_empty).unwrap());
    assert!(!hw_json.contains("tool_calls"), "empty tool_calls must be skipped");
}

#[test]
fn parity_chat_session_wire_format() {
    let hw_tc = hw::ToolCall {
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
        status: "success".into(),
        id: "".into(),
    };
    // Fully-populated session (fixed ids/timestamps for deterministic compare).
    let hw_s = hw::ChatSession {
        id: "s1".into(),
        name: "My chat".into(),
        mode: "coding".into(),
        messages: vec![hw::ChatMessage {
            id: "m1".into(),
            role: hw::Role::User,
            content: "hi".into(),
            tool_calls: vec![hw_tc],
            created_at: 100,
        }],
        created_at: 100,
        updated_at: 200,
        pending_spec_changes: vec![musk::specs::SpecChange {
            section_id: "goals".into(),
            item_id: "G1".into(),
            title: Some("new goal".into()),
            content: None,
            status: None,
            reason: "proposed by agent".into(),
        }],
        workspace_id: Some("ws1".into()),
    };
    let ag_s = ag::ChatSession {
        id: "s1".into(),
        name: "My chat".into(),
        mode: "coding".into(),
        messages: vec![ag::ChatMessage {
            id: "m1".into(),
            role: ag::Role::User,
            content: "hi".into(),
            tool_calls: vec![ag_tc],
            created_at: 100,
        }],
        created_at: 100,
        updated_at: 200,
        pending_spec_changes: vec![ag::SpecChange {
            section_id: "goals".into(),
            item_id: "G1".into(),
            title: Some("new goal".into()),
            content: None,
            status: None,
            reason: "proposed by agent".into(),
        }],
        workspace_id: Some("ws1".into()),
    };
    assert_eq!(
        serde_json::to_string(&hw_s).unwrap(),
        serde_json::to_string(&ag_s).unwrap(),
        "full ChatSession wire mismatch"
    );

    // Minimal session: empty messages / pending / workspace → all skipped.
    let hw_min = hw::ChatSession {
        id: "s2".into(),
        name: "New chat".into(),
        mode: "superpowers".into(),
        messages: vec![],
        created_at: 0,
        updated_at: 0,
        pending_spec_changes: vec![],
        workspace_id: None,
    };
    let ag_min = ag::ChatSession {
        id: "s2".into(),
        name: "New chat".into(),
        mode: "superpowers".into(),
        messages: vec![],
        created_at: 0,
        updated_at: 0,
        pending_spec_changes: vec![],
        workspace_id: None,
    };
    let hw_json = serde_json::to_string(&hw_min).unwrap();
    assert_eq!(hw_json, serde_json::to_string(&ag_min).unwrap());
    // `messages` is always serialized (no skip attr on it); only
    // pending_spec_changes + workspace_id are skipped when empty/None.
    assert!(hw_json.contains("\"messages\":[]"));
    assert!(!hw_json.contains("pending_spec_changes"));
    assert!(!hw_json.contains("workspace_id"));
}

// ──────────────────────────────────────────────────────────
// ChatSession behavior — new / append / summary parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_session_append_and_summary() {
    let mut hw_s = hw::ChatSession::new("superpowers", None);
    let mut ag_s = ag::ChatSession::new("superpowers", None);
    assert_eq!(hw_s.name, ag_s.name, "both start as 'New chat'");

    hw_s.append(hw::ChatMessage::user("List the files in this dir"));
    ag_s.append(ag::ChatMessage::user("List the files in this dir"));
    assert_eq!(hw_s.messages.len(), ag_s.messages.len());
    assert_eq!(hw_s.name, ag_s.name, "auto-name from first user message");
    assert_eq!(hw_s.name, "List the files in this dir");
    assert_eq!(hw_s.summary().preview, ag_s.summary().preview);

    // A second user message becomes the preview; assistant msgs don't.
    hw_s.append(hw::ChatMessage::assistant("Let me look."));
    hw_s.append(hw::ChatMessage::user("Second question"));
    ag_s.append(ag::ChatMessage::assistant("Let me look."));
    ag_s.append(ag::ChatMessage::user("Second question"));
    assert_eq!(hw_s.summary().preview, ag_s.summary().preview);
    assert_eq!(hw_s.summary().preview, "Second question");
    assert_eq!(hw_s.summary().message_count, ag_s.summary().message_count as usize);
    assert_eq!(hw_s.summary().message_count, 3);
}

#[test]
fn parity_summary_preview_truncates_to_80() {
    let long = "a".repeat(100);
    let mut hw_s = hw::ChatSession::new("x", None);
    hw_s.append(hw::ChatMessage::user(long.clone()));
    let mut ag_s = ag::ChatSession::new("x", None);
    ag_s.append(ag::ChatMessage::user(&long));

    let hw_preview = hw_s.summary().preview;
    let ag_preview = ag_s.summary().preview;
    assert_eq!(hw_preview, ag_preview);
    assert_eq!(hw_preview.len(), 83); // 80 'a' bytes + 3-byte UTF-8 ellipsis "…"
    assert!(hw_preview.ends_with('…'));
    assert_eq!(&hw_preview[..80], "a".repeat(80).as_str());
}

#[test]
fn parity_autoname_truncates_to_40() {
    let long_name = "b".repeat(60);
    let mut hw_s = hw::ChatSession::new("x", None);
    hw_s.append(hw::ChatMessage::user(long_name.clone()));
    let mut ag_s = ag::ChatSession::new("x", None);
    ag_s.append(ag::ChatMessage::user(&long_name));

    assert_eq!(hw_s.name, ag_s.name);
    assert_eq!(hw_s.name, "b".repeat(40));
}
