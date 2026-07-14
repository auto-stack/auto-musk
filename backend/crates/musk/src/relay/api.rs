//! Relay REST API + SSE — the HTTP surface the frontend `useRelay.ts` talks to.
//!
//! P2b.1 simplified driver: `advance` runs a step synchronously by resolving
//! the step's `role_id` into a minimal [`AgentMode`] and calling
//! `agent.run(task)`, then wraps the result in a [`HandoffDocument`] and
//! submits it. A full background driver with streaming turn events arrives in
//! P2b.2.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{sse::{Event, KeepAlive, Sse}, IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt;

use crate::relay::HandoffDocument;
use crate::relay::{AdvanceResult, GateDecision};
use crate::relay::profession::ProfessionRegistry;
use crate::relay::store::{RunEvent, RunState, RunStore, StartRunRequest};
use crate::server::AppState;
use crate::workspace::WorkspaceQuery;

// ─── Broadcast event bus (for SSE) ──────────────────────────────────────────
//
// A single process-wide broadcast channel fans run events out to all SSE
// subscribers. Each event is tagged with its run_id so subscribers filter.

#[derive(Clone)]
struct BusEvent {
    run_id: String,
    event_type: String,
    payload: serde_json::Value,
}

fn bus() -> &'static tokio::sync::broadcast::Sender<BusEvent> {
    static BUS: std::sync::OnceLock<tokio::sync::broadcast::Sender<BusEvent>> =
        std::sync::OnceLock::new();
    BUS.get_or_init(|| tokio::sync::broadcast::channel(256).0)
}

fn publish_internal(run_id: &str, event: &RunEvent) {
    // Best-effort: ignore "no receivers".
    let _ = bus().send(BusEvent {
        run_id: run_id.into(),
        event_type: event.event_type().into(),
        payload: serde_json::to_value(event).unwrap_or(serde_json::Value::Null),
    });
}

/// Publish a run event to all SSE subscribers. Public so the driver and store
/// can broadcast turn-level events.
pub fn publish(run_id: &str, event: &RunEvent) {
    publish_internal(run_id, event);
}

// ─── DTOs ───────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct ResolveGateBody {
    decision: String, // approve | reject | edit
    #[serde(default)]
    feedback: Option<String>,
}

#[derive(serde::Deserialize)]
struct SubmitHandoffBody {
    handoff: serde_json::Value,
}

#[derive(serde::Deserialize)]
struct UpdateTitleBody {
    title: String,
}

#[derive(serde::Deserialize, Default)]
struct ListRunsQuery {
    #[serde(default)]
    project_path: Option<String>,
}

// ─── Handlers ───────────────────────────────────────────────────────────────

/// `GET /api/forge/relay/runs` — list all runs (newest first).
async fn list_runs(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Query(_list_q): Query<ListRunsQuery>,
) -> impl IntoResponse {
    // useRelay.loadRuns tolerates {runs:[...]} or a bare array; return the
    // wrapper for forward-compat with pagination metadata.
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    Json(serde_json::json!({ "runs": ws.relay.list() }))
}

/// `POST /api/forge/relay/runs` — start a run.
async fn start_run(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Json(req): Json<StartRunRequest>,
) -> impl IntoResponse {
    let ws_id = q.id_or_default(&state.registry);
    let ws = state.registry.get(&ws_id);
    let (run_id, run_state) = ws.relay.start_run(&req, Some(ws_id));
    // Publish a synthetic run_started so any live listeners refresh.
    publish(
        &run_id,
        &RunEvent::RelayUpdate {
            timestamp: now_secs(),
            step_id: String::new(),
            role_id: String::new(),
            status: "idle".into(),
        },
    );
    Json(serde_json::json!({ "run_id": run_id, "state": run_state }))
}

/// `GET /api/forge/relay/runs/{run_id}` — detailed run state.
async fn get_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(q): Query<WorkspaceQuery>,
) -> Response {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.relay.get(&run_id) {
        Some(state) => Json(state).into_response(),
        None => (StatusCode::NOT_FOUND, format!("run '{run_id}' not found")).into_response(),
    }
}

/// `DELETE /api/forge/relay/runs/{run_id}` — delete a run.
async fn delete_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(q): Query<WorkspaceQuery>,
) -> Response {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    if ws.relay.delete(&run_id) {
        Json(serde_json::json!({"status": "deleted", "id": run_id})).into_response()
    } else {
        (StatusCode::NOT_FOUND, format!("run '{run_id}' not found")).into_response()
    }
}

