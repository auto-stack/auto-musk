//! Relay driver — the background loop that drives a run to completion (or to a
//! human gate). P2b.2.
//!
//! Unlike auto-forge's hand-written `turn.rs` (921 lines), musk reuses
//! auto-ai-agent's `Agent::run_stream` ReAct loop and just bridges its
//! [`StreamEvent`]s into relay [`RunEvent`]s. The driver:
//!
//! 1. `store.advance(run_id)` → if `ExecuteStep`, build an agent for the step's
//!    profession via [`MuskAgentFactory`] (Plan 008 Phase 6: implements
//!    `auto_ai_agent::orchestration::AgentFactory`).
//! 2. The `on_event` callback maps `StreamEvent → RunEvent` and pushes/publishes
//!    each one (brief store lock).
//! 3. On `Done`, wrap the accumulated output in a [`HandoffDocument`] and
//!    `store.submit_handoff` → the engine routes to the next step → loop.
//! 4. Stops at `WaitForHuman` (gate), `Completed`, `Failed`, or `Paused`.
//!
//! Spawned by the `advance` handler via `tokio::spawn` so the HTTP request
//! returns immediately and progress streams over the SSE `/events` endpoint.

use std::sync::Arc;

use auto_ai_agent::orchestration::{AgentFactory, HandoffDocument};
use auto_ai_agent::StreamEvent;

use crate::relay::AdvanceResult;
use crate::relay::store::RunEvent;
use crate::server::AppState;

// ── MuskAgentFactory (Plan 008 Phase 6) ────────────────────────────────────

/// Agent factory for musk relay steps. Implements
/// [`auto_ai_agent::orchestration::AgentFactory`] so the relay driver can
/// build agents with musk-specific context:
/// - [`crate::tool_context::ToolContext`] for orchestration tools
///   (`spawn_relay`, `dispatch`, `bring_in`)
/// - Workspace-scoped file safety
/// - Musk's full tool set (spec tools, skills, etc.)
struct MuskAgentFactory {
    state: Arc<AppState>,
    workspace_id: String,
    run_id: String,
}

impl AgentFactory for MuskAgentFactory {
    fn build_agent(
        &self,
        role_id: &str,
        handoff: Option<&HandoffDocument>,
    ) -> Result<auto_ai_agent::Agent, String> {
        // Build a one-shot mode whose profession is this step's profession.
        let mode = crate::mode::AgentMode {
            name: format!("relay-{role_id}"),
            description: String::new(),
            role: role_id.to_string(),
            skills: false,
            tools: Vec::new(),
            workflow: None,
            context_file: String::new(),
            extra_system_prompt: String::new(),
        };
        // Build agent with orchestration tool context (spawn_relay, dispatch).
        let tool_ctx = crate::tool_context::ToolContext {
            state: self.state.clone(),
            workspace_id: self.workspace_id.clone(),
            parent_conversation_id: self.run_id.clone(),
        };
        let mut agent =
            crate::build_agent_with_context(&mode, self.state.client.clone(), Some(tool_ctx))?;
        // Inject prior handoff context if this isn't the first step.
        if let Some(h) = handoff {
            let prior_md = h.render();
            if !prior_md.is_empty() {
                agent = agent.with_history(vec![("user".to_string(), prior_md)]);
            }
        }
        Ok(agent)
    }
}

// ── Driver entry points ────────────────────────────────────────────────────

/// Drive a run forward as far as possible: run every auto step until a human
/// gate, completion, failure, or pause. Designed to be `tokio::spawn`-ed.
pub async fn drive_run(state: Arc<AppState>, ws_id: String, run_id: String) {
    // Confine this task's file-tool operations to the workspace root.
    let ws = state.registry.get(&ws_id);
    crate::tool_safety::set_current_root(ws.root.clone());
    // Run the drive loop in an inner block so there's a single cleanup point —
    // clear_current_root runs on EVERY exit path (gate/completed/failed/paused).
    drive_loop(&state, &ws, &run_id).await;
    crate::tool_safety::clear_current_root();
}

