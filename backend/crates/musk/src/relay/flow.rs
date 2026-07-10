//! Flow Specification — declarative pipeline definitions.
//!
//! A flow is an ordered list of steps that the [`PipelineEngine`] executes.
//! The orchestrator is pure Rust state-machine code — zero LLM tokens are
//! spent deciding what to do next.
//!
//! Ported from auto-forge `backend/src/relay/flow.rs`, keeping `Next` and
//! `Loop` routing (the common cases). `Branch`/`Condition` routing +
//! `StepValidator`/`ToolGuard` arrive in P2b.3.

use serde::{Deserialize, Serialize};

/// A flow is an ordered list of steps with routing logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSpec {
    pub id: String,
    pub steps: Vec<FlowStep>,
}

impl FlowSpec {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            steps: Vec::new(),
        }
    }

    pub fn add_step(&mut self, step: FlowStep) -> &mut Self {
        self.steps.push(step);
        self
    }

    pub fn get_step(&self, step_id: &str) -> Option<&FlowStep> {
        self.steps.iter().find(|s| s.id == step_id)
    }

    pub fn get_step_index(&self, step_id: &str) -> Option<usize> {
        self.steps.iter().position(|s| s.id == step_id)
    }
}

/// A single step in a flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub id: String,
    pub profession_id: String,
    /// Optional agent config to use instead of the default for this profession.
    #[serde(default)]
    pub agent_config_id: Option<String>,
    pub gate: GateType,
    /// Max LLM turns before forced handoff (overrides profession default).
    #[serde(default)]
    pub max_turns: Option<u32>,
    /// How to route after this step completes.
    #[serde(default)]
    pub exit: ExitRouting,
}

impl FlowStep {
    pub fn new(id: impl Into<String>, profession_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            profession_id: profession_id.into(),
            agent_config_id: None,
            gate: GateType::Auto,
            max_turns: None,
            exit: ExitRouting::Next,
        }
    }

    pub fn with_gate(mut self, gate: GateType) -> Self {
        self.gate = gate;
        self
    }

    pub fn with_exit(mut self, exit: ExitRouting) -> Self {
        self.exit = exit;
        self
    }
}

/// Gate type controlling whether a step needs human approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateType {
    /// Proceed automatically.
    Auto,
    /// Pause for human approval before executing.
    Human,
}

impl Default for GateType {
    fn default() -> Self {
        GateType::Auto
    }
}

/// Routing logic after a step completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitRouting {
    /// Go to the next step in sequence.
    Next,
    /// Loop back to a target step (e.g., coder→tester iteration).
    Loop {
        /// Step to return to.
        target_step_id: String,
        /// Max iterations before breaking to next.
        max_iterations: u32,
    },
}

impl Default for ExitRouting {
    fn default() -> Self {
        ExitRouting::Next
    }
}

/// Build the built-in flows: default (legacy), simple, superpower, relay.
pub fn builtin_flows() -> Vec<FlowSpec> {
    vec![default_flow(), simple_flow(), superpower_flow(), relay_flow()]
}

/// The canonical spec-driven pipeline. advisor→architect carries a human gate
/// (the "goal gate"): the human approves the design before execution begins.
fn default_flow() -> FlowSpec {
    use ExitRouting::*;
    use GateType::*;

    let mut flow = FlowSpec::new("default");
    flow.add_step(FlowStep::new("advise", "advisor").with_gate(Human));
    flow.add_step(FlowStep::new("design", "architect"));
    flow.add_step(FlowStep::new("plan", "planner"));
    flow.add_step(FlowStep::new("test-first", "tester"));
    flow.add_step(
        FlowStep::new("code", "coder")
            .with_exit(Loop { target_step_id: "test-first".into(), max_iterations: 3 }),
    );
    flow.add_step(FlowStep::new("review", "reviewer"));
    flow.add_step(FlowStep::new("document", "documenter"));
    flow
}

/// A minimal two-step demo flow for quick testing.
fn simple_flow() -> FlowSpec {
    let mut flow = FlowSpec::new("simple");
    flow.add_step(FlowStep::new("advise", "advisor"));
    flow.add_step(FlowStep::new("code", "coder"));
    flow
}

/// Superpowers flow: brainstorm → plan → execute → review.
/// Medium-complexity tasks (2-6 files, focused feature/refactor).
fn superpower_flow() -> FlowSpec {
    let mut flow = FlowSpec::new("superpower");
    flow.add_step(FlowStep::new("brainstorm", "super-advisor"));
    flow.add_step(FlowStep::new("plan", "super-advisor"));
    flow.add_step(FlowStep::new("execute", "super-coder"));
    flow.add_step(FlowStep::new("review", "super-tester"));
    flow
}

/// Relay flow: brainstorm → design → plan → execute → testing → review → report.
/// Large/complex tasks needing full spec-driven multi-phase pipeline.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_flow_is_well_formed() {
        let flow = default_flow();
        assert_eq!(flow.id, "default");
        assert!(flow.steps.len() >= 7);
        // advise step has a human gate.
        assert_eq!(flow.get_step("advise").unwrap().gate, GateType::Human);
        // code step loops back to test-first.
        match &flow.get_step("code").unwrap().exit {
            ExitRouting::Loop { target_step_id, max_iterations } => {
                assert_eq!(target_step_id, "test-first");
                assert_eq!(*max_iterations, 3);
            }
            other => panic!("code step should loop, got {other:?}"),
        }
        // document is terminal (Next).
        assert!(matches!(flow.get_step("document").unwrap().exit, ExitRouting::Next));
    }

    #[test]
    fn get_builtin_by_id() {
        assert!(get_builtin_flow("default").is_some());
        assert!(get_builtin_flow("simple").is_some());
        assert!(get_builtin_flow("nope").is_none());
    }
}
