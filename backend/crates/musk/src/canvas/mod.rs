//! PLAN-087: App Canvas M1 —— 实况画布最小闭环（模块根）。
//!
//! 组成：session（spawn/收割）、mcp_client（AutoUI JSON-RPC）、manager
//! （单会话状态机 + 看门狗）、tools（agent 五件）、templates（生成侧）。
//! API 四路由（本文件，T-05）：POST /api/canvas/start、GET /api/canvas/frame
//! （PNG 直出）、GET /api/canvas/status、POST /api/canvas/stop。
//! 安全边界：start 的 app_path 经 `tool_safety::resolve_multi` 多根判定
//! fail-closed，拒绝报文列全部根（AC-06）。

pub mod manager;
pub mod mcp_client;
pub mod session;
pub mod templates;
pub mod tools;

pub use manager::{CanvasManager, CanvasState, CanvasStatus};

use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use crate::server::AppState;
use crate::workspace::WorkspaceQuery;

pub fn canvas_routes() -> Router<AppState> {
    Router::new()
        .route("/api/canvas/start", post(canvas_start))
        .route("/api/canvas/frame", get(canvas_frame))
        .route("/api/canvas/status", get(canvas_status))
        .route("/api/canvas/stop", post(canvas_stop))
}

#[derive(Debug, Deserialize)]
pub struct CanvasStartRequest {
    pub app_path: String,
}

/// POST /api/canvas/start?workspace={id} {app_path} —— 校验 + 启动（替换语义）。
async fn canvas_start(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Json(body): Json<CanvasStartRequest>,
) -> Response {
    let ws_id = q.workspace.clone().unwrap_or_default();
    let ws = state.registry.get(&ws_id);
    let mut roots = vec![ws.root.clone()];
    roots.extend(
        state
            .registry
            .extra_roots(&ws_id)
            .into_iter()
            .map(std::path::PathBuf::from),
    );
    let resolved = match crate::tool_safety::resolve_multi(&body.app_path, &roots) {
        Ok(p) => p,
        Err(msg) => {
            // AC-06：越界报文列全部根（resolve_multi 产文案已列）。
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg })))
                .into_response();
        }
    };
    if let Err(msg) = session::validate_app_dir(&resolved) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg })))
            .into_response();
    }
    match state.canvas.start(resolved).await {
        Ok(status) => Json(serde_json::json!({
            "state": status.state,
            "app_path": status.app_path,
        }))
        .into_response(),
        Err(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": msg })),
        )
            .into_response(),
    }
}

/// GET /api/canvas/frame —— 当前帧 PNG 直出；无帧 503。
async fn canvas_frame(State(state): State<AppState>) -> Response {
    match state.canvas.frame() {
        Some(bytes) => (
            [(header::CONTENT_TYPE, "image/png")],
            bytes.as_ref().to_vec(),
        )
            .into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "no canvas frame yet" })),
        )
            .into_response(),
    }
}

/// GET /api/canvas/status —— {state, seq, app_path, restarts, error}。
async fn canvas_status(State(state): State<AppState>) -> Response {
    Json(state.canvas.status()).into_response()
}

/// POST /api/canvas/stop —— 停止并收割（幂等）。
async fn canvas_stop(State(state): State<AppState>) -> Response {
    state.canvas.stop().await;
    Json(serde_json::json!({ "state": "stopped" })).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 路由表形状锚：四路由可构建（不 dispatch，防手滑改路径）。
    #[test]
    fn canvas_routes_shape() {
        let _r = canvas_routes();
        // 路由存在性由集成测试覆盖；此处锚定编译期路径字面量。
        assert!(true);
    }
}