/// `PATCH /api/forge/relay/runs/{run_id}/title` — rename a run.
async fn update_title(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(q): Query<WorkspaceQuery>,
    Json(body): Json<UpdateTitleBody>,
) -> Response {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.relay.set_title(&run_id, &body.title) {
        Some(state) => Json(state).into_response(),
        None => (StatusCode::NOT_FOUND, format!("run '{run_id}' not found")).into_response(),
    }
}

/// `POST /api/forge/relay/runs/{run_id}/advance` — kick off the background
/// driver, which runs every auto step until a human gate, completion, failure,
/// or pause. Returns the current run state immediately; step/turn progress
/// streams over `GET /runs/{run_id}/events` (SSE).
///
/// If the run is already being driven (status == running), this is a no-op.
async fn advance_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(q): Query<WorkspaceQuery>,
) -> Response {
    let ws_id = q.id_or_default(&state.registry);
    let ws = state.registry.get(&ws_id);
    // Guard: don't start a second driver if one is already running this run.
    if ws.relay.is_running(&run_id) {
        return match ws.relay.get(&run_id) {
            Some(s) => Json(s).into_response(),
            None => (StatusCode::NOT_FOUND, format!("run '{run_id}' not found")).into_response(),
        };
    }
    // Spawn the driver. It advances, runs each step's agent with streaming,
    // submits handoffs, and stops at a gate / terminal state.
    let state_arc = Arc::new(state.clone());
    let run_id_clone = run_id.clone();
    tokio::spawn(async move {
        crate::relay::driver::drive_run(state_arc, ws_id, run_id_clone).await;
    });
    // Return the current (pre-drive or just-advanced) snapshot.
    match ws.relay.get(&run_id) {
        Some(s) => Json(s).into_response(),
        None => (StatusCode::NOT_FOUND, format!("run '{run_id}' not found")).into_response(),
    }
}

/// `POST /api/forge/relay/runs/{run_id}/handoff` — submit a handoff directly.
async fn submit_handoff(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(q): Query<WorkspaceQuery>,
    Json(body): Json<SubmitHandoffBody>,
) -> Response {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let handoff: HandoffDocument = match serde_json::from_value(body.handoff) {
        Ok(h) => h,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("invalid handoff: {e}")).into_response()
        }
    };
    match ws.relay.submit_handoff(&run_id, handoff) {
        Some((result, state)) => {
            publish_advance_result(&run_id, &result);
            Json(state).into_response()
        }
        None => (StatusCode::NOT_FOUND, format!("run '{run_id}' not found")).into_response(),
    }
}

/// `POST /api/forge/relay/runs/{run_id}/gate` — resolve a pending gate.
async fn resolve_gate(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(q): Query<WorkspaceQuery>,
    Json(body): Json<ResolveGateBody>,
) -> Response {
    let ws_id = q.id_or_default(&state.registry);
    let ws = state.registry.get(&ws_id);
    let decision = match body.decision.as_str() {
        "approve" | "edit" => GateDecision::Approve,
        "reject" => GateDecision::Reject {
            feedback: body.feedback.unwrap_or_default(),
        },
        other => {
            return (
                StatusCode::BAD_REQUEST,
                format!("unknown gate decision '{other}' (want approve|reject|edit)"),
            )
                .into_response()
        }
    };
    match ws.relay.resolve_gate(&run_id, decision) {
        Some((result, run_state)) => {
            publish_advance_result(&run_id, &result);
            // After resolving a gate, resume the background driver so the run
            // continues autonomously to the next gate / terminal state.
            if matches!(result, AdvanceResult::ExecuteStep { .. }) {
                let state_arc = Arc::new(state.clone());
                let run_id_clone = run_id.clone();
                tokio::spawn(async move {
                    crate::relay::driver::drive_run(state_arc, ws_id, run_id_clone).await;
                });
            }
            Json(run_state).into_response()
        }
        None => (StatusCode::NOT_FOUND, format!("run '{run_id}' not found")).into_response(),
    }
}

