//! HTTP API server (`musk serve`) — exposes the agent over HTTP for the Vue
//! frontend.
//!
//! Vite proxies `/api/*` → `http://127.0.0.1:8080` (see
//! `gen/front/vue/vite.config.ts`), so this server listens on **:8080** and
//! mounts everything under `/api`.
//!
//! ## Endpoints
//! - `GET  /api/health`        — liveness probe.
//! - `GET  /api/professions`   — list built-in professions (name/model/temp).
//! - `POST /api/run`           — run an agent on a task, return the result.
//!
//! ## `POST /api/run` contract
//! Request:  `{ "task": "...", "profession": "coder" | "<path.at>" }`
//! Response: `{ "output": "...", "turns": N, "tool_calls": [...] }`
//!
//! `profession` is optional (defaults to "coder"). SSE streaming of partial
//! output is a later phase; this endpoint returns the full result when done.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

use auto_ai_agent::{builtin_names, load_builtin, load_profession, Client, Profession};

use crate::build_agent_from_mode;

/// Shared server state: a client that talks to the daemon, the auth store,
/// and the spec ledger store.
#[derive(Clone)]
pub struct AppState {
    pub client: Arc<dyn Client>,
    pub auth: Arc<crate::auth::AuthStore>,
    pub specs: Arc<crate::specs::SpecsStore>,
}

/// Run the HTTP server on the given address (default `127.0.0.1:8080`).
pub async fn serve(addr: &str, client: Arc<dyn Client>) -> Result<(), Box<dyn std::error::Error>> {
    let users_path = dirs::home_dir()
        .map(|h| h.join(".config/autoos/users.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("users.json"));
    let specs_path = dirs::home_dir()
        .map(|h| h.join(".config/autoos/specs.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("specs.json"));
    let state = AppState {
        client,
        auth: Arc::new(crate::auth::AuthStore::new(users_path)),
        specs: Arc::new(crate::specs::SpecsStore::new(specs_path)),
    };

    // Serve the standalone ESM config-page bundle (config-page.js) so that
    // auto-os-config can load it cross-origin via dynamic import(). The file is
    // produced by `npm run build` in frontend/ → frontend-dist/config-page.js.
    let assets_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("frontend-dist");
    let static_service = tower_http::services::ServeDir::new(&assets_path);

    // CORS: allow auto-os-config (and any localhost dev server) to load the
    // config-page bundle + config API cross-origin.
    let cors = tower_http::cors::CorsLayer::permissive()
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_origin(tower_http::cors::Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/professions", get(professions))
        .route("/api/run", post(run))
        .route("/api/run/stream", post(run_stream_handler))
        .route("/api/workflows", get(workflows))
        .route("/api/workflow/run", post(workflow_run))
        .route("/api/workflow/run/stream", post(workflow_run_stream))
        .route("/api/auth/login", post(auth_login))
        .route("/api/auth/me", get(auth_me))
        .route("/api/auth/logout", post(auth_logout))
        .route("/api/specs", get(specs_list))
        .route("/api/specs/item", post(specs_upsert))
        .route("/api/specs/transition", post(specs_transition))
        .route("/api/specs/item/{section}/{id}", axum::routing::delete(specs_delete))
        .route("/api/config", get(config_overview))
        .route("/api/modes", get(modes_list))
        .route("/api/skills", get(skills_list))
        // Serve config-page.js + any other static assets at the root.
        .fallback_service(static_service)
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("musk server listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

// ── Auth endpoints ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: crate::auth::UserInfo,
}

/// `POST /api/auth/login` — verify credentials, return a bearer token.
async fn auth_login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    match state.auth.login(&req.username, &req.password) {
        Some(session) => {
            let user = state
                .auth
                .session_user(&session.token)
                .expect("session just created");
            Json(LoginResponse {
                token: session.token,
                user,
            })
            .into_response()
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                error: "invalid credentials".into(),
            }),
        )
            .into_response(),
    }
}

/// `GET /api/auth/me` — resolve the bearer token to the user, else 401.
async fn auth_me(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    match bearer_token(&headers) {
        Some(token) => match state.auth.session_user(&token) {
            Some(user) => Json(user).into_response(),
            None => (
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    error: "invalid or expired session".into(),
                }),
            )
                .into_response(),
        },
        None => (
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                error: "missing Authorization header".into(),
            }),
        )
            .into_response(),
    }
}

/// `POST /api/auth/logout` — invalidate the bearer session.
async fn auth_logout(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    if let Some(token) = bearer_token(&headers) {
        state.auth.logout(&token);
    }
    Json(json!({"status": "logged out"}))
}

