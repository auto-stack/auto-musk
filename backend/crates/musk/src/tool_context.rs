//! Tool execution context — injected into orchestration tools (spawn_relay,
//! dispatch) so they can create + drive sub-conversations.
//!
//! The `Tool::execute(&self, args)` trait signature carries no business
//! context. Orchestration tools solve this by holding a `ToolContext` struct
//! field, injected at agent-build time via `build_agent_with_context`.

use std::sync::Arc;

use crate::server::AppState;

/// Everything an orchestration tool needs to create + drive a sub-conversation.
#[derive(Clone)]
pub struct ToolContext {
    pub state: Arc<AppState>,
    pub workspace_id: String,
    pub parent_conversation_id: String,
    /// PLAN-040 T5:工具实时进度通道(None = 无前端订阅场景,如测试/CLI)。
    /// chat 场景 = session_id,relay 场景 = run_id;run_command 等长任务工具
    /// 经此把流式 partial 推上进程级 broadcast 总线(SSE 订阅者按 id 过滤)。
    pub progress: Option<ProgressSink>,
}

/// PLAN-040 T5:工具侧进度通道——进程级 broadcast 总线的 sender + 目标 id
/// (chat session 或 relay run)。partial 是易态:无 SSE 接收者时 send 失败
/// 静默忽略;丢弃旧 partial 可接受(broadcast 背压策略,见计划风险节)。
/// 节流(100ms,pi `BASH_UPDATE_THROTTLE_MS`)由工具侧做。
#[derive(Clone)]
pub struct ProgressSink {
    run_id: String,
    bus: tokio::sync::broadcast::Sender<crate::relay::api::BusEvent>,
}

impl ProgressSink {
    /// 绑定目标 id(chat session_id / relay run_id),挂上进程级总线。
    pub fn for_run(run_id: &str) -> Self {
        Self {
            run_id: run_id.to_string(),
            bus: crate::relay::api::relay_bus().clone(),
        }
    }

    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// 推一条流式 partial(工具名 + 可空的配对 id;易态,尽力而为)。
    pub fn send(&self, tool_name: &str, tool_call_id: &str, partial: &str) {
        if partial.is_empty() {
            return;
        }
        let event = crate::relay::store::RunEvent::ToolUpdate {
            timestamp: now_secs(),
            run_id: self.run_id.clone(),
            tool_call_id: tool_call_id.to_string(),
            tool_name: tool_name.to_string(),
            partial: partial.to_string(),
        };
        let _ = self.bus.send(crate::relay::api::BusEvent {
            run_id: self.run_id.clone(),
            event_type: event.event_type().into(),
            payload: serde_json::to_value(&event).unwrap_or(serde_json::Value::Null),
        });
    }
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

    /// ProgressSink 推送:无接收者(或 Lagged)时静默不 panic;有订阅者时
    /// 收到的 BusEvent 带 run_id / event_type / tool_update payload。
    #[tokio::test]
    async fn progress_sink_publishes_tool_update_on_the_bus() {
        let mut rx = crate::relay::api::relay_bus().subscribe();
        let sink = ProgressSink::for_run("sess-40");
        sink.send("run_command", "", "partial output\n");
        let ev = rx.try_recv().expect("broadcast received");
        assert_eq!(ev.run_id, "sess-40");
        assert_eq!(ev.event_type, "tool_update");
        assert_eq!(ev.payload["type"], "tool_update");
        assert_eq!(ev.payload["tool_name"], "run_command");
        assert_eq!(ev.payload["partial"], "partial output\n");
        assert_eq!(ev.payload["run_id"], "sess-40");
    }

    #[tokio::test]
    async fn progress_sink_empty_partial_is_skipped() {
        let mut rx = crate::relay::api::relay_bus().subscribe();
        let sink = ProgressSink::for_run("sess-40b");
        sink.send("run_command", "", "");
        assert!(rx.try_recv().is_err(), "empty partial not published");
    }
}
