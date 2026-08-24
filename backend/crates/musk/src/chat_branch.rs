//! Chat session branching (PLAN-043) — hw escape-hatch routes.
//!
//! 会话树三端点：fork（从任意消息分叉重试）/ navigate（切回另一分支）/
//! tree（节点投影）。fork 与 navigate 服务端机制相同（把 `active_leaf` 切到
//! 指定消息，不复制任何数据——树由消息 `parent_id` 指针表达，turns/messages
//! append-only）；语义区分在前端（fork = 切点 + 预填输入框重发，navigate =
//! 切到分支末尾回看）。投影与链式追加的核心语义在 [`crate::chats`]（单测在
//! chats.rs）；ConversationStore 镜像保持线性 journal 不动（树单源 ChatStore，
//! Phase 0 勘察结论 #2）。

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;

use crate::server::AppState;

#[derive(Deserialize)]
struct WsQuery {
    workspace: Option<String>,
}

#[derive(Deserialize)]
struct BranchBody {
    message_id: String,
}

/// `POST /api/chats/session/{id}/fork {message_id}` — 从该消息之后分叉：
/// 活跃叶切到该消息，新消息将挂到它之下（原分支 append-only 保留可回看）。
async fn session_fork(
    State(state): State<AppState>,
    Query(q): Query<WsQuery>,
    Path(id): Path<String>,
    axum::Json(body): axum::Json<BranchBody>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.chats.set_active_leaf(&id, &body.message_id) {
        Ok(Some(session)) => {
            axum::Json(serde_json::json!({ "session": session })).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            "session or message not found".to_string(),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")).into_response(),
    }
}

/// `POST /api/chats/session/{id}/navigate {message_id}` — 切到指定分支（活跃
/// 叶 = 该消息所在分支的末尾=消息本身；下一请求按该路径重建记忆）。
/// 与 fork 同机制；前端语义上 navigate 通常传分支末条消息。
async fn session_navigate(
    State(state): State<AppState>,
    Query(q): Query<WsQuery>,
    Path(id): Path<String>,
    axum::Json(body): axum::Json<BranchBody>,
) -> impl IntoResponse {
    session_fork(State(state), Query(q), Path(id), axum::Json(body)).await
}

/// `GET /api/chats/session/{id}/tree` — Turn 树节点投影（id/role/preview/
/// children/on_active_path），供前端渲染分叉标记与分支切换器。
async fn session_tree(
    State(state): State<AppState>,
    Query(q): Query<WsQuery>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.chats.get(&id) {
        Some(session) => axum::Json(serde_json::json!({
            "id": id,
            "active_leaf": session.active_leaf,
            "nodes": session.tree_nodes(),
        }))
        .into_response(),
        None => (StatusCode::NOT_FOUND, "session not found".to_string()).into_response(),
    }
}

pub fn branch_routes() -> Router<AppState> {
    Router::new()
        .route("/api/chats/session/{id}/fork", post(session_fork))
        .route("/api/chats/session/{id}/navigate", post(session_navigate))
        .route("/api/chats/session/{id}/tree", get(session_tree))
}
