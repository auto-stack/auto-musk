//! Parity tests — verify `auto_generated::chats` behaves identically to the
//! hand-written `chats` module.
//!
//! Same framework as `parity_specs.rs`: exercise the same scenarios on both
//! the hand-written Rust (`musk::chats`) and the a2r-transpiled Auto output
//! (`musk::auto_generated::chats`).
//!
//! Scope: the transpiled module contains the data model (Role / ToolCall /
//! ChatMessage / ChatSession / ChatSessionSummary) + the pure methods
//! (`ChatMessage::user/assistant`, `ChatSession::new/summary/push_message`) +
//! a self-contained `SpecChange` mirror. Since C1 (chats CRUD 移植) the ag
//! `ChatStore` carries 10 of the 11 CRUD methods (create/list/get/rename/
//! delete/delete_all/append_message/queue_spec_change/reject_spec_change/
//! reject_all_spec_changes); `approve_spec_change` is deferred (it needs to
//! apply the change into the real specs doc — cross-module `&mut` call gap,
//! see plan 018 §12). Store IO parity tests are in the "ChatStore CRUD" section
//! below.
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
        pending_spec_changes: vec![musk::auto_generated::specs::SpecChange {
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

// ──────────────────────────────────────────────────────────
// ChatStore CRUD parity (C1: chats CRUD 移植后, 10/11 方法)
// ag 与 hw 各开一个独立 temp 文件;create/list/get/rename/delete/delete_all/
// append_message/queue/reject/reject_all 行为对齐。approve_spec_change 挂起
// (需跨模块 &mut 调用 ag specs store,见 plan 018 §12)。
// ──────────────────────────────────────────────────────────

#[test]
fn parity_store_create_and_get() {
    let hw_dir = tempfile::tempdir().unwrap();
    let ag_dir = tempfile::tempdir().unwrap();
    let hw_store = hw::ChatStore::at(hw_dir.path().join("chats.json"));
    let ag_store = ag::ChatStore::at(ag_dir.path().join("chats.json"));

    let hw_s = hw_store.create("superpowers", None).unwrap();
    let ag_s = ag_store.create("superpowers", None).unwrap();
    assert_eq!(hw_s.mode, ag_s.mode);
    assert_eq!(hw_s.name, ag_s.name);
    assert!(hw_s.messages.is_empty());
    assert!(ag_s.messages.is_empty());

    // workspace_id passthrough on both.
    let hw_s2 = hw_store.create("coding", Some("ws-1".into())).unwrap();
    let ag_s2 = ag_store.create("coding", Some("ws-1".into())).unwrap();
    assert_eq!(hw_s2.workspace_id.as_deref(), Some("ws-1"));
    assert_eq!(ag_s2.workspace_id.as_deref(), Some("ws-1"));

    // get round-trips the session; missing id → None on both.
    let hw_loaded = hw_store.get(&hw_s.id).unwrap();
    let ag_loaded = ag_store.get(&ag_s.id).unwrap();
    assert_eq!(hw_loaded.id, hw_s.id);
    assert_eq!(ag_loaded.id, ag_s.id);
    assert_eq!(hw_loaded.name, ag_loaded.name);
    assert!(hw_store.get("nope").is_none());
    assert!(ag_store.get("nope").is_none());
}

#[test]
fn parity_store_list_contains_all_sessions() {
    let hw_dir = tempfile::tempdir().unwrap();
    let ag_dir = tempfile::tempdir().unwrap();
    let hw_store = hw::ChatStore::at(hw_dir.path().join("chats.json"));
    let ag_store = ag::ChatStore::at(ag_dir.path().join("chats.json"));

    let hw_a = hw_store.create("superpowers", None).unwrap();
    let hw_b = hw_store.create("coding", None).unwrap();
    let ag_a = ag_store.create("superpowers", None).unwrap();
    let ag_b = ag_store.create("coding", None).unwrap();

    let hw_list = hw_store.list();
    let ag_list = ag_store.list();
    assert_eq!(hw_list.len(), ag_list.len());
    assert_eq!(hw_list.len(), 2);

    // ids 不可跨 store 比较:hw new_id(12) 产 24 位 hex,ag extern_impl 的 new_id
    // stub 无视参数恒产 16 位(extern stub 既有差异)。只比较结构:
    // 各自 2 个 session,id 互异,(name, mode) 集合一致。
    let mut hw_ids: Vec<&str> = hw_list.iter().map(|s| s.id.as_str()).collect();
    let mut ag_ids: Vec<&str> = ag_list.iter().map(|s| s.id.as_str()).collect();
    hw_ids.sort();
    ag_ids.sort();
    assert_eq!(hw_ids.len(), 2);
    assert_eq!(ag_ids.len(), 2);
    assert!(hw_ids[0] != hw_ids[1], "hw ids must be distinct");
    assert!(ag_ids[0] != ag_ids[1], "ag ids must be distinct");

    let mut hw_names: Vec<String> = hw_list.iter().map(|s| s.name.clone()).collect();
    let mut ag_names: Vec<String> = ag_list.iter().map(|s| s.name.clone()).collect();
    hw_names.sort();
    ag_names.sort();
    assert_eq!(hw_names, ag_names);

    // Both lists are newest-first (non-increasing updated_at).
    for w in hw_list.windows(2) {
        assert!(w[0].updated_at >= w[1].updated_at, "hw list not sorted desc");
    }
    for w in ag_list.windows(2) {
        assert!(w[0].updated_at >= w[1].updated_at, "ag list not sorted desc");
    }

    // mode values agree (as a sorted set).
    let mut hw_modes: Vec<String> = hw_list.iter().map(|s| s.mode.clone()).collect();
    let mut ag_modes: Vec<String> = ag_list.iter().map(|s| s.mode.clone()).collect();
    hw_modes.sort();
    ag_modes.sort();
    assert_eq!(hw_modes, ag_modes);
}

#[test]
fn parity_store_rename_persists() {
    let hw_dir = tempfile::tempdir().unwrap();
    let ag_dir = tempfile::tempdir().unwrap();
    let hw_store = hw::ChatStore::at(hw_dir.path().join("chats.json"));
    let ag_store = ag::ChatStore::at(ag_dir.path().join("chats.json"));

    let hw_s = hw_store.create("x", None).unwrap();
    let ag_s = ag_store.create("x", None).unwrap();

    let hw_r = hw_store.rename(&hw_s.id, "My task").unwrap().unwrap();
    let ag_r = ag_store.rename(&ag_s.id, "My task").unwrap().unwrap();
    assert_eq!(hw_r.name, "My task");
    assert_eq!(ag_r.name, "My task");

    // Persists across reload.
    let hw_rt = hw_store.get(&hw_s.id).unwrap();
    let ag_rt = ag_store.get(&ag_s.id).unwrap();
    assert_eq!(hw_rt.name, "My task");
    assert_eq!(ag_rt.name, "My task");

    // Missing id → Ok(None) on both.
    assert!(hw_store.rename("nope", "x").unwrap().is_none());
    assert!(ag_store.rename("nope", "x").unwrap().is_none());
}

#[test]
fn parity_store_delete_and_delete_all() {
    let hw_dir = tempfile::tempdir().unwrap();
    let ag_dir = tempfile::tempdir().unwrap();
    let hw_store = hw::ChatStore::at(hw_dir.path().join("chats.json"));
    let ag_store = ag::ChatStore::at(ag_dir.path().join("chats.json"));

    let hw_a = hw_store.create("x", None).unwrap();
    let hw_b = hw_store.create("y", None).unwrap();
    let ag_a = ag_store.create("x", None).unwrap();
    let ag_b = ag_store.create("y", None).unwrap();

    // delete removes exactly one session; second delete → false.
    assert!(hw_store.delete(&hw_a.id).unwrap());
    assert!(ag_store.delete(&ag_a.id).unwrap());
    assert!(!hw_store.delete(&hw_a.id).unwrap());
    assert!(!ag_store.delete(&ag_a.id).unwrap());
    assert!(hw_store.get(&hw_a.id).is_none());
    assert!(ag_store.get(&ag_a.id).is_none());

    // delete_all clears the rest.
    hw_store.delete_all().unwrap();
    ag_store.delete_all().unwrap();
    assert!(hw_store.list().is_empty());
    assert!(ag_store.list().is_empty());

    // delete on a missing id is Ok(false), not an error, on both.
    assert!(!hw_store.delete("ghost").unwrap());
    assert!(!ag_store.delete("ghost").unwrap());
}

#[test]
fn parity_store_append_message_autonames_and_persists() {
    let hw_dir = tempfile::tempdir().unwrap();
    let ag_dir = tempfile::tempdir().unwrap();
    let hw_store = hw::ChatStore::at(hw_dir.path().join("chats.json"));
    let ag_store = ag::ChatStore::at(ag_dir.path().join("chats.json"));

    let hw_s = hw_store.create("superpowers", None).unwrap();
    let ag_s = ag_store.create("superpowers", None).unwrap();

    let hw_upd = hw_store
        .append_message(&hw_s.id, hw::ChatMessage::user("List the files"))
        .unwrap()
        .unwrap();
    let ag_upd = ag_store
        .append_message(&ag_s.id, ag::ChatMessage::user("List the files"))
        .unwrap()
        .unwrap();
    assert_eq!(hw_upd.messages.len(), 1);
    assert_eq!(ag_upd.messages.len(), 1);
    assert_eq!(hw_upd.name, "List the files");
    assert_eq!(ag_upd.name, "List the files");
    assert_eq!(hw_upd.summary().preview, ag_upd.summary().preview);

    // Persists across reload.
    let hw_rt = hw_store.get(&hw_s.id).unwrap();
    let ag_rt = ag_store.get(&ag_s.id).unwrap();
    assert_eq!(hw_rt.messages.len(), 1);
    assert_eq!(ag_rt.messages.len(), 1);
    assert_eq!(hw_rt.name, ag_rt.name);

    // Missing id → Ok(None) on both.
    assert!(hw_store
        .append_message("nope", hw::ChatMessage::user("x"))
        .unwrap()
        .is_none());
    assert!(ag_store
        .append_message("nope", ag::ChatMessage::user("x"))
        .unwrap()
        .is_none());
}

#[test]
fn parity_store_queue_and_reject_spec_change() {
    let hw_dir = tempfile::tempdir().unwrap();
    let ag_dir = tempfile::tempdir().unwrap();
    let hw_store = hw::ChatStore::at(hw_dir.path().join("chats.json"));
    let ag_store = ag::ChatStore::at(ag_dir.path().join("chats.json"));

    let hw_s = hw_store.create("superpowers", None).unwrap();
    let ag_s = ag_store.create("superpowers", None).unwrap();

    let hw_change = musk::specs::SpecChange {
        section_id: "goals".into(),
        item_id: "G1".into(),
        title: Some("new goal".into()),
        content: None,
        status: None,
        reason: "agent proposal".into(),
    };
    let ag_change = musk::auto_generated::specs::SpecChange {
        section_id: "goals".into(),
        item_id: "G1".into(),
        title: Some("new goal".into()),
        content: None,
        status: None,
        reason: "agent proposal".into(),
    };
    let hw_upd = hw_store.queue_spec_change(&hw_s.id, hw_change).unwrap().unwrap();
    let ag_upd = ag_store.queue_spec_change(&ag_s.id, ag_change).unwrap().unwrap();
    assert_eq!(hw_upd.pending_spec_changes.len(), 1);
    assert_eq!(ag_upd.pending_spec_changes.len(), 1);
    assert_eq!(hw_upd.pending_spec_changes[0].item_id, "G1");
    assert_eq!(ag_upd.pending_spec_changes[0].item_id, "G1");

    // Reject at index 0 → queue empty on both.
    let hw_after = hw_store.reject_spec_change(&hw_s.id, 0).unwrap().unwrap();
    let ag_after = ag_store.reject_spec_change(&ag_s.id, 0).unwrap().unwrap();
    assert!(hw_after.pending_spec_changes.is_empty());
    assert!(ag_after.pending_spec_changes.is_empty());

    // Out-of-range index and missing session → Err on both.
    assert!(hw_store.reject_spec_change(&hw_s.id, 5).is_err());
    assert!(ag_store.reject_spec_change(&ag_s.id, 5).is_err());
    assert!(hw_store.reject_spec_change("ghost", 0).is_err());
    assert!(ag_store.reject_spec_change("ghost", 0).is_err());
}

#[test]
fn parity_store_reject_all_spec_changes() {
    let hw_dir = tempfile::tempdir().unwrap();
    let ag_dir = tempfile::tempdir().unwrap();
    let hw_store = hw::ChatStore::at(hw_dir.path().join("chats.json"));
    let ag_store = ag::ChatStore::at(ag_dir.path().join("chats.json"));

    let hw_s = hw_store.create("superpowers", None).unwrap();
    let ag_s = ag_store.create("superpowers", None).unwrap();

    for i in 0..2 {
        hw_store
            .queue_spec_change(
                &hw_s.id,
                musk::specs::SpecChange {
                    section_id: "goals".into(),
                    item_id: format!("G{i}"),
                    title: None,
                    content: None,
                    status: None,
                    reason: "x".into(),
                },
            )
            .unwrap();
        ag_store
            .queue_spec_change(
                &ag_s.id,
                musk::auto_generated::specs::SpecChange {
                    section_id: "goals".into(),
                    item_id: format!("G{i}"),
                    title: None,
                    content: None,
                    status: None,
                    reason: "x".into(),
                },
            )
            .unwrap();
    }
    assert_eq!(
        hw_store.get(&hw_s.id).unwrap().pending_spec_changes.len(),
        ag_store.get(&ag_s.id).unwrap().pending_spec_changes.len(),
    );

    let hw_after = hw_store.reject_all_spec_changes(&hw_s.id).unwrap().unwrap();
    let ag_after = ag_store.reject_all_spec_changes(&ag_s.id).unwrap().unwrap();
    assert!(hw_after.pending_spec_changes.is_empty());
    assert!(ag_after.pending_spec_changes.is_empty());

    // Missing session → Err on both.
    assert!(hw_store.reject_all_spec_changes("ghost").is_err());
    assert!(ag_store.reject_all_spec_changes("ghost").is_err());
}

/// approve_spec_change 对齐:第 11 个方法(本切片完成)。
/// 状态迁移路径 + upsert 路径 + 错误路径,两边逐一比对。
#[test]
fn parity_store_approve_spec_change() {
    let hw_dir = tempfile::tempdir().unwrap();
    let ag_dir = tempfile::tempdir().unwrap();
    let hw_store = hw::ChatStore::at(hw_dir.path().join("chats.json"));
    let ag_store = ag::ChatStore::at(ag_dir.path().join("chats.json"));
    let hw_specs = musk::specs::SpecsStore::new(hw_dir.path().join("specs.json"));
    let ag_specs = musk::auto_generated::specs::SpecsStore::new(ag_dir.path().join("specs.json"));

    // Seed a goal at Empty in both spec docs.
    let mut hw_doc = hw_specs.load().unwrap();
    hw_specs
        .upsert_item(&mut hw_doc, "goals", musk::specs::SpecItem::new("G1", "g"))
        .unwrap();
    hw_specs.save(&hw_doc).unwrap();
    let mut ag_doc = ag_specs.load().unwrap();
    ag_specs
        .upsert_item(&mut ag_doc, "goals", musk::auto_generated::specs::SpecItem::new("G1", "g"))
        .unwrap();
    ag_specs.save(ag_doc.clone()).unwrap();

    let hw_s = hw_store.create("superpowers", None).unwrap();
    let ag_s = ag_store.create("superpowers", None).unwrap();

    // ── status-transition path: queue Empty -> Proposed (legal for Goals) ──
    hw_store
        .queue_spec_change(
            &hw_s.id,
            musk::specs::SpecChange {
                section_id: "goals".into(),
                item_id: "G1".into(),
                title: None,
                content: None,
                status: Some(musk::specs::SpecStatus::Proposed),
                reason: "advance".into(),
            },
        )
        .unwrap();
    ag_store
        .queue_spec_change(
            &ag_s.id,
            musk::auto_generated::specs::SpecChange {
                section_id: "goals".into(),
                item_id: "G1".into(),
                title: None,
                content: None,
                status: Some(musk::auto_generated::specs::SpecStatus::Proposed),
                reason: "advance".into(),
            },
        )
        .unwrap();

    let (hw_change, hw_session) = hw_store
        .approve_spec_change(&hw_s.id, 0, &hw_specs)
        .unwrap()
        .unwrap();
    let (ag_change, ag_session) = ag_store
        .approve_spec_change(&ag_s.id, 0, ag_specs.clone())
        .unwrap()
        .unwrap();
    assert_eq!(hw_change.item_id, "G1");
    assert_eq!(ag_change.item_id, "G1");
    assert_eq!(hw_change.section_id, ag_change.section_id);
    assert!(hw_session.pending_spec_changes.is_empty(), "hw queue must drain");
    assert!(ag_session.pending_spec_changes.is_empty(), "ag queue must drain");

    // The status landed in the spec doc on both.
    let hw_doc = hw_specs.load().unwrap();
    let ag_doc = ag_specs.load().unwrap();
    let hw_g = hw_doc
        .sections
        .iter()
        .find(|x| x.id == "goals")
        .unwrap()
        .items
        .iter()
        .find(|i| i.id == "G1")
        .unwrap();
    let ag_g = ag_doc
        .sections
        .iter()
        .find(|x| x.id == "goals")
        .unwrap()
        .items
        .iter()
        .find(|i| i.id == "G1")
        .unwrap();
    assert_eq!(hw_g.status.to_str(), "proposed");
    assert_eq!(ag_g.status.to_str(), "proposed");

    // ── upsert path: title-only change lands in the doc ──
    hw_store
        .queue_spec_change(
            &hw_s.id,
            musk::specs::SpecChange {
                section_id: "goals".into(),
                item_id: "G1".into(),
                title: Some("approved goal".into()),
                content: None,
                status: None,
                reason: "agent proposal".into(),
            },
        )
        .unwrap();
    ag_store
        .queue_spec_change(
            &ag_s.id,
            musk::auto_generated::specs::SpecChange {
                section_id: "goals".into(),
                item_id: "G1".into(),
                title: Some("approved goal".into()),
                content: None,
                status: None,
                reason: "agent proposal".into(),
            },
        )
        .unwrap();
    hw_store.approve_spec_change(&hw_s.id, 0, &hw_specs).unwrap().unwrap();
    ag_store
        .approve_spec_change(&ag_s.id, 0, ag_specs.clone())
        .unwrap()
        .unwrap();

    let hw_doc2 = hw_specs.load().unwrap();
    let ag_doc2 = ag_specs.load().unwrap();
    let hw_g2 = hw_doc2
        .sections
        .iter()
        .find(|x| x.id == "goals")
        .unwrap()
        .items
        .iter()
        .find(|i| i.id == "G1")
        .unwrap();
    let ag_g2 = ag_doc2
        .sections
        .iter()
        .find(|x| x.id == "goals")
        .unwrap()
        .items
        .iter()
        .find(|i| i.id == "G1")
        .unwrap();
    assert_eq!(hw_g2.title, "approved goal");
    assert_eq!(ag_g2.title, "approved goal");

    // ── error paths ──
    // Out-of-range index → Err on both.
    assert!(hw_store.approve_spec_change(&hw_s.id, 5, &hw_specs).is_err());
    assert!(ag_store
        .approve_spec_change(&ag_s.id, 5, ag_specs.clone())
        .is_err());
    // Missing session → Err on both.
    assert!(hw_store.approve_spec_change("ghost", 0, &hw_specs).is_err());
    assert!(ag_store
        .approve_spec_change("ghost", 0, ag_specs.clone())
        .is_err());
}
