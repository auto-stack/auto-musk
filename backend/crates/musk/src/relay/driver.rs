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
use crate::relay::GateDecision;
use crate::relay::store::RunEvent;
use crate::server::AppState;

// ── MuskAgentFactory (Plan 008 Phase 6) ────────────────────────────────────

/// PLAN-086 T-02: flow-level handoff-injection switch. The `plan` flow's
/// inter-phase context is the plan file alone (mechanically passed via the
/// run's `plan_file` context var) — injecting the prior phase's AI-generated
/// handoff render would re-introduce the hallucination surface PLAN-086
/// retires, so `plan`/`plan-merge` skip it. Deprecated flows keep the legacy
/// injection (parity tests compare on `simple`). Unknown flow (vanished run)
/// fails open to the legacy behavior.
fn injects_handoff(flow_id: Option<&str>) -> bool {
    !matches!(flow_id, Some("plan") | Some("plan-merge"))
}

/// Agent factory for musk relay steps. Implements
/// [`auto_ai_agent::orchestration::AgentFactory`] so the relay driver can
/// build agents with musk-specific context:
/// - [`crate::tool_context::ToolContext`] for orchestration tools
///   (`spawn_relay`, `dispatch`, `bring_in`)
/// - Workspace-scoped file safety
/// - Musk's full tool set (spec tools, skills, etc.)
/// `pub`: constructible by the a2r extern_impl `factory_build_agent` stub and
/// the parity_relay tests (Plan 018 §11 ③ delegation).
pub struct MuskAgentFactory {
    pub state: Arc<AppState>,
    pub workspace_id: String,
    pub run_id: String,
}

