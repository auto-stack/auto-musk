//! Parity tests — verify `auto_generated::task_plan` behaves identically to the
//! hand-written `relay::task_plan` (Plan 018 Phase 3.3).
//!
//! Scope: the transpiled module contains the data model (TaskPlan / Phase /
//! RunRef / TaskMode / PhaseMode), the builders (new / add_phase / validate /
//! detect_cycle) and the Atom parse path (`TaskPlan::from_node`, the a2r
//! equivalent of the hand-written `impl TryFrom<Node>`).
//!
//! Known deviation: the hand-written `TryFrom<Node>` trait impl is expressed
//! in Auto as a `static fn from_node` (a2r can't express trait impls);
//! behavior is asserted to be equivalent.

use musk::relay::task_plan as hw;      // hand-written
use musk::auto_generated::task_plan as ag; // a2r-transpiled Auto

use auto_atom::AtomParser;
use auto_val::Node;

/// Parse Atom text → Node (shared by hw/ag parse paths).
fn parse_node(src: &str) -> Node {
    let atom = AtomParser::parse(src).unwrap();
    match atom {
        auto_atom::Atom::Node(node) => node,
        other => panic!("expected node, got {:?}", other.to_value()),
    }
}

// ──────────────────────────────────────────────────────────
// TaskMode / PhaseMode — wire format + semantics parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_task_mode_wire_format() {
    for (hw_v, ag_v, expected) in [
        (hw::TaskMode::Gsd, ag::TaskMode::Gsd, "\"gsd\""),
        (hw::TaskMode::Check, ag::TaskMode::Check, "\"check\""),
    ] {
        assert_eq!(serde_json::to_string(&hw_v).unwrap(), expected);
        assert_eq!(serde_json::to_string(&ag_v).unwrap(), expected);
    }
    // Default: both pick Gsd.
    assert_eq!(hw::TaskMode::default(), hw::TaskMode::Gsd);
    assert_eq!(ag::TaskMode::default(), ag::TaskMode::Gsd);
}

#[test]
fn parity_phase_mode_wire_format() {
    for (hw_v, ag_v, expected) in [
        (hw::PhaseMode::Serial, ag::PhaseMode::Serial, "\"serial\""),
        (hw::PhaseMode::Parallel, ag::PhaseMode::Parallel, "\"parallel\""),
    ] {
        assert_eq!(serde_json::to_string(&hw_v).unwrap(), expected);
        assert_eq!(serde_json::to_string(&ag_v).unwrap(), expected);
    }
    assert_eq!(hw::PhaseMode::default(), hw::PhaseMode::Serial);
    assert_eq!(ag::PhaseMode::default(), ag::PhaseMode::Serial);
}

// ──────────────────────────────────────────────────────────
// Data model — wire format parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_task_plan_wire_format() {
    let hw_plan = hw::TaskPlan::new("api_v2").add_phase(
        hw::Phase::new("discovery").add_run(hw::RunRef::new("discover", "goal-discovery")),
    );
    let ag_plan = ag::TaskPlan::new("api_v2").add_phase(
        ag::Phase::new("discovery").add_run(ag::RunRef::new("discover", "goal-discovery")),
    );
    let hw_json = serde_json::to_value(&hw_plan).unwrap();
    let ag_json = serde_json::to_value(&ag_plan).unwrap();
    assert_eq!(hw_json, ag_json, "TaskPlan wire mismatch");
}

#[test]
fn parity_run_ref_builders_wire_format() {
    let hw_run = hw::RunRef::new("discover", "goal-discovery")
        .with_input("Build the API.")
        .with_input_from(vec!["feat.plan.handoff.summary".into()])
        .with_context("ctx")
        .with_mode_override(hw::TaskMode::Check);
    let ag_run = ag::RunRef::new("discover", "goal-discovery")
        .with_input("Build the API.")
        .with_input_from(vec!["feat.plan.handoff.summary".into()])
        .with_context("ctx")
        .with_mode_override(ag::TaskMode::Check);
    assert_eq!(
        serde_json::to_value(&hw_run).unwrap(),
        serde_json::to_value(&ag_run).unwrap(),
        "RunRef builder wire mismatch"
    );
}

// ──────────────────────────────────────────────────────────
// validate() — builder semantics parity
// ──────────────────────────────────────────────────────────

#[test]
fn parity_validate_accepts_wellformed_plan() {
    let hw_plan = hw::TaskPlan::new("x")
        .add_phase(hw::Phase::new("a").add_run(hw::RunRef::new("r1", "f")))
        .add_phase(hw::Phase::new("b").depends_on(vec!["a".into()]));
    let ag_plan = ag::TaskPlan::new("x")
        .add_phase(ag::Phase::new("a").add_run(ag::RunRef::new("r1", "f")))
        .add_phase(ag::Phase::new("b").depends_on(vec!["a".into()]));
    assert!(hw_plan.validate().is_ok());
    assert!(ag_plan.validate().is_ok());
}

