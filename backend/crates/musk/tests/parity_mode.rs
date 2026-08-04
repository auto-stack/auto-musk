//! Parity tests — verify `auto_generated::mode` behaves identically to the
//! hand-written `mode` module.
//!
//! Same framework as `parity_specs.rs`: exercise the same scenarios on both
//! the hand-written Rust (`musk::mode`) and the a2r-transpiled Auto output
//! (`musk::auto_generated::mode`).
//!
//! C8 closure: `ModeRegistry::DEFAULT` is now a real associated const in the
//! transpiled module (was a `static fn default_name()` workaround before a2r
//! supported ext-block consts) — this is the headline parity item here.
//!
//! Scope limits: the hand-written `ModeRegistry.modes` field is private and the
//! only constructor is `load()` (user-dir scan — a hand-written boundary, absent
//! from the transpiled module), so populated-registry behavior can't be compared
//! head-to-head. The parity surface is: the `DEFAULT` const, empty-registry
//! behavior, and the `AgentMode` data model. `parse_mode_at`/`BUILTIN_MODES`/
//! `load` are hand-written boundaries.

use musk::mode as hw;                 // hand-written
use musk::auto_generated::mode as ag; // a2r-transpiled Auto

/// A canonical AgentMode (fixed fields) for data-model parity.
fn agent_mode_hw() -> hw::AgentMode {
    hw::AgentMode {
        name: "coding".into(),
        description: "TDD workflow".into(),
        role: "coder".into(),
        skills: true,
        tools: vec!["cargo".into(), "pytest".into()],
        workflow: Some("feature-dev".into()),
        context_file: ".musk.md".into(),
        extra_system_prompt: "Write tests first.".into(),
    }
}

fn agent_mode_ag() -> ag::AgentMode {
    ag::AgentMode {
        name: "coding".into(),
        description: "TDD workflow".into(),
        role: "coder".into(),
        skills: true,
        tools: vec!["cargo".into(), "pytest".into()],
        workflow: Some("feature-dev".into()),
        context_file: ".musk.md".into(),
        extra_system_prompt: "Write tests first.".into(),
    }
}

// ──────────────────────────────────────────────────────────
// DEFAULT const — C8 headline
// ──────────────────────────────────────────────────────────

#[test]
fn parity_default_const() {
    // Hand-written: `pub const DEFAULT: &'static str = "superpowers"`.
    // Transpiled (C8): `pub const DEFAULT: &str = "superpowers"`.
    assert_eq!(hw::ModeRegistry::DEFAULT, "superpowers");
    assert_eq!(ag::ModeRegistry::DEFAULT, "superpowers");
    assert_eq!(hw::ModeRegistry::DEFAULT, ag::ModeRegistry::DEFAULT);
}

// ──────────────────────────────────────────────────────────
// AgentMode — data model parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_agent_mode_fields() {
    let h = agent_mode_hw();
    let a = agent_mode_ag();
    assert_eq!(h.name, a.name);
    assert_eq!(h.description, a.description);
    assert_eq!(h.role, a.role);
    assert_eq!(h.skills, a.skills);
    assert_eq!(h.tools, a.tools);
    assert_eq!(h.workflow, a.workflow);
    assert_eq!(h.context_file, a.context_file);
    assert_eq!(h.extra_system_prompt, a.extra_system_prompt);
}

// ──────────────────────────────────────────────────────────
// Empty-registry behavior parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_empty_registry() {
    // Hand-written Default (empty HashMap) vs transpiled empty Vec.
    let hw_reg = hw::ModeRegistry::default();
    let ag_reg = ag::ModeRegistry { modes: vec![] };

    assert_eq!(hw_reg.names(), ag_reg.names(), "empty names() mismatch");
    assert!(hw_reg.names().is_empty());

    assert!(hw_reg.get("superpowers").is_none());
    assert!(ag_reg.get("superpowers").is_none());
}

// ──────────────────────────────────────────────────────────
// Transpiled registry behavior (ag-only — hw has no pub constructor)
// ──────────────────────────────────────────────────────────

#[test]
fn transpiled_registry_register_lookup() {
    let mut reg = ag::ModeRegistry { modes: vec![] };
    assert_eq!(reg.names(), Vec::<String>::new());
    assert!(!reg.contains("coding"));

    reg.register(agent_mode_ag());
    reg.register(ag::AgentMode {
        name: "review".into(),
        description: "Review mode".into(),
        role: "reviewer".into(),
        skills: false,
        tools: vec![],
        workflow: None,
        context_file: String::new(),
        extra_system_prompt: String::new(),
    });

    // names() is sorted; contains/get resolve; get returns a clone.
    assert_eq!(reg.names(), vec!["coding".to_string(), "review".to_string()]);
    assert!(reg.contains("coding"));
    assert!(reg.contains("review"));
    assert!(!reg.contains("basic"));
    let m = reg.get("coding").expect("registered mode found");
    assert_eq!(m.name, "coding");
    assert_eq!(m.role, "coder");

    // Same-name register overrides in place (no duplicate).
    reg.register(ag::AgentMode {
        name: "coding".into(),
        description: "Updated".into(),
        role: "coder".into(),
        skills: false,
        tools: vec![],
        workflow: None,
        context_file: String::new(),
        extra_system_prompt: String::new(),
    });
    assert_eq!(reg.names(), vec!["coding".to_string(), "review".to_string()]);
    assert_eq!(reg.get("coding").unwrap().description, "Updated");
}
