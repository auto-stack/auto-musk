//! Musk-specific built-in flow definitions.
//! These are app-level product decisions (which steps, which gates, which loops).
//! The generic FlowSpec/FlowStep types come from auto_ai_agent::orchestration.

use auto_ai_agent::orchestration::{ExitRouting, FlowSpec, FlowStep, GateType};

/// Build the built-in flows: default (legacy), simple, superpower, relay.
pub fn builtin_flows() -> Vec<FlowSpec> {
    vec![default_flow(), simple_flow(), superpower_flow(), relay_flow()]
}

/// The canonical spec-driven pipeline. advisor→architect carries a human gate.
fn default_flow() -> FlowSpec {
    use ExitRouting::*;
    use GateType::*;
    let mut flow = FlowSpec::new("default");
    flow.add_step(FlowStep::new("advise", "advisor").with_gate(Human));
    flow.add_step(FlowStep::new("design", "architect"));
    flow.add_step(FlowStep::new("plan", "planner"));
    flow.add_step(FlowStep::new("test-first", "tester"));
    flow.add_step(FlowStep::new("code", "coder").with_exit(Loop { target_step_id: "test-first".into(), max_iterations: 3 }));
    flow.add_step(FlowStep::new("review", "reviewer"));
    flow.add_step(FlowStep::new("document", "documenter"));
    flow
}

fn simple_flow() -> FlowSpec {
    let mut flow = FlowSpec::new("simple");
    flow.add_step(FlowStep::new("advise", "advisor"));
    flow.add_step(FlowStep::new("code", "coder"));
    flow
}

fn superpower_flow() -> FlowSpec {
    let mut flow = FlowSpec::new("superpower");
    flow.add_step(FlowStep::new("brainstorm", "super-advisor"));
    flow.add_step(FlowStep::new("plan", "super-advisor"));
    flow.add_step(FlowStep::new("execute", "super-coder"));
    flow.add_step(FlowStep::new("review", "super-tester"));
    flow
}

fn relay_flow() -> FlowSpec {
    use GateType::*;
    let mut flow = FlowSpec::new("relay");
    flow.add_step(FlowStep::new("brainstorm", "advisor").with_gate(Human));
    flow.add_step(FlowStep::new("design", "architect"));
    flow.add_step(FlowStep::new("plan", "planner"));
    flow.add_step(FlowStep::new("execute", "coder"));
    flow.add_step(FlowStep::new("testing", "tester"));
    flow.add_step(FlowStep::new("review", "reviewer"));
    flow.add_step(FlowStep::new("report", "documenter"));
    flow
}

/// Look up a built-in flow by id.
pub fn get_builtin_flow(id: &str) -> Option<FlowSpec> {
    builtin_flows().into_iter().find(|f| f.id == id)
}
