//! feature-dev workflow — the legacy `/api/workflow/run` (+ `/stream`) engine,
//! rebuilt on `auto_ai_agent::orchestration::PipelineEngine`.
//!
//! (Plan 017 Phase 1: migrate the last deprecated-`Workflow` consumer to the
//! new orchestration spine, then delete the old engine.)
//!
//! The old `.at` workflow (`workflows/feature-dev.at`) was parsed by the
//! deprecated `auto_ai_agent::workflow` module: per-step `input` templates with
//! `$var` substitution, an optional `condition` skip, and full step outputs
//! passed forward. PipelineEngine is a generic state machine — it has no
//! concept of per-step prompt templates or conditional skips — so this module
//! reproduces those *application-level* semantics on top of it:
//!
//! - `flow()` builds the same architect → coder → tester → reviewer `FlowSpec`
//!   the engine drives.
//! - `drive()` renders each step's input template from the accumulated output
//!   map (multi-step lookback: reviewer sees both `$code` and `$test_report`),
//!   evaluates the reviewer `condition`, and runs a fresh agent per step.
//! - The engine still owns the state machine: step ordering, handoff
//!   auto-correction, budget tracking, status.
//!
//! Behavioral notes vs. the old engine:
//! - Only the built-in `feature-dev` is supported (custom `.at` workflow files
//!   are dropped — the `.at` workflow parser is being retired).
//! - Streaming emits the same step-level SSE events as the old
//!   `WorkflowEvent` shapes (`step_start`/`step_done`/`step_skipped`/`finished`).

use std::collections::HashMap;
use std::sync::Arc;

use auto_ai_agent::orchestration::{
    AdvanceResult, FlowSpec, FlowStep, HandoffDocument, PipelineEngine,
};

use crate::server::AppState;
use crate::workspace::WorkspaceStores;

/// The tool whitelist for every feature-dev step — the old `shared_tools()`
/// set (read/write/run). Matches the original workflow's tool exposure.
const STEP_TOOLS: &[&str] = &["read_file", "write_file", "run_command"];

/// One step of the feature-dev workflow.
struct StepSpec {
    step_id: &'static str,
    /// The profession this step runs (musk role id).
    profession: &'static str,
    /// Per-step input template; `$var` refs are substituted from prior outputs.
    input_template: &'static str,
    /// The output variable this step's result is stored under (e.g. `$design`).
    output_var: &'static str,
    /// Optional skip condition (`$var` non-empty, or `$var.contains("...")`).
    condition: Option<&'static str>,
}

/// Port of `backend/crates/musk/workflows/feature-dev.at` (architect → coder →
/// tester → reviewer), verbatim prompt templates.
const FEATURE_DEV_STEPS: &[StepSpec] = &[
    StepSpec {
        step_id: "architect",
        profession: "architect",
        input_template: "Design a plan for the following task. Output only the plan.\n\nTask:\n$user_request",
        output_var: "$design",
        condition: None,
    },
    StepSpec {
        step_id: "coder",
        profession: "coder",
        input_template: "Implement the following plan using the available tools (read_file/write_file/run_command). Write real code.\n\nPlan:\n$design",
        output_var: "$code",
        condition: None,
    },
    StepSpec {
        step_id: "tester",
        profession: "tester",
        input_template: "Write and run tests for the implementation described below. Use run_command to execute them. Report pass/fail.\n\nImplementation:\n$code",
        output_var: "$test_report",
        condition: None,
    },
    StepSpec {
        step_id: "reviewer",
        profession: "reviewer",
        input_template: "Review the implementation and test results. List issues, or confirm approval.\n\nImplementation:\n$code\n\nTests:\n$test_report",
        output_var: "$review",
        // Review only if the tests actually ran (skip if tester produced nothing).
        condition: Some("$test_report"),
    },
];

fn step_spec(step_id: &str) -> Option<&'static StepSpec> {
    FEATURE_DEV_STEPS.iter().find(|s| s.step_id == step_id)
}

/// All built-in workflow names.
pub fn builtin_names() -> &'static [&'static str] {
    &["feature-dev"]
}

/// Validate a workflow spec: only the built-in name is accepted. Custom
/// `.at` workflow files are no longer supported (the old parser is retired).
pub fn require_builtin(spec: &str) -> Result<(), String> {
    if builtin_names().contains(&spec) {
        Ok(())
    } else {
        Err(format!(
            "unknown workflow '{spec}' (built-ins: {}) — custom .at workflows are retired",
            builtin_names().join(", ")
        ))
    }
}

/// The linear FlowSpec PipelineEngine drives for feature-dev.
pub fn flow() -> FlowSpec {
    let mut flow = FlowSpec::new("feature-dev");
    for spec in FEATURE_DEV_STEPS {
        flow.add_step(FlowStep::new(spec.step_id, spec.profession));
    }
    flow
}