/// Extract a bearer token from `Authorization: Bearer <token>`.
fn bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let h = headers.get("authorization")?.to_str().ok()?;
    let t = h.strip_prefix("Bearer ")?.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

// ── Spec Ledger endpoints ───────────────────────────────────────────────────

/// `GET /api/specs` — return the full spec document (all sections + items).
async fn specs_list(State(state): State<AppState>) -> impl IntoResponse {
    match state.specs.load() {
        Ok(doc) => Json(doc).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { error: format!("load specs: {e}") }),
        )
            .into_response(),
    }
}

/// `POST /api/specs/item` — create or update a spec item.
#[derive(Debug, Deserialize)]
pub struct SpecsUpsertRequest {
    pub section_id: String,
    pub item: crate::specs::SpecItem,
}

async fn specs_upsert(
    State(state): State<AppState>,
    Json(req): Json<SpecsUpsertRequest>,
) -> impl IntoResponse {
    let mut doc = match state.specs.load() {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { error: format!("load: {e}") }),
            )
                .into_response()
        }
    };
    match state
        .specs
        .upsert_item(&mut doc, &req.section_id, req.item)
    {
        Ok(_) => match state.specs.save(&doc) {
            Ok(_) => Json(&doc).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { error: format!("save: {e}") }),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiError { error: e }),
        )
            .into_response(),
    }
}

/// `POST /api/specs/transition` — change an item's status (validated by the
/// state machine).
#[derive(Debug, Deserialize)]
pub struct SpecsTransitionRequest {
    pub section_id: String,
    pub item_id: String,
    pub new_status: String,
}

async fn specs_transition(
    State(state): State<AppState>,
    Json(req): Json<SpecsTransitionRequest>,
) -> impl IntoResponse {
    let new_status = crate::specs::SpecStatus::from_str_lossy(&req.new_status);
    let mut doc = match state.specs.load() {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { error: format!("load: {e}") }),
            )
                .into_response()
        }
    };
    match state
        .specs
        .transition_item(&mut doc, &req.section_id, &req.item_id, new_status)
    {
        Ok(_) => match state.specs.save(&doc) {
            Ok(_) => Json(json!({"status": "ok", "new_status": req.new_status})).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { error: format!("save: {e}") }),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiError { error: e }),
        )
            .into_response(),
    }
}

/// `DELETE /api/specs/item/:section/:id` — remove a spec item.
async fn specs_delete(
    State(state): State<AppState>,
    axum::extract::Path((section_id, item_id)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let mut doc = match state.specs.load() {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { error: format!("load: {e}") }),
            )
                .into_response()
        }
    };
    match state.specs.delete_item(&mut doc, &section_id, &item_id) {
        Ok(true) => match state.specs.save(&doc) {
            Ok(_) => Json(json!({"status": "deleted"})).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { error: format!("save: {e}") }),
            )
                .into_response(),
        },
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ApiError { error: "item not found".into() }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiError { error: e }),
        )
            .into_response(),
    }
}

async fn professions() -> impl IntoResponse {
    let list: Vec<serde_json::Value> = builtin_names()
        .iter()
        .filter_map(|name| {
            load_builtin(name).map(|p| {
                json!({
                    "name": name,
                    "tier": format!("{:?}", p.model_tier()).to_lowercase(),
                    "model": p.model(),
                    "temperature": p.temperature(),
                    "max_turns": p.max_turns(),
                })
            })
        })
        .collect();
    Json(json!({"professions": list}))
}

// ── Config page endpoints ───────────────────────────────────────────────────

/// `GET /api/config` — combined overview of modes, professions, skills.
async fn config_overview() -> impl IntoResponse {
    let reg = crate::mode::ModeRegistry::load();
    let modes: Vec<serde_json::Value> = reg
        .names()
        .iter()
        .filter_map(|n| {
            reg.get(n).map(|m| {
                json!({
                    "name": m.name,
                    "description": m.description,
                    "profession": m.profession,
                    "skills": m.skills,
                    "tool_count": m.tools.len(),
                })
            })
        })
        .collect();

    let profs: Vec<serde_json::Value> = auto_ai_agent::builtin_names()
        .iter()
        .filter_map(|name| {
            auto_ai_agent::load_builtin(name).map(|p| {
                json!({
                    "name": name,
                    "tier": format!("{:?}", p.model_tier()).to_lowercase(),
                    "temperature": p.temperature(),
                    "max_turns": p.max_turns(),
                })
            })
        })
        .collect();

    let skills_dir = dirs::home_dir().map(|h| h.join(".config/autoos/skills"));
    let skills: Vec<serde_json::Value> = if let Some(dir) = skills_dir {
        let reg = std::sync::Arc::new(auto_ai_agent::SkillRegistry::scan(&dir));
        reg.descriptions()
            .iter()
            .map(|(name, desc)| json!({ "name": name, "description": desc }))
            .collect()
    } else {
        vec![]
    };

    Json(json!({ "modes": modes, "professions": profs, "skills": skills }))
}

