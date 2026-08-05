//! Parity tests — verify `auto_generated::handoff_store` behaves identically to
//! the hand-written `relay::handoff_store` (Plan 018 Phase 3.5).
//!
//! Scope: HandoffStore (data_dir + Mutex cache) save / load / resolve_path /
//! save_from_run. Known deviations:
//! - cache key: hw uses `(String, String, String)` tuple, ag uses string
//!   `"tp/phase/run"` (a2r tuple-key 缺陷, cache 是私有字段,行为等价).
//! - `new` takes `PathBuf` (hw: `impl Into<PathBuf>`, C6 已知退化).
//! - `save` returns `Result<bool, String>` (hw: `Result<(), String>` — a2r 无法
//!   表达 unit 类型, bool 载荷承载).

use musk::relay::handoff_store as hw;      // hand-written
use musk::auto_generated::handoff_store as ag; // a2r-transpiled Auto
use musk::relay::HandoffDocument;

use serde_json::Value;
use tempfile::TempDir;

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
