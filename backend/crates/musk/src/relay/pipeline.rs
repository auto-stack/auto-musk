//! Pipeline Engine — the deterministic state machine that executes flows.
//!
//! Pure Rust code: zero LLM tokens are spent on orchestration. The engine's
//! `advance() / submit_handoff() / resolve_gate()` triad drives an external
//! loop (the relay API's simplified driver in P2b.1, the full background
//! driver in P2b.2).
//!
//! Ported from auto-forge `backend/src/relay/pipeline.rs`. P2b.1 keeps
//! `Next`/`Loop` routing, gate handling, budget enforcement, and handoff
//! auto-correction. Auto-validation retry/escalation and `Branch`/`Condition`
//! routing arrive in P2b.3.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::relay::budget::{BudgetAction, BudgetTracker, TokenBudget};
use crate::relay::flow::{ExitRouting, FlowSpec, GateType};
use crate::relay::handoff::HandoffDocument;

/// Execution mode controlling human-gate behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelayMode {
    /// Get Shit Done — autonomous; human gates still pause for approval.
    GSD,
    /// Human reviews every configured gate.
    Check,
}

impl Default for RelayMode {
    fn default() -> Self {
        RelayMode::GSD
    }
}

/// Result of advancing the pipeline — tells the caller what to do next.
#[derive(Debug, Clone, PartialEq)]
pub enum AdvanceResult {
    /// Run the agent for this step, then call `submit_handoff()`.
    ExecuteStep {
        step_id: String,
        profession_id: String,
        agent_config_id: Option<String>,
    },
    /// Pause for human approval at a gate.
    WaitForHuman {
        gate: GateType,
        step_id: String,
    },
    /// Flow completed successfully.
    Completed,
    /// Flow failed.
    Failed { error: String },
    /// Loop reached max iterations — manual resume required.
    Paused { step_id: String, reason: String },
}

/// A human decision at a gate.
#[derive(Debug, Clone, PartialEq)]
pub enum GateDecision {
    /// Approve and continue.
    Approve,
    /// Reject and redraft the same step, with feedback.
    Reject { feedback: String },
    /// Approve with edit notes carried forward as context.
    Edit { changes: String },
}

/// Record of a completed step execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub step_id: String,
    pub profession_id: String,
    pub handoff: Option<HandoffDocument>,
    pub started_at: u64,
    pub completed_at: u64,
    pub iteration: u32,
}

/// The pipeline engine state machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineEngine {
    pub flow: FlowSpec,
    /// Index into `flow.steps` of the current (or next) step.
    pub current_step: usize,
    pub status: PipelineStatus,
    pub run_id: String,
    /// History of completed steps.
    pub step_history: Vec<StepRecord>,
    /// Loop iteration counters per step_id.
    pub loop_counters: HashMap<String, u32>,
    /// Pending human gate (when status is `WaitingForHuman`).
    pub pending_gate: Option<PendingGate>,
    /// Feedback from rejected gates, keyed by step_id.
    pub gate_feedback: HashMap<String, Vec<String>>,
    /// Which step had its gate resolved for the current attempt.
    pub gate_resolved_for_step: Option<String>,
    /// If set, this step was resumed from a paused state and should receive a resume hint.
    #[serde(default)]
    pub resumed_step_id: Option<String>,
    /// Accumulated token usage across all steps.
    pub cumulative_tokens: u64,
    /// Budget tracker for runaway-cost prevention and analytics.
    pub budget_tracker: BudgetTracker,
    /// Execution mode.
    pub mode: RelayMode,
}

/// Current state of the pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PipelineStatus {
    Idle,
    Running {
        step_id: String,
        profession_id: String,
        started_at: u64,
    },
    WaitingForHuman {
        gate: GateType,
        step_id: String,
        since: u64,
    },
    Completed,
    Failed {
        error: String,
    },
    Paused {
        at_step: usize,
    },
}

impl PipelineStatus {
    /// Clean, human-readable status string (matches frontend expectations).
    pub fn to_status_str(&self) -> String {
        match self {
            PipelineStatus::Idle => "idle".into(),
            PipelineStatus::Running { .. } => "running".into(),
            PipelineStatus::WaitingForHuman { .. } => "waiting_approval".into(),
            PipelineStatus::Completed => "completed".into(),
            PipelineStatus::Failed { .. } => "failed".into(),
            PipelineStatus::Paused { .. } => "paused".into(),
        }
    }
}

