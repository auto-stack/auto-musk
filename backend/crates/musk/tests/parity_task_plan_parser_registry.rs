//! parity_task_plan_parser_registry.rs — Plan 020 Phase B: task_plan_parser.at
//! + task_plan_registry.at 移植 parity。
//!
//! - parser: parse_task_plan 对同一 atom 输入产生与 hw 等价的 plan(wire),
//!   拒绝路径(missing id / bad mode / cycle)与 hw 同为 Err。
//! - registry:TaskPlanSource / TaskPlanSummary wire、load_builtins_only 的
//!   内建 deferred-decompose、builtin 不可删、insert/remove user plan、
//!   register(校验 flow + 写文件)、new() 从目录加载 user plans。

use std::path::PathBuf;

use musk::auto_generated::task_plan_parser as ag_parser;
use musk::auto_generated::task_plan_registry as ag_reg;
use musk::relay::task_plan_parser as hw_parser;
use musk::relay::task_plan_registry as hw_reg;
use tempfile::TempDir;

// ── parser ──────────────────────────────────────────────────────────────────

fn example_input() -> &'static str {
    r#"
    task_plan(id: "api_v2", version: 1) {
        title: "Build v2 API"
        default_mode: "gsd"

        phase(name: "discovery") {
            mode: "serial"
            run(name: "discover", flow_id: "goal-discovery") {
                input: "Build the v2 API."
            }
        }

        phase(name: "design") {
            mode: "serial"
            depends_on: ["discovery"]
            run(name: "architecture", flow_id: "default") {
                input_from: ["discovery.discover.handoff.goals"]
            }
        }
    }
    "#
}

#[test]
fn parity_parse_full_example() {
    let hw_plan = hw_parser::parse_task_plan(example_input()).unwrap();
    let ag_plan = ag_parser::parse_task_plan(example_input()).unwrap();

    assert_eq!(ag_plan.id, hw_plan.id, "id");
    assert_eq!(ag_plan.version, hw_plan.version, "version");
    assert_eq!(ag_plan.title, hw_plan.title, "title");
    assert_eq!(
        serde_json::to_value(&ag_plan).unwrap(),
        serde_json::to_value(&hw_plan).unwrap(),
        "full plan wire parity"
    );
    assert_eq!(ag_plan.phases.len(), 2, "two phases");
    assert_eq!(ag_plan.phases[1].depends_on, vec!["discovery"]);
    assert_eq!(
        ag_plan.phases[0].runs[0].input.as_deref(),
        Some("Build the v2 API.")
    );
}

#[test]
fn parity_parse_rejections() {
    // Same accept/reject decisions for malformed inputs.
    let cases = [
        r#"task_plan { }"#,                                          // missing id
        r#"task_plan(id: "x") { default_mode: "fast" }"#,            // bad mode
        r#"
        task_plan(id: "x") {
            phase(name: "a", depends_on: ["b"]) { run(name: "r", flow_id: "f") }
            phase(name: "b", depends_on: ["a"]) { run(name: "r", flow_id: "f") }
        }
        "#, // cycle
    ];
    for input in cases {
        let hw_res = hw_parser::parse_task_plan(input);
        let ag_res = ag_parser::parse_task_plan(input);
        assert_eq!(hw_res.is_err(), true, "hw rejects: {input}");
        assert_eq!(ag_res.is_err(), true, "ag rejects: {input}");
    }
}

#[test]
fn parity_parse_builtin_deferred_decompose() {
    // The builtin plan shipped with musk must parse identically.
    let atom = musk::auto_generated::extern_impl::task_plan_builtin_atom();
    let hw_plan = hw_parser::parse_task_plan(&atom).unwrap();
    let ag_plan = ag_parser::parse_task_plan(&atom).unwrap();
    assert_eq!(ag_plan.id, hw_plan.id, "builtin id");
    assert_eq!(ag_plan.id, "deferred-decompose");
    assert_eq!(ag_plan.phases.len(), 1);
    assert_eq!(ag_plan.phases[0].runs.len(), 1);
    assert_eq!(
        serde_json::to_value(&ag_plan).unwrap(),
        serde_json::to_value(&hw_plan).unwrap(),
        "builtin plan wire parity"
    );
}

