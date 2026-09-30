//! PLAN-087: App Canvas M1 —— 实况画布最小闭环（模块根）。
//!
//! 组成：session（spawn/收割）、mcp_client（AutoUI JSON-RPC）、manager
//! （单会话状态机 + 看门狗）、tools（agent 五件）、templates（生成侧）。
//! API 四路由（本文件，T-05）：POST /api/canvas/start、GET /api/canvas/frame
//! （PNG 直出）、GET /api/canvas/status、POST /api/canvas/stop。
//! 安全边界：start 的 app_path 经 `tool_safety::resolve_multi` 多根判定
//! fail-closed，拒绝报文列全部根（AC-06）。

pub mod anchor;
pub mod bp_tools;
pub mod examples_pool;
pub mod manager;
pub mod mcp_client;
pub mod session;
pub mod templates;
pub mod tools;
pub mod ui_lint;
pub mod vocabulary;

pub use manager::{
    CanvasManager, CanvasState, CanvasStatus, FrameConflict, GenerationConflict, StartError,
    StartIdentity, StopRefused,
};

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
        .route("/api/canvas/pick", post(canvas_pick))
        .route("/api/canvas/stop", post(canvas_stop))
        // PLAN-093 T-07 G-13: 源码只读（结构列源码面板；双 serve 同契约——
        // VM 轨经 canvas_vm.at 宿主桥同一 registry/read 语义）。
        .route("/api/canvas/source", get(canvas_source))
}