/// A gate awaiting human resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingGate {
    pub step_id: String,
    pub gate: GateType,
    pub since: u64,
}

/// Internal next-step resolution result.
enum NextStep {
    Index(usize),
    Complete,
    Error(String),
    Pause { reason: String, resume_step_id: String },
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl PipelineEngine {
    pub fn new(flow: FlowSpec, run_id: impl Into<String>) -> Self {
        Self::with_budget(flow, run_id, TokenBudget::new(10_000_000))
    }

    pub fn with_budget(
        flow: FlowSpec,
        run_id: impl Into<String>,
        run_budget: TokenBudget,
    ) -> Self {
        Self {
            flow,
            current_step: 0,
            status: PipelineStatus::Idle,
            run_id: run_id.into(),
            step_history: Vec::new(),
            loop_counters: HashMap::new(),
            pending_gate: None,
            gate_feedback: HashMap::new(),
            gate_resolved_for_step: None,
            resumed_step_id: None,
            cumulative_tokens: 0,
            budget_tracker: BudgetTracker::new(run_budget),
            mode: RelayMode::GSD,
        }
    }

    /// Advance the pipeline by one logical action.
    pub fn advance(&mut self) -> AdvanceResult {
        match &self.status {
            PipelineStatus::Completed => return AdvanceResult::Completed,
            PipelineStatus::Failed { error } => {
                return AdvanceResult::Failed { error: error.clone() }
            }
            PipelineStatus::WaitingForHuman { .. } => {
                return AdvanceResult::Failed {
                    error: "Cannot advance while waiting for human gate. Call resolve_gate() first."
                        .into(),
                };
            }
            PipelineStatus::Paused { at_step } => {
                let step = &self.flow.steps[*at_step];
                return AdvanceResult::Paused {
                    step_id: step.id.clone(),
                    reason: format!("Run paused at step '{}'. Call resume to continue.", step.id),
                };
            }
            _ => {}
        }

        // Exhausted all steps?
        if self.current_step >= self.flow.steps.len() {
            self.status = PipelineStatus::Completed;
            return AdvanceResult::Completed;
        }

        let step = &self.flow.steps[self.current_step];
        let now = now_secs();

        // Human gate: pause unless already resolved for this attempt. Both GSD
        // and Check pause at human gates — the difference (GSD only at the goal
        // gate) is a flow-design concern; a step marked Human always pauses.
        if step.gate == GateType::Human
            && self.gate_resolved_for_step.as_ref() != Some(&step.id)
        {
            self.status = PipelineStatus::WaitingForHuman {
                gate: GateType::Human,
                step_id: step.id.clone(),
                since: now,
            };
            self.pending_gate = Some(PendingGate {
                step_id: step.id.clone(),
                gate: GateType::Human,
                since: now,
            });
            return AdvanceResult::WaitForHuman {
                gate: GateType::Human,
                step_id: step.id.clone(),
            };
        }

        // Transition to Running.
        self.status = PipelineStatus::Running {
            step_id: step.id.clone(),
            profession_id: step.profession_id.clone(),
            started_at: now,
        };
        // Clear the resume hint once the step actually starts.
        self.resumed_step_id = None;

        AdvanceResult::ExecuteStep {
            step_id: step.id.clone(),
            profession_id: step.profession_id.clone(),
            agent_config_id: step.agent_config_id.clone(),
        }
    }

    /// Submit the result of an agent turn to continue the pipeline.
    pub fn submit_handoff(&mut self, mut handoff: HandoffDocument) -> AdvanceResult {
        let now = now_secs();

        // Record the completed step.
        let (step_id, started_at) = match &self.status {
            PipelineStatus::Running { step_id, started_at, .. } => (step_id.clone(), *started_at),
            _ => {
                self.status = PipelineStatus::Failed {
                    error: "submit_handoff called but no step is running".into(),
                };
                return self.advance();
            }
        };

        // Consume the gate resolution — a fresh attempt needs re-approval.
        self.gate_resolved_for_step = None;

        let profession_id = self.flow.steps[self.current_step].profession_id.clone();
        let exit = self.flow.steps[self.current_step].exit.clone();

        // ─── Handoff target auto-correction ──────────────────────────────────
        // If the flow's exit routing implies a deterministic next profession and
        // the handoff disagrees, correct the target and note it as feedback.
        let expected_prof = match &exit {
            ExitRouting::Next => {
                let next_idx = self.current_step + 1;
                self.flow
                    .steps
                    .get(next_idx)
                    .map(|s| s.profession_id.clone())
            }
            ExitRouting::Loop { target_step_id, .. } => self
                .flow
                .get_step_index(target_step_id)
                .map(|idx| self.flow.steps[idx].profession_id.clone()),
        };
        if let Some(expected) = expected_prof {
            if handoff.to != expected {
                tracing::warn!(
                    "Handoff target '{}' != flow-expected '{}'; correcting.",
                    handoff.to,
                    expected
                );
                self.gate_feedback
                    .entry(step_id.clone())
                    .or_default()
                    .push(format!(
                        "[AUTO-CORRECTION] Handoff target was '{}' but flow routing expects '{}'. Corrected.",
                        handoff.to, expected
                    ));
                handoff.to = expected;
            }
        }

        self.step_history.push(StepRecord {
            step_id: step_id.clone(),
            profession_id: profession_id.clone(),
            handoff: Some(handoff.clone()),
            started_at,
            completed_at: now,
            iteration: *self.loop_counters.get(&step_id).unwrap_or(&0),
        });

        // Update cumulative tokens.
        let step_tokens = handoff.token_usage.step_input + handoff.token_usage.step_output;
        self.cumulative_tokens += step_tokens;
        self.budget_tracker
            .record(&profession_id, handoff.token_usage.step_input, handoff.token_usage.step_output);

        // Budget enforcement.
        if self.budget_tracker.check(&profession_id) == BudgetAction::HardStop {
            let error = format!(
                "Budget exceeded: {} tokens spent vs {} limit",
                self.budget_tracker.cumulative, self.budget_tracker.run_budget.limit
            );
            self.status = PipelineStatus::Failed { error: error.clone() };
            return AdvanceResult::Failed { error };
        }

        // Determine the next step from exit routing.
        let next = self.resolve_next_step(&step_id, &exit);
        match next {
            NextStep::Index(idx) => {
                self.current_step = idx;
                self.advance()
            }
            NextStep::Complete => {
                self.current_step = self.flow.steps.len();
                self.status = PipelineStatus::Completed;
                AdvanceResult::Completed
            }
            NextStep::Error(msg) => {
                self.status = PipelineStatus::Failed { error: msg.clone() };
                AdvanceResult::Failed { error: msg }
            }
            NextStep::Pause { reason, resume_step_id } => {
                if let Some(idx) = self.flow.get_step_index(&resume_step_id) {
                    self.current_step = idx;
                }
                self.status = PipelineStatus::Paused {
                    at_step: self.current_step,
                };
                AdvanceResult::Paused {
                    step_id: self.flow.steps[self.current_step].id.clone(),
                    reason,
                }
            }
        }
    }

    /// Resolve a pending human gate.
    pub fn resolve_gate(&mut self, decision: GateDecision) -> AdvanceResult {
        let pending = match self.pending_gate.take() {
            Some(g) => g,
            None => {
                return AdvanceResult::Failed {
                    error: "No pending gate to resolve".into(),
                }
            }
        };

        match decision {
            GateDecision::Approve | GateDecision::Edit { .. } => {
                self.gate_resolved_for_step = Some(pending.step_id.clone());
                self.status = PipelineStatus::Idle;
                self.advance()
            }
            GateDecision::Reject { feedback } => {
                // Store feedback and redraft the same step.
                self.gate_feedback
                    .entry(pending.step_id.clone())
                    .or_default()
                    .push(feedback);
                self.gate_resolved_for_step = Some(pending.step_id.clone());
                self.status = PipelineStatus::Idle;
                self.advance()
            }
        }
    }

    /// Resolve the next step index from exit routing (Next / Loop only in P2b.1).
    fn resolve_next_step(&mut self, step_id: &str, exit: &ExitRouting) -> NextStep {
        match exit {
            ExitRouting::Next => {
                let next = self.current_step + 1;
                if next >= self.flow.steps.len() {
                    NextStep::Complete
                } else {
                    NextStep::Index(next)
                }
            }
            ExitRouting::Loop {
                target_step_id,
                max_iterations,
            } => {
                let count = self.loop_counters.entry(step_id.to_string()).or_insert(0);
                *count += 1;
                if *count >= *max_iterations {
                    NextStep::Pause {
                        reason: format!(
                            "Step '{}' loop reached max iterations ({}). Manual resume required.",
                            step_id, max_iterations
                        ),
                        resume_step_id: target_step_id.clone(),
                    }
                } else {
                    match self.flow.get_step_index(target_step_id) {
                        Some(idx) => NextStep::Index(idx),
                        None => NextStep::Error(format!("Loop target '{}' not found", target_step_id)),
                    }
                }
            }
        }
    }

    /// Pause at the current position.
    pub fn pause(&mut self) {
        if matches!(self.status, PipelineStatus::Running { .. }) {
            self.status = PipelineStatus::Paused {
                at_step: self.current_step,
            };
        }
    }

    /// Resume from a paused state, resetting loop counters for a fresh start.
    pub fn resume(&mut self) -> Option<AdvanceResult> {
        if matches!(self.status, PipelineStatus::Paused { .. }) {
            for step in &self.flow.steps {
                self.loop_counters.insert(step.id.clone(), 0);
            }
            if let Some(step) = self.flow.steps.get(self.current_step) {
                self.resumed_step_id = Some(step.id.clone());
            }
            self.status = PipelineStatus::Idle;
            Some(self.advance())
        } else {
            None
        }
    }

    /// Rerun from the current failed step, resetting its retry state.
    pub fn rerun(&mut self) -> Option<AdvanceResult> {
        if matches!(self.status, PipelineStatus::Failed { .. }) {
            let step_id = self.flow.steps.get(self.current_step)?.id.clone();
            self.loop_counters.insert(step_id.clone(), 0);
            self.gate_feedback.remove(&step_id);
            self.gate_resolved_for_step = None;
            self.status = PipelineStatus::Idle;
            Some(self.advance())
        } else {
            None
        }
    }

    /// Which profession is currently/next expected.
    pub fn current_profession_id(&self) -> Option<&str> {
        self.flow
            .steps
            .get(self.current_step)
            .map(|s| s.profession_id.as_str())
    }

    /// Current step ID.
    pub fn current_step_id(&self) -> Option<&str> {
        self.flow
            .steps
            .get(self.current_step)
            .map(|s| s.id.as_str())
    }

    /// Feedback accumulated for a step (gate rejections, auto-corrections).
    pub fn feedback_for(&self, step_id: &str) -> Vec<String> {
        self.gate_feedback.get(step_id).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::flow::FlowStep;

    fn two_step_flow() -> FlowSpec {
        let mut f = FlowSpec::new("test");
        f.add_step(FlowStep::new("a", "advisor"));
        f.add_step(FlowStep::new("b", "architect"));
        f
    }

    fn handoff(from: &str, to: &str, run_id: &str) -> HandoffDocument {
        let mut h = HandoffDocument::new(from, to, run_id, 0);
        h.summary = "done".into();
        h
    }

    #[test]
    fn advance_executes_then_completes() {
        let mut eng = PipelineEngine::new(two_step_flow(), "run-1");
        // Step A.
        let r = eng.advance();
        assert!(matches!(r, AdvanceResult::ExecuteStep { ref step_id, .. } if step_id == "a"));
        // Submit A's handoff → routes to step B.
        let r = eng.submit_handoff(handoff("advisor", "architect", "run-1"));
        assert!(matches!(r, AdvanceResult::ExecuteStep { ref step_id, .. } if step_id == "b"));
        // Submit B's handoff → completed (no more steps).
        let r = eng.submit_handoff(handoff("architect", "documenter", "run-1"));
        assert_eq!(r, AdvanceResult::Completed);
        assert_eq!(eng.status, PipelineStatus::Completed);
        assert_eq!(eng.step_history.len(), 2);
    }

    #[test]
    fn human_gate_pauses_then_resolves() {
        let mut f = FlowSpec::new("gated");
        f.add_step(FlowStep::new("advise", "advisor").with_gate(GateType::Human));
        f.add_step(FlowStep::new("code", "coder"));
        let mut eng = PipelineEngine::new(f, "run-2");

        // First advance pauses at the human gate.
        let r = eng.advance();
        assert!(matches!(r, AdvanceResult::WaitForHuman { .. }));
        assert!(matches!(eng.status, PipelineStatus::WaitingForHuman { .. }));

        // Cannot advance while waiting.
        let r = eng.advance();
        assert!(matches!(r, AdvanceResult::Failed { .. }));

        // Approve → executes the step.
        let r = eng.resolve_gate(GateDecision::Approve);
        assert!(matches!(r, AdvanceResult::ExecuteStep { ref step_id, .. } if step_id == "advise"));

        // Submit handoff → next step.
        let r = eng.submit_handoff(handoff("advisor", "coder", "run-2"));
        assert!(matches!(r, AdvanceResult::ExecuteStep { .. }));
    }

    #[test]
    fn gate_reject_redrafts_with_feedback() {
        let mut f = FlowSpec::new("gated");
        f.add_step(FlowStep::new("advise", "advisor").with_gate(GateType::Human));
        let mut eng = PipelineEngine::new(f, "run-3");
        eng.advance(); // WaitForHuman
        let r = eng.resolve_gate(GateDecision::Reject {
            feedback: "needs more detail".into(),
        });
        // Reject still re-enters the step (redraft).
        assert!(matches!(r, AdvanceResult::ExecuteStep { ref step_id, .. } if step_id == "advise"));
        assert!(eng.feedback_for("advise").iter().any(|s| s.contains("needs more detail")));
    }

    #[test]
    fn handoff_target_auto_corrected() {
        let mut eng = PipelineEngine::new(two_step_flow(), "run-4");
        eng.advance(); // ExecuteStep a
        // Hand off to the WRONG target — engine should correct to "architect".
        let r = eng.submit_handoff(handoff("advisor", "wrong-target", "run-4"));
        assert!(matches!(r, AdvanceResult::ExecuteStep { .. }));
        assert!(eng.feedback_for("a").iter().any(|s| s.contains("AUTO-CORRECTION")));
    }

    #[test]
    fn loop_pauses_at_max_iterations() {
        // code → loop back to test-first, max 2 iterations.
        let mut f = FlowSpec::new("loop");
        f.add_step(FlowStep::new("test", "tester"));
        f.add_step(
            FlowStep::new("code", "coder")
                .with_exit(ExitRouting::Loop { target_step_id: "test".into(), max_iterations: 2 }),
        );
        let mut eng = PipelineEngine::new(f, "run-5");

        eng.advance(); // test
        eng.submit_handoff(handoff("tester", "coder", "run-5")); // → code
        eng.submit_handoff(handoff("coder", "tester", "run-5")); // iter 1 → test
        eng.submit_handoff(handoff("tester", "coder", "run-5")); // → code
        // 2nd handoff from code hits max iterations → Pause.
        let r = eng.submit_handoff(handoff("coder", "tester", "run-5"));
        assert!(matches!(r, AdvanceResult::Paused { .. }));
        assert!(matches!(eng.status, PipelineStatus::Paused { .. }));
    }

    #[test]
    fn budget_hardstop_fails_run() {
        let mut eng = PipelineEngine::with_budget(
            two_step_flow(),
            "run-6",
            TokenBudget::new(100),
        );
        eng.advance(); // ExecuteStep a
        let mut h = handoff("advisor", "architect", "run-6");
        h.token_usage.step_input = 200; // exceeds 100-token budget
        let r = eng.submit_handoff(h);
        assert!(matches!(r, AdvanceResult::Failed { .. }));
        assert!(matches!(eng.status, PipelineStatus::Failed { .. }));
    }

    #[test]
    fn rerun_from_failure() {
        let mut eng = PipelineEngine::new(two_step_flow(), "run-7");
        eng.status = PipelineStatus::Failed { error: "boom".into() };
        eng.current_step = 1;
        let r = eng.rerun();
        assert!(matches!(r, Some(AdvanceResult::ExecuteStep { .. })));
        assert!(matches!(eng.status, PipelineStatus::Running { .. }));
    }
}
