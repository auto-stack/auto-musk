//! Tool approval gate endpoints (PLAN-069 W3).
//!
//! hw escape-hatch 路由：决议 tool_gate hub 中挂起的 run_command 审批门。

use axum::{
    extract::Path,
    routing::post,
    Json,
    Router,
};

pub fn tool_gate_routes() -> Router<crate::server::AppState> {
    Router::new()
        .route("/api/chats/tool-gate/{gid}/approve", post(gate_approve))
        .route("/api/chats/tool-gate/{gid}/deny", post(gate_deny))
}

async fn gate_approve(Path(gid): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "resolved": crate::tool_gate::resolve(&gid, true) }))
}

async fn gate_deny(Path(gid): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "resolved": crate::tool_gate::resolve(&gid, false) }))
}
