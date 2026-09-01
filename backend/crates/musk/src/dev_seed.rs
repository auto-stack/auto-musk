//! dev_seed.rs — 开发/演示用数据种子路由（hw 逃生舱，仿 workspace::pick_routes）。
//!
//! POST /api/dev/seed-run?workspace=<id>，请求体为完整 RunEntry JSON。relay
//! run 是纯内存态（RunStore 不落盘），真实 run 只能来自 agent 链路；本路由
//! 让演示/测试场景能注入一段编排好的事件史（Run 窗口块型全谱展示等）。
//! 注入在进程退出后即失，重启后由种子方重新 POST。
//!
//! 仅本地开发用途：无鉴权（与同级 workspace 端点口径一致）、不改盘。

use axum::{
    extract::{Query, State},
    routing::post,
    Json, Router,
};

use crate::relay::store::RunEntry;
use crate::server::AppState;
use crate::workspace::WorkspaceQuery;

async fn dev_seed_run(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Json(entry): Json<RunEntry>,
) -> Json<serde_json::Value> {
    let ws_id = q.id_or_default(&state.registry);
    let ws = state.registry.get(&ws_id);
    let run_id = entry.run_id.clone();
    ws.relay.seed(entry);
    tracing::info!(run_id = %run_id, ws_id = %ws_id, "dev seed: run injected");
    Json(serde_json::json!({ "seeded": run_id, "workspace": ws_id }))
}

/// dev 种子路由。
pub fn dev_routes() -> Router<AppState> {
    Router::new().route("/api/dev/seed-run", post(dev_seed_run))
}