/// `GET /api/modes` — list all agent modes.
async fn modes_list() -> impl IntoResponse {
    let reg = crate::mode::ModeRegistry::load();
    let modes: Vec<serde_json::Value> = reg
        .names()
        .iter()
        .filter_map(|n| {
            reg.get(n).map(|m| {
                json!({
                    "name": m.name,
                    "description": m.description,
                    "profession": m.profession,
                    "skills": m.skills,
                    "tool_count": m.tools.len(),
                })
            })
        })
        .collect();
    Json(json!({ "modes": modes }))
}

/// `GET /api/skills` — list all configured skills.
async fn skills_list() -> impl IntoResponse {
    let skills_dir = dirs::home_dir().map(|h| h.join(".config/autoos/skills"));
    let skills: Vec<serde_json::Value> = if let Some(dir) = skills_dir {
        let reg = std::sync::Arc::new(auto_ai_agent::SkillRegistry::scan(&dir));
        reg.descriptions()
            .iter()
            .map(|(name, desc)| json!({ "name": name, "description": desc }))
            .collect()
    } else {
        vec![]
    };
    Json(json!({ "skills": skills }))
}

/// `POST /api/run` request body.
#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub task: String,
    /// Agent mode: built-in name (superpowers/basic/coding/review) or path to
    /// a `.at` mode file. Defaults to "superpowers".
    #[serde(default = "default_mode")]
    pub mode: String,
}

fn default_mode() -> String {
    "superpowers".to_string()
}

/// One tool-call record in the response.
#[derive(Debug, Serialize)]
pub struct ToolCallOut {
    pub tool: String,
    pub args: serde_json::Value,
    pub result: String,
}

/// `POST /api/run` response body (on success).
#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub output: String,
    pub turns: usize,
    pub tool_calls: Vec<ToolCallOut>,
}

/// Error response body.
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}

/// Core logic for `POST /api/run`, returning a typed Result so it's testable
/// without going through the HTTP layer.
async fn run_inner(
    state: AppState,
    req: RunRequest,
) -> Result<RunResponse, (StatusCode, Json<ApiError>)> {
    // Resolve the mode from the request.
    let reg = crate::mode::ModeRegistry::load();
    let mode = reg.get(&req.mode).cloned().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: format!(
                    "unknown mode '{}'; available: {}",
                    req.mode,
                    reg.names().join(", ")
                ),
            }),
        )
    })?;

    let mut agent = crate::build_agent_from_mode(&mode, state.client.clone())
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: format!("build agent: {e}"),
                }),
            )
        })?;
    let result = agent.run(&req.task).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: format!("agent failed: {e}"),
            }),
        )
    })?;

    let tool_calls = result
        .tool_calls
        .iter()
        .map(|tc| ToolCallOut {
            tool: tc.tool.clone(),
            args: tc.args.clone(),
            result: tc.result.clone(),
        })
        .collect();
    Ok(RunResponse {
        output: result.output,
        turns: result.turns,
        tool_calls,
    })
}

async fn run(
    State(state): State<AppState>,
    Json(req): Json<RunRequest>,
) -> impl IntoResponse {
    match run_inner(state, req).await {
        Ok(resp) => Json(resp).into_response(),
        Err(err) => err.into_response(),
    }
}