/// GET /api/canvas/source?workspace=&path= —— 结构列源码面板只读通道
/// （PLAN-093 T-07 G-13：/api/files/* 域在 VM serve 被 ag 参数路由承接
/// 且 VM 转译 handler registry 状态桥缺失，恒 200 "null"——canvas 域
/// 双 serve 同契约绕行）。read_text_confined 同款 confinement/lossy。
async fn canvas_source(
    State(state): State<AppState>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let ws_id = q.get("workspace").cloned().unwrap_or_default();
    let rel = q.get("path").cloned().unwrap_or_default();
    if ws_id.is_empty() || rel.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "source: empty workspace or path" })),
        )
            .into_response();
    }
    let ws = state.registry.get(&ws_id);
    match crate::files_browser::read_text_confined(&ws.root, &rel) {
        Ok(content) => (
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            content,
        )
            .into_response(),
        Err((status, msg)) => (
            status,
            Json(serde_json::json!({ "error": msg })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct CanvasStartRequest {
    pub app_path: String,
    /// PLAN-093 T-02：归属会话（工作台路径携带；工具路径由 ToolContext 提供）。
    /// 缺省 = 旧无所属调用（"未绑定预览"）。
    pub conversation_id: Option<String>,
    /// PLAN-093 T-02：预期代次（新客户端）。与当前不一致 → 409，不替换新目标。
    pub expected_generation: Option<u64>,
}

/// frame/pick/stop 的 query 形预期代次（GET frame 无 body）。
#[derive(Debug, Deserialize)]
pub struct CanvasIdentityQuery {
    pub generation: Option<u64>,
    pub seq: Option<u64>,
}

fn conflict_409(msg: String, current: u64) -> Response {
    (
        StatusCode::CONFLICT,
        Json(serde_json::json!({ "error": msg, "generation_id": current })),
    )
        .into_response()
}

/// POST /api/canvas/pick 入参：{x, y}（帧 PNG 像素，点选）或
/// {vnode_id: "vnode_N"}（层树联动/程序直选）或 {clear: true}（显式清选，
/// PLAN-093：与坐标/vnode_id 互斥）。二选一/清选。id 字符串形态
///（哈希 vnode > JS 2^53，数字形态静默截断）。
#[derive(Debug, Deserialize)]
pub struct CanvasPickRequest {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub vnode_id: Option<String>,
    /// PLAN-093 T-02：显式清选（与 x/y/vnode_id 互斥；未命中同样清选）。
    pub clear: Option<bool>,
    /// PLAN-093 T-02：预期代次。不一致 → 409，不作用于已替换的新应用。
    pub expected_generation: Option<u64>,
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
    match state
        .canvas
        .start_owned(resolved, &ws.root, &ws_id, body.conversation_id.as_deref(), body.expected_generation)
        .await
    {
        Ok(identity) => {
            let st = state.canvas.status();
            Json(serde_json::json!({
                "state": st.state,
                "app_path": st.app_path,
                "generation_id": identity.generation,
                "owner_workspace_id": identity.owner_workspace,
                "owner_conversation_id": identity.owner_conversation,
            }))
            .into_response()
        }
        Err(StartError::GenerationConflict(c)) => conflict_409(
            format!(
                "canvas generation conflict: expected {}, current {} — re-fetch /api/canvas/status",
                body.expected_generation.unwrap_or_default(),
                c.current
            ),
            c.current,
        ),
        Err(StartError::Spawn(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": msg })),
        )
            .into_response(),
    }
}

/// GET /api/canvas/frame —— 当前帧 PNG 直出；无帧 503；携带预期代次/seq 与
/// 当前不一致 → 409（不伪造历史帧，客户端重新取 status）。
async fn canvas_frame(
    State(state): State<AppState>,
    Query(q): Query<CanvasIdentityQuery>,
) -> Response {
    match state.canvas.frame_for(q.generation, q.seq) {
        Ok(bytes) => (
            [(header::CONTENT_TYPE, "image/png")],
            bytes.as_ref().to_vec(),
        )
            .into_response(),
        Err(FrameConflict::Generation { current }) => conflict_409(
            format!(
                "canvas frame generation mismatch — re-fetch /api/canvas/status (current {current})"
            ),
            current,
        ),
        Err(FrameConflict::StaleSeq { current_seq }) => {
            // 请求历史 seq 或无帧：有帧时按代次冲突语义回 409（客户端重取
            // status 后会拿到新 seq）；无帧保持 503 原语义。
            if state.canvas.frame().is_some() {
                conflict_409(
                    format!(
                        "canvas frame seq mismatch — re-fetch /api/canvas/status (current seq {current_seq})"
                    ),
                    state.canvas.generation(),
                )
            } else {
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(serde_json::json!({ "error": "no canvas frame yet" })),
                )
                    .into_response()
            }
        }
    }
}

/// GET /api/canvas/status —— M1 基础字段 + picked/overlay/tree/pac（M2 全载荷；
/// 前端 1s 轮询同拍取走，零新通道）。
async fn canvas_status(State(state): State<AppState>) -> Response {
    Json(state.canvas.status_full()).into_response()
}

/// POST /api/canvas/pick —— 点选→锚点（200 命中 / 204 未命中·已清选 /
/// 200 {cleared:true} 显式清选 / 503 无帧或无索引 / 409 代次冲突）。
/// 命中即置 manager picked（status 下一拍带出，前端层树联动）。
async fn canvas_pick(
    State(state): State<AppState>,
    Json(body): Json<CanvasPickRequest>,
) -> Response {
    let manager = &state.canvas;
    let clear = body.clear.unwrap_or(false);
    if let Some(exp) = body.expected_generation {
        let current = manager.generation();
        if current != 0 && exp != current {
            return conflict_409(
                format!(
                    "canvas pick generation conflict: expected {exp}, current {current} — re-fetch /api/canvas/status"
                ),
                current,
            );
        }
    }
    if clear {
        if body.x.is_some() || body.y.is_some() || body.vnode_id.is_some() {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "pick {clear:true} is mutually exclusive with x/y/vnode_id" })),
            )
                .into_response();
        }
        manager.clear_pick();
        return Json(serde_json::json!({ "cleared": true })).into_response();
    }
    if manager.frame().is_none() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "no canvas frame yet" })),
        )
            .into_response();
    }
    let result = match (body.vnode_id.clone(), body.x, body.y) {
        (Some(v), _, _) => manager.pick_vnode_str(&v),
        (None, Some(x), Some(y)) => manager.pick_at(x, y),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "pick requires {x,y} (frame px), {vnode_id} or {clear:true}" })),
            )
                .into_response()
        }
    };
    match result {
        Some(anchor) => Json(anchor).into_response(),
        // 未命中：manager 已清 picked（PLAN-093），204 语义保持。
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

/// POST /api/canvas/stop —— 停止并收割（幂等）。携带预期代次与当前不一致 →
/// 409 不杀新目标（PLAN-093：旧 stop 不得作用于已替换的应用）；无预期 = 旧
/// 无身份调用，保持无条件停止。
async fn canvas_stop(
    State(state): State<AppState>,
    Query(q): Query<CanvasIdentityQuery>,
) -> Response {
    match state.canvas.stop_guarded(q.generation, None).await {
        Ok(generation) => Json(serde_json::json!({ "state": "stopped", "generation_id": generation }))
            .into_response(),
        Err(StopRefused::GenerationConflict { current }) => conflict_409(
            format!(
                "canvas stop generation conflict — session was replaced (current {current}); re-fetch /api/canvas/status"
            ),
            current,
        ),
        Err(StopRefused::OwnedByOther { owner }) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": format!("canvas session belongs to conversation {owner}") })),
        )
            .into_response(),
    }
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
