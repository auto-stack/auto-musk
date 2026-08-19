//! Orchestration tools — `spawn_relay`, `dispatch`, and `bring_in`.
//!
//! These tools let an agent create nested sub-conversations. When called, they:
//! 1. Create a child Conversation (kind=Flow for spawn_relay, kind=Errand for dispatch/bring_in)
//! 2. Record a Turn with `child_conversation` in the parent conversation
//! 3. Drive the sub-conversation asynchronously (spawn_relay) or synchronously (dispatch/bring_in)
//! 4. Wait for completion, then return the summary as the tool_result string
//!
//! The Tool trait's `execute(&self, args)` carries no business context, so these
//! tools hold a [`ToolContext`] struct field injected at agent-build time.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use auto_ai_agent::{Tool, ToolError};

use crate::conversation::{
    self, ConversationKind, ConversationStatus, Driver, GateInfo, Turn, TurnKind,
};
use crate::relay::store::{RunEvent, StartRunRequest};
use crate::tool_context::ToolContext;

// ─── SpawnRelay ─────────────────────────────────────────────────────────────

/// `spawn_relay` — start a relay flow (multi-agent pipeline) as a child
/// conversation. Returns the flow's final status + a summary when it completes.
pub struct SpawnRelay {
    ctx: ToolContext,
}

impl SpawnRelay {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for SpawnRelay {
    fn name(&self) -> &str {
        "spawn_relay"
    }

    fn description(&self) -> &str {
        "Start a relay flow (multi-agent pipeline) in the background. Use for \
         complex tasks that need multiple specialists (advisor → architect → \
         coder → tester → reviewer). Args: flow_id (optional, default='default'), \
         task (required, a one-sentence description of what to accomplish). \
         Returns {run_id, status:'started'} immediately — the flow keeps running \
         (pausing at human gates); do NOT wait or poll for its completion, just \
         tell the user the run has started and where to follow/approve it."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "flow_id": {
                    "type": "string",
                    "description": "Flow template id (default, simple). Defaults to 'default'."
                },
                "task": {
                    "type": "string",
                    "description": "A clear one-sentence description of the task."
                }
            },
            "required": ["task"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let flow_id = args["flow_id"].as_str().unwrap_or("default").to_string();
        let task = args["task"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if task.is_empty() {
            return Err(ToolError::Exec("spawn_relay: 'task' is required".into()));
        }

        let ws = self.ctx.state.registry.get(&self.ctx.workspace_id);

        // 1. Create child conversation.
        let child = ws.conversations.create(
            ConversationKind::Flow,
            self.ctx.workspace_id.clone(),
            Driver::Flow { flow_id: flow_id.clone() },
            None,
            Some(task.clone()),
        );

        // 2. Record parent Turn linking to child.
        let parent_turn = build_toolcall_turn(
            "spawn_relay",
            args,
            &child.id,
        );
        ws.conversations
            .append_turn(&self.ctx.parent_conversation_id, parent_turn);

        // 3. Start the relay run (reuses RunStore + driver).
        let req = StartRunRequest {
            run_id: None,
            flow_id: Some(flow_id.clone()),
            steps: Vec::new(),
            task: Some(task.clone()),
        };
        let (run_id, _initial_state) =
            ws.relay.start_run(&req, Some(self.ctx.workspace_id.clone()));

        // 4. Spawn the driver.
        let state = self.ctx.state.clone();
        let ws_id = self.ctx.workspace_id.clone();
        let run_id_clone = run_id.clone();
        tokio::spawn(async move {
            // Plan 020 Phase G: switched to the transpiled ag drive_run.
            let _ = crate::auto_generated::relay_driver::drive_run(state, &ws_id, &run_id_clone).await;
        });

        // 5. Detached watcher: mirror the run's terminal status onto the child
        //    conversation (data hygiene only — it never blocks the tool result).
        //    Human gates can wait arbitrarily long, so there is no short timeout;
        //    the watcher just exits when the run reaches a terminal state.
        {
            let ws = ws.clone();
            let child_id = child.id.clone();
            let run_id_clone = run_id.clone();
            tokio::spawn(async move {
                let poll_interval = std::time::Duration::from_secs(2);
                loop {
                    let status = ws.relay.status(&run_id_clone);
                    match status.as_deref() {
                        Some("completed") => {
                            ws.conversations
                                .set_status(&child_id, ConversationStatus::Completed);
                            break;
                        }
                        Some("failed") => {
                            ws.conversations.set_status(
                                &child_id,
                                ConversationStatus::Failed {
                                    error: "Relay flow failed".into(),
                                },
                            );
                            break;
                        }
                        _ => {} // still running / waiting_gate / paused — keep watching
                    }
                    tokio::time::sleep(poll_interval).await;
                }
            });
        }

        // 6. Return the run handle immediately. spawn_relay is async by design:
        //    blocking the chat stream until the flow finishes deadlocks whenever
        //    a step has a human gate (the run sits in waiting_for_human while
        //    the poll loop here ignores it, and the chat SSE shows nothing).
        //    The JSON shape feeds the frontend's extractRunId (result.run_id).
        Ok(json!({
            "run_id": run_id,
            "flow_id": flow_id,
            "status": "started",
            "detail": "relay run started in the background; it advances until the first human gate. Track it via GET /api/forge/relay/runs/{run_id} (or the chat run box) and approve gates via POST /api/forge/relay/runs/{run_id}/gate",
        })
        .to_string())
    }
}

// ─── Dispatch (errand) ──────────────────────────────────────────────────────

/// `dispatch` — send a short task to a target agent (e.g. gofer) as a child
/// errand conversation. The target agent runs a single turn and returns.
pub struct Dispatch {
    ctx: ToolContext,
}

impl Dispatch {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for Dispatch {
    fn name(&self) -> &str {
        "dispatch"
    }

