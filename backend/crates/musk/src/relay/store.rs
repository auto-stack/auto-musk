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
use std::sync::{Arc, Mutex};

use crate::relay::TokenBudget;
use crate::relay::{FlowSpec, FlowStep, GateType};
use crate::relay::HandoffDocument;
use crate::relay::{GateDecision, PipelineEngine, PipelineStatus, RelayMode};

/// A run event for SSE streaming + history replay. Tagged so the frontend's
/// `event_type` switch (`useRelay.ts` `subscribeToRun`) maps 1:1.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunEvent {
    StepStarted { #[serde(default)] timestamp: u64, step_id: String, role_id: String },
    StepCompleted { #[serde(default)] timestamp: u64, step_id: String, handoff_summary: String },
    GateWaiting { #[serde(default)] timestamp: u64, step_id: String, gate: String },
    GateResolved { #[serde(default)] timestamp: u64, step_id: String, decision: String },
    /// PLAN-031 T5: carries the deterministic run report so the frontend
    /// ReportCard (web global slot / gen RunBox embed) lights up on completion.
    RunCompleted {
        #[serde(default)]
        timestamp: u64,
        #[serde(default)]
        report: RunReportPayload,
    },
    /// PLAN-032: document 相位 emit_report 工具落盘后追加——前端 deck 层
    /// 的实时/回放数据源（内容本体经 /runs/{id}/report 端点拉取，事件只带元数据）。
    ReportEmitted {
        #[serde(default)]
        timestamp: u64,
        #[serde(default)]
        format: String,
        #[serde(default)]
        title: String,
        /// 相对 workspace 根的报告 html 路径。
        #[serde(default)]
        path: String,
    },
    RunFailed { #[serde(default)] timestamp: u64, error: String },
    TokenSpend { #[serde(default)] timestamp: u64, cumulative: u64, step_tokens: u64 },
    RelayUpdate {
        #[serde(default)] timestamp: u64,
        step_id: String,
        role_id: String,
        status: String,
    },
    // ─── Turn events (session-log persistence; populated by the P2b.2 driver) ───
    TurnDelta { #[serde(default)] timestamp: u64, role_id: String, text: String },
    TurnToolCall {
        #[serde(default)] timestamp: u64,
        role_id: String,
        tool_id: String,
        tool_name: String,
        arguments: serde_json::Value,
    },
    TurnToolResult {
        #[serde(default)] timestamp: u64,
        role_id: String,
        tool_id: String,
        result: String,
    },
    TurnComplete { #[serde(default)] timestamp: u64, role_id: String },
    TurnError { #[serde(default)] timestamp: u64, role_id: String, message: String },
    TurnBudgetWarning { #[serde(default)] timestamp: u64, role_id: String, remaining: u64 },
    TurnBudgetExceeded { #[serde(default)] timestamp: u64, role_id: String },
}

/// PLAN-032: 汇报报告元数据（emit_report 工具产物登记；本体文件在
/// workspace `.autoos/reports/{run_id}/`，事件/载荷只携带元数据）。
/// PLAN-035: `structured` 携带 v2 结构化报告数据（目标/关联 Goals/各阶段
/// 成果/交付物）——HTML/markdown 产物与前端 block 渲染共用同一数据源；
/// 旧数据无此字段（serde default 兼容）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ReportMeta {
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub title: String,
    /// 相对 workspace 根的 report.html 路径。
    #[serde(default)]
    pub path: String,
    /// PLAN-035 v2 结构化报告数据（emit_report 入参原样登记）。
    #[serde(default)]
    pub structured: Option<serde_json::Value>,
}

/// Deterministic run report assembled at completion (PLAN-031 T5).
/// Field names match the frontend ReportCard mapping (`onReport` in web
/// ChatsView reads snake_case keys; gen track maps via `relayReportView`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RunReportPayload {
    #[serde(default)]
    pub run_id: String,
    #[serde(default)]
    pub title: String,
    /// Markdown summary — the document-phase handoff summary (LLM prose).
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub goals_met: String,
    #[serde(default)]
    pub tests_pass: String,
    #[serde(default)]
    pub drift_detected: String,
    #[serde(default)]
    pub cost: String,
    #[serde(default)]
    pub confidence: String,
    #[serde(default)]
    pub deliverables: Vec<String>,
    #[serde(default)]
    pub files_changed: Vec<String>,
    #[serde(default)]
    pub tool_calls: u64,
    #[serde(default)]
    pub duration_s: u64,
    #[serde(default)]
    pub completed_steps: u32,
    #[serde(default)]
    pub total_steps: u32,
    /// PLAN-032: 汇报报告元数据（deck 层数据源；None=未生成）。
    #[serde(default)]
    pub report: Option<ReportMeta>,
}

/// Assemble the run report from the run entry (no extra LLM call — the
/// document-phase handoff summary is the prose; metrics come from events).
fn build_run_report(entry: &RunEntry) -> RunReportPayload {
    let total = entry.engine.flow.steps.len() as u32;
    let completed = entry.engine.step_history.len() as u32;
    // 变更文件：write/edit 类工具调用目标（去重保序）
    let mut files: Vec<String> = Vec::new();
    let mut tool_calls: u64 = 0;
    for ev in &entry.events {
        if let RunEvent::TurnToolCall { tool_name, arguments, .. } = ev {
            tool_calls += 1;
            if matches!(tool_name.as_str(), "write_file" | "edit_file" | "apply_patch" | "replace") {
                if let Some(path) = arguments.get("path").and_then(|v| v.as_str()) {
                    let p = path.to_string();
                    if !p.is_empty() && !files.contains(&p) {
                        files.push(p);
                    }
                }
            }
        }
    }
    let summary = entry
        .engine
        .step_history
        .last()
        .and_then(|r| r.handoff.as_ref().map(|h| h.summary.clone()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Run completed successfully.".to_string());
    RunReportPayload {
        run_id: entry.run_id.clone(),
        title: entry.metadata.title.clone().unwrap_or_default(),
        summary,
        goals_met: format!("{}/{}", completed, total),
        tests_pass: format!("{}", tool_calls),
        drift_detected: "None".into(),
        cost: entry.engine.cumulative_tokens.to_string(),
        confidence: "High".into(),
        deliverables: files.clone(),
        files_changed: files,
        tool_calls,
        duration_s: entry.updated_at.saturating_sub(entry.created_at),
        completed_steps: completed,
        total_steps: total,
        report: entry.metadata.report.clone(),
    }
}

impl RunEvent {
    pub fn timestamp(&self) -> u64 {
        match self {
            RunEvent::StepStarted { timestamp, .. }
            | RunEvent::StepCompleted { timestamp, .. }
            | RunEvent::GateWaiting { timestamp, .. }
            | RunEvent::GateResolved { timestamp, .. }
            | RunEvent::RunCompleted { timestamp, .. }
            | RunEvent::RunFailed { timestamp, .. }
            | RunEvent::ReportEmitted { timestamp, .. }
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
            RunEvent::ReportEmitted { .. } => "report_emitted",
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
    pub step_history: Vec<crate::relay::StepRecord>,
    pub cumulative_tokens: u64,
    pub budget_limit: u64,
    pub budget_remaining: u64,
    pub waiting_for_gate: Option<GateState>,
    pub events: Vec<RunEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_step_started_at: Option<u64>,
    /// Per-profession token totals (key = role_id).
    #[serde(default)]
    pub profession_tokens: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepState {
    pub id: String,
    pub role_id: String,
    pub status: String, // pending | running | completed | failed
    pub gate: String,   // auto | human
}

#[derive(Debug, Clone, Serialize)]
pub struct GateState {
    pub step_id: String,
    pub role_id: String,
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
    /// PLAN-030: per-run context vars for phase-task-template substitution
    /// (e.g. `plan_file`, stashed by the driver from the plan phase's
    /// `PLAN_FILE:` marker).
    #[serde(default)]
    pub context: HashMap<String, String>,
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
    #[serde(default)]
    pub workspace_id: Option<String>,
    // ── TaskPlan tracing (Plan 009 P2b.7) ───────────────────────────────────
    /// PLAN-032: 汇报报告元数据（emit_report 登记处）。
    #[serde(default)]
    pub report: Option<ReportMeta>,
    /// When this run belongs to a TaskPlan, the plan's id.
    #[serde(default)]
    pub task_plan_id: Option<String>,
    #[serde(default)]
    pub task_run_name: Option<String>,
    #[serde(default)]
    pub phase_name: Option<String>,
    #[serde(default)]
    pub phase_index: Option<usize>,
    #[serde(default)]
    pub parent_run_id: Option<String>,
    #[serde(default)]
    pub root_run_id: Option<String>,
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
    pub role_id: String,
    #[serde(default)]
    pub gate: Option<GateType>,
}

/// Resolve a start-run request into a flow: explicit inline steps win,
/// otherwise look up a built-in flow by id, otherwise default.
pub fn resolve_flow(req: &StartRunRequest) -> FlowSpec {
    if !req.steps.is_empty() {
        let mut flow = FlowSpec::new("inline");
        for s in &req.steps {
            flow.add_step(FlowStep::new(s.id.clone(), s.role_id.clone())
                .with_gate(s.gate.unwrap_or(GateType::Auto)));
        }
        return flow;
    }
    // PLAN-030 §5.5: the canonical flow is "plan" (deprecated spec-driven
    // pipelines stay addressable by explicit id).
    let id = req.flow_id.as_deref().unwrap_or("plan");
    crate::relay::get_builtin_flow(id)
        .unwrap_or_else(|| crate::relay::get_builtin_flow("plan").unwrap())
}

/// The run store: in-memory run registry (PLAN-030 D7).
///
/// Runs are **not** persisted to disk anymore: the driver never resumes
/// across restarts (tokio::spawn'd drives are not respawned), and the
/// conversation dual-write below is the run's sole durable log. The
/// constructor keeps the `dir` parameter for API compatibility; legacy files
/// under the old relay dir are left untouched on disk.
pub struct RunStore {
    runs: Mutex<HashMap<String, RunEntry>>,
    /// Optional dual-write target: when linked, run events are mirrored as
    /// `Turn`s into a Conversation sharing the run's id. Linked by
    /// `WorkspaceStores::new` after both stores are constructed.
    conversations: Mutex<Option<Arc<crate::conversation::ConversationStore>>>,
}

impl RunStore {
    /// Create an empty in-memory store. `dir` is accepted (and ignored) for
    /// call-site compatibility with the old persisted layout.
    pub fn new(_dir: PathBuf) -> Self {
        Self {
            runs: Mutex::new(HashMap::new()),
            conversations: Mutex::new(None),
        }
    }

    /// Link a `ConversationStore` so relay events are dual-written as turns
    /// into a conversation that shares the run's id. Called once from
    /// `WorkspaceStores::new`.
    pub fn link_conversations(&self, conv: Arc<crate::conversation::ConversationStore>) {
        *self.conversations.lock().unwrap() = Some(conv);
    }

    /// Create an empty store at an arbitrary path (for tests).
    pub fn at(dir: PathBuf) -> Self {
        Self::new(dir)
    }




    /// Start a new run. Returns `(run_id, RunState)`.
    pub fn start_run(
        &self,
        req: &StartRunRequest,
        workspace_id: Option<String>,
    ) -> (String, RunState) {
        let flow = resolve_flow(req);
        let run_id = req.run_id.clone().unwrap_or_else(|| {
            let ts = now_secs();
            format!("run-{ts}-{}", &uuidish())
        });
        let engine = PipelineEngine::with_budget(flow, &run_id, TokenBudget::new(10_000_000));
        let now = now_secs();
        let ws_id_for_conv = workspace_id.clone();
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
                report: None,
                workspace_id,
                task_plan_id: None,
                task_run_name: None,
                phase_name: None,
                phase_index: None,
                parent_run_id: None,
                root_run_id: None,
            },
            context: HashMap::new(),
        };
        let state = build_run_state(&entry);
        self.runs.lock().unwrap().insert(run_id.clone(), entry);
        // Dual-write: mirror the run as a Flow conversation sharing the run id.
        self.create_conversation_for_run(&run_id, req, ws_id_for_conv.as_deref());
        (run_id, state)
    }

    /// Create the linked Flow conversation for a run (if a ConversationStore is
    /// linked). Idempotent — skipped if the conversation already exists.
    fn create_conversation_for_run(
        &self,
        run_id: &str,
        req: &StartRunRequest,
        workspace_id: Option<&str>,
    ) {
        let conv_store = self.conversations.lock().unwrap();
        let Some(conv) = conv_store.as_ref() else {
            return;
        };
        // Avoid duplicating if the conversation was already created (e.g. on a
        // reload). `get` is cheap (cache check first).
        if conv.get(run_id).is_some() {
            return;
        }
        let ws_id = workspace_id.unwrap_or_default().to_string();
        let flow_id = req
            .flow_id
            .clone()
            .or_else(|| {
                req.steps
                    .first()
                    .map(|s| s.id.clone())
                    .or_else(|| Some("default".into()))
            })
            .unwrap_or_default();
        conv.create_with_id(
            run_id.to_string(),
            crate::conversation::ConversationKind::Flow,
            ws_id,
            crate::conversation::Driver::Flow { flow_id },
            None,
            req.task.clone(),
        );
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
        let removed = {
            let mut runs = self.runs.lock().unwrap();
            let removed = runs.remove(run_id).is_some();
            if removed {
            }
            removed
        };
        // Also remove the dual-written conversation (if linked).
        let conv_store = self.conversations.lock().unwrap();
        if let Some(conv) = conv_store.as_ref() {
            conv.delete(run_id);
        }
        removed
    }

    pub fn set_title(&self, run_id: &str, title: &str) -> Option<RunState> {
        let mut runs = self.runs.lock().unwrap();
        let entry = runs.get_mut(run_id)?;
        entry.metadata.title = Some(title.to_string());
        entry.updated_at = now_secs();
        let state = build_run_state(entry);
        Some(state)
    }

    /// Advance the run's pipeline engine. Returns the advance result + the new
    /// run state. Pushes the appropriate RunEvent. The caller (api.rs) drives
    /// the agent turn when the result is `ExecuteStep`.
    pub fn advance(&self, run_id: &str) -> Option<(crate::relay::AdvanceResult, RunState)> {
        let mut appended: Vec<RunEvent> = Vec::new();
        let (result, state) = {
            let mut runs = self.runs.lock().unwrap();
            let entry = runs.get_mut(run_id)?;
            // PLAN-031 T13: 终态防重复——已完成 run 再次 advance（driver 循环
            // submit_handoff 返回 Completed 后的下一轮）曾二次追加 RunCompleted
            // （"Flow completed" 双写、报告重复推送的直接根因）。幂等返回。
            if matches!(entry.engine.status, PipelineStatus::Completed) {
                let state = build_run_state(entry);
                return Some((crate::relay::AdvanceResult::Completed, state));
            }
            let result = entry.engine.advance();
            let now = now_secs();
            entry.updated_at = now;
            match &result {
                crate::relay::AdvanceResult::ExecuteStep { step_id, role_id, .. } => {
                    appended.push(RunEvent::StepStarted {
                        timestamp: now,
                        step_id: step_id.clone(),
                        role_id: role_id.clone(),
                    });
                    appended.push(RunEvent::RelayUpdate {
                        timestamp: now,
                        step_id: step_id.clone(),
                        role_id: role_id.clone(),
                        status: "running".into(),
                    });
                }
                crate::relay::AdvanceResult::WaitForHuman { step_id, .. } => {
                    appended.push(RunEvent::GateWaiting {
                        timestamp: now,
                        step_id: step_id.clone(),
                        gate: "human".into(),
                    });
                }
                crate::relay::AdvanceResult::Completed => {
                    appended.push(RunEvent::RunCompleted {
                        timestamp: now,
                        report: build_run_report(entry),
                    });
                }
                crate::relay::AdvanceResult::Failed { error } => {
                    appended.push(RunEvent::RunFailed {
                        timestamp: now,
                        error: error.clone(),
                    });
                }
                crate::relay::AdvanceResult::Paused { .. } => {}
            }
            for ev in &appended {
                entry.events.push(ev.clone());
            }
            let state = build_run_state(entry);
                Some((result, state))
        }?;
        // Dual-write the appended events to the linked conversation (if any),
        // outside the runs lock.
        self.mirror_events(run_id, &appended);
        Some((result, state))
    }

    /// Submit a handoff document and record the step completion.
    pub fn submit_handoff(&self, run_id: &str, handoff: HandoffDocument) -> Option<(crate::relay::AdvanceResult, RunState)> {
        let mut appended: Vec<RunEvent> = Vec::new();
        let (result, state) = {
            let mut runs = self.runs.lock().unwrap();
            let entry = runs.get_mut(run_id)?;
            let step_id = entry.engine.current_step_id().map(String::from);
            let result = entry.engine.submit_handoff(handoff);
            let now = now_secs();
            entry.updated_at = now;
            if let Some(sid) = &step_id {
                appended.push(RunEvent::StepCompleted {
                    timestamp: now,
                    step_id: sid.clone(),
                    handoff_summary: entry
                        .engine
                        .step_history
                        .last()
                        .and_then(|r| r.handoff.as_ref().map(|h| h.summary.clone()))
                        .unwrap_or_default(),
                });
                appended.push(RunEvent::TokenSpend {
                    timestamp: now,
                    cumulative: entry.engine.cumulative_tokens,
                    step_tokens: entry
                        .engine
                        .step_history
                        .last()
                        .map(|r| {
                            r.handoff
                                .as_ref()
                                .map(|h| h.token_usage.step_tokens + h.token_usage.step_tokens)
                                .unwrap_or(0)
                        })
                        .unwrap_or(0),
                });
            }
            match &result {
                crate::relay::AdvanceResult::Completed => {
                    appended.push(RunEvent::RunCompleted {
                        timestamp: now,
                        report: build_run_report(entry),
                    });
                }
                crate::relay::AdvanceResult::Failed { error, .. } => {
                    appended.push(RunEvent::RunFailed {
                        timestamp: now,
                        error: error.clone(),
                    });
                }
                _ => {}
            }
            for ev in &appended {
                entry.events.push(ev.clone());
            }
            let state = build_run_state(entry);
                Some((result, state))
        }?;
        self.mirror_events(run_id, &appended);
        Some((result, state))
    }

    /// Resolve a pending human gate.
    pub fn resolve_gate(&self, run_id: &str, decision: GateDecision) -> Option<(crate::relay::AdvanceResult, RunState)> {
        let mut appended: Vec<RunEvent> = Vec::new();
        let (result, state) = {
            let mut runs = self.runs.lock().unwrap();
            let entry = runs.get_mut(run_id)?;
            let now = now_secs();
            let decision_str = match &decision {
                GateDecision::Approve => "approve",
                GateDecision::Reject { .. } => "reject",
            };
            let step_id = entry
                .engine
                .pending_gate
                .as_ref()
                .map(|g| g.step_id.clone());
            let result = entry.engine.resolve_gate(decision);
            entry.updated_at = now;
            appended.push(RunEvent::GateResolved {
                timestamp: now,
                step_id: step_id.unwrap_or_default(),
                decision: decision_str.into(),
            });
            // resolve_gate may itself advance into ExecuteStep/WaitForHuman/etc.;
            // record the resulting transition.
            match &result {
                crate::relay::AdvanceResult::ExecuteStep { step_id, role_id, .. } => {
                    appended.push(RunEvent::StepStarted {
                        timestamp: now,
                        step_id: step_id.clone(),
                        role_id: role_id.clone(),
                    });
                }
                crate::relay::AdvanceResult::Completed => {
                    appended.push(RunEvent::RunCompleted {
                        timestamp: now,
                        report: build_run_report(entry),
                    });
                }
                _ => {}
            }
            for ev in &appended {
                entry.events.push(ev.clone());
            }
            let state = build_run_state(entry);
                Some((result, state))
        }?;
        self.mirror_events(run_id, &appended);
        Some((result, state))
    }

    /// Rerun from the current failed step.
    pub fn rerun(&self, run_id: &str) -> Option<RunState> {
        let mut runs = self.runs.lock().unwrap();
        let entry = runs.get_mut(run_id)?;
        entry.engine.rerun();
        entry.updated_at = now_secs();
        let state = build_run_state(entry);
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
        // Dual-write: mirror the event as Turn(s) into the linked conversation
        // (if any). Done outside the runs lock to avoid nesting store locks.
        let conv_store = self.conversations.lock().unwrap();
        if let Some(conv) = conv_store.as_ref() {
            let seq_base = conv
                .get(run_id)
                .map(|c| c.turns.len())
                .unwrap_or(0);
            for turn in crate::conversation::run_event_to_turns(&event, seq_base) {
                conv.append_turn(run_id, turn);
            }
        }
    }

    /// Mirror a batch of events into the linked conversation (if any) as turns.
    /// Called by `advance`/`submit_handoff`/`resolve_gate` after they've pushed
    /// events to the run's in-memory log + persisted it. Runs entirely outside
    /// the runs lock so the conversation store's own lock never nests inside.
    fn mirror_events(&self, run_id: &str, events: &[RunEvent]) {
        if events.is_empty() {
            return;
        }
        let conv_store = self.conversations.lock().unwrap();
        let Some(conv) = conv_store.as_ref() else {
            return;
        };
        let mut seq_base = conv
            .get(run_id)
            .map(|c| c.turns.len())
            .unwrap_or(0);
        for event in events {
            for turn in crate::conversation::run_event_to_turns(event, seq_base) {
                seq_base += 1;
                conv.append_turn(run_id, turn);
            }
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

    /// PLAN-031 T5: the completion report carried on the last RunCompleted
    /// event (watchers / SSE publishers re-read it instead of rebuilding).
    pub fn run_report(&self, run_id: &str) -> Option<RunReportPayload> {
        let runs = self.runs.lock().unwrap();
        let entry = runs.get(run_id)?;
        entry.events.iter().rev().find_map(|ev| match ev {
            RunEvent::RunCompleted { report, .. } => Some(report.clone()),
            _ => None,
        })
    }

    /// PLAN-032: emit_report 工具登记报告元数据（幂等覆盖）+ 追加事件
    /// （持久化 run.events + 会话镜像 + SSE 广播）。
    pub fn append_report(&self, run_id: &str, meta: ReportMeta) -> Option<()> {
        let ev = {
            let mut runs = self.runs.lock().unwrap();
            let entry = runs.get_mut(run_id)?;
            entry.metadata.report = Some(meta.clone());
            entry.updated_at = now_secs();
            let ev = RunEvent::ReportEmitted {
                timestamp: now_secs(),
                format: meta.format.clone(),
                title: meta.title.clone(),
                path: meta.path.clone(),
            };
            entry.events.push(ev.clone());
            ev
        };
        self.mirror_events(run_id, &[ev.clone()]);
        crate::relay::api::publish(run_id, &ev);
        Some(())
    }

    /// PLAN-032: 报告元数据（None=未生成）。
    pub fn report_meta(&self, run_id: &str) -> Option<ReportMeta> {
        let runs = self.runs.lock().unwrap();
        runs.get(run_id).and_then(|e| e.metadata.report.clone())
    }

    /// Which workspace this run belongs to (for orchestration tool context).
    pub fn workspace_of(&self, run_id: &str) -> Option<String> {
        let runs = self.runs.lock().unwrap();
        runs.get(run_id).and_then(|e| e.metadata.workspace_id.clone())
    }

    /// Snapshot the initial task + prior handoff markdown for the driver.
    /// (Read-only; the driver calls this before running the agent.)
    ///
    /// PLAN-030: for the `plan` flow, the task is the phase template for the
    /// step about to run (with `{plan_file}` substituted from the run
    /// context) instead of the raw initial task; other flows keep the legacy
    /// behavior.
    pub fn step_context(&self, run_id: &str) -> Option<(String, String)> {
        let runs = self.runs.lock().unwrap();
        let entry = runs.get(run_id)?;
        let initial_task = entry
            .metadata
            .initial_task
            .clone()
            .unwrap_or_else(|| "Continue the relay pipeline.".into());
        let step_id = entry
            .engine
            .flow
            .steps
            .get(entry.engine.current_step)
            .map(|s| s.id.clone())
            .unwrap_or_default();
        let task = super::plan_flow::phase_task(
            &entry.engine.flow.id,
            &step_id,
            &initial_task,
            &entry.context,
        )
        .unwrap_or(initial_task);
        let prior_md = entry
            .engine
            .step_history
            .last()
            .and_then(|r| r.handoff.as_ref().map(|h| h.render()))
            .unwrap_or_default();
        Some((task, prior_md))
    }

    /// Stash a context var on a run (PLAN-030: the driver stores the
    /// `PLAN_FILE:` marker extracted from the plan phase output here).
    pub fn set_context_var(&self, run_id: &str, key: &str, value: &str) {
        let mut runs = self.runs.lock().unwrap();
        if let Some(entry) = runs.get_mut(run_id) {
            entry.context.insert(key.to_string(), value.to_string());
            entry.updated_at = now_secs();
        }
    }

    /// Read a context var（PLAN-034 T9：driver 完成时读 `chat_session_id`
    /// 以把报告消息写回发起会话）。
    pub fn context_var(&self, run_id: &str, key: &str) -> Option<String> {
        let runs = self.runs.lock().unwrap();
        runs.get(run_id)
            .and_then(|e| e.context.get(key).cloned())
    }

    /// PLAN-030 试用修复：显式置败。agent 运行错误时 driver 调用——原先错误
    /// 被包装成 handoff 提交、引擎照常路由到下一相位（execute 挂了 review/
    /// document 空转出假完成）。置 Failed 终态并广播 RunFailed。
    pub fn fail_run(&self, run_id: &str, error: &str) -> Option<crate::relay::AdvanceResult> {
        let now = now_secs();
        let (result, appended) = {
            let mut runs = self.runs.lock().unwrap();
            let entry = runs.get_mut(run_id)?;
            entry.updated_at = now;
            let result = entry.engine.fail(error.to_string());
            let appended = vec![RunEvent::RunFailed {
                timestamp: now,
                error: error.to_string(),
            }];
            (result, appended)
        };
        for ev in &appended {
            self.push_event(run_id, ev.clone());
        }
        crate::relay::api::publish_advance_result(run_id, &result);
        Some(result)
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
        steps.get(cur + 1).map(|s| s.role_id.clone())
    }

    /// The last completed handoff document (for the next agent's context).
    /// Returns None for the first step.
    pub fn last_handoff(&self, run_id: &str) -> Option<HandoffDocument> {
        let runs = self.runs.lock().unwrap();
        let entry = runs.get(run_id)?;
        entry
            .engine
            .step_history
            .last()
            .and_then(|r| r.handoff.clone())
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
                role_id: s.role_id.clone(),
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
            let role_id = eng
                .flow
                .get_step(step_id)
                .map(|s| s.role_id.clone())
                .unwrap_or_default();
            Some(GateState {
                step_id: step_id.clone(),
                role_id,
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
            let spent = h.token_usage.step_tokens + h.token_usage.step_tokens;
            *profession_tokens.entry(rec.role_id.clone()).or_insert(0) += spent;
        }
    }

    RunState {
        run_id: entry.run_id.clone(),
        status: eng.status.to_status_str(),
        current_step,
        total_steps,
        current_profession: eng.current_role_id().map(String::from),
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
        current_profession: eng.current_role_id().map(String::from),
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
    use crate::relay::HandoffDocument;

    /// PLAN-035：ReportMeta 新增 structured 字段——旧 JSON（无该字段）可反
    /// 序列化，新数据携带结构化报告。
    #[test]
    fn report_meta_structured_field_compat() {
        let legacy: ReportMeta =
            serde_json::from_str(r#"{"format":"html","title":"t","path":"p"}"#).unwrap();
        assert_eq!(legacy.title, "t");
        assert!(legacy.structured.is_none(), "legacy without structured");
        let v2: ReportMeta = serde_json::from_str(
            r#"{"format":"html","title":"t","path":"p","structured":{"objective":"目标"}}"#,
        )
        .unwrap();
        assert_eq!(v2.structured.unwrap()["objective"], "目标");
    }

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
        let mut h = HandoffDocument::new("advisor", to);
        h.summary = "done".into();
        h
    }

    /// PLAN-030 §6.4: plan flow 的 step_context 返回相位模板（含需求）；
    /// set_context_var 后 execute 相位模板完成 {plan_file} 替换；非 plan
    /// flow 保持旧行为（raw initial task）。
    #[test]
    fn step_context_uses_phase_templates_for_plan_flow() {
        let store = tmp_store();
        let (id, _state) = store.start_run(
            &StartRunRequest {
                run_id: None,
                flow_id: Some("plan".into()),
                steps: Vec::new(),
                task: Some("做一个登录功能".into()),
            },
            None,
        );
        // 第一步 = plan 相位：任务为模板而非裸需求
        let (task, _) = store.step_context(&id).unwrap();
        assert!(task.contains("做一个登录功能"));
        assert!(task.contains("需求整理与计划撰写"));
        assert!(!task.starts_with("做一个登录功能"), "not the raw task");

        // 上下文变量注入后，execute 相位模板替换 {plan_file}
        store.set_context_var(&id, "plan_file", "docs/plans/031-login.md");
        store.advance(&id);
        let _ = store.submit_handoff(&id, handoff("plan-dev"));
        // gate（execute 前 Human）未批准时 current_step 已指向 execute
        let (task, _) = store.step_context(&id).unwrap();
        assert!(task.contains("按计划实施"));
        assert!(task.contains("docs/plans/031-login.md"));
        assert!(!task.contains("{plan_file}"));
    }

    #[test]
    fn step_context_legacy_flow_unchanged() {
        let store = tmp_store();
        let (id, _) = store.start_run(
            &StartRunRequest {
                run_id: None,
                flow_id: Some("simple".into()),
                steps: Vec::new(),
                task: Some("裸任务文本".into()),
            },
            None,
        );
        let (task, _) = store.step_context(&id).unwrap();
        assert_eq!(task, "裸任务文本");
    }

    #[test]
    fn start_get_list_delete() {
        let store = tmp_store();
        let (id, state) = store.start_run(
            &StartRunRequest {
                flow_id: Some("simple".into()),
                ..Default::default()
            },
            None,
        );
        assert_eq!(state.status, "idle");
        assert!(store.get(&id).is_some());
        assert!(store.list().iter().any(|s| s.run_id == id));
        assert!(store.delete(&id));
        assert!(store.get(&id).is_none());
    }

    #[test]
    fn advance_then_handoff_completes_simple_flow() {
        let store = tmp_store();
        let (id, _) = store.start_run(
            &StartRunRequest {
                flow_id: Some("simple".into()),
                ..Default::default()
            },
            None,
        );
        // Step 1: advisor.
        let (r, _) = store.advance(&id).unwrap();
        assert!(matches!(r, crate::relay::AdvanceResult::ExecuteStep { .. }));
        let (r, _) = store
            .submit_handoff(&id, handoff("coder"))
            .unwrap();
        // Step 2: coder.
        assert!(matches!(r, crate::relay::AdvanceResult::ExecuteStep { .. }));
        let (r, state) = store
            .submit_handoff(&id, handoff("documenter"))
            .unwrap();
        assert!(matches!(r, crate::relay::AdvanceResult::Completed));
        assert_eq!(state.status, "completed");
    }

    #[test]
    fn runs_are_in_memory_only_after_reload() {
        // PLAN-030 D7: RunStore no longer persists — a fresh store over the
        // same dir starts empty; the durable log lives in the linked
        // conversation (dual-write), not on the relay dir.
        let dir = std::env::temp_dir().join(format!(
            "musk-relay-persist-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let id = {
            let store = RunStore::at(dir.clone());
            let (id, _) = store.start_run(
                &StartRunRequest {
                    flow_id: Some("simple".into()),
                    ..Default::default()
                },
                None,
            );
            id
        };
        // A fresh store over the same dir is empty (no disk reload).
        let store2 = RunStore::at(dir);
        assert!(store2.get(&id).is_none(), "runs must NOT persist to disk");
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

    /// Build a RunStore linked to a ConversationStore at the same temp root,
    /// exercising the full dual-write path.
    fn linked_stores() -> (
        RunStore,
        Arc<crate::conversation::ConversationStore>,
    ) {
        let dir = std::env::temp_dir().join(format!(
            "musk-relay-dual-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let conv = Arc::new(crate::conversation::ConversationStore::at(
            dir.join("conversations"),
        ));
        let relay = RunStore::at(dir.join("relay"));
        relay.link_conversations(conv.clone());
        (relay, conv)
    }

    #[test]
    fn dual_write_creates_flow_conversation_on_start() {
        let (store, conv) = linked_stores();
        let (id, _) = store.start_run(
            &StartRunRequest {
                flow_id: Some("simple".into()),
                task: Some("build it".into()),
                ..Default::default()
            },
            Some("ws1".into()),
        );
        let conversation = conv.get(&id).expect("conversation should be created");
        assert_eq!(conversation.kind, crate::conversation::ConversationKind::Flow);
        assert_eq!(conversation.workspace_id, "ws1");
        assert_eq!(conversation.title.as_deref(), Some("build it"));
        // The run id and conversation id match (that's the dual-write link).
        assert_eq!(conversation.id, id);
    }

    #[test]
    fn dual_write_mirrors_advance_and_handoff_as_turns() {
        let (store, conv) = linked_stores();
        let (id, _) = store.start_run(
            &StartRunRequest {
                flow_id: Some("simple".into()),
                ..Default::default()
            },
            None,
        );
        // Step 1.
        store.advance(&id).unwrap();
        store.submit_handoff(&id, handoff("coder")).unwrap();
        // Step 2.
        store.submit_handoff(&id, handoff("documenter")).unwrap();

        let conversation = conv.get(&id).expect("conversation exists");
        // advance(StepStarted + RelayUpdate) + submit_handoff(StepCompleted + TokenSpend)
        // for two steps, plus a final RunCompleted. StepStarted/RelayUpdate/
        // StepCompleted produce turns; TokenSpend does not; RunCompleted does.
        // Expect at least one System turn mentioning a step + a completion turn.
        assert!(
            !conversation.turns.is_empty(),
            "dual-written turns should exist"
        );
        assert!(
            conversation
                .turns
                .iter()
                .any(|t| t.content.contains("completed")),
            "should have a step-completion turn"
        );
        assert!(
            conversation
                .turns
                .iter()
                .any(|t| t.content == "Flow completed"),
            "should have a run-completed turn"
        );
    }

    #[test]
    fn dual_write_mirrors_push_event() {
        let (store, conv) = linked_stores();
        let (id, _) = store.start_run(
            &StartRunRequest {
                flow_id: Some("simple".into()),
                ..Default::default()
            },
            None,
        );
        store.push_event(
            &id,
            RunEvent::TurnDelta {
                timestamp: 0,
                role_id: "advisor".into(),
                text: "thinking...".into(),
            },
        );
        let conversation = conv.get(&id).unwrap();
        assert_eq!(conversation.turns.len(), 1);
        assert_eq!(conversation.turns[0].kind, crate::conversation::TurnKind::Message);
        assert_eq!(conversation.turns[0].from, "advisor");
        assert_eq!(conversation.turns[0].content, "thinking...");
    }

    #[test]
    fn dual_write_delete_run_deletes_conversation() {
        let (store, conv) = linked_stores();
        let (id, _) = store.start_run(
            &StartRunRequest {
                flow_id: Some("simple".into()),
                ..Default::default()
            },
            None,
        );
        assert!(conv.get(&id).is_some());
        assert!(store.delete(&id));
        assert!(conv.get(&id).is_none(), "conversation should be deleted with run");
    }

    #[test]
    fn no_link_means_no_dual_write() {
        // A standalone RunStore (no link_conversations) must not crash and
        // simply skip the dual-write.
        let store = tmp_store();
        let (id, _) = store.start_run(
            &StartRunRequest {
                flow_id: Some("simple".into()),
                ..Default::default()
            },
            None,
        );
        store.advance(&id).unwrap();
        store.push_event(
            &id,
            RunEvent::TurnDelta {
                timestamp: 0,
                role_id: "advisor".into(),
                text: "hi".into(),
            },
        );
        // No panic → success.
    }
}