// ── registry ────────────────────────────────────────────────────────────────

#[test]
fn parity_task_plan_source_wire() {
    assert_eq!(
        serde_json::to_string(&ag_reg::TaskPlanSource::Builtin).unwrap(),
        serde_json::to_string(&hw_reg::TaskPlanSource::Builtin).unwrap(),
        "Builtin wire"
    );
    assert_eq!(
        serde_json::to_string(&ag_reg::TaskPlanSource::User).unwrap(),
        serde_json::to_string(&hw_reg::TaskPlanSource::User).unwrap(),
        "User wire"
    );
}

#[test]
fn parity_registry_builtins_and_semantics() {
    let hw_registry = hw_reg::TaskPlanRegistry::load_builtins_only();
    let ag_registry = ag_reg::TaskPlanRegistry::load_builtins_only();

    // Builtin deferred-decompose loads on both sides.
    let hw_plan = hw_registry.get("deferred-decompose").unwrap();
    let ag_plan = ag_registry.get("deferred-decompose").unwrap();
    assert_eq!(ag_plan.phases.len(), hw_plan.phases.len(), "builtin phases");
    assert_eq!(ag_plan.phases[0].runs.len(), 1);
    assert_eq!(hw_registry.source("deferred-decompose"), Some(hw_reg::TaskPlanSource::Builtin));
    assert_eq!(ag_registry.source("deferred-decompose"), Some(ag_reg::TaskPlanSource::Builtin));

    // list() summaries agree on counts + sources.
    let hw_list = hw_registry.list();
    let ag_list = ag_registry.list();
    assert_eq!(ag_list.len(), hw_list.len(), "same list length");
    let hw_s = hw_list.iter().find(|s| s.id == "deferred-decompose").unwrap();
    let ag_s = ag_list.iter().find(|s| s.id == "deferred-decompose").unwrap();
    assert_eq!(ag_s.phase_count as usize, hw_s.phase_count, "phase_count");
    assert_eq!(ag_s.run_count as usize, hw_s.run_count, "run_count");
    assert_eq!(
        serde_json::to_value(&ag_s).unwrap(),
        serde_json::to_value(&hw_s).unwrap(),
        "summary wire parity"
    );

    // Builtin plans cannot be removed.
    let mut ag_registry = ag_reg::TaskPlanRegistry::load_builtins_only();
    assert!(ag_registry.remove("deferred-decompose").is_none());
    assert!(ag_registry.get("deferred-decompose").is_some());
}

#[test]
fn parity_registry_insert_and_remove_user_plan() {
    let mut ag_registry = ag_reg::TaskPlanRegistry::load_builtins_only();
    let plan = musk::auto_generated::task_plan::TaskPlan::new("custom");
    ag_registry.insert(plan, ag_reg::TaskPlanSource::User);
    assert!(ag_registry.get("custom").is_some());
    assert_eq!(ag_registry.source("custom"), Some(ag_reg::TaskPlanSource::User));
    assert!(ag_registry.remove("custom").is_some());
    assert!(ag_registry.get("custom").is_none());

    // insert with empty user_dir → remove still works (no file to delete).
    let mut ag_registry = ag_reg::TaskPlanRegistry::load_builtins_only();
    ag_registry.insert(
        musk::auto_generated::task_plan::TaskPlan::new("custom"),
        ag_reg::TaskPlanSource::User,
    );
    assert!(ag_registry.remove("custom").is_some());
}