impl MuskAgentFactory {
    /// PLAN-096 T-04: the execution scope of the step about to run — the
    /// registered dev-worktree root for `execute`/`review` phases of a bound
    /// plan run, None otherwise (plan/document phases and unbound runs keep
    /// the workspace multi-root resolution).
    pub fn execution_scope_for_current_step(&self) -> Option<std::sync::Arc<std::path::PathBuf>> {
        let ws = self.state.registry.get(&self.workspace_id);
        let state = ws.relay.get(&self.run_id)?;
        let step_id = state.steps.get(state.current_step).map(|s| s.id.clone())?;
        if !matches!(step_id.as_str(), "execute" | "review") {
            return None;
        }
        let binding = ws
            .relay
            .plan_execution(&self.run_id)
            .and_then(|pe| pe.binding)?;
        let root = binding.execution_root?;
        Some(std::sync::Arc::new(std::path::PathBuf::from(root)))
    }
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
        // PLAN-096 T-04（§5.3/AC-03）：execute/review 相位的显式执行作用域 =
        // 登记的开发 worktree（binding.execution_root）——文件/命令工具与
        // Canvas 路径解析落 worktree，主检出代码不注册为可写根；plan/document
        // 相位与无绑定 run 维持 workspace 多根（计划共享状态走 plan 工具）。
        let tool_ctx = crate::tool_context::ToolContext {
            approval_mode: None,
            state: self.state.clone(),
            workspace_id: self.workspace_id.clone(),
            parent_conversation_id: self.run_id.clone(),
            // PLAN-040 T5:relay 步骤的工具进度挂在 run_id 上——run SSE
            // (/runs/{id}/events)订阅同一条总线,ToolUpdate 自动透传前端。
            progress: Some(crate::tool_context::ProgressSink::for_run(&self.run_id)),
            execution_root: self.execution_scope_for_current_step(),
        };
        let mut agent =
            crate::build_agent_with_context(&mode, self.state.client.clone(), Some(tool_ctx))?;
        // Inject prior handoff context if this isn't the first step — unless
        // the flow retired the channel (PLAN-086 T-02: plan/plan-merge rely
        // on the plan file as the sole inter-phase carrier). The ag factory
        // delegates here (extern_impl factory_build_agent), so the switch is
        // single-point across both drivers.
        let flow_id = self
            .state
            .registry
            .get(&self.workspace_id)
            .relay
            .flow_of(&self.run_id);
        if injects_handoff(flow_id.as_deref()) {
            if let Some(h) = handoff {
                let prior_md = h.render();
                if !prior_md.is_empty() {
                    agent = agent.with_history(vec![("user".to_string(), prior_md)]);
                }
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
    // PLAN-034 T9：完成事件在 submit_handoff 内已落库——Completed 分支不可
    // 达，此处按终态判断写回报告消息（gate/failed 不写）。
    if ws.relay.status(&run_id).as_deref() == Some("completed") {
        crate::auto_generated::extern_impl::relay_append_report_message_to(&ws, &run_id);
    }
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
                    // Agent build/run failure → 置败停车（PLAN-030 试用修复：
                    // 原先错误被当 handoff 提交并级联到后续相位空转）。
                    tracing::error!("drive_run: step agent failed for {run_id}: {e}");
                    let _ = ws.relay.fail_run(run_id, &format!("[agent error] {e}"));
                    return;
                }
                // 3. The agent step submitted its own handoff inside run_step;
                //    loop back to advance the next step.
                continue;
            }
            AdvanceResult::WaitForHuman { step_id, .. } => {
                // Gate: stop driving and wait for POST /gate to resolve it.
                tracing::info!("drive_run: {run_id} paused at human gate");
                // PLAN-067 T-04(a)：镜像 gate_waiting 到发起会话的 chat 流。
                // 总线桥（chat_run_stream 内）按 run_id==session_id 过滤,而
                // relay run 事件携带的是 run 自身 id——此前 chat 级审批门
                // 事件无生产者（前端 OnStreamEvent 的 relay_gate_waiting
                // 分支为死线）,审批卡永不出现。此处读发起会话 id,以其为
                // bus run_id 发 relay_gate_waiting（payload.run_id 仍是
                // relay run id,前端据此更新 .relays 卡片状态）。
                if let Some(chat_sid) = ws.relay.context_var(run_id, "chat_session_id") {
                    if !chat_sid.is_empty() {
                        crate::relay::api::publish_task_plan_event(
                            &chat_sid,
                            "relay_gate_waiting",
                            serde_json::json!({ "run_id": run_id, "step_id": step_id }),
                        );
                    }
                }
                // PLAN-067 T-05：审批模式 = auto 时即刻放行并继续驱动
                // （resolve_gate 内部 advance 出 ExecuteStep/Completed）。
                // 放行本身经 store.resolve_gate 落 GateResolved 审计事件。
                // PLAN-094 T-03（UAT K4/K5）：放行前先过 execute 门不变式——
                // plan 流 execute 门上的 approve 必须有已落实的计划文件；缺失
                // 时不放行、直接置败响亮失败（plan D1 退化：引擎 redraft 只会
                // 重做被门守卫的 execute 相位，定向重跑 plan 相位不可表达）。
                if ws.relay.context_var(run_id, "approval_mode").as_deref() == Some("auto") {
                    let gate_step = ws.relay.pending_gate_step(run_id);
                    if crate::relay::plan_flow::execute_gate_action(
                        ws.relay.flow_of(run_id).as_deref(),
                        gate_step.as_deref(),
                        ws.relay.context_var(run_id, "plan_file").as_deref(),
                    ) == Some(crate::relay::plan_flow::ExecuteGateAction::Fail)
                    {
                        tracing::error!(
                            "drive_run: {run_id} execute gate without plan_file — failing run (no silent pass)"
                        );
                        let _ = ws
                            .relay
                            .fail_run(run_id, crate::relay::plan_flow::PLAN_GATE_FAIL_ERROR);
                        return;
                    }
                    tracing::info!("drive_run: {run_id} approval_mode=auto, auto-approving gate");
                    let (res, _st) = match ws.relay.resolve_gate_with_note(
                        run_id,
                        GateDecision::Approve,
                        Some(crate::relay::plan_flow::AUTO_APPROVE_NOTE),
                    ) {
                        Some(v) => v,
                        None => return,
                    };
                    match res {
                        AdvanceResult::ExecuteStep { role_id, .. } => {
                            if let Err(e) = run_step(state, ws, run_id, &role_id).await {
                                tracing::error!("drive_run: step agent failed for {run_id}: {e}");
                                let _ = ws.relay.fail_run(run_id, &format!("[agent error] {e}"));
                            }
                            continue;
                        }
                        AdvanceResult::Completed => {
                            crate::auto_generated::extern_impl::relay_append_report_message_to(ws, run_id);
                            return;
                        }
                        _ => return,
                    }
                }
                return;
            }
            AdvanceResult::Completed => {
                tracing::info!("drive_run: {run_id} completed");
                // PLAN-034 T9：run 完成后把报告作为助手消息写回发起它的
                // chat 会话——报告卡在对话流内渲染（刷新持久），Run 卡片与
                // 报告卡互链。`chat_session_id` 由 plan-merge 短路 / spawn_relay
                // 在启动 run 时写入 context。（实现在 ag extern_impl，hw/ag
                // 两个驱动共用；完成事件在 submit_handoff 内落库，此分支通常
                // 不可达，实际写回在 drive_run 收尾按终态判断。）
                crate::auto_generated::extern_impl::relay_append_report_message_to(ws, run_id);
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
            StreamEvent::ToolStart { .. } => {
                // The Tool event follows with the actual result; nothing to
                // persist yet (kept for state-tracking parity with server.rs).
            }
            // auto-ai PLAN-026 新增的 turn 边界事件:relay store 不消费,
            // 仅覆盖以保持 match 穷尽(完整适配待正式对齐时重做)。
            StreamEvent::TurnStart { .. } | StreamEvent::TurnEnd { .. } => {}
            StreamEvent::Tool {
                tool,
                args,
                result,
                details,
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
                        // PLAN-042:结构化载荷透传(run events SSE + 会话回放)。
                        details: details.clone(),
                    },
                );
            }
            StreamEvent::Warning { text } => {
                tracing::warn!("relay turn warning: {text}");
            }
            StreamEvent::Thinking { text } => {
                // Reasoning is not persisted into the relay run store — the
                // handoff/output should carry the final answer, not the
                // chain-of-thought. (Surfaced only in the direct chat SSE.)
                tracing::debug!("relay thinking: {}…", &text[..text.len().min(60)]);
            }
            StreamEvent::Done { .. } | StreamEvent::Error { .. } => {
                // Handled below via the return value.
            }
            StreamEvent::Cancelled { result } => {
                // The driver never sets the cancel flag; belt-and-braces so
                // partial output still lands in the handoff.
                acc.lock().unwrap().push_str(&result.output);
                tracing::warn!("relay turn cancelled (unexpected)");
            }
        }
    });

    // No cancellation endpoint yet — the run flag is never set.
    let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let result = agent
        .run_stream(&task, on_event, cancel)
        .await
        .map_err(|e| format!("agent: {e}"))?;

    let output = std::mem::take(&mut *accumulated.lock().unwrap());
    let final_output = if output.trim().is_empty() {
        result.output.clone()
    } else {
        output
    };

    // PLAN-030: stash the plan-file marker (emitted by the plan flow's plan
    // phase) into the run context so later phases' templates carry the exact
    // path instead of relying on the handoff summary.
    // PLAN-086 T-03: the marker is now the FALLBACK channel — when the
    // create_plan binding already wrote plan_file tool-time, the marker must
    // not overwrite it (binding > marker > hint).
    if let Some(plan_file) = crate::relay::plan_flow::plan_file_marker_write(
        ws.relay.context_var(run_id, "plan_file"),
        &final_output,
    ) {
        ws.relay.set_context_var(run_id, "plan_file", &plan_file);
    }

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

    use super::injects_handoff;

    /// PLAN-086 T-02 / AC-02: plan 系流程相位输入=模板+计划文件，prior
    /// handoff render 注入退役；deprecated 流保留注入（parity 对拍面）；
    /// 未知 flow（run 消失）fail-open 走旧行为。
    #[test]
    fn handoff_injection_is_flow_gated() {
        // plan 系两流：不注入。
        assert!(!injects_handoff(Some("plan")));
        assert!(!injects_handoff(Some("plan-merge")));
        // deprecated 流：照旧注入。
        assert!(injects_handoff(Some("simple")));
        assert!(injects_handoff(Some("default")));
        assert!(injects_handoff(Some("relay")));
        assert!(injects_handoff(Some("superpower")));
        // fail-open：run 不存在/查询不到 flow 时保持旧行为。
        assert!(injects_handoff(None));
    }
}