/// `POST /api/run/stream` — streaming variant. Streams the agent's progress as
/// SSE events so the frontend can render tokens live.
///
/// SSE events (each is a `data:` line with JSON):
/// - `{"type":"delta","text":"…"}`   — a text chunk
/// - `{"type":"tool",…}`             — a tool call + result
/// - `{"type":"done",…}`             — loop finished (full result)
/// - `{"type":"error","message":"…"}`— loop failed
async fn run_stream_handler(
    State(state): State<AppState>,
    Json(req): Json<RunRequest>,
) -> impl IntoResponse {
    use axum::body::Body;
    use axum::response::Response;
    use tokio::sync::mpsc;

    let (tx, mut rx) = mpsc::channel::<serde_json::Value>(64);

    // Resolve the mode up front so we can fail fast on a bad spec.
    let reg = crate::mode::ModeRegistry::load();
    let mode = match reg.get(&req.mode).cloned() {
        Some(m) => m,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: format!(
                        "unknown mode '{}'; available: {}",
                        req.mode,
                        reg.names().join(", ")
                    ),
                }),
            )
                .into_response();
        }
    };

    // Spawn the agent run, pushing StreamEvents into the channel as SSE JSON.
    let client = state.client.clone();
    tokio::spawn(async move {
        let mut agent = match crate::build_agent_from_mode(&mode, client) {
            Ok(a) => a,
            Err(e) => {
                let _ = tx.try_send(json!({"type": "error", "message": format!("build agent: {e}")}));
                return;
            }
        };
        let tx2 = tx.clone();
        let on_event: Arc<dyn Fn(auto_ai_agent::StreamEvent) + Send + Sync> =
            Arc::new(move |ev| {
                let value = stream_event_to_json(&ev);
                let _ = tx2.try_send(value);
            });
        match agent.run_stream(&req.task, on_event).await {
            Ok(_) => {
                // Done event already emitted by run_stream; nothing more.
            }
            Err(e) => {
                let _ = tx.try_send(json!({"type": "error", "message": format!("{e}")}));
            }
        }
    });

    let stream = async_stream::stream! {
        while let Some(value) = rx.recv().await {
            yield Ok::<_, std::convert::Infallible>(format!("data: {value}\n\n"));
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap()
}

/// Serialize a [`auto_ai_agent::StreamEvent`] to the SSE JSON shape.
fn stream_event_to_json(ev: &auto_ai_agent::StreamEvent) -> serde_json::Value {
    use auto_ai_agent::StreamEvent;
    match ev {
        StreamEvent::Delta { text } => json!({"type": "delta", "text": text}),
        StreamEvent::Tool { tool, args, result } => json!({
            "type": "tool",
            "tool": tool,
            "args": args,
            "result": result,
        }),
        StreamEvent::Done { result } => json!({
            "type": "done",
            "output": result.output,
            "turns": result.turns,
            "tool_calls": result.tool_calls.iter().map(|tc| json!({
                "tool": tc.tool, "args": tc.args, "result": tc.result,
            })).collect::<Vec<_>>(),
        }),
        StreamEvent::Error { message } => json!({"type": "error", "message": message}),
    }
}

// ── Workflow endpoints ─────────────────────────────────────────────────────

/// The shared tool set every workflow agent may use (read/write/run).
fn shared_tools() -> Vec<Arc<dyn auto_ai_agent::Tool>> {
    vec![
        Arc::new(crate::tools::ReadFile),
        Arc::new(crate::tools::WriteFile),
        Arc::new(crate::tools::RunCommand),
    ]
}

/// `GET /api/workflows` — list built-in workflows.
async fn workflows() -> impl IntoResponse {
    Json(json!({"workflows": crate::workflow::builtin_names()}))
}

/// `POST /api/workflow/run` request.
#[derive(Debug, Deserialize)]
pub struct WorkflowRunRequest {
    /// The task / user request (seeds `$user_request`).
    pub task: String,
    /// Built-in workflow name (e.g. "feature-dev") or path to a `.at` file.
    pub workflow: String,
}

/// `POST /api/workflow/run` response.
#[derive(Debug, Serialize)]
pub struct WorkflowRunResponse {
    /// Each step id → its output.
    pub steps: std::collections::HashMap<String, String>,
    /// Each output variable → its value.
    pub outputs: std::collections::HashMap<String, String>,
}

async fn workflow_run(
    State(state): State<AppState>,
    Json(req): Json<WorkflowRunRequest>,
) -> Result<Json<WorkflowRunResponse>, (StatusCode, Json<ApiError>)> {
    let wf = crate::workflow::load(&req.workflow).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: format!("invalid workflow '{}': {e}", req.workflow),
            }),
        )
    })?;

    wf.run(&shared_tools(), state.client.clone(), &req.task)
        .await
        .map(|result| {
            Json(WorkflowRunResponse {
                steps: result.step_outputs,
                outputs: result.outputs,
            })
        })
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: format!("workflow failed: {e}"),
                }),
            )
        })
}