    fn description(&self) -> &str {
        "Dispatch a short task to a sub-agent (e.g. 'gofer'). Use for \
         quick lookups, searches, or simple operations you don't want to \
         do yourself. Args: target (agent id, default 'gofer'), task (clear \
         instruction). Returns the sub-agent's result."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Agent id to dispatch to (default: 'gofer')."
                },
                "task": {
                    "type": "string",
                    "description": "A clear, self-contained instruction for the sub-agent."
                }
            },
            "required": ["task"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let target = args["target"].as_str().unwrap_or("gofer").to_string();
        let task = args["task"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if task.is_empty() {
            return Err(ToolError::Exec("dispatch: 'task' is required".into()));
        }

        let ws = self.ctx.state.registry.get(&self.ctx.workspace_id);

        // 1. Create child errand conversation.
        let child = ws.conversations.create(
            ConversationKind::Errand,
            self.ctx.workspace_id.clone(),
            Driver::Agent {
                agent_id: target.clone(),
            },
            Some(target.clone()),
            Some(task.clone()),
        );

        // 2. Record parent Turn linking to child.
        let parent_turn = build_toolcall_turn("dispatch", args, &child.id);
        ws.conversations
            .append_turn(&self.ctx.parent_conversation_id, parent_turn);

        // 3. Record the task as the first turn in the child conversation.
        ws.conversations.append_turn(
            &child.id,
            Turn {
                id: conversation::new_id(8),
                seq: 0,
                from: "system".into(),
                to: Some(target.clone()),
                kind: TurnKind::Message,
                content: task.clone(),
                tool: None,
                gate: None,
                child_conversation: None,
                tokens: None,
                timestamp: conversation::now_secs(),
            },
        );

        // 4. Build + run the target agent for one turn (synchronous — errands
        //    are short by design).
        crate::tool_safety::set_current_root(ws.root.clone());
        let result = run_errand_agent(&self.ctx.state, &target, &task, &child.id, &ws).await;
        crate::tool_safety::clear_current_root();

        match result {
            Ok(output) => {
                // Record the agent's reply in the child conversation.
                ws.conversations.append_turn(
                    &child.id,
                    Turn {
                        id: conversation::new_id(8),
                        seq: 1,
                        from: target.clone(),
                        to: None,
                        kind: TurnKind::Message,
                        content: output.clone(),
                        tool: None,
                        gate: None,
                        child_conversation: None,
                        tokens: None,
                        timestamp: conversation::now_secs(),
                    },
                );
                ws.conversations
                    .set_status(&child.id, ConversationStatus::Completed);
                Ok(output)
            }
            Err(e) => {
                ws.conversations.set_status(
                    &child.id,
                    ConversationStatus::Failed {
                        error: e.clone(),
                    },
                );
                Err(ToolError::Exec(format!("dispatch to '{}' failed: {}", target, e)))
            }
        }
    }
}

// ─── BringIn (specialist sub-task) ──────────────────────────────────────────

/// `bring_in` — bring in a specialist agent (e.g. coder, architect) to handle
/// a sub-task. Unlike `dispatch` (which targets gofer for quick lookups),
/// `bring_in` targets a full specialist that runs a complete multi-turn session.
pub struct BringIn {
    ctx: ToolContext,
}

impl BringIn {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for BringIn {
    fn name(&self) -> &str {
        "bring_in"
    }

