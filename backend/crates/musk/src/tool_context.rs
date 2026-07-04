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
}