/// `POST /api/workflow/run/stream` — streaming workflow run.
///
/// Emits step-by-step SSE events so a long multi-step workflow doesn't block
/// a single HTTP response. Events:
/// - `{"type":"step_start","step_id":"architect","profession":"architect","input":"…"}`
/// - `{"type":"step_done","step_id":"architect","output":"…"}`
/// - `{"type":"step_skipped","step_id":"reviewer"}`
/// - `{"type":"finished",…}` (or `{"type":"error","message":"…"}`)
async fn workflow_run_stream(
    State(state): State<AppState>,
    Json(req): Json<WorkflowRunRequest>,
) -> axum::response::Response {
    use axum::body::Body;
    use axum::response::Response;
    use tokio::sync::mpsc;

    let (tx, mut rx) = mpsc::channel::<serde_json::Value>(64);

    let wf = match crate::workflow::load(&req.workflow) {
        Ok(w) => w,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: format!("invalid workflow '{}': {e}", req.workflow),
                }),
            )
                .into_response();
        }
    };

    let client = state.client.clone();
    let task = req.task.clone();
    tokio::spawn(async move {
        let on_event: Arc<dyn Fn(auto_ai_agent::WorkflowEvent) + Send + Sync> =
            Arc::new(move |ev| {
                let value = workflow_event_to_json(&ev);
                let _ = tx.try_send(value);
            });
        if let Err(e) = wf
            .run_with_progress(&shared_tools(), client, &task, on_event)
            .await
        {
            // Errors also surfaced via the event stream (StepDone error), but
            // a top-level failure (e.g. cycle) goes here:
            tracing::error!("workflow stream failed: {e}");
        }
    });

    let stream = async_stream::stream! {
        while let Some(value) = rx.recv().await {
            yield Ok::<_, std::convert::Infallible>(format!("data: {value}\n\n"));
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap()
}

/// Serialize a [`auto_ai_agent::WorkflowEvent`] to the SSE JSON shape.
fn workflow_event_to_json(ev: &auto_ai_agent::WorkflowEvent) -> serde_json::Value {
    use auto_ai_agent::WorkflowEvent;
    match ev {
        WorkflowEvent::StepStart {
            step_id,
            profession,
            input,
        } => json!({
            "type": "step_start",
            "step_id": step_id,
            "profession": profession,
            "input": input,
        }),
        WorkflowEvent::StepDone { step_id, output } => {
            json!({"type": "step_done", "step_id": step_id, "output": output})
        }
        WorkflowEvent::StepSkipped { step_id } => {
            json!({"type": "step_skipped", "step_id": step_id})
        }
        WorkflowEvent::Finished { result } => json!({
            "type": "finished",
            "steps": result.step_outputs,
            "outputs": result.outputs,
        }),
    }
}

/// Resolve a profession spec: built-in name, else `.at` file path.
fn resolve(spec: &str) -> Result<Arc<dyn Profession>, String> {
    if let Some(p) = load_builtin(spec) {
        return Ok(p);
    }
    let content = std::fs::read_to_string(spec)
        .map_err(|e| format!("not a builtin, cannot read '{spec}': {e}"))?;
    load_profession(&content).map_err(|e| format!("parse '{spec}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};

    /// A mock client that returns a canned text answer (no daemon needed).
    struct MockClient;
    #[async_trait]
    impl Client for MockClient {
        async fn complete(
            &self,
            _req: &CompletionRequest,
        ) -> Result<CompletionResponse, ClientError> {
            Ok(CompletionResponse {
                content: "mock answer".into(),
                tool_calls: vec![],
                stop_reason: Some("end_turn".into()),
                usage: None,
                model: "mock".into(),
                error: None,
            })
        }
    }

    fn tmp_auth() -> Arc<crate::auth::AuthStore> {
        let path = std::env::temp_dir().join(format!(
            "musk_server_auth_test_{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        Arc::new(crate::auth::AuthStore::new(path))
    }

    fn tmp_specs() -> Arc<crate::specs::SpecsStore> {
        let path = std::env::temp_dir().join(format!(
            "musk_server_specs_test_{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        Arc::new(crate::specs::SpecsStore::new(path))
    }

    #[tokio::test]
    async fn run_endpoint_returns_result() {
        let state = AppState {
            client: Arc::new(MockClient) as Arc<dyn Client>,
            auth: tmp_auth(),
            specs: tmp_specs(),
        };
        let req = RunRequest {
            task: "say hello".into(),
            profession: "coder".into(),
            skills: true,
        };
        let resp = run_inner(state, req).await.unwrap();
        assert_eq!(resp.output, "mock answer");
        assert_eq!(resp.turns, 1);
    }

    #[tokio::test]
    async fn run_endpoint_bad_profession_errors() {
        let state = AppState {
            client: Arc::new(MockClient) as Arc<dyn Client>,
            auth: tmp_auth(),
            specs: tmp_specs(),
        };
        let req = RunRequest {
            task: "x".into(),
            profession: "nonexistent".into(),
            skills: true,
        };
        let err = run_inner(state, req).await.unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn resolve_builtin() {
        let p = resolve("coder").unwrap();
        assert_eq!(p.name(), "coder");
    }

    #[test]
    fn resolve_unknown_errors() {
        assert!(resolve("does-not-exist").is_err());
    }
}