#[test]
fn parity_validate_rejects_duplicate_phase_names() {
    let hw_plan = hw::TaskPlan::new("x")
        .add_phase(hw::Phase::new("a"))
        .add_phase(hw::Phase::new("a"));
    let ag_plan = ag::TaskPlan::new("x")
        .add_phase(ag::Phase::new("a"))
        .add_phase(ag::Phase::new("a"));
    assert!(hw_plan.validate().is_err());
    assert!(ag_plan.validate().is_err());
    assert_eq!(
        hw_plan.validate().unwrap_err().to_string(),
        ag_plan.validate().unwrap_err().to_string()
    );
}

#[test]
fn parity_validate_rejects_duplicate_run_names() {
    let hw_plan = hw::TaskPlan::new("x").add_phase(
        hw::Phase::new("a")
            .add_run(hw::RunRef::new("r1", "f"))
            .add_run(hw::RunRef::new("r1", "f")),
    );
    let ag_plan = ag::TaskPlan::new("x").add_phase(
        ag::Phase::new("a")
            .add_run(ag::RunRef::new("r1", "f"))
            .add_run(ag::RunRef::new("r1", "f")),
    );
    assert!(hw_plan.validate().is_err());
    assert!(ag_plan.validate().is_err());
    assert_eq!(
        hw_plan.validate().unwrap_err().to_string(),
        ag_plan.validate().unwrap_err().to_string()
    );
}

#[test]
fn parity_validate_rejects_unknown_dependency() {
    let hw_plan = hw::TaskPlan::new("x")
        .add_phase(hw::Phase::new("a").depends_on(vec!["missing".into()]));
    let ag_plan = ag::TaskPlan::new("x")
        .add_phase(ag::Phase::new("a").depends_on(vec!["missing".into()]));
    assert!(hw_plan.validate().is_err());
    assert!(ag_plan.validate().is_err());
    assert_eq!(
        hw_plan.validate().unwrap_err().to_string(),
        ag_plan.validate().unwrap_err().to_string()
    );
}

#[test]
fn parity_validate_rejects_cycle() {
    let hw_plan = hw::TaskPlan::new("x")
        .add_phase(hw::Phase::new("a").depends_on(vec!["b".into()]))
        .add_phase(hw::Phase::new("b").depends_on(vec!["a".into()]));
    let ag_plan = ag::TaskPlan::new("x")
        .add_phase(ag::Phase::new("a").depends_on(vec!["b".into()]))
        .add_phase(ag::Phase::new("b").depends_on(vec!["a".into()]));
    assert!(hw_plan.validate().is_err());
    assert!(ag_plan.validate().is_err());
    assert_eq!(
        hw_plan.validate().unwrap_err().to_string(),
        ag_plan.validate().unwrap_err().to_string()
    );
}

#[test]
fn parity_validate_rejects_self_cycle() {
    let hw_plan = hw::TaskPlan::new("x")
        .add_phase(hw::Phase::new("a").depends_on(vec!["a".into()]));
    let ag_plan = ag::TaskPlan::new("x")
        .add_phase(ag::Phase::new("a").depends_on(vec!["a".into()]));
    assert!(hw_plan.validate().is_err());
    assert!(ag_plan.validate().is_err());
}

#[test]
fn parity_validate_rejects_invalid_handoff_path() {
    let hw_plan = hw::TaskPlan::new("x").add_phase(
        hw::Phase::new("a").add_run(hw::RunRef::new("r1", "f").with_input_from(vec!["bad".into()])),
    );
    let ag_plan = ag::TaskPlan::new("x").add_phase(
        ag::Phase::new("a").add_run(ag::RunRef::new("r1", "f").with_input_from(vec!["bad".into()])),
    );
    assert!(hw_plan.validate().is_err());
    assert!(ag_plan.validate().is_err());
    assert_eq!(
        hw_plan.validate().unwrap_err().to_string(),
        ag_plan.validate().unwrap_err().to_string()
    );
}

#[test]
fn parity_validate_accepts_valid_handoff_path() {
    let hw_plan = hw::TaskPlan::new("x").add_phase(
        hw::Phase::new("a")
            .add_run(hw::RunRef::new("r1", "f").with_input_from(vec!["b.run.handoff.summary".into()])),
    );
    let ag_plan = ag::TaskPlan::new("x").add_phase(
        ag::Phase::new("a")
            .add_run(ag::RunRef::new("r1", "f").with_input_from(vec!["b.run.handoff.summary".into()])),
    );
    assert!(hw_plan.validate().is_ok());
    assert!(ag_plan.validate().is_ok());
}

// ──────────────────────────────────────────────────────────
// from_node — Atom parse path parity (TryFrom<Node> equivalent)
// ──────────────────────────────────────────────────────────