/// `POST /api/forge/relay/runs/{run_id}/rerun` — rerun from the failed step.
async fn rerun_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(q): Query<WorkspaceQuery>,
) -> Response {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.relay.rerun(&run_id) {
        Some(state) => Json(state).into_response(),
        None => (StatusCode::NOT_FOUND, format!("run '{run_id}' not found")).into_response(),
    }
}

/// `GET /api/forge/relay/runs/{run_id}/events` — SSE stream of run events.
async fn run_events(Path(run_id): Path<String>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = bus().subscribe();
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .filter_map(move |res| match res {
            Ok(ev) if ev.run_id == run_id => Some(ev),
            _ => None,
        })
        .map(|ev| {
            Ok(Event::default()
                .event("run_event")
                .json_data(serde_json::json!({
                    "event_type": ev.event_type,
                    "payload": ev.payload,
                }))
                .unwrap_or_else(|_| Event::default()))
        });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

/// `GET /api/forge/relay/professions` — list professions.
async fn list_professions(State(_state): State<AppState>) -> impl IntoResponse {
    let reg = ProfessionRegistry::load();
    Json(serde_json::json!({ "professions": reg.list() }))
}

/// `GET /api/forge/relay/souls` — list souls (empty in P2b.1; soul.rs comes later).
async fn list_souls() -> impl IntoResponse {
    Json(serde_json::json!({ "souls": [] }))
}

/// `GET /api/forge/relay/flows` — list built-in flows.
async fn list_flows() -> impl IntoResponse {
    let flows: Vec<serde_json::Value> = crate::relay::builtin_flows()
        .into_iter()
        .map(|f| {
            serde_json::json!({
                "id": f.id,
                "steps": f.steps.iter().map(|s| {
                    serde_json::json!({
                        "id": s.id,
                        "role_id": s.role_id,
                        "gate": match s.gate {
                            crate::relay::GateType::Auto => "auto",
                            crate::relay::GateType::Human => "human",
                        },
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect();
    Json(serde_json::json!({ "flows": flows }))
}

// ─── Driver dispatch ────────────────────────────────────────────────────────
//
// The background driver lives in `crate::relay::driver`. The `advance` and
// `resolve_gate` handlers `tokio::spawn` it; here we only keep the SSE-bus
// publishing helpers the driver + store call back into.

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn publish_advance_result(run_id: &str, result: &AdvanceResult) {
    let now = now_secs();
    match result {
        AdvanceResult::ExecuteStep { step_id, role_id, .. } => {
            publish(run_id, &RunEvent::StepStarted {
                timestamp: now,
                step_id: step_id.clone(),
                role_id: role_id.clone(),
            });
        }
        AdvanceResult::WaitForHuman { step_id, .. } => {
            publish(run_id, &RunEvent::GateWaiting {
                timestamp: now,
                step_id: step_id.clone(),
                gate: "human".into(),
            });
        }
        AdvanceResult::Completed => {
            publish(run_id, &RunEvent::RunCompleted { timestamp: now });
        }
        AdvanceResult::Failed { error } => {
            publish(run_id, &RunEvent::RunFailed {
                timestamp: now,
                error: error.clone(),
            });
        }
        AdvanceResult::Paused { .. } => {}
    }
}

// Placeholder removed — publish_advance_result no longer needs an AppState.

// ─── Router ─────────────────────────────────────────────────────────────────

/// All relay routes. Paths match `useRelay.ts` exactly; `.merge`-ed into the
/// main router without a prefix.
pub fn relay_routes() -> Router<AppState> {
    Router::new()
        .route("/api/forge/relay/runs", get(list_runs).post(start_run))
        .route(
            "/api/forge/relay/runs/{run_id}",
            get(get_run).delete(delete_run),
        )
        .route("/api/forge/relay/runs/{run_id}/title", patch(update_title))
        .route("/api/forge/relay/runs/{run_id}/advance", post(advance_run))
        .route("/api/forge/relay/runs/{run_id}/rerun", post(rerun_run))
        .route("/api/forge/relay/runs/{run_id}/handoff", post(submit_handoff))
        .route("/api/forge/relay/runs/{run_id}/gate", post(resolve_gate))
        .route("/api/forge/relay/runs/{run_id}/events", get(run_events))
        .route("/api/forge/relay/professions", get(list_professions))
        .route("/api/forge/relay/souls", get(list_souls))
        .route("/api/forge/relay/flows", get(list_flows))
}