    fn description(&self) -> &str {
        "Bring in a specialist agent (e.g. 'coder', 'architect', 'tester') to \
         handle a sub-task. Use for tasks that need a specific specialist's \
         expertise but don't require a full relay pipeline. Args: target \
         (agent id, e.g. 'coder'), task (clear, self-contained instruction \
         with full context). Returns the specialist's output."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Specialist agent id: 'coder', 'architect', 'tester', 'reviewer', etc."
                },
                "task": {
                    "type": "string",
                    "description": "A clear, self-contained instruction with full context for the specialist."
                }
            },
            "required": ["target", "task"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let target = args["target"]
            .as_str()
            .unwrap_or("coder")
            .to_string();
        let task = args["task"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if task.is_empty() {
            return Err(ToolError::Exec("bring_in: 'task' is required".into()));
        }

        let ws = self.ctx.state.registry.get(&self.ctx.workspace_id);

        // 1. Create child errand conversation.
        let child = ws.conversations.create(
            ConversationKind::Errand,
            self.ctx.workspace_id.clone(),
            Driver::Agent {
                agent_id: target.clone(),
            },
            Some(target.clone()),
            Some(task.clone()),
        );

        // 2. Record parent Turn linking to child.
        let parent_turn = build_toolcall_turn("bring_in", args, &child.id);
        ws.conversations
            .append_turn(&self.ctx.parent_conversation_id, parent_turn);

        // 3. Record the task as the first turn in the child conversation.
        ws.conversations.append_turn(
            &child.id,
            Turn {
                id: conversation::new_id(8),
                seq: 0,
                from: "assistant".into(),
                to: Some(target.clone()),
                kind: TurnKind::Message,
                content: task.clone(),
                tool: None,
                gate: None,
                child_conversation: None,
                tokens: None,
                timestamp: conversation::now_secs(),
            },
        );

        // 4. Build + run the specialist agent (synchronous, multi-turn).
        crate::tool_safety::set_current_root(ws.root.clone());
        let result = run_errand_agent(&self.ctx.state, &target, &task, &child.id, &ws).await;
        crate::tool_safety::clear_current_root();

        match result {
            Ok(output) => {
                ws.conversations.append_turn(
                    &child.id,
                    Turn {
                        id: conversation::new_id(8),
                        seq: 1,
                        from: target.clone(),
                        to: None,
                        kind: TurnKind::Message,
                        content: output.clone(),
                        tool: None,
                        gate: None,
                        child_conversation: None,
                        tokens: None,
                        timestamp: conversation::now_secs(),
                    },
                );
                ws.conversations
                    .set_status(&child.id, ConversationStatus::Completed);
                Ok(output)
            }
            Err(e) => {
                ws.conversations.set_status(
                    &child.id,
                    ConversationStatus::Failed {
                        error: e.clone(),
                    },
                );
                Err(ToolError::Exec(format!("bring_in '{}' failed: {}", target, e)))
            }
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Build a ToolCall Turn that links parent → child conversation.
fn build_toolcall_turn(tool_name: &str, args: &Value, child_id: &str) -> Turn {
    Turn {
        id: conversation::new_id(8),
        seq: 0, // Will be overwritten by append_turn
        from: "assistant".into(),
        to: None,
        kind: TurnKind::ToolCall,
        content: String::new(),
        tool: Some(conversation::ToolRecord {
            name: tool_name.into(),
            args: args.clone(),
            result: String::new(),
            tool_id: None,
        }),
        gate: None,
        child_conversation: Some(child_id.into()),
        tokens: None,
        timestamp: conversation::now_secs(),
    }
}

/// Run a single agent turn for an errand. Builds the agent from the target
/// profession, runs run_stream, returns the output.
async fn run_errand_agent(
    state: &crate::server::AppState,
    target: &str,
    task: &str,
    _child_id: &str,
    ws: &Arc<crate::workspace::WorkspaceStores>,
) -> Result<String, String> {
    use crate::mode::AgentMode;

    let mode = AgentMode {
        name: format!("errand-{target}"),
        description: String::new(),
        role: target.into(),
        skills: false,
        tools: vec![],
        workflow: None,
        context_file: String::new(),
        extra_system_prompt: String::new(),
    };
    let mut agent = crate::build_agent_from_mode(&mode, state.client.clone())?;

    let result = agent.run(task).await.map_err(|e| format!("agent: {e}"))?;
    Ok(result.output)
}

// Tests for orchestration tools require a full AppState + workspace, so they're
// exercised via integration testing (curl + real agent runs). The tool metadata
// (name/description/parameters) is verified at registration time by build_agent_with_context.

// ─── SpawnTaskPlan (Plan 009 P2b.7) ──────────────────────────────────────────

/// `spawn_task_plan` — launch a registered TaskPlan (a DAG of relay phases) in
/// the background and return its instance id + initial status. The plan runs to
/// completion asynchronously; the caller may poll `GET /api/forge/relay/task_plans/runs/{instance_id}`.
pub struct SpawnTaskPlan {
    ctx: ToolContext,
}

impl SpawnTaskPlan {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for SpawnTaskPlan {
    fn name(&self) -> &str {
        "spawn_task_plan"
    }

    fn description(&self) -> &str {
        "Launch a registered TaskPlan (a multi-phase relay orchestration). Use \
         for large goals that decompose into several relay pipelines with \
         dependencies between them. Args: task_plan_id (required, must already \
         be registered), initial_input (required, the top-level goal). Returns \
         the instance id and 'started' status."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task_plan_id": {
                    "type": "string",
                    "description": "Id of a registered TaskPlan (see list_task_plans / register_task_plan)."
                },
                "initial_input": {
                    "type": "string",
                    "description": "The top-level goal handed to the plan's first phase."
                }
            },
            "required": ["task_plan_id", "initial_input"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let task_plan_id = args["task_plan_id"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let initial_input = args["initial_input"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if task_plan_id.is_empty() {
            return Err(ToolError::Exec(
                "spawn_task_plan: 'task_plan_id' is required".into(),
            ));
        }
        if initial_input.is_empty() {
            return Err(ToolError::Exec(
                "spawn_task_plan: 'initial_input' is required".into(),
            ));
        }

        let ws = self.ctx.state.registry.get(&self.ctx.workspace_id);

        // Look up the plan in the workspace registry.
        let plan = {
            let reg = ws.task_plans.lock().unwrap();
            reg.get(&task_plan_id).ok_or_else(|| {
                ToolError::Exec(format!(
                    "spawn_task_plan: unknown task_plan_id '{task_plan_id}'"
                ))
            })?
        };

        let mut engine = crate::relay::task_plan_engine::TaskPlanEngine::new(plan, initial_input.clone());
        let instance_id = engine.instance_id.clone();
        // Run the plan id-to-id validation before backgrounding (surfaces flow errors fast).
        if let Err(e) = engine.validate() {
            return Err(ToolError::Exec(format!(
                "spawn_task_plan: plan '{task_plan_id}' failed validation: {e}"
            )));
        }

        // Record a parent Turn linking to this plan instance (reuses the child-conversation link shape).
        let parent_turn = build_toolcall_turn("spawn_task_plan", args, &instance_id);
        ws.conversations
            .append_turn(&self.ctx.parent_conversation_id, parent_turn);

        // Drive the plan in the background.
        let state = (*self.ctx.state).clone();
        let ws_id = self.ctx.workspace_id.clone();
        let handoffs = ws.handoffs.clone();
        tokio::spawn(async move {
            let ctx = crate::relay::task_plan_engine::TaskPlanContext { state, workspace_id: ws_id };
            let result = engine.execute(&handoffs, |req| {
                let ctx = ctx.clone();
                async move { crate::relay::task_plan_engine::drive_task_plan_run(&ctx, req).await }
            }).await;
            if let Err(e) = result {
                tracing::error!("TaskPlan instance failed: {}", e);
            }
        });

        Ok(json!({
            "task_plan_spawned": true,
            "instance_id": instance_id,
            "task_plan_id": task_plan_id,
            "initial_input": initial_input,
            "status": "started"
        })
        .to_string())
    }
}

// ─── RegisterTaskPlan (Plan 009 P2b.7) ───────────────────────────────────────

/// `register_task_plan` — parse + validate an Atom TaskPlan and persist it to
/// the workspace's `.autoos/task_plans/` directory so it can be spawned.
pub struct RegisterTaskPlan {
    ctx: ToolContext,
}

impl RegisterTaskPlan {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for RegisterTaskPlan {
    fn name(&self) -> &str {
        "register_task_plan"
    }

    fn description(&self) -> &str {
        "Register a new TaskPlan from Atom source. The plan is parsed, \
         validated (structure + that every run's flow_id exists), and written \
         to <workspace>/.autoos/task_plans/<id>.atom. Args: atom (required, \
         full Atom source). Returns the plan id, phase count, and run count."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "atom": {
                    "type": "string",
                    "description": "Full Atom source of the TaskPlan, e.g. task_plan(id: \"my-plan\", version: 1) { phase(name: \"p\") { run(name: \"r\", flow_id: \"default\") } }"
                }
            },
            "required": ["atom"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let atom = args["atom"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if atom.trim().is_empty() {
            return Err(ToolError::Exec(
                "register_task_plan: 'atom' is required".into(),
            ));
        }

        let ws = self.ctx.state.registry.get(&self.ctx.workspace_id);
        let (plan, phase_count, run_count) = {
            let mut reg = ws.task_plans.lock().unwrap();
            let plan = reg.register(&atom).map_err(ToolError::Exec)?;
            let phase_count = plan.phases.len();
            let run_count = plan.phases.iter().map(|p| p.runs.len()).sum::<usize>();
            (plan, phase_count, run_count)
        };

        Ok(json!({
            "task_plan_registered": true,
            "id": plan.id,
            "phase_count": phase_count,
            "run_count": run_count
        })
        .to_string())
    }
}
