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
        "Start a relay flow (multi-agent pipeline). Use for complex tasks that \
         need multiple specialists (advisor → architect → coder → tester → \
         reviewer). Args: flow_id (optional, default='default'), task (required, \
         a one-sentence description of what to accomplish). Returns the flow's \
         final output summary."
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
            crate::relay::driver::drive_run(state, ws_id, run_id_clone).await;
        });

        // 5. Wait for the run to reach a terminal state (poll every 2s, up to 15 min).
        let timeout_secs = 900u64;
        let poll_interval = std::time::Duration::from_secs(2);
        let start = std::time::Instant::now();
        loop {
            if start.elapsed().as_secs() > timeout_secs {
                // Update child conversation status.
                ws.conversations.set_status(
                    &child.id,
                    ConversationStatus::Failed {
                        error: "Timed out waiting for relay to complete".into(),
                    },
                );
                return Err(ToolError::Exec(format!(
                    "spawn_relay: timed out after {}s waiting for run {}",
                    timeout_secs, run_id
                )));
            }

            let status = ws.relay.status(&run_id);
            match status.as_deref() {
                Some("completed") => {
                    ws.conversations
                        .set_status(&child.id, ConversationStatus::Completed);
                    break;
                }
                Some("failed") => {
                    ws.conversations.set_status(
                        &child.id,
                        ConversationStatus::Failed {
                            error: "Relay flow failed".into(),
                        },
                    );
                    break;
                }
                _ => {} // still running / waiting_gate / paused — keep waiting
            }
            tokio::time::sleep(poll_interval).await;
        }

        // 6. Return summary.
        let final_state = ws.relay.get(&run_id);
        let summary = final_state
            .as_ref()
            .map(|s| {
                format!(
                    "Relay flow '{}' completed. Status: {}. Steps: {}/{}. Tokens: {}.",
                    flow_id, s.status, s.current_step, s.total_steps, s.cumulative_tokens
                )
            })
            .unwrap_or_else(|| format!("Relay flow '{}' finished.", flow_id));

        Ok(summary)
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