/// Outcome of a feature-dev run — the same shape the old `WorkflowResult`
/// exposed (`/api/workflow/run` response contract).
#[derive(Debug, Clone, Default)]
pub struct FeatureDevResult {
    /// Each step id → its textual output (skipped steps are absent).
    pub steps: HashMap<String, String>,
    /// Each output variable (no `$`) → its value.
    pub outputs: HashMap<String, String>,
    /// Total tokens consumed across all steps.
    pub total_tokens: u64,
}

/// Step-level progress events — byte-for-byte the old `WorkflowEvent` SSE
/// shapes (`{"type":...}`), so existing consumers are unaffected.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowStreamEvent {
    StepStart { step_id: String, role: String, input: String },
    StepDone { step_id: String, output: String },
    StepSkipped { step_id: String },
    Finished { steps: HashMap<String, String>, outputs: HashMap<String, String> },
}

/// Run feature-dev to completion (non-streaming).
pub async fn run(
    state: &AppState,
    ws: &Arc<WorkspaceStores>,
    task: &str,
) -> Result<FeatureDevResult, String> {
    drive(state, ws, task, None).await
}

/// Run feature-dev, emitting step-level events as it goes.
pub async fn run_stream(
    state: &AppState,
    ws: &Arc<WorkspaceStores>,
    task: &str,
    on_event: Arc<dyn Fn(WorkflowStreamEvent) + Send + Sync>,
) -> Result<FeatureDevResult, String> {
    drive(state, ws, task, Some(on_event.as_ref())).await
}

/// The shared drive loop. `on_event` is None for the non-streaming path.
async fn drive(
    state: &AppState,
    ws: &Arc<WorkspaceStores>,
    task: &str,
    on_event: Option<&(dyn Fn(WorkflowStreamEvent) + Send + Sync)>,
) -> Result<FeatureDevResult, String> {
    // Confine this run's file-tool operations to the workspace root.
    crate::tool_safety::set_current_root(ws.root.clone());

    let run_id = format!("workflow-feature-dev-{}", now_secs());
    let mut engine = PipelineEngine::new(flow(), run_id);

    // Shared output map (key without `$`), seeded with the user request.
    let mut vars: HashMap<String, String> = HashMap::new();
    vars.insert("user_request".to_string(), task.to_string());

    let mut result = FeatureDevResult::default();

    let drive_result = loop {
        match engine.advance() {
            AdvanceResult::ExecuteStep { step_id, role_id } => {
                let spec = step_spec(&step_id)
                    .ok_or_else(|| format!("feature-dev: unknown step '{step_id}'"))?;

                // Condition skip (old `condition : "$test_report"` semantics).
                if let Some(cond) = spec.condition {
                    if !eval_condition(cond, &vars) {
                        tracing::info!("workflow step '{step_id}' skipped (condition false)");
                        if let Some(ev) = on_event {
                            ev(WorkflowStreamEvent::StepSkipped { step_id: step_id.clone() });
                        }
                        // The engine has no skip primitive — submit an empty
                        // handoff so it routes to the next step. The step is
                        // NOT recorded in the result (old behavior: skipped
                        // steps are absent).
                        let h = HandoffDocument::new(&role_id, "");
                        match engine.submit_handoff(h) {
                            AdvanceResult::Completed => break Ok(result),
                            AdvanceResult::Failed { error } => break Err(error),
                            AdvanceResult::Paused { step_id, reason } => {
                                break Err(format!("paused at '{step_id}': {reason}"))
                            }
                            _ => continue,
                        }
                    }
                }

                let input = substitute(spec.input_template, &vars);
                if let Some(ev) = on_event {
                    ev(WorkflowStreamEvent::StepStart {
                        step_id: step_id.clone(),
                        role: role_id.clone(),
                        input: input.clone(),
                    });
                }

                // Build a fresh agent for this step (same builder the relay
                // driver uses; tool whitelist matches the old shared_tools()).
                let mode = crate::mode::AgentMode {
                    name: format!("workflow-{step_id}"),
                    description: String::new(),
                    role: role_id.clone(),
                    skills: false,
                    tools: STEP_TOOLS.iter().map(|s| s.to_string()).collect(),
                    workflow: None,
                    context_file: String::new(),
                    extra_system_prompt: String::new(),
                };
                let mut agent = crate::build_agent_from_mode(&mode, state.client.clone())
                    .map_err(|e| format!("build agent '{role_id}': {e}"))?;

                let agent_result = agent
                    .run(&input)
                    .await
                    .map_err(|e| format!("agent '{step_id}' failed: {e}"))?;
                let output = agent_result.output;

                if let Some(ev) = on_event {
                    ev(WorkflowStreamEvent::StepDone {
                        step_id: step_id.clone(),
                        output: output.clone(),
                    });
                }

                // Record into the shared output map + result.
                result.steps.insert(step_id.clone(), output.clone());
                let key = spec.output_var.trim_start_matches('$').to_string();
                vars.insert(key.clone(), output.clone());
                result.outputs.insert(key, output.clone());
                result.total_tokens += agent_result.total_tokens as u64;

                // Handoff carries the full output (empty `to` = "engine
                // decides" — the engine fills the expected next role).
                let mut h = HandoffDocument::new(&role_id, "");
                h.summary = output;
                h.token_usage.step_tokens = agent_result.total_tokens as u64;

                match engine.submit_handoff(h) {
                    AdvanceResult::Completed => break Ok(result),
                    AdvanceResult::Failed { error } => break Err(error),
                    AdvanceResult::Paused { step_id, reason } => {
                        break Err(format!("paused at '{step_id}': {reason}"))
                    }
                    _ => continue,
                }
            }
            AdvanceResult::Completed => break Ok(result),
            AdvanceResult::Failed { error } => break Err(error),
            AdvanceResult::Paused { step_id, reason } => {
                break Err(format!("paused at '{step_id}': {reason}"))
            }
            AdvanceResult::WaitForHuman { .. } => {
                break Err("unexpected human gate in feature-dev".into())
            }
        }
    };

    crate::tool_safety::clear_current_root();

    let final_result = drive_result?;
    if let Some(ev) = on_event {
        ev(WorkflowStreamEvent::Finished {
            steps: final_result.steps.clone(),
            outputs: final_result.outputs.clone(),
        });
    }
    Ok(final_result)
}

