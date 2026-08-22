//! Musk-specific built-in flow definitions.
//! These are app-level product decisions (which steps, which gates, which loops).
//! The generic FlowSpec/FlowStep types come from auto_ai_agent::orchestration.

use auto_ai_agent::orchestration::{ExitRouting, FlowSpec, FlowStep, GateType};

/// Build the built-in flows: plan (PLAN-030 canonical), plan-merge
/// (PLAN-034 smart deposit), simple, superpower, plus the deprecated
/// spec-driven pipelines (default/relay, kept for comparison runs).
pub fn builtin_flows() -> Vec<FlowSpec> {
    vec![
        plan_flow(),
        plan_merge_flow(),
        default_flow(),
        simple_flow(),
        superpower_flow(),
        relay_flow(),
    ]
}

/// Plan-driven dev flow — 单角色四相位，计划文件为交接载体（PLAN-030）。
///
/// All four steps run as the SAME profession (`plan-dev`): role/soul/model
/// tier/toolset never change, so there is no persona handoff. Phase behavior
/// comes from the musk-side phase task templates (`relay/plan_flow.rs`)
/// injected in `step_context`; the plan file (located via the `PLAN_FILE:`
/// marker the driver extracts) is the full inter-phase context. The Human
/// gate before `execute` is the plan-confirmation checkpoint (superpowers
/// present-for-confirmation discipline).
fn plan_flow() -> FlowSpec {
    use GateType::*;
    let mut flow = FlowSpec::new("plan");
    flow.add_step(FlowStep::new("plan", "plan-dev"));
    flow.add_step(FlowStep::new("execute", "plan-dev").with_gate(Human));
    flow.add_step(FlowStep::new("review", "plan-dev"));
    flow.add_step(FlowStep::new("document", "plan-dev"));
    flow
}

/// Smart deposit flow（PLAN-034）— 单相位 document，直接沉淀。
/// 由计划页"沉淀到 Spec"按钮经 Chats `/auto-plan:merge PLAN-NNN` 触发：
/// Agent 先 `merge_plan`（机械沉淀+归档，幂等）→ 按 spec-impact 更新
/// `docs/specs/` 模块树 → `emit_report` 生成 HTML 报告（relay run 内合法）。
fn plan_merge_flow() -> FlowSpec {
    let mut flow = FlowSpec::new("plan-merge");
    flow.add_step(FlowStep::new("document", "plan-dev"));
    flow
}

/// DEPRECATED (PLAN-030): the 7-role spec-driven pipeline. Kept for
/// comparison runs; the canonical flow is now `plan`. advisor→architect
/// carries a human gate.
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

/// DEPRECATED (PLAN-030): the 7-role spec-driven relay pipeline. Kept for
/// comparison runs; the canonical flow is now `plan`.
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

    /// PLAN-030 A1: the plan flow is 4 steps, all `plan-dev`, with a single
    /// Human gate before `execute` (plan-confirmation checkpoint).
    #[test]
    fn plan_flow_is_four_same_role_steps_with_one_human_gate() {
        let flow = get_builtin_flow("plan").expect("plan flow registered");
        assert_eq!(flow.steps.len(), 4);
        let ids: Vec<&str> = flow.steps.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["plan", "execute", "review", "document"]);
        for s in &flow.steps {
            assert_eq!(s.role_id, "plan-dev", "step {} role", s.id);
        }
        assert_eq!(flow.steps[0].gate, GateType::Auto);
        assert_eq!(flow.steps[1].gate, GateType::Human);
        assert_eq!(flow.steps[2].gate, GateType::Auto);
        assert_eq!(flow.steps[3].gate, GateType::Auto);
    }

    #[test]
    fn plan_flow_is_first_registered() {
        // resolve_flow's fallback chain ends at "default"; ensure "plan" is
        // addressable by id and present in the builtin list.
        assert!(builtin_flows().iter().any(|f| f.id == "plan"));
        assert!(get_builtin_flow("default").is_some(), "deprecated default kept");
    }

    /// PLAN-034: plan-merge 是单步 document 相位、plan-dev 角色、无 gate。
    #[test]
    fn plan_merge_flow_is_single_document_step() {
        let flow = get_builtin_flow("plan-merge").expect("plan-merge flow registered");
        assert_eq!(flow.steps.len(), 1);
        assert_eq!(flow.steps[0].id.as_str(), "document");
        assert_eq!(flow.steps[0].role_id, "plan-dev");
        assert_eq!(flow.steps[0].gate, GateType::Auto);
    }
}