#[test]
fn parity_registry_validate_flow_id() {
    let ag_registry = ag_reg::TaskPlanRegistry::load_builtins_only();
    let hw_registry = hw_reg::TaskPlanRegistry::load_builtins_only();

    let ag_plan = musk::auto_generated::task_plan::TaskPlan::new("x").add_phase(
        musk::auto_generated::task_plan::Phase::new("p")
            .add_run(musk::auto_generated::task_plan::RunRef::new("r", "nonexistent-flow")),
    );
    let hw_plan = musk::relay::task_plan::TaskPlan::new("x").add_phase(
        musk::relay::task_plan::Phase::new("p")
            .add_run(musk::relay::task_plan::RunRef::new("r", "nonexistent-flow")),
    );
    let ag_err = ag_registry.validate(ag_plan).unwrap_err();
    let hw_err = hw_registry.validate(&hw_plan).unwrap_err();
    assert_eq!(ag_err, hw_err, "unknown-flow validate error text");
    assert!(ag_err.contains("unknown flow"));
}

#[test]
fn parity_registry_register_and_load_from_dir() {
    let dir = TempDir::new().unwrap();
    let ag_dir = dir.path().join("ag");
    let hw_dir = dir.path().join("hw");

    // register(): unknown flow rejected; known builtin flow accepted + file written.
    let mut ag_registry = ag_reg::TaskPlanRegistry::new(ag_dir.clone());
    let mut hw_registry = hw_reg::TaskPlanRegistry::new(hw_dir.clone());

    let bad = r#"task_plan(id: "bad") { phase(name: "p") { run(name: "r", flow_id: "nope") } }"#;
    assert!(ag_registry.register(bad).is_err(), "ag rejects unknown flow");
    assert!(hw_registry.register(bad).is_err(), "hw rejects unknown flow");

    let good = r#"task_plan(id: "good") { phase(name: "p") { run(name: "r", flow_id: "default") } }"#;
    let ag_plan = ag_registry.register(good).unwrap();
    let hw_plan = hw_registry.register(good).unwrap();
    assert_eq!(ag_plan.id, hw_plan.id, "registered id");
    assert!(ag_registry.get("good").is_some());
    assert_eq!(ag_registry.source("good"), Some(ag_reg::TaskPlanSource::User));
    // The user file was written (both sides).
    assert!(ag_dir.join("task_plans").join("good.atom").exists());
    assert!(hw_dir.join("task_plans").join("good.atom").exists());

    // A fresh registry on the same dir loads the user plan from disk.
    let ag_reload = ag_reg::TaskPlanRegistry::new(ag_dir);
    assert!(ag_reload.get("good").is_some());
    assert_eq!(ag_reload.source("good"), Some(ag_reg::TaskPlanSource::User));
}

#[test]
fn parity_registry_new_loads_user_plans_from_dir() {
    let dir = TempDir::new().unwrap();
    let ag_dir = dir.path().join("ag");
    let plans_dir = ag_dir.join("task_plans");
    std::fs::create_dir_all(&plans_dir).unwrap();
    std::fs::write(
        plans_dir.join("my.atom"),
        r#"task_plan(id: "my") { phase(name: "p") { run(name: "r", flow_id: "default") } }"#,
    )
    .unwrap();

    let registry = ag_reg::TaskPlanRegistry::new(ag_dir);
    assert!(registry.get("my").is_some());
    assert_eq!(registry.source("my"), Some(ag_reg::TaskPlanSource::User));

    // Same load behavior as hw against an equivalent dir.
    let hw_dir = dir.path().join("hw");
    let hw_plans = hw_dir.join("task_plans");
    std::fs::create_dir_all(&hw_plans).unwrap();
    std::fs::write(
        hw_plans.join("my.atom"),
        r#"task_plan(id: "my") { phase(name: "p") { run(name: "r", flow_id: "default") } }"#,
    )
    .unwrap();
    let hw_registry = hw_reg::TaskPlanRegistry::new(&hw_dir);
    assert_eq!(
        serde_json::to_value(registry.get("my").unwrap()).unwrap(),
        serde_json::to_value(hw_registry.get("my").unwrap()).unwrap(),
        "loaded user plan wire parity"
    );
}
