//! Run Store — in-memory run registry + disk persistence + the read models
//! the frontend (`useRelay.ts`) consumes.
//!
//! Ported from auto-forge `backend/src/relay/store.rs`, injected as
//! `Arc<RunStore>` via `AppState` (not a global singleton) for testability.
//! Persistence is synchronous in P2b.1 (a background async queue arrives with
//! the full driver in P2b.2).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::relay::budget::TokenBudget;
use crate::relay::flow::{FlowSpec, FlowStep, GateType};
use crate::relay::handoff::HandoffDocument;
use crate::relay::pipeline::{GateDecision, PipelineEngine, PipelineStatus, RelayMode};

/// A run event for SSE streaming + history replay. Tagged so the frontend's
/// `event_type` switch (`useRelay.ts` `subscribeToRun`) maps 1:1.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunEvent {
    StepStarted { #[serde(default)] timestamp: u64, step_id: String, profession_id: String },
    StepCompleted { #[serde(default)] timestamp: u64, step_id: String, handoff_summary: String },
    GateWaiting { #[serde(default)] timestamp: u64, step_id: String, gate: String },
    GateResolved { #[serde(default)] timestamp: u64, step_id: String, decision: String },
    RunCompleted { #[serde(default)] timestamp: u64 },
    RunFailed { #[serde(default)] timestamp: u64, error: String },
    TokenSpend { #[serde(default)] timestamp: u64, cumulative: u64, step_tokens: u64 },
    RelayUpdate {
        #[serde(default)] timestamp: u64,
        step_id: String,
        profession_id: String,
        status: String,
    },
    // ─── Turn events (session-log persistence; populated by the P2b.2 driver) ───
    TurnDelta { #[serde(default)] timestamp: u64, profession_id: String, text: String },
    TurnToolCall {
        #[serde(default)] timestamp: u64,
        profession_id: String,
        tool_id: String,
        tool_name: String,
        arguments: serde_json::Value,
    },
    TurnToolResult {
        #[serde(default)] timestamp: u64,
        profession_id: String,
        tool_id: String,
        result: String,
    },
    TurnComplete { #[serde(default)] timestamp: u64, profession_id: String },
    TurnError { #[serde(default)] timestamp: u64, profession_id: String, message: String },
    TurnBudgetWarning { #[serde(default)] timestamp: u64, profession_id: String, remaining: u64 },
    TurnBudgetExceeded { #[serde(default)] timestamp: u64, profession_id: String },
}

impl RunEvent {
    pub fn timestamp(&self) -> u64 {
        match self {
            RunEvent::StepStarted { timestamp, .. }
            | RunEvent::StepCompleted { timestamp, .. }
            | RunEvent::GateWaiting { timestamp, .. }
            | RunEvent::GateResolved { timestamp, .. }
            | RunEvent::RunCompleted { timestamp }
            | RunEvent::RunFailed { timestamp, .. }
            | RunEvent::TokenSpend { timestamp, .. }
            | RunEvent::RelayUpdate { timestamp, .. }
            | RunEvent::TurnDelta { timestamp, .. }
            | RunEvent::TurnToolCall { timestamp, .. }
            | RunEvent::TurnToolResult { timestamp, .. }
            | RunEvent::TurnComplete { timestamp, .. }
            | RunEvent::TurnError { timestamp, .. }
            | RunEvent::TurnBudgetWarning { timestamp, .. }
            | RunEvent::TurnBudgetExceeded { timestamp, .. } => *timestamp,
        }
    }

    /// Snake-case event_type the frontend switches on.
    pub fn event_type(&self) -> &'static str {
        match self {
            RunEvent::StepStarted { .. } => "step_started",
            RunEvent::StepCompleted { .. } => "step_completed",
            RunEvent::GateWaiting { .. } => "gate_waiting",
            RunEvent::GateResolved { .. } => "gate_resolved",
            RunEvent::RunCompleted { .. } => "run_completed",
            RunEvent::RunFailed { .. } => "run_failed",
            RunEvent::TokenSpend { .. } => "token_spend",
            RunEvent::RelayUpdate { .. } => "relay_update",
            RunEvent::TurnDelta { .. } => "turn_delta",
            RunEvent::TurnToolCall { .. } => "turn_tool_call",
            RunEvent::TurnToolResult { .. } => "turn_tool_result",
            RunEvent::TurnComplete { .. } => "turn_complete",
            RunEvent::TurnError { .. } => "turn_error",
            RunEvent::TurnBudgetWarning { .. } => "turn_budget_warning",
            RunEvent::TurnBudgetExceeded { .. } => "turn_budget_exceeded",
        }
    }
}

/// Lightweight summary for list views.
#[derive(Debug, Clone, Serialize)]
pub struct RunSummary {
    pub run_id: String,
    pub status: String,
    pub current_step: usize,
    pub total_steps: usize,
    pub current_profession: Option<String>,
    pub cumulative_tokens: u64,
    pub created_at: u64,
    pub updated_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
}

/// Detailed run state for the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct RunState {
    pub run_id: String,
    pub status: String,
    pub current_step: usize,
    pub total_steps: usize,
    pub current_profession: Option<String>,
    pub steps: Vec<StepState>,
    pub step_history: Vec<crate::relay::pipeline::StepRecord>,
    pub cumulative_tokens: u64,
    pub budget_limit: u64,
    pub budget_remaining: u64,
    pub waiting_for_gate: Option<GateState>,
    pub events: Vec<RunEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_step_started_at: Option<u64>,
    /// Per-profession token totals (key = profession_id).
    #[serde(default)]
    pub profession_tokens: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepState {
    pub id: String,
    pub profession_id: String,
    pub status: String, // pending | running | completed | failed
    pub gate: String,   // auto | human
}

#[derive(Debug, Clone, Serialize)]
pub struct GateState {
    pub step_id: String,
    pub profession_id: String,
    pub since: u64,
}

/// A persisted relay run: the pipeline engine + event log + metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEntry {
    pub run_id: String,
    pub engine: PipelineEngine,
    pub created_at: u64,
    pub updated_at: u64,
    #[serde(default)]
    pub events: Vec<RunEvent>,
    #[serde(default)]
    pub metadata: RunMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunMetadata {
    #[serde(default)]
    pub title: Option<String>,
    /// Original task description.
    #[serde(default)]
    pub initial_task: Option<String>,
    #[serde(default)]
    pub originating_chat_session: Option<String>,
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Request body for `POST /api/forge/relay/runs` (matches `useRelay.ts`
/// `StartRunRequest`).
#[derive(Debug, Clone, Deserialize)]
pub struct StartRunRequest {
    #[serde(default)]
    pub run_id: Option<String>,
    pub flow_id: Option<String>,
    #[serde(default)]
    pub steps: Vec<StartRunStep>,
    #[serde(default)]
    pub task: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StartRunStep {
    pub id: String,
    pub profession_id: String,
    #[serde(default)]
    pub gate: Option<GateType>,
}

/// Resolve a start-run request into a flow: explicit inline steps win,
/// otherwise look up a built-in flow by id, otherwise default.
pub fn resolve_flow(req: &StartRunRequest) -> FlowSpec {
    if !req.steps.is_empty() {
        let mut flow = FlowSpec::new("inline");
        for s in &req.steps {
            flow.add_step(FlowStep::new(s.id.clone(), s.profession_id.clone())
                .with_gate(s.gate.unwrap_or(GateType::Auto)));
        }
        return flow;
    }
    let id = req.flow_id.as_deref().unwrap_or("default");
    crate::relay::flow::get_builtin_flow(id)
        .unwrap_or_else(|| crate::relay::flow::get_builtin_flow("default").unwrap())
}

/// The run store: in-memory map keyed by run_id, with disk persistence.
pub struct RunStore {
    runs: Mutex<HashMap<String, RunEntry>>,
    dir: PathBuf,
}

impl RunStore {
    /// Create a store rooted at `dir` (e.g. `~/.config/autoos/relay`), loading
    /// any persisted runs from disk.
    pub fn new(dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&dir);
        let store = Self {
            runs: Mutex::new(HashMap::new()),
            dir,
        };
        store.load_all();
        store
    }

    /// Create an empty store at an arbitrary path (for tests).
    pub fn at(dir: PathBuf) -> Self {
        Self::new(dir)
    }

    fn load_all(&self) {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return;
        };
        let mut runs = self.runs.lock().unwrap();
        for entry in entries.flatten() {
            let path = entry.path().join("run.json");
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(entry) = serde_json::from_str::<RunEntry>(&content) {
                    runs.insert(entry.run_id.clone(), entry);
                }
            }
        }
    }

    fn save_run(&self, entry: &RunEntry) {
        let dir = self.dir.join(&entry.run_id);
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        if let Ok(json) = serde_json::to_string_pretty(entry) {
            let _ = std::fs::write(dir.join("run.json"), json);
        }
    }

    fn delete_run_disk(&self, run_id: &str) {
        let _ = std::fs::remove_dir_all(self.dir.join(run_id));
    }

    /// Start a new run. Returns `(run_id, RunState)`.
    pub fn start_run(&self, req: &StartRunRequest) -> (String, RunState) {
        let flow = resolve_flow(req);
        let run_id = req.run_id.clone().unwrap_or_else(|| {
            let ts = now_secs();
            format!("run-{ts}-{}", &uuidish())
        });
        let engine = PipelineEngine::with_budget(flow, &run_id, TokenBudget::new(10_000_000));
        let now = now_secs();
        let entry = RunEntry {
            run_id: run_id.clone(),
            engine,
            created_at: now,
            updated_at: now,
            events: Vec::new(),
            metadata: RunMetadata {
                title: req.task.as_ref().map(|t| truncate_title(t)),
                initial_task: req.task.clone(),
                originating_chat_session: None,
            },
        };
        self.save_run(&entry);
        let state = build_run_state(&entry);
        self.runs.lock().unwrap().insert(run_id.clone(), entry);
        (run_id, state)
    }

    pub fn get(&self, run_id: &str) -> Option<RunState> {
        let runs = self.runs.lock().unwrap();
        runs.get(run_id).map(build_run_state)
    }

    pub fn list(&self) -> Vec<RunSummary> {
        let runs = self.runs.lock().unwrap();
        let mut summaries: Vec<RunSummary> = runs.values().map(build_run_summary).collect();
        // Newest first — the frontend re-sorts by updated_at, but pre-sort anyway.
        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        summaries
    }

    pub fn delete(&self, run_id: &str) -> bool {
        let mut runs = self.runs.lock().unwrap();
        let removed = runs.remove(run_id).is_some();
        if removed {
            self.delete_run_disk(run_id);
        }
        removed
    }

    pub fn set_title(&self, run_id: &str, title: &str) -> Option<RunState> {
        let mut runs = self.runs.lock().unwrap();
        let entry = runs.get_mut(run_id)?;
        entry.metadata.title = Some(title.to_string());
        entry.updated_at = now_secs();
        let state = build_run_state(entry);
        self.save_run(entry);
        Some(state)
    }

    /// Advance the run's pipeline engine. Returns the advance result + the new
    /// run state. Pushes the appropriate RunEvent. The caller (api.rs) drives
    /// the agent turn when the result is `ExecuteStep`.
    pub fn advance(&self, run_id: &str) -> Option<(crate::relay::pipeline::AdvanceResult, RunState)> {
        let mut runs = self.runs.lock().unwrap();
        let entry = runs.get_mut(run_id)?;
        let result = entry.engine.advance();
        let now = now_secs();
        entry.updated_at = now;
        match &result {
            crate::relay::pipeline::AdvanceResult::ExecuteStep { step_id, profession_id, .. } => {
                entry.events.push(RunEvent::StepStarted {
                    timestamp: now,
                    step_id: step_id.clone(),
                    profession_id: profession_id.clone(),
                });
                entry.events.push(RunEvent::RelayUpdate {
                    timestamp: now,
                    step_id: step_id.clone(),
                    profession_id: profession_id.clone(),
                    status: "running".into(),
                });
            }
            crate::relay::pipeline::AdvanceResult::WaitForHuman { step_id, .. } => {
                entry.events.push(RunEvent::GateWaiting {
                    timestamp: now,
                    step_id: step_id.clone(),
                    gate: "human".into(),
                });
            }
            crate::relay::pipeline::AdvanceResult::Completed => {
                entry.events.push(RunEvent::RunCompleted { timestamp: now });
            }
            crate::relay::pipeline::AdvanceResult::Failed { error } => {
                entry.events.push(RunEvent::RunFailed {
                    timestamp: now,
                    error: error.clone(),
                });
            }
            crate::relay::pipeline::AdvanceResult::Paused { .. } => {}
        }
        let state = build_run_state(entry);
        self.save_run(entry);
        Some((result, state))
    }

    /// Submit a handoff document and record the step completion.
    pub fn submit_handoff(&self, run_id: &str, handoff: HandoffDocument) -> Option<(crate::relay::pipeline::AdvanceResult, RunState)> {
        let mut runs = self.runs.lock().unwrap();
        let entry = runs.get_mut(run_id)?;
        let step_id = entry.engine.current_step_id().map(String::from);
        let result = entry.engine.submit_handoff(handoff);
        let now = now_secs();
        entry.updated_at = now;
        if let Some(sid) = &step_id {
            entry.events.push(RunEvent::StepCompleted {
                timestamp: now,
                step_id: sid.clone(),
                handoff_summary: entry
                    .engine
                    .step_history
                    .last()
                    .and_then(|r| r.handoff.as_ref().map(|h| h.summary.clone()))
                    .unwrap_or_default(),
            });
            entry.events.push(RunEvent::TokenSpend {
                timestamp: now,
                cumulative: entry.engine.cumulative_tokens,
                step_tokens: entry
                    .engine
                    .step_history
                    .last()
                    .map(|r| {
                        r.handoff
                            .as_ref()
                            .map(|h| h.token_usage.step_input + h.token_usage.step_output)
                            .unwrap_or(0)
                    })
                    .unwrap_or(0),
            });
        }
        match &result {
            crate::relay::pipeline::AdvanceResult::Completed => {
                entry.events.push(RunEvent::RunCompleted { timestamp: now });
            }
            crate::relay::pipeline::AdvanceResult::Failed { error, .. } => {
                entry.events.push(RunEvent::RunFailed {
                    timestamp: now,
                    error: error.clone(),
                });
            }
            _ => {}
        }
        let state = build_run_state(entry);
        self.save_run(entry);
        Some((result, state))
    }

    /// Resolve a pending human gate.
    pub fn resolve_gate(&self, run_id: &str, decision: GateDecision) -> Option<(crate::relay::pipeline::AdvanceResult, RunState)> {
        let mut runs = self.runs.lock().unwrap();
        let entry = runs.get_mut(run_id)?;
        let now = now_secs();
        let decision_str = match &decision {
            GateDecision::Approve => "approve",
            GateDecision::Reject { .. } => "reject",
            GateDecision::Edit { .. } => "edit",
        };
        let step_id = entry
            .engine
            .pending_gate
            .as_ref()
            .map(|g| g.step_id.clone());
        let result = entry.engine.resolve_gate(decision);
        entry.updated_at = now;
        entry.events.push(RunEvent::GateResolved {
            timestamp: now,
            step_id: step_id.unwrap_or_default(),
            decision: decision_str.into(),
        });
        // resolve_gate may itself advance into ExecuteStep/WaitForHuman/etc.;
        // record the resulting transition.
        match &result {
            crate::relay::pipeline::AdvanceResult::ExecuteStep { step_id, profession_id, .. } => {
                entry.events.push(RunEvent::StepStarted {
                    timestamp: now,
                    step_id: step_id.clone(),
                    profession_id: profession_id.clone(),
                });
            }
            crate::relay::pipeline::AdvanceResult::Completed => {
                entry.events.push(RunEvent::RunCompleted { timestamp: now });
            }
            _ => {}
        }
        let state = build_run_state(entry);
        self.save_run(entry);
        Some((result, state))
    }

    /// Rerun from the current failed step.
    pub fn rerun(&self, run_id: &str) -> Option<RunState> {
        let mut runs = self.runs.lock().unwrap();
        let entry = runs.get_mut(run_id)?;
        entry.engine.rerun();
        entry.updated_at = now_secs();
        let state = build_run_state(entry);
        self.save_run(entry);
        Some(state)
    }

    // ─── P2b.2 driver support ──────────────────────────────────────────────

    /// Append a turn-level event to a run's history + publish it to the SSE
    /// bus. Used by the driver's `on_event` callback (StreamEvent → RunEvent).
    /// Fast + lock-only-around-the-push.
    pub fn push_event(&self, run_id: &str, event: RunEvent) {
        let mut runs = self.runs.lock().unwrap();
        if let Some(entry) = runs.get_mut(run_id) {
            entry.events.push(event.clone());
            entry.updated_at = now_secs();
            let snapshot = build_run_state(entry);
            // Persist + publish outside the lock would be ideal, but the lock is
            // held only briefly; saving a clone keeps the publish accurate.
            drop(runs);
            crate::relay::api::publish(run_id, &event);
            let _ = snapshot;
        }
    }

    /// True if the run is currently being driven (status == Running).
    pub fn is_running(&self, run_id: &str) -> bool {
        let runs = self.runs.lock().unwrap();
        runs.get(run_id)
            .map(|e| matches!(e.engine.status, PipelineStatus::Running { .. }))
            .unwrap_or(false)
    }

    /// Read the run's pending-gate / terminal status without mutation.
    pub fn status(&self, run_id: &str) -> Option<String> {
        let runs = self.runs.lock().unwrap();
        runs.get(run_id).map(|e| e.engine.status.to_status_str())
    }

    /// Snapshot the initial task + prior handoff markdown for the driver.
    /// (Read-only; the driver calls this before running the agent.)
    pub fn step_context(&self, run_id: &str) -> Option<(String, String)> {
        let runs = self.runs.lock().unwrap();
        let entry = runs.get(run_id)?;
        let task = entry
            .metadata
            .initial_task
            .clone()
            .unwrap_or_else(|| "Continue the relay pipeline.".into());
        let prior_md = entry
            .engine
            .step_history
            .last()
            .and_then(|r| r.handoff.as_ref().map(|h| h.render()))
            .unwrap_or_default();
        Some((task, prior_md))
    }

    /// The profession of the step *after* the current one (for handoff `to`).
    /// Called by the driver before submit_handoff.
    pub fn next_profession(&self, run_id: &str) -> Option<String> {
        let runs = self.runs.lock().unwrap();
        let entry = runs.get(run_id)?;
        let cur = entry.engine.current_step;
        let steps = &entry.engine.flow.steps;
        // current_step points at the step about to run; after it completes the
        // "next" is current+1.
        steps.get(cur + 1).map(|s| s.profession_id.clone())
    }
}

/// Build the frontend read-model from a run entry.
fn build_run_state(entry: &RunEntry) -> RunState {
    let eng = &entry.engine;
    let total_steps = eng.flow.steps.len();
    let current_step = eng.current_step.min(total_steps);

    let steps: Vec<StepState> = eng
        .flow
        .steps
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let status = if idx < eng.current_step {
                "completed"
            } else if idx == eng.current_step && matches!(eng.status, PipelineStatus::Running { .. }) {
                "running"
            } else if idx == eng.current_step && matches!(eng.status, PipelineStatus::WaitingForHuman { .. }) {
                "waiting_gate"
            } else {
                "pending"
            };
            StepState {
                id: s.id.clone(),
                profession_id: s.profession_id.clone(),
                status: status.into(),
                gate: match s.gate {
                    GateType::Auto => "auto".into(),
                    GateType::Human => "human".into(),
                },
            }
        })
        .collect();

    let waiting_for_gate = match &eng.status {
        PipelineStatus::WaitingForHuman { step_id, since, .. } => {
            let profession_id = eng
                .flow
                .get_step(step_id)
                .map(|s| s.profession_id.clone())
                .unwrap_or_default();
            Some(GateState {
                step_id: step_id.clone(),
                profession_id,
                since: *since,
            })
        }
        _ => None,
    };

    let current_step_started_at = match &eng.status {
        PipelineStatus::Running { started_at, .. } => Some(*started_at),
        _ => None,
    };

    let budget_remaining = eng
        .budget_tracker
        .run_budget
        .limit
        .saturating_sub(eng.budget_tracker.cumulative);

    // Per-profession token totals.
    let mut profession_tokens: HashMap<String, u64> = HashMap::new();
    for rec in &eng.step_history {
        if let Some(h) = &rec.handoff {
            let spent = h.token_usage.step_input + h.token_usage.step_output;
            *profession_tokens.entry(rec.profession_id.clone()).or_insert(0) += spent;
        }
    }

    RunState {
        run_id: entry.run_id.clone(),
        status: eng.status.to_status_str(),
        current_step,
        total_steps,
        current_profession: eng.current_profession_id().map(String::from),
        steps,
        step_history: eng.step_history.clone(),
        cumulative_tokens: eng.cumulative_tokens,
        budget_limit: eng.budget_tracker.run_budget.limit,
        budget_remaining,
        waiting_for_gate,
        events: trim_events(&entry.events),
        title: entry.metadata.title.clone(),
        current_step_started_at,
        profession_tokens,
    }
}

fn build_run_summary(entry: &RunEntry) -> RunSummary {
    let eng = &entry.engine;
    RunSummary {
        run_id: entry.run_id.clone(),
        status: eng.status.to_status_str(),
        current_step: eng.current_step.min(eng.flow.steps.len()),
        total_steps: eng.flow.steps.len(),
        current_profession: eng.current_profession_id().map(String::from),
        cumulative_tokens: eng.cumulative_tokens,
        created_at: entry.created_at,
        updated_at: entry.updated_at,
        title: entry.metadata.title.clone(),
        task: entry.metadata.initial_task.clone(),
    }
}

/// Cap the in-memory event log (keep the last 500).
fn trim_events(events: &[RunEvent]) -> Vec<RunEvent> {
    if events.len() <= 500 {
        events.to_vec()
    } else {
        events[events.len() - 500..].to_vec()
    }
}

fn truncate_title(s: &str) -> String {
    const MAX: usize = 60;
    if s.chars().count() <= MAX {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(MAX).collect();
        t.push('…');
        t
    }
}

/// A short, mostly-unique id without pulling in the uuid crate.
fn uuidish() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("{now:06x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::handoff::HandoffDocument;

    fn tmp_store() -> RunStore {
        let dir = std::env::temp_dir().join(format!(
            "musk-relay-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        RunStore::at(dir)
    }

    fn handoff(to: &str) -> HandoffDocument {
        let mut h = HandoffDocument::new("advisor", to, "run", 0);
        h.summary = "done".into();
        h
    }

    #[test]
    fn start_get_list_delete() {
        let store = tmp_store();
        let (id, state) = store.start_run(&StartRunRequest {
            flow_id: Some("simple".into()),
            ..Default::default()
        });
        assert_eq!(state.status, "idle");
        assert!(store.get(&id).is_some());
        assert!(store.list().iter().any(|s| s.run_id == id));
        assert!(store.delete(&id));
        assert!(store.get(&id).is_none());
    }

    #[test]
    fn advance_then_handoff_completes_simple_flow() {
        let store = tmp_store();
        let (id, _) = store.start_run(&StartRunRequest {
            flow_id: Some("simple".into()),
            ..Default::default()
        });
        // Step 1: advisor.
        let (r, _) = store.advance(&id).unwrap();
        assert!(matches!(r, crate::relay::pipeline::AdvanceResult::ExecuteStep { .. }));
        let (r, _) = store
            .submit_handoff(&id, handoff("coder"))
            .unwrap();
        // Step 2: coder.
        assert!(matches!(r, crate::relay::pipeline::AdvanceResult::ExecuteStep { .. }));
        let (r, state) = store
            .submit_handoff(&id, handoff("documenter"))
            .unwrap();
        assert!(matches!(r, crate::relay::pipeline::AdvanceResult::Completed));
        assert_eq!(state.status, "completed");
    }

    #[test]
    fn persists_across_reload() {
        let dir = std::env::temp_dir().join(format!(
            "musk-relay-persist-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let id = {
            let store = RunStore::at(dir.clone());
            let (id, _) = store.start_run(&StartRunRequest {
                flow_id: Some("simple".into()),
                ..Default::default()
            });
            id
        };
        // A fresh store over the same dir reloads the run.
        let store2 = RunStore::at(dir);
        assert!(store2.get(&id).is_some(), "run should persist across reload");
    }

    impl Default for StartRunRequest {
        fn default() -> Self {
            StartRunRequest {
                run_id: None,
                flow_id: None,
                steps: Vec::new(),
                task: None,
            }
        }
    }
}
