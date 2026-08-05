//! Parity tests — verify `auto_generated::handoff_store` behaves identically to
//! the hand-written `relay::handoff_store` (Plan 018 Phase 3.5).
//!
//! Scope: HandoffStore (data_dir + Mutex cache) save / load / resolve_path /
//! save_from_run. Known deviations:
//! - `new` takes `PathBuf` (hw: `impl Into<PathBuf>`, C6 已知退化).
//! - `save` returns `Result<bool, String>` (hw: `Result<(), String>` — a2r 无法
//!   表达 unit 类型, bool 载荷承载).
//! - cache key 两边均为 `(String, String, String)` tuple (§14 W1 闭环后 ag 恢复
//!   hw 同构 tuple key, 去字符串拼接变通).

use musk::relay::handoff_store as hw;      // hand-written
use musk::auto_generated::handoff_store as ag; // a2r-transpiled Auto
use musk::relay::HandoffDocument;
use musk::relay::store::{RunStore, StartRunRequest, StartRunStep};

use serde_json::Value;
use tempfile::TempDir;

/// Build a RunStore with one completed step whose handoff is seeded, so that
/// `last_handoff(run_id)` returns Some. Mirrors how a real relay run leaves a
/// handoff behind for the next agent. Requires `advance` (to mark a step
/// running) before `submit_handoff` will record into step_history.
fn run_store_with_handoff(handoff: HandoffDocument) -> (tempfile::TempDir, RunStore, String) {
    let dir = tempfile::TempDir::new().unwrap();
    let store = RunStore::at(dir.path().to_path_buf());
    let req = StartRunRequest {
        run_id: Some("run-x".to_string()),
        flow_id: None,
        steps: vec![
            StartRunStep { id: "s1".to_string(), role_id: "coder".to_string(), gate: None },
            StartRunStep { id: "s2".to_string(), role_id: "tester".to_string(), gate: None },
        ],
        task: Some("do thing".to_string()),
    };
    let (run_id, _) = store.start_run(&req, None);
    let _ = store.advance(&run_id); // mark s1 running
    store.submit_handoff(&run_id, handoff);
    (dir, store, run_id)
}

// ──────────────────────────────────────────────────────────
// save / load — 行为 parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_save_and_load_handoff() {
    let hw_dir = TempDir::new().unwrap();
    let ag_dir = TempDir::new().unwrap();
    let hw_store = hw::HandoffStore::new(hw_dir.path());
    let ag_store = ag::HandoffStore::new(ag_dir.path().to_path_buf());

    let hw_handoff = HandoffDocument::new("coder", "tester");
    let ag_handoff = HandoffDocument::new("coder", "tester");

    hw_store.save("tp", "phase", "run", &hw_handoff).unwrap();
    assert!(ag_store.save("tp", "phase", "run", ag_handoff).unwrap());

    let hw_loaded = hw_store.load("tp", "phase", "run").unwrap();
    let ag_loaded = ag_store.load("tp", "phase", "run").unwrap();
    assert_eq!(hw_loaded.from, "coder");
    assert_eq!(ag_loaded.from, "coder");
    assert_eq!(hw_loaded.to, "tester");
    assert_eq!(ag_loaded.to, "tester");
}

#[test]
fn parity_missing_handoff_returns_none() {
    let hw_dir = TempDir::new().unwrap();
    let ag_dir = TempDir::new().unwrap();
    let hw_store = hw::HandoffStore::new(hw_dir.path());
    let ag_store = ag::HandoffStore::new(ag_dir.path().to_path_buf());

    assert!(hw_store.load("tp", "phase", "run").is_none());
    assert!(ag_store.load("tp", "phase", "run").is_none());
}

#[test]
fn parity_load_after_reload_from_disk() {
    // After save, a NEW store instance (same dir) must read from disk —
    // proving the file round-trip (cache is per-instance).
    let dir = TempDir::new().unwrap();
    {
        let store = ag::HandoffStore::new(dir.path().to_path_buf());
        assert!(store
            .save("tp", "phase", "run", HandoffDocument::new("coder", "tester"))
            .unwrap());
    }
    let store2 = ag::HandoffStore::new(dir.path().to_path_buf());
    let loaded = store2.load("tp", "phase", "run").unwrap();
    assert_eq!(loaded.from, "coder");
    assert_eq!(loaded.to, "tester");
}