/// Replace `$var` references in `template` with values from `vars`. Unknown
/// vars are left as-is so a missing dependency surfaces visibly. (Port of the
/// old `WorkflowContext::substitute`.)
fn substitute(template: &str, vars: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end > start {
                let name = std::str::from_utf8(&bytes[start..end]).unwrap_or("");
                if let Some(val) = vars.get(name) {
                    out.push_str(val);
                } else {
                    out.push('$');
                    out.push_str(name);
                }
                i = end;
                continue;
            }
        }
        let ch_len = utf8_len_at(bytes, i);
        out.push_str(&template[i..i + ch_len]);
        i += ch_len;
    }
    out
}

/// Length in bytes of the UTF-8 code point starting at `idx`.
fn utf8_len_at(bytes: &[u8], idx: usize) -> usize {
    if idx >= bytes.len() {
        return 0;
    }
    let b = bytes[idx];
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else {
        4
    }
}

/// Evaluate a step condition (port of the old `evaluate_condition`):
/// `$var` → true iff non-empty; `$var.contains("lit")` → substring check.
/// Unknown shapes fail open (run the step).
fn eval_condition(expr: &str, vars: &HashMap<String, String>) -> bool {
    let expr = expr.trim();
    if let Some(rest) = expr.strip_prefix('$') {
        if let Some(paren) = rest.find(".contains(") {
            let var = &rest[..paren];
            let after = &rest[paren + ".contains(".len()..];
            if let Some(close) = after.rfind(')') {
                let literal = after[..close].trim().trim_matches('"').trim_matches('\'');
                return vars.get(var).map(|v| v.contains(literal)).unwrap_or(false);
            }
        }
        return vars.get(rest).map(|v| !v.is_empty()).unwrap_or(false);
    }
    tracing::warn!("workflow: unrecognized condition '{expr}', treating as true");
    true
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_is_linear_four_steps() {
        let f = flow();
        assert_eq!(f.id, "feature-dev");
        assert_eq!(f.steps.len(), 4);
        assert_eq!(f.steps[0].role_id, "architect");
        assert_eq!(f.steps[3].role_id, "reviewer");
    }

    #[test]
    fn builtin_validation_accepts_feature_dev_only() {
        assert!(require_builtin("feature-dev").is_ok());
        assert!(require_builtin("nope").is_err());
        assert!(require_builtin("/some/custom.at").is_err());
    }

    #[test]
    fn substitute_replaces_known_vars_and_keeps_unknown() {
        let mut vars = HashMap::new();
        vars.insert("user_request".to_string(), "build x".to_string());
        vars.insert("code".to_string(), "fn main() {}".to_string());
        let out = substitute("Task:\n$user_request\nCode:\n$code\n$missing", &vars);
        assert_eq!(out, "Task:\nbuild x\nCode:\nfn main() {}\n$missing");
    }

    #[test]
    fn condition_bare_var_and_contains() {
        let mut vars = HashMap::new();
        vars.insert("test_report".to_string(), "all pass".to_string());
        assert!(eval_condition("$test_report", &vars));
        assert!(eval_condition("$test_report.contains(pass)", &vars));
        assert!(!eval_condition("$test_report.contains(fail)", &vars));
        vars.insert("empty".to_string(), String::new());
        assert!(!eval_condition("$empty", &vars));
        assert!(!eval_condition("$unknown", &vars));
    }
}
