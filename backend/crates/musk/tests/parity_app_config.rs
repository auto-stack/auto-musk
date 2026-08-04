//! Parity tests — verify `auto_generated::app_config` behaves identically to
//! the hand-written `app_config` module.
//!
//! Same framework as `parity_specs.rs`: exercise the same scenarios on both
//! the hand-written Rust (`musk::app_config`) and the a2r-transpiled Auto
//! output (`musk::auto_generated::app_config`).
//!
//! Scope: the transpiled module contains the two structs (`MuskAppConfig` /
//! `HarnessSelection`) plus the `effective_*` readers. The load/parse/serialize
//! path (`load`/`parse_from_at`/`to_at_source`) and env mutation
//! (`apply_to_env`) are hand-written boundaries (auto_atom/auto_val + env) and
//! are excluded here.
//!
//! Known divergence (documented, not a test failure): the transpiled
//! `effective_daemon_url` skips the `AAID_URL` env-var override — a2r cannot
//! express `env::var(...).ok()` (verified 2026-08-04), so env fallback stays a
//! hand-written concern.

use musk::app_config as hw;                 // hand-written
use musk::auto_generated::app_config as ag; // a2r-transpiled Auto

/// A fully-populated config (every field set) — mirrors the hand-written
/// roundtrip tests' shapes.
fn full_config_hw() -> hw::MuskAppConfig {
    hw::MuskAppConfig {
        daemon_url: Some("http://example:17654".into()),
        default_mode: Some("coding".into()),
        context_file: Some(".musk.md".into()),
        serve_addr: Some("127.0.0.1:9999".into()),
        auto_start_daemon: Some(false),
        harness: hw::HarnessSelection {
            roles: vec!["coder".into(), "architect".into()],
            skills: vec!["test-driven-development".into()],
            modes: vec!["superpowers".into()],
        },
    }
}

fn full_config_ag() -> ag::MuskAppConfig {
    ag::MuskAppConfig {
        daemon_url: Some("http://example:17654".into()),
        default_mode: Some("coding".into()),
        context_file: Some(".musk.md".into()),
        serve_addr: Some("127.0.0.1:9999".into()),
        auto_start_daemon: Some(false),
        harness: ag::HarnessSelection {
            roles: vec!["coder".into(), "architect".into()],
            skills: vec!["test-driven-development".into()],
            modes: vec!["superpowers".into()],
        },
    }
}

// ──────────────────────────────────────────────────────────
// Wire format — serde parity (C4/C5 guarantees)
// ──────────────────────────────────────────────────────────

#[test]
fn parity_serde_full_config_wire() {
    assert_eq!(
        serde_json::to_string(&full_config_hw()).unwrap(),
        serde_json::to_string(&full_config_ag()).unwrap(),
        "full config wire format mismatch"
    );
}

#[test]
fn parity_serde_default_config_wire() {
    // All-None configs (what a missing config file yields) must serialize the same.
    assert_eq!(
        serde_json::to_string(&hw::MuskAppConfig::default()).unwrap(),
        serde_json::to_string(&ag::MuskAppConfig::default()).unwrap(),
        "default config wire format mismatch"
    );
    assert_eq!(
        serde_json::to_string(&hw::HarnessSelection::default()).unwrap(),
        serde_json::to_string(&ag::HarnessSelection::default()).unwrap(),
        "default harness wire format mismatch"
    );
}

#[test]
fn parity_serde_deserialize_missing_fields_default() {
    // `{}` deserializes to all defaults in both versions (#[serde(default)]).
    let hw_cfg: hw::MuskAppConfig = serde_json::from_str("{}").unwrap();
    let ag_cfg: ag::MuskAppConfig = serde_json::from_str("{}").unwrap();
    assert_eq!(
        serde_json::to_string(&hw_cfg).unwrap(),
        serde_json::to_string(&ag_cfg).unwrap(),
        "empty-JSON defaults mismatch"
    );
    assert!(hw_cfg.daemon_url.is_none());
    assert!(ag_cfg.daemon_url.is_none());
    assert!(hw_cfg.harness.roles.is_empty());
    assert!(ag_cfg.harness.roles.is_empty());
}

#[test]
fn parity_serde_roundtrip_via_json() {
    // Full config → JSON → deserialize → JSON must agree between versions.
    let json_hw = serde_json::to_string(&full_config_hw()).unwrap();
    let json_ag = serde_json::to_string(&full_config_ag()).unwrap();
    assert_eq!(json_hw, json_ag);

    let back_hw: hw::MuskAppConfig = serde_json::from_str(&json_hw).unwrap();
    let back_ag: ag::MuskAppConfig = serde_json::from_str(&json_ag).unwrap();
    assert_eq!(
        serde_json::to_string(&back_hw).unwrap(),
        serde_json::to_string(&back_ag).unwrap(),
        "round-trip mismatch"
    );
}

// ──────────────────────────────────────────────────────────
// effective_* readers — pure logic parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_effective_default_mode() {
    assert_eq!(
        hw::MuskAppConfig::default().effective_default_mode(),
        ag::MuskAppConfig::default().effective_default_mode(),
    );
    assert_eq!(hw::MuskAppConfig::default().effective_default_mode(), "superpowers");

    let hw_c = full_config_hw();
    let ag_c = full_config_ag();
    assert_eq!(hw_c.effective_default_mode(), ag_c.effective_default_mode());
    assert_eq!(hw_c.effective_default_mode(), "coding");
}

#[test]
fn parity_effective_daemon_url() {
    // Deterministic environment: remove any AAID_URL so both sides agree on the
    // compiled default (the transpiled version never reads the env var).
    std::env::remove_var("AAID_URL");

    let hw_c = full_config_hw();
    let ag_c = full_config_ag();
    // Configured URL wins on both.
    assert_eq!(hw_c.effective_daemon_url(), ag_c.effective_daemon_url());
    assert_eq!(hw_c.effective_daemon_url(), "http://example:17654");

    // Unset → compiled default on both.
    assert_eq!(
        hw::MuskAppConfig::default().effective_daemon_url(),
        ag::MuskAppConfig::default().effective_daemon_url(),
    );
    assert_eq!(hw::MuskAppConfig::default().effective_daemon_url(), "http://127.0.0.1:17654");
}

/// Documented divergence — the transpiled `effective_daemon_url` skips the
/// `AAID_URL` env override (a2r can't express `env::var(...).ok()`; env access
/// is a plan-014 hand-written boundary). This test pins the *current* behavior
/// of each side; if a future dogfooding loop closes the gap, it should change
/// here.
#[test]
fn documented_divergence_env_override_skipped_in_ag() {
    std::env::set_var("AAID_URL", "http://env:9999");
    // Hand-written merges env AAID_URL into the effective URL...
    assert_eq!(
        hw::MuskAppConfig::default().effective_daemon_url(),
        "http://env:9999",
    );
    // ...the transpiled version falls straight back to the compiled default.
    assert_eq!(
        ag::MuskAppConfig::default().effective_daemon_url(),
        "http://127.0.0.1:17654",
    );
    std::env::remove_var("AAID_URL");
}