// ──────────────────────────────────────────────────────────
// resolve_path — 行为 parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_resolve_path_summary_and_nested_token() {
    let hw_dir = TempDir::new().unwrap();
    let ag_dir = TempDir::new().unwrap();
    let hw_store = hw::HandoffStore::new(hw_dir.path());
    let ag_store = ag::HandoffStore::new(ag_dir.path().to_path_buf());

    let mut hw_handoff = HandoffDocument::new("coder", "tester");
    hw_handoff.summary = "Implemented auth".to_string();
    hw_handoff.token_usage.step_tokens = 100;
    hw_handoff.token_usage.cumulative = 150;
    hw_store.save("tp", "phase", "run", &hw_handoff).unwrap();

    let mut ag_handoff = HandoffDocument::new("coder", "tester");
    ag_handoff.summary = "Implemented auth".to_string();
    ag_handoff.token_usage.step_tokens = 100;
    ag_handoff.token_usage.cumulative = 150;
    ag_store.save("tp", "phase", "run", ag_handoff).unwrap();

    let hw_summary = hw_store.resolve_path("tp.phase.run.handoff.summary");
    let ag_summary = ag_store.resolve_path("tp.phase.run.handoff.summary");
    assert_eq!(hw_summary, Some(Value::String("Implemented auth".to_string())));
    assert_eq!(ag_summary, hw_summary, "summary wire mismatch");

    let hw_cum = hw_store.resolve_path("tp.phase.run.handoff.token_usage.cumulative");
    let ag_cum = ag_store.resolve_path("tp.phase.run.handoff.token_usage.cumulative");
    assert_eq!(hw_cum, Some(Value::Number(150.into())));
    assert_eq!(ag_cum, hw_cum, "token_usage wire mismatch");
}

#[test]
fn parity_resolve_path_missing_field_returns_none() {
    let dir = TempDir::new().unwrap();
    let store = ag::HandoffStore::new(dir.path().to_path_buf());
    assert!(store
        .save("tp", "phase", "run", HandoffDocument::new("coder", "tester"))
        .unwrap());
    // Field that doesn't exist in the handoff JSON.
    assert!(store
        .resolve_path("tp.phase.run.handoff.no_such_field")
        .is_none());
}

#[test]
fn parity_rejects_path_without_handoff_segment() {
    let hw_dir = TempDir::new().unwrap();
    let ag_dir = TempDir::new().unwrap();
    let hw_store = hw::HandoffStore::new(hw_dir.path());
    let ag_store = ag::HandoffStore::new(ag_dir.path().to_path_buf());
    // Third segment is not "handoff" → None (both).
    assert!(hw_store.resolve_path("tp.phase.run.output.summary").is_none());
    assert!(ag_store.resolve_path("tp.phase.run.output.summary").is_none());
}

#[test]
fn parity_rejects_too_short_path() {
    let dir = TempDir::new().unwrap();
    let store = ag::HandoffStore::new(dir.path().to_path_buf());
    // < 5 segments → None.
    assert!(store.resolve_path("tp.phase.run").is_none());
    assert!(store.resolve_path("tp.phase.run.handoff").is_none());
}

// ──────────────────────────────────────────────────────────
// save_from_run — 复审修复: ag 版此前只 load 不 save(休眠分歧)。
// 现已对齐 hw: load → self.save 持久化 → 返回。验证持久化真实生效。
// ──────────────────────────────────────────────────────────

#[test]
fn parity_save_from_run_persists_to_disk() {
    let handoff = HandoffDocument::new("coder", "tester");

    // hw 版: save_from_run 持久化到 hw_dir。
    let (hw_dir, hw_run, hw_run_id) = run_store_with_handoff(handoff.clone());
    let hw_store = hw::HandoffStore::new(hw_dir.path());
    let hw_result = hw_store.save_from_run(&hw_run, "tp", "phase", "run", &hw_run_id);
    assert!(hw_result.is_some(), "hw save_from_run returns the handoff");
    // hw 落盘后, 一个全新 store 实例(同目录)能从磁盘读回 → 证明持久化。
    let hw_store2 = hw::HandoffStore::new(hw_dir.path());
    assert!(hw_store2.load("tp", "phase", "run").is_some(), "hw persisted to disk");

    // ag 版: 同样应持久化(修复前不会落盘, load 返回 None)。
    let (ag_dir, ag_run, ag_run_id) = run_store_with_handoff(handoff);
    let ag_store = ag::HandoffStore::new(ag_dir.path().to_path_buf());
    let ag_result = ag_store.save_from_run(&ag_run, "tp", "phase", "run", &ag_run_id);
    assert_eq!(ag_result.map(|h| h.from), Some("coder".to_string()), "ag save_from_run returns the handoff");
    // ag 落盘后, 全新实例能读回 → 修复后行为与 hw 一致。
    let ag_store2 = ag::HandoffStore::new(ag_dir.path().to_path_buf());
    let ag_loaded = ag_store2.load("tp", "phase", "run");
    assert_eq!(ag_loaded.map(|h| h.from), Some("coder".to_string()), "ag now persists (was: not persisted)");
}

#[test]
fn parity_save_from_run_none_when_no_handoff() {
    // RunStore 没有 handoff 时, 两边都返回 None。
    let dir = tempfile::TempDir::new().unwrap();
    let run = RunStore::at(dir.path().to_path_buf());
    let req = StartRunRequest {
        run_id: Some("run-empty".to_string()),
        flow_id: None,
        steps: vec![StartRunStep { id: "s1".to_string(), role_id: "coder".to_string(), gate: None }],
        task: Some("t".to_string()),
    };
    let (run_id, _) = run.start_run(&req, None);
    // 不 submit_handoff → last_handoff 返回 None。

    let hw_store = hw::HandoffStore::new(dir.path());
    let ag_store = ag::HandoffStore::new(dir.path().to_path_buf());
    assert!(hw_store.save_from_run(&run, "tp", "phase", "run", &run_id).is_none());
    assert!(ag_store.save_from_run(&run, "tp", "phase", "run", &run_id).is_none());
}