const PLAN_AT: &str = r#"
task_plan(id: "api_v2", version: 2) {
    title: "Build v2 API"
    description: "Second-gen"
    default_mode: "check"

    phase(name: "discovery") {
        mode: "serial"
        run(name: "discover", flow_id: "goal-discovery") {
            input: "Build the v2 API."
            input_from: ["feat.plan.handoff.summary"]
            context: "the ctx"
            mode_override: "gsd"
        }
    }

    phase(name: "design") {
        mode: "parallel"
        depends_on: ["discovery"]
    }
}
"#;

#[test]
fn parity_from_node_full_plan() {
    let node = parse_node(PLAN_AT);
    let hw_plan = hw::TaskPlan::try_from(node.clone()).unwrap();
    let ag_plan = ag::TaskPlan::from_node(node).unwrap();

    assert_eq!(hw_plan.id, ag_plan.id);
    assert_eq!(hw_plan.version, ag_plan.version);
    assert_eq!(hw_plan.title, ag_plan.title);
    assert_eq!(hw_plan.description, ag_plan.description);
    assert_eq!(
        serde_json::to_string(&hw_plan.default_mode).unwrap(),
        serde_json::to_string(&ag_plan.default_mode).unwrap()
    );
    assert_eq!(hw_plan.phases.len(), ag_plan.phases.len());

    for (hw_phase, ag_phase) in hw_plan.phases.iter().zip(ag_plan.phases.iter()) {
        assert_eq!(hw_phase.name, ag_phase.name);
        assert_eq!(
            serde_json::to_string(&hw_phase.mode).unwrap(),
            serde_json::to_string(&ag_phase.mode).unwrap()
        );
        assert_eq!(hw_phase.depends_on, ag_phase.depends_on);
        assert_eq!(hw_phase.runs.len(), ag_phase.runs.len());
        for (hw_run, ag_run) in hw_phase.runs.iter().zip(ag_phase.runs.iter()) {
            assert_eq!(hw_run.name, ag_run.name);
            assert_eq!(hw_run.flow_id, ag_run.flow_id);
            assert_eq!(hw_run.input, ag_run.input);
            assert_eq!(hw_run.input_from, ag_run.input_from);
            assert_eq!(hw_run.context, ag_run.context);
            assert_eq!(
                serde_json::to_string(&hw_run.mode_override).unwrap(),
                serde_json::to_string(&ag_run.mode_override).unwrap()
            );
        }
    }

    // Both validate the parsed plan.
    assert!(hw_plan.validate().is_ok());
    assert!(ag_plan.validate().is_ok());
}

#[test]
fn parity_from_node_rejects_cycle() {
    let src = r#"
task_plan(id: "cyc") {
    phase(name: "a") { depends_on: ["b"] }
    phase(name: "b") { depends_on: ["a"] }
}
"#;
    let node = parse_node(src);
    let hw_err = hw::TaskPlan::try_from(node.clone()).unwrap_err();
    let ag_err = ag::TaskPlan::from_node(node).unwrap_err();
    assert_eq!(hw_err.to_string(), ag_err.to_string());
}

#[test]
fn parity_from_node_rejects_wrong_root() {
    let src = r#"not_a_plan(id: "x") {}"#;
    let node = parse_node(src);
    let hw_err = hw::TaskPlan::try_from(node.clone()).unwrap_err();
    let ag_err = ag::TaskPlan::from_node(node).unwrap_err();
    assert_eq!(hw_err.to_string(), ag_err.to_string());
}

#[test]
fn parity_from_node_defaults() {
    // Minimal plan: everything falls back to defaults.
    let src = r#"task_plan(id: "min") { phase(name: "p") { run(name: "r", flow_id: "f") } }"#;
    let node = parse_node(src);
    let hw_plan = hw::TaskPlan::try_from(node.clone()).unwrap();
    let ag_plan = ag::TaskPlan::from_node(node).unwrap();

    assert_eq!(hw_plan.version, 1);
    assert_eq!(ag_plan.version, 1);
    assert_eq!(hw_plan.default_mode, hw::TaskMode::Gsd);
    assert_eq!(ag_plan.default_mode, ag::TaskMode::Gsd);
    assert_eq!(hw_plan.phases[0].mode, hw::PhaseMode::Serial);
    assert_eq!(ag_plan.phases[0].mode, ag::PhaseMode::Serial);
    assert_eq!(hw_plan.phases[0].runs[0].mode_override, None);
    assert_eq!(ag_plan.phases[0].runs[0].mode_override, None);
}

#[test]
fn parity_from_node_invalid_task_mode() {
    let src = r#"task_plan(id: "x") { default_mode: "bogus" }"#;
    let node = parse_node(src);
    assert!(hw::TaskPlan::try_from(node.clone()).is_err());
    assert!(ag::TaskPlan::from_node(node).is_err());
}