/// The actual advance/run loop, factored out so the caller can guarantee the
/// thread-local root is cleared exactly once on return.
async fn drive_loop(
    state: &AppState,
    ws: &std::sync::Arc<crate::workspace::WorkspaceStores>,
    run_id: &str,
) {
    loop {
        // 1. Advance the state machine.
        let (result, _state) = match ws.relay.advance(run_id) {
            Some(v) => v,
            None => {
                tracing::warn!("drive_run: run {run_id} vanished mid-drive");
                return;
            }
        };
        // Publish the transition (StepStarted / GateWaiting / RunCompleted / ...).
        crate::relay::api::publish_advance_result(run_id, &result);

        match result {
            AdvanceResult::ExecuteStep {
                role_id, ..
            } => {
                // 2. Run the agent for this step (outside the store lock).
                if let Err(e) = run_step(state, ws, run_id, &role_id).await {
                    // Agent build/run failure → fail the run with a handoff carrying the error.
                    tracing::error!("drive_run: step agent failed for {run_id}: {e}");
                    let mut h = HandoffDocument::new(&role_id, "");
                    h.summary = format!("[agent error] {e}");
                    let _ = ws.relay.submit_handoff(run_id, h);
                    continue;
                }
                // 3. The agent step submitted its own handoff inside run_step;
                //    loop back to advance the next step.
                continue;
            }
            AdvanceResult::WaitForHuman { .. } => {
                // Gate: stop driving and wait for POST /gate to resolve it.
                tracing::info!("drive_run: {run_id} paused at human gate");
                return;
            }
            AdvanceResult::Completed => {
                tracing::info!("drive_run: {run_id} completed");
                return;
            }
            AdvanceResult::Failed { error } => {
                tracing::info!("drive_run: {run_id} failed: {error}");
                return;
            }
            AdvanceResult::Paused { reason, .. } => {
                tracing::info!("drive_run: {run_id} paused: {reason}");
                return;
            }
        }
    }
}

/// Run a single step's agent and submit the resulting handoff. Returns the
/// accumulated output on success.
async fn run_step(
    state: &AppState,
    ws: &std::sync::Arc<crate::workspace::WorkspaceStores>,
    run_id: &str,
    role_id: &str,
) -> Result<String, String> {
    // Compose the task + prior-step context.
    let (task, _prior_md) = ws
        .relay
        .step_context(run_id)
        .unwrap_or(("Continue the relay pipeline.".to_string(), String::new()));

    // Build the agent via MuskAgentFactory (Plan 008 Phase 6).
    let factory = MuskAgentFactory {
        state: Arc::new(state.clone()),
        workspace_id: ws.relay.workspace_of(run_id).unwrap_or_default(),
        run_id: run_id.to_string(),
    };
    let prior_handoff = ws.relay.last_handoff(run_id);
    let mut agent = factory.build_agent(role_id, prior_handoff.as_ref())?;

    // Stream events into the run's history + SSE bus. The callback is `Fn` (not
    // async) so it must be cheap; it locks the store only to push an event.
    let store = ws.relay.clone();
    let run_id_owned = run_id.to_string();
    let profession_owned = role_id.to_string();
    let accumulated = Arc::new(std::sync::Mutex::new(String::new()));
    let acc = accumulated.clone();
    let on_event: Arc<dyn Fn(StreamEvent) + Send + Sync> = Arc::new(move |ev| {
        match &ev {
            StreamEvent::Delta { text } => {
                acc.lock().unwrap().push_str(text);
                store.push_event(
                    &run_id_owned,
                    RunEvent::TurnDelta {
                        timestamp: now_secs(),
                        role_id: profession_owned.clone(),
                        text: text.clone(),
                    },
                );
            }
            StreamEvent::Tool {
                tool,
                args,
                result,
            } => {
                store.push_event(
                    &run_id_owned,
                    RunEvent::TurnToolCall {
                        timestamp: now_secs(),
                        role_id: profession_owned.clone(),
                        tool_id: String::new(),
                        tool_name: tool.clone(),
                        arguments: args.clone(),
                    },
                );
                store.push_event(
                    &run_id_owned,
                    RunEvent::TurnToolResult {
                        timestamp: now_secs(),
                        role_id: profession_owned.clone(),
                        tool_id: String::new(),
                        result: result.clone(),
                    },
                );
            }
            StreamEvent::Done { .. } | StreamEvent::Error { .. } => {
                // Handled below via the return value.
            }
        }
    });

    let result = agent
        .run_stream(&task, on_event)
        .await
        .map_err(|e| format!("agent: {e}"))?;

    let output = std::mem::take(&mut *accumulated.lock().unwrap());
    let final_output = if output.trim().is_empty() {
        result.output.clone()
    } else {
        output
    };

    // TurnComplete event.
    ws.relay.push_event(
        run_id,
        RunEvent::TurnComplete {
            timestamp: now_secs(),
            role_id: role_id.into(),
        },
    );

    // Wrap into a HandoffDocument and submit (the engine routes to the next step).
    let next_profession = ws.relay.next_profession(run_id).unwrap_or_default();
    let mut handoff = HandoffDocument::new(role_id, &next_profession);
    handoff.summary = final_output.clone();
    handoff.token_usage.step_tokens = result.total_tokens / 2;
    handoff.token_usage.step_tokens =
        result.total_tokens.saturating_sub(result.total_tokens / 2);

    ws.relay
        .submit_handoff(run_id, handoff)
        .ok_or_else(|| "run vanished after step".to_string())?;
    // submit_handoff already pushes StepCompleted/TokenSpend + publishes.
    let _ = result;
    Ok(final_output)
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    // The driver itself needs a live AppState + client to run an agent, so it's
    // exercised via the curl/integration layer. The state-machine behavior it
    // relies on (advance/submit_handoff/gate) is covered by pipeline/store tests.
}
