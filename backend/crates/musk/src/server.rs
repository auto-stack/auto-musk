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
//! Request:  `{ "task": "...", "role": "coder" | "<path.at>" }`
//! Response: `{ "output": "...", "turns": N, "tool_calls": [...] }`
//!
//! `role` is optional (defaults to "coder"). SSE streaming of partial
//! output is a later phase; this endpoint returns the full result when done.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

use auto_ai_agent::{load_builtin, load_role, Client, Role};

use crate::workspace::WorkspaceQuery;

/// Guess a Content-Type from a file extension (mirrors `wiki::guess_mime` but
/// local to the server module so the `/api/files` endpoint stays self-contained).

/// Shared server state: a client that talks to the daemon, the auth store,
/// and the workspace registry (which resolves per-workspace specs/chats/wiki/
/// relay stores via `?workspace=<id>`).
#[derive(Clone)]
pub struct AppState {
    pub client: Arc<dyn Client>,
    // 接线运行(计划 018 §11):auth 数据层是 a2r 转译版 AuthStore(①);
    // auth 端点由 a2r 转译 handler 服务(C2/C3),手写 auth handler 已删除。
    pub auth: Arc<crate::auto_generated::auth::AuthStore>,
    pub registry: Arc<crate::workspace::WorkspaceRegistry>,
}

/// Run the HTTP server on the given address (default `127.0.0.1:8080`).
pub async fn serve(addr: &str, client: Arc<dyn Client>) -> Result<(), Box<dyn std::error::Error>> {
    let users_path = dirs::home_dir()
        .map(|h| h.join(".config/autoos/users.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("users.json"));
    let config_dir = dirs::home_dir()
        .map(|h| h.join(".config/autoos"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let default_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let registry =
        crate::workspace::WorkspaceRegistry::load(config_dir.join("workspaces.json"), default_root);
    registry.migrate_global_data(&config_dir);
    let state = AppState {
        client,
        auth: Arc::new(crate::auto_generated::auth::AuthStore::new(users_path)),
        registry: Arc::new(registry),
    };

    // Static assets: the web app (Chats/Specs SPA) lives at `web/dist`
    // (built by `npm run build` in web/). The legacy config-page ESM bundle
    // (`frontend-dist/config-page.js`, if present) is served as a fallback so
    // auto-os-config can still load it cross-origin.
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // web/dist is two levels up from backend/crates/musk → auto-musk/web/dist.
    let web_dist = manifest
        .join("../../../web/dist")
        .canonicalize()
        .unwrap_or_else(|_| manifest.join("../../../web/dist"));
    // The config-page ESM bundles (served to auto-os-config) live at
    // backend/crates/musk/frontend-dist/ (that's where vite lib mode outputs).
    let frontend_dist = manifest.join("frontend-dist");
    // Serve static files: web/dist (the SPA) first, then frontend-dist (config
    // bundles for auto-os-config) as a nested fallback, then index.html (SPA
    // client-side routing). The nesting matters: each layer only falls through
    // if the previous didn't find the file.
    let index_html = web_dist.join("index.html");
    let static_service = tower_http::services::ServeDir::new(&web_dist)
        .fallback(
            tower_http::services::ServeDir::new(&frontend_dist)
                .fallback(tower_http::services::ServeFile::new(&index_html)),
        );

    // Warn (not fail) if the web app wasn't built — the API still works, but
    // the browser UI will be missing. Tells the user how to build it.
    if !web_dist.join("index.html").exists() {
        tracing::warn!(
            "web app not built at {} — `cd web && npm install && npm run build` for the UI. \
             The HTTP API is still available; the browser will show nothing until the web app is built.",
            web_dist.display()
        );
    }

    // CORS: allow auto-os-config (and any localhost dev server) to load the
    // config-page bundle + config API cross-origin.
    let cors = tower_http::cors::CorsLayer::permissive()
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_origin(tower_http::cors::Any);

    // ④ 整体接入(plan 018 §11):转译的 ag build_router(38 路由)作为主 router。
    // Plan 019:6 个 🔴 daemon/SSE handler 全部切到 ag server_stream
    // (Phase 1c 非流式 run/workflow_run + Phase 2-4 流式 run_stream/
    // workflow_run_stream/chat_stream/conversation_stream,均经 extern_impl
    // 真实委托)。
    // Plan 020 Phase F + Plan 021 Phase A:所有业务端点(含 /api/files 文件服务)
    // 全部由 ag handler 服务。唯一残留 hw:静态文件/CORS/serve 外壳。
    let app = crate::auto_generated::server::build_router()
        // daemon/SSE handlers — all served by transpiled server_stream handlers.
        .route("/api/run", post(crate::auto_generated::server_stream::run))
        .route("/api/run/stream", post(crate::auto_generated::server_stream::run_stream_handler))
        .route("/api/workflow/run", post(crate::auto_generated::server_stream::workflow_run))
        .route("/api/workflow/run/stream", post(crate::auto_generated::server_stream::workflow_run_stream))
        // settings_link(Plan 020 Phase E):ag server_stream handler + extern
        // settings_link_do(reqwest::blocking 封装)。
        .route("/api/settings-link", post(crate::auto_generated::server_stream::settings_link))
        .route("/api/chats/session/{id}/stream", get(crate::auto_generated::server_stream::chat_stream))
        .route("/api/conversations/{id}/stream", get(crate::auto_generated::server_stream::conversation_stream))
        // Relay (Flows) orchestration engine — ag relay_api handlers(Plan 020 Phase D)。
        .merge(crate::auto_generated::relay_api::relay_routes())
        // TaskPlan orchestration (Plan 009 P2b.7) — ag relay_api handlers。
        .merge(crate::auto_generated::relay_api::task_plan_routes())
        // Wiki knowledge base (Phase 4) — ag wiki handlers(Plan 020 Phase D)。
        .merge(crate::auto_generated::wiki::wiki_routes())
        // Serve config-page.js + any other static assets at the root.
        .fallback_service(static_service)
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("musk server listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
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

// ── Auth endpoints (Phase C: 完全由 a2r 转译 handler 服务,见
//    auto_generated::server::auth_login/auth_me/auth_logout —— 原手写版已删除) ──

// ── Spec Ledger endpoints ───────────────────────────────────────────────────
// ── Wiki (Flows) ── relay routes now live in `crate::relay::api`. ──────────

// ── Config page endpoints ───────────────────────────────────────────────────
// ── Plan 004: Agent Roles endpoints ─────────────────────────────────────────
// ── App runtime config (musk) ───────────────────────────────────────────────
//
// ── App runtime config (musk) ───────────────────────────────────────────────
//
// musk's runtime config lives in `crate::app_config` (shared with the CLI).
// The handlers here read/persist it; the CLI applies it to the environment.
// Per the unified-Harness design, app config is "how this app runs", not
// "which capabilities it inherits".

// NOTE: `POST /api/settings-link`(Plan 020 Phase E)已 Auto 化 —— 由
// auto_generated::server_stream::settings_link 服务(extern settings_link_do
// 封装 reqwest::blocking),serve() 路由已切换;原手写 handler 已删除。

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
    Query(q): Query<WorkspaceQuery>,
    Json(req): Json<RunRequest>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    crate::tool_safety::set_current_root(ws.root.clone());
    let result = run_inner(state, req).await;
    crate::tool_safety::clear_current_root();
    match result {
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
    Query(q): Query<WorkspaceQuery>,
    Json(req): Json<RunRequest>,
) -> impl IntoResponse {
    use axum::body::Body;
    use axum::response::Response;
    use tokio::sync::mpsc;

    let (tx, mut rx) = mpsc::channel::<serde_json::Value>(64);

    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let ws_root = ws.root.clone();

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
        // Confine this task's file-tool operations to the workspace root.
        crate::tool_safety::set_current_root(ws_root.clone());
        let mut agent = match crate::build_agent_from_mode(&mode, client) {
            Ok(a) => a,
            Err(e) => {
                crate::tool_safety::clear_current_root();
                let _ = tx.try_send(json!({"type": "error", "message": format!("build agent: {e}")}));
                return;
            }
        };
        let tx2 = tx.clone();
        let tc_counter = std::sync::Arc::new(std::sync::Mutex::new(0usize));
        let tc_stack: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let on_event: Arc<dyn Fn(auto_ai_agent::StreamEvent) + Send + Sync> =
            Arc::new(move |ev| {
                use auto_ai_agent::StreamEvent;
                let id = match &ev {
                    StreamEvent::ToolStart { .. } => {
                        let n = { let mut c = tc_counter.lock().unwrap(); *c += 1; *c };
                        let id = format!("tc-{n}");
                        tc_stack.lock().unwrap().push(id.clone());
                        Some(id)
                    }
                    StreamEvent::Tool { .. } => tc_stack.lock().unwrap().pop(),
                    _ => None,
                };
                let value = stream_event_to_json(&ev, id.as_deref());
                let _ = tx2.try_send(value);
            });
        // No cancellation endpoint yet — the run flag is never set.
        let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
        match agent.run_stream(&req.task, on_event, cancel).await {
            Ok(_) => {
                // Done event already emitted by run_stream; nothing more.
            }
            Err(e) => {
                let _ = tx.try_send(json!({"type": "error", "message": format!("{e}")}));
            }
        }
        crate::tool_safety::clear_current_root();
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
///
/// Tool events are emitted as the `tool_call` / `tool_result` pair the Vue
/// frontend expects (field names `name` / `arguments`, not `tool` / `args`).
/// `id` ties a `tool_result` back to its `tool_call` so the frontend can fill
/// in the result on the same card it already rendered as "running".
fn stream_event_to_json(ev: &auto_ai_agent::StreamEvent, id: Option<&str>) -> serde_json::Value {
    use auto_ai_agent::StreamEvent;
    let id_val = |id: Option<&str>| -> serde_json::Value {
        match id { Some(s) => json!(s), None => json!(null) }
    };
    match ev {
        StreamEvent::Delta { text } => json!({"type": "delta", "text": text}),
        StreamEvent::Thinking { text } => json!({"type": "thinking", "thinking": text}),
        StreamEvent::ToolStart { tool, args } => json!({
            "type": "tool_call",
            "id": id_val(id),
            "name": tool,
            "arguments": args,
        }),
        StreamEvent::Tool { tool, args, result } => json!({
            "type": "tool_result",
            "id": id_val(id),
            "name": tool,
            "arguments": args,
            "result": result,
            "status": "success",
        }),
        StreamEvent::Warning { text } => json!({"type": "warning", "text": text}),
        StreamEvent::Done { result } => json!({
            "type": "done",
            "output": result.output,
            "turns": result.turns,
            "tool_calls": result.tool_calls.iter().map(|tc| json!({
                "name": tc.tool, "arguments": tc.args, "result": tc.result,
            })).collect::<Vec<_>>(),
        }),
        StreamEvent::Cancelled { result } => json!({
            "type": "cancelled",
            "output": result.output,
            "turns": result.turns,
            "tool_calls": result.tool_calls.iter().map(|tc| json!({
                "name": tc.tool, "arguments": tc.args, "result": tc.result,
            })).collect::<Vec<_>>(),
        }),
        StreamEvent::Error { message } => json!({"type": "error", "message": message}),
    }
}

// ── App Harness endpoints (Design 005) ──────────────────────────────────────
//
// Merged view: for each kind (roles/skills/modes), show OS-level harnesses
// (with `selected` flag from the app's reference list) + app-level custom
// harnesses (scanned from apps/musk/harness/<kind>/).

/// The app-level harness dir: `~/.config/autoos/apps/musk/harness/<kind>/`.
///
/// Plan 021 C2: honors the `AUTOOS_HOME` env var when set, so tests can redirect
/// harness I/O away from the real `~/.config/autoos`. Precedence:
/// `AUTOOS_HOME` env > `~/.config/autoos`. Default (no env) unchanged.
pub(crate) fn app_harness_dir(kind: &str) -> Option<std::path::PathBuf> {
    if let Ok(custom) = std::env::var("AUTOOS_HOME") {
        if !custom.is_empty() {
            return Some(std::path::PathBuf::from(custom).join(format!("apps/musk/harness/{kind}")));
        }
    }
    dirs::home_dir().map(|h| h.join(format!(".config/autoos/apps/musk/harness/{kind}")))
}
/// Scan `dir` for app-level custom roles (*.at files).
pub(crate) fn scan_app_roles(dir: &std::path::Path) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("at") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = auto_ai_agent::parse_at_role(&content) {
                    out.push(json!({
                        "name": cfg.name.unwrap_or_else(|| {
                            path.file_stem().and_then(|s| s.to_str()).unwrap_or("?").to_string()
                        }),
                        "description": cfg.description.unwrap_or_default(),
                        "tier": cfg.model_tier.map(|t| format!("{:?}", t).to_lowercase()).unwrap_or("mid".into()),
                        "is_builtin": false,
                    }));
                }
            }
        }
    }
    out
}

/// Scan `dir` for app-level custom skills (<name>/SKILL.md).
pub(crate) fn scan_app_skills(dir: &std::path::Path) -> Vec<serde_json::Value> {
    let reg = auto_ai_agent::SkillRegistry::scan(dir);
    reg.descriptions().iter().map(|(name, desc)| {
        json!({ "name": name, "description": desc, "is_builtin": false })
    }).collect()
}

/// Scan `dir` for app-level custom modes (*.at files).
pub(crate) fn scan_app_modes(dir: &std::path::Path) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("at") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mode) = crate::mode::parse_mode_at(&content) {
                    out.push(json!({
                        "name": mode.name,
                        "description": mode.description,
                        "is_builtin": false,
                    }));
                }
            }
        }
    }
    out
}
// ── Chats endpoints (Plan 008) ──────────────────────────────────────────────
/// `GET /api/chats/session/{id}/stream` — run the last queued user message as
/// an agent turn, streaming SSE events (delta/tool/done/error). On completion,
/// the assistant reply (+ tool calls) is persisted to the session.
///
/// The agent is rebuilt from the session's mode and pre-loaded with the
/// conversation history (all prior user/assistant turns), so it continues the
/// multi-turn context across the stateless HTTP boundary.
async fn chat_stream(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    use axum::body::Body;
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    // Load the session + its history.
    let session = match ws.chats.get(&id) {
        Some(s) => s,
        None => {
            return (StatusCode::NOT_FOUND, format!("session '{id}' not found")).into_response()
        }
    };
    let mode = session.mode.clone();

    // The user message to run = the last user turn in history.
    let user_msg = match session.messages.iter().rev().find(|m| m.role == crate::chats::Role::User) {
        Some(m) => m.content.clone(),
        None => {
            return (StatusCode::BAD_REQUEST, "no user message to run").into_response();
        }
    };

    // Build (role, content) history pairs for prior turns (exclude the last
    // user message — that's the one we're about to run).
    let mut history: Vec<(String, String)> = Vec::new();
    let mut seen_last_user = false;
    for m in session.messages.iter().rev() {
        if !seen_last_user && m.role == crate::chats::Role::User {
            seen_last_user = true;
            continue; // skip the message we're running now
        }
        let role = match m.role {
            crate::chats::Role::User => "user",
            crate::chats::Role::Assistant => "assistant",
            crate::chats::Role::Tool => continue, // tool observations aren't plain turns
        };
        history.push((role.to_string(), m.content.clone()));
    }
    history.reverse(); // chronological order for the agent

    // Spawn the agent run, streaming events.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<serde_json::Value>(64);
    let client = state.client.clone();
    let chats = ws.chats.clone();
    let conversations = ws.conversations.clone();
    let session_id = id.clone();
    let history_for_agent = history.clone();
    let ws_root = ws.root.clone();
    let ws_id_for_ctx = q.id_or_default(&state.registry);
    let state_for_ctx = Arc::new(state.clone());
    // Resolve the session's mode to an AgentMode (built-in or user .at).
    let mode_reg = crate::mode::ModeRegistry::load();
    let agent_mode = match mode_reg.get(&mode).cloned() {
        Some(m) => m,
        None => mode_reg.get("superpowers").cloned().unwrap_or_else(|| {
            // Fallback: a minimal superpowers-like mode if the registry is empty.
            crate::mode::AgentMode {
                name: "superpowers".into(),
                description: String::new(),
                role: "coder".into(),
                skills: true,
                tools: vec![],
                workflow: None,
                context_file: String::new(),
                extra_system_prompt: String::new(),
            }
        }),
    };
    tokio::spawn(async move {
        // Confine this task's file-tool operations to the workspace root.
        crate::tool_safety::set_current_root(ws_root.clone());
        // Build agent with orchestration tool context (spawn_relay, dispatch).
        let tool_ctx = crate::tool_context::ToolContext {
            state: state_for_ctx.clone(),
            workspace_id: ws_id_for_ctx.clone(),
            parent_conversation_id: session_id.clone(),
        };
        let mut agent = match crate::build_agent_with_context(&agent_mode, client, Some(tool_ctx)) {
            Ok(a) => a,
            Err(e) => {
                crate::tool_safety::clear_current_root();
                let _ = tx.try_send(json!({"type": "error", "message": format!("build agent: {e}")}));
                return;
            }
        };
        // Pre-load the conversation history so the agent has context.
        agent = agent.with_history(history_for_agent);

        // Accumulate the streamed text + tool calls to persist on completion.
        let accumulated = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let tool_calls: std::sync::Arc<std::sync::Mutex<Vec<crate::chats::ToolCall>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        // tx is moved into the on_event closure; keep a clone for the error path.
        let tx_err = tx.clone();
        let acc2 = accumulated.clone();
        let tc2 = tool_calls.clone();
        // Tool-call id pairing: ToolStart has no shared id with Tool, so we hand
        // out a sequential id on ToolStart and re-use it for the matching Tool.
        // Start/result are strictly nested (a tool always finishes before the
        // next begins in the current agent loop), so a simple stack suffices.
        let tc_counter = std::sync::Arc::new(std::sync::Mutex::new(0usize));
        let tc_stack: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let on_event: Arc<dyn Fn(auto_ai_agent::StreamEvent) + Send + Sync> =
            Arc::new(move |ev| {
                // Assign / reuse an id for tool start/result pairing.
                use auto_ai_agent::StreamEvent;
                let id = match &ev {
                    StreamEvent::ToolStart { .. } => {
                        let n = { let mut c = tc_counter.lock().unwrap(); *c += 1; *c };
                        let id = format!("tc-{n}");
                        tc_stack.lock().unwrap().push(id.clone());
                        Some(id)
                    }
                    StreamEvent::Tool { .. } => tc_stack.lock().unwrap().pop(),
                    _ => None,
                };
                let value = stream_event_to_json(&ev, id.as_deref());
                // capture for persistence
                if let Some(text) = value.get("text").and_then(|t| t.as_str()) {
                    acc2.lock().unwrap().push_str(text);
                }
                if value.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                    let tool = value.get("name").and_then(|t| t.as_str()).unwrap_or("").to_string();
                    let args = value.get("arguments").cloned().unwrap_or(json!(null));
                    let result = value.get("result").and_then(|t| t.as_str()).unwrap_or("").to_string();
                    tc2.lock().unwrap().push(crate::chats::ToolCall {
                        tool, args, result,
                        status: String::from("success"),
                        id: id.unwrap_or_default(),
                    });
                }
                let _ = tx.try_send(value);
            });
        // No cancellation endpoint yet — the run flag is never set.
        let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
        match agent.run_stream(&user_msg, on_event, cancel).await {
            Ok(_) => {
                // Persist the assistant reply + tool calls.
                let text = std::mem::take(&mut *accumulated.lock().unwrap());
                let tcs = std::mem::take(&mut *tool_calls.lock().unwrap());
                let mut msg = crate::chats::ChatMessage::assistant(text);
                msg.tool_calls = tcs;
                let _ = chats.append_message(&session_id, msg.clone());
                // Dual-write: mirror the assistant message (+ tool calls) into
                // the conversation as turns.
                let seq_base = conversations
                    .get(&session_id)
                    .map(|c| c.turns.len())
                    .unwrap_or(0);
                for turn in
                    crate::conversation::chat_message_to_turns(&msg, seq_base)
                {
                    let _ = conversations.append_turn(&session_id, turn);
                }
                crate::tool_safety::clear_current_root();
            }
            Err(e) => {
                crate::tool_safety::clear_current_root();
                let _ = tx_err.try_send(json!({"type": "error", "message": format!("{e}")}));
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

// ── Spec-change approval endpoints (Plan 009 P1b) ──────────────────────────
// ── Workspace management endpoints ──────────────────────────────────────────
// ── Conversation endpoints (unified chat + flow) ────────────────────────────
/// `GET /api/conversations/{id}/stream?workspace=<id>` — SSE stream of
/// conversation events (appended turns + status changes). Events from other
/// conversations are filtered out client-side here.
async fn conversation_stream(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Query(q): Query<WorkspaceQuery>,
) -> Response {
    use axum::response::sse::{Event, KeepAlive, Sse};
    use std::convert::Infallible;
    use std::time::Duration;
    use tokio_stream::StreamExt;

    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let rx = ws.conversations.subscribe();

    let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .filter_map(move |res| match res {
            Ok(ev) if ev.conversation_id == id => Some(ev),
            _ => None,
        })
        .map(|ev| {
            Ok::<_, Infallible>(
                Event::default()
                    .event("conversation_event")
                    .json_data(serde_json::json!({
                        "conversation_id": ev.conversation_id,
                        "turn": ev.turn,
                        "status": ev.status,
                    }))
                    .unwrap_or_else(|_| Event::default()),
            )
        });

    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
        .into_response()
}

// ── Workflow endpoints ─────────────────────────────────────────────────────

/// `POST /api/workflow/run` request.
#[derive(Debug, Deserialize)]
pub struct WorkflowRunRequest {
    /// The task / user request (seeds `$user_request`).
    pub task: String,
    /// Built-in workflow name (e.g. "feature-dev").
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
    Query(q): Query<WorkspaceQuery>,
    Json(req): Json<WorkflowRunRequest>,
) -> Result<Json<WorkflowRunResponse>, (StatusCode, Json<ApiError>)> {
    crate::relay::feature_dev::require_builtin(&req.workflow).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: format!("invalid workflow '{}': {e}", req.workflow),
            }),
        )
    })?;

    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let result = crate::relay::feature_dev::run(&state, &ws, &req.task).await;

    result
        .map(|r| {
            Json(WorkflowRunResponse {
                steps: r.steps,
                outputs: r.outputs,
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
/// - `{"type":"step_start","step_id":"architect","role":"architect","input":"…"}`
/// - `{"type":"step_done","step_id":"architect","output":"…"}`
/// - `{"type":"step_skipped","step_id":"reviewer"}`
/// - `{"type":"finished",…}`
async fn workflow_run_stream(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Json(req): Json<WorkflowRunRequest>,
) -> axum::response::Response {
    use axum::body::Body;
    use axum::response::Response;
    use tokio::sync::mpsc;

    if let Err(e) = crate::relay::feature_dev::require_builtin(&req.workflow) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: format!("invalid workflow '{}': {e}", req.workflow),
            }),
        )
            .into_response();
    }

    let (tx, mut rx) = mpsc::channel::<serde_json::Value>(64);

    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let ws_root = ws.root.clone();
    let state = state.clone();
    let task = req.task.clone();
    tokio::spawn(async move {
        // Confine this task's file-tool operations to the workspace root.
        crate::tool_safety::set_current_root(ws_root);
        let on_event: Arc<dyn Fn(crate::relay::feature_dev::WorkflowStreamEvent) + Send + Sync> =
            Arc::new(move |ev| {
                let _ = tx.try_send(
                    serde_json::to_value(&ev).unwrap_or(serde_json::Value::Null),
                );
            });
        if let Err(e) =
            crate::relay::feature_dev::run_stream(&state, &ws, &task, on_event).await
        {
            // A top-level failure (e.g. agent build error) lands here — the
            // event stream carries whatever step events ran before it.
            tracing::error!("workflow stream failed: {e}");
        }
        crate::tool_safety::clear_current_root();
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

/// Resolve a role spec: built-in name, else `.at` file path.
fn resolve(spec: &str) -> Result<Arc<dyn Role>, String> {
    if let Some(p) = load_builtin(spec) {
        return Ok(p);
    }
    let content = std::fs::read_to_string(spec)
        .map_err(|e| format!("not a builtin, cannot read '{spec}': {e}"))?;
    load_role(&content).map_err(|e| format!("parse '{spec}': {e}"))
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

    fn tmp_auth() -> Arc<crate::auto_generated::auth::AuthStore> {
        // 唯一路径(时间戳 + 自增),避免并行测试共用同一 users.json 竞态。
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "musk_server_auth_test_{}_{}.json",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_file(&path);
        Arc::new(crate::auto_generated::auth::AuthStore::new(path))
    }

    fn tmp_state() -> AppState {
        let dir = std::env::temp_dir().join(format!(
            "musk-server-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let registry =
            crate::workspace::WorkspaceRegistry::load(dir.join("workspaces.json"), dir.clone());
        AppState {
            client: Arc::new(MockClient) as Arc<dyn Client>,
            auth: tmp_auth(),
            registry: Arc::new(registry),
        }
    }

    /// 接线运行(计划 018 §11 ①):真实 HTTP 请求打到 auth 端点,数据层是 a2r
    /// 转译的 ag::AuthStore(真 sha2 哈希 + 文件持久化 + Mutex sessions)。
    /// 断言 wire 形状与手写版一致:{ token, user: { username, role } }。
    #[tokio::test]
    async fn auth_endpoints_run_on_transpiled_store() {
        use axum::body::Body;
        use crate::auto_generated::server as ag_server;
        use tower::ServiceExt;

        let app = axum::Router::new()
            .route("/api/auth/login", axum::routing::post(ag_server::auth_login))
            .route("/api/auth/me", axum::routing::get(ag_server::auth_me))
            .route("/api/auth/logout", axum::routing::post(ag_server::auth_logout))
            .with_state(tmp_state());

        // login with the default admin (created by ensure_default_admin).
        let resp = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"username":"admin","password":"admin"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let token = json["token"].as_str().expect("login returns token").to_string();
        assert_eq!(json["user"]["username"], "admin");
        assert_eq!(json["user"]["role"], "Admin");

        // Bad credentials → 401 (C3 状态码模型,与手写版一致)。
        let resp = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"username":"admin","password":"wrong"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);

        // me with the bearer token resolves the user from the transpiled store.
        let resp = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/auth/me")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["role"], "Admin");

        // logout invalidates the session → me is 401 afterwards.
        let resp = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/auth/logout")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/auth/me")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn run_endpoint_returns_result() {
        let state = tmp_state();
        let req = RunRequest {
            task: "say hello".into(),
            mode: "superpowers".into(),
        };
        let resp = run_inner(state, req).await.unwrap();
        assert_eq!(resp.output, "mock answer");
        assert_eq!(resp.turns, 1);
    }

    /// C1 重新评估 PoC(plan 018 §11 ②):extern_impl 的 specs_load 走
    /// AppState.registry 的真实 workspace stores(hw SpecsStore),返回真实 doc
    /// —— 证明"换 store 类型(41 处级联)"非必需,委托路径可行。
    #[test]
    fn specs_extern_delegation_returns_real_doc() {
        let state = tmp_state();
        // Seed a spec item into the default workspace's real (hw) specs store.
        let ws = state.registry.get("");
        let mut doc = ws.specs.load().unwrap();
        ws.specs
            .upsert_item(&mut doc, "goals", crate::specs::SpecItem::new("G1", "goal"))
            .unwrap();
        ws.specs.save(&doc).unwrap();

        let state_wrapper = axum::extract::State(state);
        let q = axum::extract::Query(crate::auto_generated::server::WorkspaceQuery {
            workspace: None,
        });
        let value = crate::auto_generated::extern_impl::specs_load(&state_wrapper, q);

        // The delegated value is the real doc (same wire shape as hw
        // specs_list's Json(doc)).
        let doc: crate::specs::SpecsDocument = serde_json::from_value(value).unwrap();
        let goals = doc.sections.iter().find(|s| s.id == "goals").unwrap();
        assert_eq!(goals.items.len(), 1);
        assert_eq!(goals.items[0].id, "G1");
        assert_eq!(goals.items[0].title, "goal");
    }

    /// ② 接线验收:specs/chats 端点由转译 handler(经 extern_impl 委托到真实
    /// workspace stores)服务 —— 与 hw 手写版行为一致的真实 CRUD。
    #[tokio::test]
    async fn specs_chats_endpoints_run_on_transpiled_handlers() {
        use axum::body::Body;
        use crate::auto_generated::server as ag_server;
        use tower::ServiceExt;

        let app = axum::Router::new()
            .route("/api/specs", axum::routing::get(ag_server::specs_list))
            .route("/api/specs/item", axum::routing::post(ag_server::specs_upsert))
            .route("/api/specs/transition", axum::routing::post(ag_server::specs_transition))
            .route(
                "/api/specs/item/{section}/{id}",
                axum::routing::delete(ag_server::specs_delete),
            )
            .route("/api/specs/overview", axum::routing::get(ag_server::specs_overview))
            .route("/api/chats/sessions", axum::routing::get(ag_server::chat_list))
            .route("/api/chats/session", axum::routing::post(ag_server::chat_create))
            .route(
                "/api/chats/session/{id}",
                axum::routing::get(ag_server::chat_get)
                    .patch(ag_server::chat_rename)
                    .delete(ag_server::chat_delete),
            )
            .route("/api/chats/session/{id}/message", axum::routing::post(ag_server::chat_message))
            .with_state(tmp_state());

        // ── specs: upsert → 真实 doc 持久化,list 可读回 ──
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/specs/item")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"section":"goals","item":{"id":"G1","title":"goal","content":"body","status":"empty"}}"#,
                ))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let doc: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(doc["sections"][0]["items"][0]["id"], "G1");

        // GET /api/specs returns the persisted doc (via specs_load 委托)。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/specs")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let doc: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(doc["sections"][0]["items"][0]["id"], "G1");

        // specs transition: Empty → Proposed(Goals 合法)真实落库。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/specs/transition")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"section":"goals","item_id":"G1","new_status":"proposed"}"#))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["new_status"], "proposed");

        // specs overview 返回各 section 的聚合。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/specs/overview")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["sections"].as_array().map(|a| a.len()).unwrap_or(0), 7);

        // ── chats: create → {"session": {...}},list/get 可读回 ──
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/chats/session")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"mode":"superpowers"}"#))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let session_id = v["session"]["id"].as_str().expect("create returns session").to_string();
        assert_eq!(v["session"]["mode"], "superpowers");

        // append message → 会话消息数 +1。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/chats/session/{session_id}/message"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"content":"List the files"}"#))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["session"]["messages"].as_array().map(|a| a.len()).unwrap_or(0), 1);
        assert_eq!(v["queued"]["content"], "List the files");

        // list 返回该 session 的 summary。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/chats/sessions")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let sessions = v["sessions"].as_array().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0]["id"], session_id);

        // get 返回完整 session。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/chats/session/{session_id}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["session"]["id"], session_id);

        // rename + delete。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!("/api/chats/session/{session_id}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"renamed"}"#))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["session"]["name"], "renamed");

        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!("/api/chats/session/{session_id}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["status"], "deleted");
        // 复审 A1:删除响应返回真实 id(此前 path_inner 空串 → id="")。
        assert_eq!(v["id"], session_id, "delete response carries the real session id");
    }

    /// ② workspace 路由接线验收:workspace list/open/status/browse/initialize
    /// 由转译 handler 服务(经 extern_impl 委托到真实 registry),wire 与 hw 一致。
    #[tokio::test]
    async fn workspace_endpoints_run_on_transpiled_handlers() {
        use axum::body::Body;
        use crate::auto_generated::server as ag_server;
        use tower::ServiceExt;

        let state = tmp_state();
        let app = axum::Router::new()
            .route("/api/workspace/list", axum::routing::get(ag_server::workspace_list))
            .route("/api/workspace/open", axum::routing::post(ag_server::workspace_open))
            .route("/api/workspace/status", axum::routing::get(ag_server::workspace_status))
            .route("/api/workspace/browse", axum::routing::get(ag_server::workspace_browse))
            .route("/api/workspace/initialize", axum::routing::post(ag_server::workspace_initialize))
            .with_state(state);

        // list → 默认 workspace 已 seed,含完整 meta(last_opened/is_empty)。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/workspace/list")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let wss = v["workspaces"].as_array().unwrap();
        assert_eq!(wss.len(), 1);
        assert!(wss[0]["id"].is_string());
        assert!(wss[0]["path"].is_string());
        let default_id = wss[0]["id"].as_str().unwrap().to_string();

        // open 一个已有路径 → {"workspace": {meta}}。
        let open_path = std::env::temp_dir().join("musk-open-test").to_string_lossy().to_string();
        std::fs::create_dir_all(&open_path).unwrap();
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/workspace/open")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "path": open_path }).to_string()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(v["workspace"]["id"].is_string());
        // registry.open 会 canonicalize(Windows 上有 \\?\ 前缀),按 canonical 比较。
        let canonical = std::fs::canonicalize(&open_path).unwrap();
        assert_eq!(v["workspace"]["path"], canonical.to_string_lossy().to_string());

        // status → {"workspace", "root_exists"}。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/workspace/status")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["workspace"]["id"], default_id);
        assert!(v["root_exists"].is_boolean());

        // browse → {"entries", "parent"}(默认 workspace 根里有 .autoos)。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/workspace/browse")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(v["entries"].is_array());
        assert!(v["parent"].is_null() || v["parent"].is_string());

        // initialize → 写 .autoos/initialized 标记。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/workspace/initialize")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["status"], "initialized");
        assert_eq!(v["workspace"], default_id);

        let _ = std::fs::remove_dir_all(&open_path);
    }

    /// ③ config 页接线验收:professions/config/modes/skills/roles 读端点由转译
    /// handler 服务(经 extern_impl 委托到真实 registry),wire 与 hw 一致。
    /// (role save/delete 会写用户真实 ~/.config/autoos/roles —— 不在此测。)
    #[tokio::test]
    async fn config_endpoints_run_on_transpiled_handlers() {
        use axum::body::Body;
        use crate::auto_generated::server as ag_server;
        use tower::ServiceExt;

        let app = axum::Router::new()
            .route("/api/professions", axum::routing::get(ag_server::professions))
            .route("/api/config", axum::routing::get(ag_server::config_overview))
            .route("/api/modes", axum::routing::get(ag_server::modes_list))
            .route("/api/skills", axum::routing::get(ag_server::skills_list))
            .route("/api/roles", axum::routing::get(ag_server::roles_list))
            .with_state(tmp_state());

        let get = |uri: String| {
            let app = app.clone();
            async move {
                let resp = app
                    .oneshot(
                        axum::http::Request::builder()
                            .method("GET")
                            .uri(uri)
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(resp.status(), 200, "GET should be 200");
                let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
                serde_json::from_slice::<serde_json::Value>(&body).unwrap()
            }
        };

        // professions:builtin 有数据,每项含 name/tier/model。
        let v = get("/api/professions".to_string()).await;
        let profs = v["professions"].as_array().unwrap();
        assert!(!profs.is_empty(), "builtin professions expected");
        assert!(profs[0]["name"].is_string());
        assert!(profs[0]["tier"].is_string());

        // modes:数组形状(空与否取决于用户配置)。
        let v = get("/api/modes".to_string()).await;
        assert!(v["modes"].is_array());

        // skills:数组形状。
        let v = get("/api/skills".to_string()).await;
        assert!(v["skills"].is_array());

        // config:三个键齐全。
        let v = get("/api/config".to_string()).await;
        assert!(v["modes"].is_array());
        assert!(v["professions"].is_array());
        assert!(v["skills"].is_array());

        // roles:builtin 有数据,每项含 name/tier/is_builtin。
        let v = get("/api/roles".to_string()).await;
        let roles = v["roles"].as_array().unwrap();
        assert!(!roles.is_empty(), "builtin roles expected");
        assert!(roles[0]["name"].is_string());
    }

    /// ③ conversations 接线验收:list/get/rename/delete 由转译 handler 服务
    /// (经 extern_impl 委托到 workspace ConversationStore)。
    #[tokio::test]
    async fn conversations_endpoints_run_on_transpiled_handlers() {
        use axum::body::Body;
        use crate::auto_generated::server as ag_server;
        use tower::ServiceExt;

        let app = axum::Router::new()
            .route("/api/chats/session", axum::routing::post(ag_server::chat_create))
            .route("/api/conversations", axum::routing::get(ag_server::conversation_list))
            .route(
                "/api/conversations/{id}",
                axum::routing::get(ag_server::conversation_get).delete(ag_server::conversation_delete),
            )
            .route("/api/conversations/{id}/title", axum::routing::patch(ag_server::conversation_rename))
            .with_state(tmp_state());

        // 建 chat session(双写 conversation,id 相同)。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/chats/session")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"mode":"superpowers"}"#))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let session_id = v["session"]["id"].as_str().unwrap().to_string();

        // list → 该 conversation 出现。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/conversations")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let convs = v["conversations"].as_array().unwrap();
        assert_eq!(convs.len(), 1);
        assert_eq!(convs[0]["id"], session_id);

        // get → 完整 conversation。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/conversations/{session_id}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["id"], session_id);

        // rename → 标题更新。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!("/api/conversations/{session_id}/title"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"renamed"}"#))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["title"], "renamed");

        // delete → 删除。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!("/api/conversations/{session_id}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["status"], "deleted");
        // 复审 A1:删除响应返回真实 id(此前 path_inner 空串 → id="")。
        assert_eq!(v["id"], session_id, "delete response carries the real conversation id");
    }

    /// ③ app-config + harness 接线验收(只测读端点;save 会写用户真实
    /// ~/.config/autoos 配置,不在此测)。
    #[tokio::test]
    async fn app_config_endpoints_run_on_transpiled_handlers() {
        use axum::body::Body;
        use crate::auto_generated::server as ag_server;
        use tower::ServiceExt;

        let app = axum::Router::new()
            .route("/api/app-config", axum::routing::get(ag_server::app_config_get))
            .route("/api/app-harness/{kind}", axum::routing::get(ag_server::app_harness_list))
            .with_state(tmp_state());

        // app-config → {stored, effective} 形状。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/app-config")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(v["stored"].is_object(), "stored config expected");
        assert!(v["effective"].is_object(), "effective config expected");

        // harness list (roles kind) → {os_available, app_custom} 形状。
        let resp = app.clone().oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/app-harness/roles")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(v["os_available"].is_array(), "os_available expected");
        assert!(v["app_custom"].is_array(), "app_custom expected");
    }

    #[tokio::test]
    async fn run_endpoint_bad_profession_errors() {
        let state = tmp_state();
        let req = RunRequest {
            task: "x".into(),
            mode: "nonexistent-mode".into(),
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

    // ── Plan 017: feature-dev on PipelineEngine (replaces deprecated Workflow) ──

    /// The feature-dev runner drives all four steps through the engine; the
    /// mock client answers each turn, so the reviewer condition is satisfied
    /// (tester produced a non-empty report) and no step is skipped.
    #[tokio::test]
    async fn workflow_run_end_to_end_runs_four_steps() {
        let state = tmp_state();
        let ws = state.registry.get("default");
        let result =
            crate::relay::feature_dev::run(&state, &ws, "implement binary search").await.unwrap();
        assert_eq!(result.steps.len(), 4);
        assert_eq!(result.steps.get("architect").unwrap(), "mock answer");
        assert_eq!(result.steps.get("reviewer").unwrap(), "mock answer");
        assert_eq!(result.outputs.get("design").unwrap(), "mock answer");
        assert_eq!(result.outputs.get("review").unwrap(), "mock answer");
        assert_eq!(result.outputs.len(), 4);
        // Token accounting is driven by client-reported usage; the mock
        // reports none, so the total stays 0 (nothing to assert there).
    }

    /// The streaming variant emits the old step-level SSE shapes: four
    /// step_start/step_done pairs, no skips, then a finished event.
    #[tokio::test]
    async fn workflow_run_stream_emits_step_events() {
        use crate::relay::feature_dev::WorkflowStreamEvent;
        let state = tmp_state();
        let ws = state.registry.get("default");
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = events.clone();
        let on_event: Arc<dyn Fn(WorkflowStreamEvent) + Send + Sync> =
            Arc::new(move |ev| sink.lock().unwrap().push(ev));
        let _ = crate::relay::feature_dev::run_stream(&state, &ws, "task", on_event)
            .await
            .unwrap();
        let got = events.lock().unwrap();
        let starts = got
            .iter()
            .filter(|e| matches!(e, WorkflowStreamEvent::StepStart { .. }))
            .count();
        let dones = got
            .iter()
            .filter(|e| matches!(e, WorkflowStreamEvent::StepDone { .. }))
            .count();
        let skips = got
            .iter()
            .filter(|e| matches!(e, WorkflowStreamEvent::StepSkipped { .. }))
            .count();
        assert_eq!(starts, 4, "expected 4 step_start events: {got:?}");
        assert_eq!(dones, 4, "expected 4 step_done events: {got:?}");
        assert_eq!(skips, 0, "no step should be skipped with a live tester");
        assert!(
            got.iter().any(|e| matches!(e, WorkflowStreamEvent::Finished { .. })),
            "missing finished event: {got:?}"
        );
    }

    /// Unknown workflow specs are rejected before any agent runs (the handler
    /// validates via require_builtin; the runner itself drives the built-in
    /// flow unconditionally).
    #[test]
    fn workflow_spec_validation() {
        assert!(crate::relay::feature_dev::require_builtin("not-a-workflow").is_err());
        assert!(crate::relay::feature_dev::require_builtin("feature-dev").is_ok());
        // Custom .at paths are retired with the old workflow parser.
        assert!(crate::relay::feature_dev::require_builtin("workflows/custom.at").is_err());
    }

    /// Phase C (计划 018 §11 C2):a2r 转译的 auth handler 栈经真实委托
    /// (ag::AuthStore) 产生真实行为 —— login 返回真实 session token,
    /// me 解析真实用户,logout 使 session 失效。wire 形状与手写版一致
    /// ({token, user:{username, role}} / me→{username, role})。
    /// 注:ag handler 无 HTTP 状态码模型(§11 C3 边界),无效凭据返回空数据
    /// 而非 401 —— 由手写版保持 401,生产接线待 error-status 模型。
    #[tokio::test]
    async fn ag_auth_handlers_produce_real_behavior() {
        use axum::body::Body;
        use crate::auto_generated::server as ag_server;
        use tower::ServiceExt;

        let app = axum::Router::new()
            .route("/api/auth/login", axum::routing::post(ag_server::auth_login))
            .route("/api/auth/me", axum::routing::get(ag_server::auth_me))
            .route("/api/auth/logout", axum::routing::post(ag_server::auth_logout))
            .with_state(tmp_state());

        // login: real session token + real user (default admin).
        let resp = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"username":"admin","password":"admin"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let token = json["token"].as_str().expect("real token").to_string();
        assert!(!token.is_empty());
        assert_eq!(json["user"]["username"], "admin");
        assert_eq!(json["user"]["role"], "Admin");

        // me: resolves the real user from the token.
        let resp = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/auth/me")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["username"], "admin");
        assert_eq!(json["role"], "Admin");

        // logout: session invalidated → me returns 401 (C3 状态码模型)。
        let resp = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/auth/logout")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/auth/me")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    /// ④ 整体接入(plan 018 §11):真实 serve() 的 router 组合 —— ag build_router
    /// (38 路由) 作为主 router + 🔴 流式/daemon 路由 + relay/task_plan/wiki 合并。
    /// Plan 019 Phase 2-4:6 个 daemon/SSE 路由全部由 ag server_stream handler
    /// 服务(与 serve() 一致)。axum 在构造期对重复路由 panic —— 本测试保证
    /// 组合无冲突,且转译端点实际可服务。
    #[tokio::test]
    async fn production_router_composition_serves_core_endpoints() {
        use axum::body::Body;
        use tower::ServiceExt;
        use crate::auto_generated::server_stream as ag;

        let app = crate::auto_generated::server::build_router()
            .route("/api/run", axum::routing::post(ag::run))
            .route("/api/run/stream", axum::routing::post(ag::run_stream_handler))
            .route("/api/workflow/run", axum::routing::post(ag::workflow_run))
            .route("/api/workflow/run/stream", axum::routing::post(ag::workflow_run_stream))
            .route("/api/settings-link", axum::routing::post(ag::settings_link))
            .route("/api/chats/session/{id}/stream", axum::routing::get(ag::chat_stream))
            .route("/api/conversations/{id}/stream", axum::routing::get(ag::conversation_stream))
            // Plan 021 Phase A:/api/files 现由 build_router() 内的 ag workspace_file
            // 服务(不再单独 .route)。
            // Plan 020 Phase F:relay/task_plan/wiki 合并全部切到 ag handler
            // (与 serve() 一致)。
            .merge(crate::auto_generated::relay_api::relay_routes())
            .merge(crate::auto_generated::relay_api::task_plan_routes())
            .merge(crate::auto_generated::wiki::wiki_routes())
            .with_state(tmp_state());

        // 转译端点由主 router 服务:health + workflows(ag workflows → 与 hw 同形状)。
        let resp = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);

        let resp = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/workflows")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["workflows"][0], "feature-dev");

        // specs delete 路由(serve() 原有、④ 时补进 ag build_router)可被命中。
        // 复审 A3:不存在的 item → 404(此前错误语义回归返回 200+空 id)。
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("DELETE")
                    .uri("/api/specs/item/no-such-section/no-such-item")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            axum::http::StatusCode::NOT_FOUND,
            "deleting a non-existent spec item returns 404 (A3 error semantics)"
        );
    }

    /// 复审 A3:委托端点错误语义 —— 不存在/失败返回 4xx(此前错误回归为 200+null)。
    #[tokio::test]
    async fn delegated_endpoints_return_http_errors() {
        use axum::body::Body;
        use crate::auto_generated::server as ag_server;
        use tower::ServiceExt;

        let app = axum::Router::new()
            .route(
                "/api/chats/session/{id}",
                axum::routing::get(ag_server::chat_get).delete(ag_server::chat_delete),
            )
            .route(
                "/api/specs/item/{section}/{id}",
                axum::routing::delete(ag_server::specs_delete),
            )
            .route("/api/workspace/status", axum::routing::get(ag_server::workspace_status))
            .with_state(tmp_state());

        // 不存在的 chat:get/delete → 404(hw 语义;此前 200 null / 200 空 id)。
        for (method, uri) in [
            ("GET", "/api/chats/session/no-such"),
            ("DELETE", "/api/chats/session/no-such"),
        ] {
            let resp = app
                .clone()
                .oneshot(
                    axum::http::Request::builder()
                        .method(method)
                        .uri(uri)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(resp.status(), axum::http::StatusCode::NOT_FOUND, "{method} {uri}");
        }

        // 不存在的 workspace → status 404。
        let resp = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/workspace/status?workspace=no-such")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::NOT_FOUND);
    }

    // ── Plan 019 Phase 0b: 流式/daemon handler 契约金标准 ───────────────────
    //
    // 这些测试锚定切换到 ag handler 前后都必须成立的 wire 形状契约。它们是
    // "行为等价金标准":先在 hw 行为上写绿,切换后继续跑,任何回归都会被抓到。
    //
    // 三类契约:
    //   (1) stream_event_to_json —— run/chat stream 的 SSE JSON 形状
    //       (字段名 name/arguments + tc-{n} id 配对 + done.turns + cancelled)
    //   (2) WorkflowStreamEvent 序列化 —— workflow stream 的 {"type":...} snake_case
    //   (3) sse_event —— ag 侧 SSE 帧格式必须无 event 行(前端 onmessage 才能收到)

    /// (1a) `stream_event_to_json` 的 ToolStart/Tool 用 `tc-{n}` id 配对,
    /// 字段名是 `name`/`arguments`(不是 `tool`/`args`)。前端按此渲染 tool 卡片。
    #[test]
    fn contract_stream_event_tool_pairing_uses_name_arguments_and_tc_id() {
        use auto_ai_agent::{StreamEvent, ToolCallRecord};
        // ToolStart → tool_call, id 由调用方分配( hw 用 tc_counter/tc_stack)。
        let start = StreamEvent::ToolStart {
            tool: "read_file".into(),
            args: json!({"path": "/tmp/x"}),
        };
        let v = stream_event_to_json(&start, Some("tc-1"));
        assert_eq!(v["type"], "tool_call", "ToolStart → type=tool_call");
        assert_eq!(v["id"], "tc-1", "id 透传 tc-N");
        assert_eq!(v["name"], "read_file", "字段名是 name(非 tool)");
        assert_eq!(v["arguments"]["path"], "/tmp/x", "字段名是 arguments(非 args)");

        // Tool → tool_result, 复用同一 id, 多了 result + status=success。
        let tool = StreamEvent::Tool {
            tool: "read_file".into(),
            args: json!({"path": "/tmp/x"}),
            result: "ok".into(),
        };
        let v = stream_event_to_json(&tool, Some("tc-1"));
        assert_eq!(v["type"], "tool_result", "Tool → type=tool_result");
        assert_eq!(v["id"], "tc-1", "result 复用 start 的 id");
        assert_eq!(v["name"], "read_file");
        assert_eq!(v["arguments"]["path"], "/tmp/x");
        assert_eq!(v["result"], "ok");
        assert_eq!(v["status"], "success");
    }

    /// (1b) Delta/Warning/Error 走 `type` 字段,无 id(只有 tool 事件配对)。
    #[test]
    fn contract_stream_event_text_variants_have_no_id() {
        use auto_ai_agent::StreamEvent;
        let delta = StreamEvent::Delta { text: "hi".into() };
        let v = stream_event_to_json(&delta, None);
        assert_eq!(v["type"], "delta");
        assert_eq!(v["text"], "hi");
        assert!(v.get("id").is_none(), "delta 无 id");

        let warn = StreamEvent::Warning { text: "cap".into() };
        let v = stream_event_to_json(&warn, None);
        assert_eq!(v["type"], "warning");
        assert_eq!(v["text"], "cap");

        let err = StreamEvent::Error { message: "boom".into() };
        let v = stream_event_to_json(&err, None);
        assert_eq!(v["type"], "error");
        assert_eq!(v["message"], "boom");
    }

    /// (1c) Done/Cancelled 携带 output + turns + tool_calls(每条用 name/arguments)。
    /// 这是 RunResponse.turns 的流式对应物 —— 前端靠 turns 显示迭代次数。
    #[test]
    fn contract_stream_event_done_carries_turns_and_tool_calls() {
        use auto_ai_agent::{AgentResult, StreamEvent, ToolCallRecord};
        let result = AgentResult {
            output: "answer".into(),
            turns: 3,
            tool_calls: vec![ToolCallRecord {
                tool: "read_file".into(),
                args: json!({"path": "/a"}),
                result: "r".into(),
            }],
            total_tokens: 0,
        };
        let done = StreamEvent::Done { result };
        let v = stream_event_to_json(&done, None);
        assert_eq!(v["type"], "done");
        assert_eq!(v["output"], "answer");
        assert_eq!(v["turns"], 3, "done 携带 turns(前端依赖)");
        assert_eq!(v["tool_calls"][0]["name"], "read_file");
        assert_eq!(v["tool_calls"][0]["arguments"]["path"], "/a");
        assert_eq!(v["tool_calls"][0]["result"], "r");

        // Cancelled 同形(除 type 外)。
        let cancelled = StreamEvent::Cancelled {
            result: AgentResult {
                output: "partial".into(),
                turns: 1,
                tool_calls: vec![],
                total_tokens: 0,
            },
        };
        let v = stream_event_to_json(&cancelled, None);
        assert_eq!(v["type"], "cancelled");
        assert_eq!(v["output"], "partial");
        assert_eq!(v["turns"], 1);
    }

    /// (2) WorkflowStreamEvent 序列化为 `{"type":"step_start",...}` snake_case
    /// —— 前端按 type 字段路由 workflow 事件。ag 的 WorkflowEventDto 必须对齐。
    #[test]
    fn contract_workflow_stream_event_serializes_to_snake_case_tag() {
        use crate::relay::feature_dev::WorkflowStreamEvent;
        let ev = WorkflowStreamEvent::StepStart {
            step_id: "architect".into(),
            role: "architect".into(),
            input: "task".into(),
        };
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["type"], "step_start", "serde tag=type, snake_case");
        assert_eq!(v["step_id"], "architect");
        assert_eq!(v["role"], "architect");
        assert_eq!(v["input"], "task");

        let ev = WorkflowStreamEvent::StepSkipped { step_id: "reviewer".into() };
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["type"], "step_skipped");

        let ev = WorkflowStreamEvent::Finished {
            steps: std::collections::HashMap::from([("architect".into(), "out".into())]),
            outputs: std::collections::HashMap::new(),
        };
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["type"], "finished");
        assert_eq!(v["steps"]["architect"], "out");
    }

    /// (3) **根因修复金标准** —— ag 的 sse_event 产出的帧必须无 `event:` 行。
    /// 前端只用 EventSource.onmessage;按 SSE 协议带 event 行的消息不进 onmessage。
    /// axum Event 经 Sse 包装序列化后,无 event 名的帧只有 `data: {json}\n\n`。
    #[tokio::test]
    async fn contract_sse_event_frame_has_no_event_line() {
        use crate::auto_generated::extern_impl::sse_event;
        use axum::response::sse::{KeepAlive, Sse};
        // sse_event 的第一个参数(name)现在被忽略 —— 无论传什么,帧都不含 event 行。
        let event = sse_event("run", json!({"type": "delta", "text": "hi"}));
        // 用含单个 event 的 stream 构造 Sse,经 IntoResponse 读 body 字节。
        let stream = async_stream::stream! { yield Ok::<_, std::convert::Infallible>(event) };
        let sse = Sse::new(stream).keep_alive(KeepAlive::new());
        let resp = sse.into_response();
        let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        let frame = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(
            !frame.contains("event:"),
            "sse_event 帧不得含 event: 行(否则前端 onmessage 收不到): {frame}"
        );
        assert!(
            frame.contains("data:"),
            "sse_event 帧必须有 data: 行: {frame}"
        );
    }

    /// (4a) HTTP 层契约:POST /api/workflow/run 成功 → 200 + application/json,
    /// body 含 steps/outputs(hw 金标准)。切换到 ag handler 后必须等价。
    #[tokio::test]
    async fn contract_workflow_run_http_returns_json() {
        use axum::body::Body;
        use tower::ServiceExt;
        let app = axum::Router::new()
            .route("/api/workflow/run", axum::routing::post(workflow_run))
            .with_state(tmp_state());
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/workflow/run")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"task":"implement binary search","workflow":"feature-dev"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            "application/json"
        );
        let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(v["steps"].is_object(), "workflow/run body 含 steps");
        assert!(v["outputs"].is_object(), "workflow/run body 含 outputs");
    }

    /// (4b) HTTP 层契约:POST /api/run 用未知 mode → 400 + 错误形状。
    /// 切换到 ag handler 后必须保持错误路径(或登记 KNOWN-DEBT)。
    #[tokio::test]
    async fn contract_run_http_bad_mode_returns_400() {
        use axum::body::Body;
        use tower::ServiceExt;
        let app = axum::Router::new()
            .route("/api/run", axum::routing::post(run))
            .with_state(tmp_state());
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/run")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"task":"x","mode":"no-such-mode"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(v["error"].is_string(), "错误 body 含 error 字段");
    }

    // ── Plan 019 Phase 1d: ag handler 等价性验收 ───────────────────────────
    //
    // 切换到 ag server_stream handler 后,这两条路由由转译 handler(经 extern_impl
    // 真实委托)服务。这里断言 ag handler 产出与 hw 等价的 wire 形状 ——
    // /api/workflow/run 的 steps/outputs + /api/run 的 output/turns/tool_calls。

    /// ag workflow_run(经 extern wf_run → feature_dev::run)产出与 hw 等价的
    /// steps/outputs(MultiClient 每个 step 答 "mock answer")。
    #[tokio::test]
    async fn ag_workflow_run_produces_real_steps_and_outputs() {
        use axum::body::Body;
        use tower::ServiceExt;
        use crate::auto_generated::server_stream as ag;
        let app = axum::Router::new()
            .route("/api/workflow/run", axum::routing::post(ag::workflow_run))
            .with_state(tmp_state());
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/workflow/run")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"task":"implement binary search","workflow":"feature-dev"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        // 与 hw workflow_run_end_to_end_runs_four_steps 同断言:4 step + outputs。
        assert_eq!(v["steps"].as_object().unwrap().len(), 4, "ag 产出 4 steps");
        assert_eq!(v["steps"]["architect"], "mock answer");
        assert_eq!(v["steps"]["reviewer"], "mock answer");
        assert_eq!(v["outputs"]["design"], "mock answer");
        assert!(v["outputs"].as_object().unwrap().len() >= 2, "ag 产出 outputs");
    }

    /// ag run(经 extern agent_run → build_agent + agent.run)产出与 hw 等价的
    /// output + turns + tool_calls。MockClient 答 "mock answer",无 tool 调用,
    /// 跑 1 turn。
    #[tokio::test]
    async fn ag_run_produces_output_turns_tool_calls() {
        use axum::body::Body;
        use tower::ServiceExt;
        use crate::auto_generated::server_stream as ag;
        let app = axum::Router::new()
            .route("/api/run", axum::routing::post(ag::run))
            .with_state(tmp_state());
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/run")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"task":"say hello","mode":"superpowers"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["output"], "mock answer", "ag run output");
        assert_eq!(v["turns"], 1, "ag run turns(与 hw run_endpoint_returns_result 一致)");
        assert!(v["tool_calls"].is_array(), "ag run tool_calls 是数组");
    }

    // ── Plan 019 Phase 2-4: ag 流式 handler + 状态码模型等价性验收 ──────────
    //
    // 切换后,4 个流式 handler 由 ag server_stream(经 extern side-table +
    // sink 桥接)服务;run/workflow_run 由 ~Response + 错误包络服务。这些测试
    // 断言 ag 与 hw 等价:流式产出的 SSE wire 形状 + 流终止 + 400/500 错误码。

    /// ag run(经 ~Response + 错误包络)未知 mode → 400(与 hw run_inner 等价;
    /// 此前是 200 + 空 RunResponse)。
    #[tokio::test]
    async fn ag_run_unknown_mode_returns_400() {
        use axum::body::Body;
        use tower::ServiceExt;
        use crate::auto_generated::server_stream as ag;
        let app = axum::Router::new()
            .route("/api/run", axum::routing::post(ag::run))
            .with_state(tmp_state());
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/run")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"task":"x","mode":"no-such-mode"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(v["error"].is_string(), "ag 错误 body 含 error 字段: {v}");
    }

    /// ag workflow_run 坏 workflow → 400(与 hw require_builtin 等价;此前 200+空)。
    #[tokio::test]
    async fn ag_workflow_run_invalid_workflow_returns_400() {
        use axum::body::Body;
        use tower::ServiceExt;
        use crate::auto_generated::server_stream as ag;
        let app = axum::Router::new()
            .route("/api/workflow/run", axum::routing::post(ag::workflow_run))
            .with_state(tmp_state());
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/workflow/run")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"task":"x","workflow":"bogus"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(v["error"].is_string(), "ag 错误 body 含 error 字段: {v}");
    }

    /// ag run_stream(经 side-table + extern 桥接)产出与 hw 等价的 SSE:
    /// delta/done 事件、name/arguments 字段、tc-{n} id 配对;run 结束后
    /// channel 关闭 → 流终止(body 有限,不挂起)。
    #[tokio::test]
    async fn ag_run_stream_produces_sse_events() {
        use axum::body::Body;
        use tower::ServiceExt;
        use crate::auto_generated::server_stream as ag;
        let app = axum::Router::new()
            .route("/api/run/stream", axum::routing::post(ag::run_stream_handler))
            .with_state(tmp_state());
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/run/stream")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"task":"say hello","mode":"superpowers"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
        let body = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            axum::body::to_bytes(resp.into_body(), 1 << 20),
        )
        .await
        .expect("ag run_stream 必须终止(close_channel 后 recv None → break)")
        .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("data: "), "SSE 有 data 帧: {text}");
        assert!(text.contains("\"type\":\"delta\""), "delta 事件: {text}");
        assert!(text.contains("\"type\":\"done\""), "done 事件: {text}");
        assert!(text.contains("\"turns\":1"), "done 携带 turns: {text}");
    }

    /// ag workflow_run_stream 产出 step_start/step_done/finished 事件
    /// (WorkflowStreamEvent → WorkflowEventDto 无损回读),4 step + 流终止。
    #[tokio::test]
    async fn ag_workflow_run_stream_emits_step_events() {
        use axum::body::Body;
        use tower::ServiceExt;
        use crate::auto_generated::server_stream as ag;
        let app = axum::Router::new()
            .route(
                "/api/workflow/run/stream",
                axum::routing::post(ag::workflow_run_stream),
            )
            .with_state(tmp_state());
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/workflow/run/stream")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"task":"implement binary search","workflow":"feature-dev"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let text = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            axum::body::to_bytes(resp.into_body(), 1 << 20),
        )
        .await
        .expect("ag workflow_run_stream 必须终止")
        .unwrap();
        let text = String::from_utf8(text.to_vec()).unwrap();
        assert_eq!(
            text.matches("\"type\":\"step_start\"").count(),
            4,
            "4 个 step_start: {text}"
        );
        assert_eq!(
            text.matches("\"type\":\"step_done\"").count(),
            4,
            "4 个 step_done: {text}"
        );
        assert!(text.contains("\"type\":\"finished\""), "finished: {text}");
    }

    /// §6.1 验收:ag run_stream_handler 坏 mode → 400 JSON(而非 200 SSE + error 帧)。
    /// 前置 mode_exists 校验在建 mpsc channel / 提交 SSE 响应前,与 hw run_stream_handler
    /// 的 "Resolve the mode up front so we can fail fast" 等价。
    #[tokio::test]
    async fn ag_run_stream_bad_mode_returns_400() {
        use axum::body::Body;
        use tower::ServiceExt;
        use crate::auto_generated::server_stream as ag;
        let app = axum::Router::new()
            .route("/api/run/stream", axum::routing::post(ag::run_stream_handler))
            .with_state(tmp_state());
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/run/stream")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"task":"x","mode":"no-such-mode"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        // §6.1:坏 mode 在 SSE 提交前 → 400(而非 200 text/event-stream)。
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "坏 mode 应返回 400,非 200 SSE"
        );
        let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("application/json"), "400 是 JSON 错误: {ct}");
        let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(v["error"].is_string(), "错误 body 含 error 字段: {v}");
    }

    /// §6.1 验收:ag workflow_run_stream 坏 workflow → 400 JSON(而非 200 SSE + error 帧)。
    /// 前置 workflow_exists 校验在建 mpsc channel 前,与 hw workflow_run_stream 的
    /// require_builtin 前置 400 等价。
    #[tokio::test]
    async fn ag_workflow_run_stream_invalid_workflow_returns_400() {
        use axum::body::Body;
        use tower::ServiceExt;
        use crate::auto_generated::server_stream as ag;
        let app = axum::Router::new()
            .route(
                "/api/workflow/run/stream",
                axum::routing::post(ag::workflow_run_stream),
            )
            .with_state(tmp_state());
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/workflow/run/stream")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"task":"x","workflow":"not-a-workflow"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "坏 workflow 应返回 400,非 200 SSE"
        );
        let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(v["error"].is_string(), "错误 body 含 error 字段: {v}");
    }

    /// ag chat_stream(经 extern 真实化)流式输出 delta/done,run 完成后把
    /// assistant 回复持久化到 session(与 hw chat_stream 一致)。
    #[tokio::test]
    async fn ag_chat_stream_persists_and_streams() {
        use axum::body::Body;
        use tower::ServiceExt;
        use crate::auto_generated::server_stream as ag;
        let state = tmp_state();
        let ws = state.registry.get("");
        let sess = ws
            .chats
            .create("superpowers", Some(String::new()))
            .expect("create session");
        ws.chats
            .append_message(
                &sess.id,
                crate::chats::ChatMessage::user("say hello"),
            )
            .expect("append user message");
        let app = axum::Router::new()
            .route(
                "/api/chats/session/{id}/stream",
                axum::routing::get(ag::chat_stream),
            )
            .with_state(state);
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri(&format!("/api/chats/session/{}/stream", sess.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let text = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            axum::body::to_bytes(resp.into_body(), 1 << 20),
        )
        .await
        .expect("ag chat_stream 必须终止")
        .unwrap();
        let text = String::from_utf8(text.to_vec()).unwrap();
        assert!(text.contains("\"type\":\"delta\""), "delta 事件: {text}");
        assert!(text.contains("\"type\":\"done\""), "done 事件: {text}");
        // Persistence: the assistant reply is now on the session.
        let updated = ws.chats.get(&sess.id).expect("session still exists");
        assert!(
            updated
                .messages
                .iter()
                .any(|m| m.role == crate::chats::Role::Assistant),
            "assistant 回复已持久化"
        );
    }

    /// ag conversation_stream 订阅 broadcast 并按 conversation_id 过滤:
    /// 打开流后再 append turn,应收到该会话的 conversation_event 帧。
    #[tokio::test]
    async fn ag_conversation_stream_filters_events() {
        use axum::body::Body;
        use tower::ServiceExt;
        use futures::StreamExt;
        use crate::auto_generated::server_stream as ag;
        let state = tmp_state();
        let ws = state.registry.get("");
        let conv = ws.conversations.create(
            crate::conversation::ConversationKind::Chat,
            String::new(),
            crate::conversation::Driver::Human,
            Some("superpowers".into()),
            Some("t".into()),
        );
        // Open the stream BEFORE broadcasting, so the receiver is subscribed.
        let app = axum::Router::new()
            .route(
                "/api/conversations/{id}/stream",
                axum::routing::get(ag::conversation_stream),
            )
            .with_state(state);
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri(&format!("/api/conversations/{}/stream", conv.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let mut body = resp.into_body().into_data_stream();

        // Append a turn → broadcasts a ConversationEvent for this conversation.
        let turn = crate::conversation::chat_message_to_turns(
            &crate::chats::ChatMessage::user("hi"),
            0,
        )
        .pop()
        .expect("one turn");
        ws.conversations.append_turn(&conv.id, turn);

        // Read frames until we see the matching conversation_event.
        let mut saw = String::new();
        while let Some(Ok(bytes)) = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            body.next(),
        )
        .await
        .ok()
        .flatten()
        {
            saw.push_str(&String::from_utf8_lossy(&bytes));
            if saw.contains(&format!("\"conversation_id\":\"{}\"", conv.id)) {
                break;
            }
        }
        assert!(
            saw.contains(&format!("\"conversation_id\":\"{}\"", conv.id)),
            "收到匹配会话的事件: {saw}"
        );
        assert!(
            saw.contains("\"turn\""),
            "事件携带完整 Turn 结构: {saw}"
        );
    }

    /// §6.2 验收:BroadcastSub drop → broadcast receiver 析构(receiver_count 回落)。
    /// 直接测 extern 层:subscribe 后 receiver_count +1,drop BroadcastSub 后回到基线。
    #[tokio::test]
    async fn broadcast_sub_drop_reclaims_receiver() {
        use crate::auto_generated::extern_impl::conversations_subscribe;
        let state = tmp_state();
        let ws = state.registry.get("");
        let before = ws.conversations.receiver_count();
        let q = axum::extract::Query(crate::auto_generated::server_stream::WorkspaceQuery { workspace: None });
        let s = axum::extract::State(state.clone());
        {
            let _sub = conversations_subscribe(&s, q, "some-conv-id");
            // subscribe 后 receiver_count +1(rx 活跃)。
            assert_eq!(
                ws.conversations.receiver_count(),
                before + 1,
                "subscribe 注册一个 receiver"
            );
            // _sub 离开作用域 → drop → rx 析构 → receiver_count 回落。
        }
        assert_eq!(
            ws.conversations.receiver_count(),
            before,
            "BroadcastSub drop 后 receiver 回收,无累积泄漏"
        );
    }

    /// §6.2 验收:HTTP 层 —— conversation_stream 的 SSE stream drop(模拟客户端
    /// 断开)后,broadcast receiver 回收(receiver_count 回落)。打开流 → 取 body
    /// stream → drop body stream(模拟断开)→ receiver_count 回到基线。
    #[tokio::test]
    async fn conversation_stream_drop_reclaims_receiver() {
        use axum::body::Body;
        use tower::ServiceExt;
        use crate::auto_generated::server_stream as ag;
        let state = tmp_state();
        let ws = state.registry.get("");
        let conv = ws.conversations.create(
            crate::conversation::ConversationKind::Chat,
            String::new(),
            crate::conversation::Driver::Human,
            Some("superpowers".into()),
            Some("t".into()),
        );
        let before = ws.conversations.receiver_count();
        let app = axum::Router::new()
            .route(
                "/api/conversations/{id}/stream",
                axum::routing::get(ag::conversation_stream),
            )
            .with_state(state);
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri(&format!("/api/conversations/{}/stream", conv.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        // SSE 响应已建立 → conv_event_stream 持有 sub(clone)→ rx 活跃。
        // 注:oneshot 返回后 handler future drop,但其 sub 已 clone 进 stream
        // (在 Response body 里),receiver_count 保持 +1。
        assert_eq!(
            ws.conversations.receiver_count(),
            before + 1,
            "stream 建立后 receiver 活跃(+1)"
        );
        // 模拟客户端断开:drop body stream(Sse + conv_event_stream + sub clone 析构)。
        let body = resp.into_body().into_data_stream();
        drop(body);
        // yield 一下让 Drop 生效。
        tokio::task::yield_now().await;
        assert_eq!(
            ws.conversations.receiver_count(),
            before,
            "客户端断开(stream drop)后 receiver 回收,无累积泄漏"
        );
    }

    /// ag SseEventDto 的 tool_call/tool_result 序列化必须与 hw
    /// `stream_event_to_json` 完全一致(name/arguments + tc-{n} id + status),
    /// 且 stream_event_map 可无损回读(MockClient 不产工具事件,此处直接锚定
    /// wire 形状——run/chat stream 前端渲染工具卡片的金标准)。
    #[test]
    fn ag_sse_dto_tool_events_match_hw_wire_shape() {
        use crate::auto_generated::server_stream::SseEventDto;
        use crate::auto_generated::extern_impl::stream_event_map;
        let args = json!({"path": "/tmp/x"});
        let dto = SseEventDto::ToolCall {
            id: Some("tc-1".into()),
            name: "read_file".into(),
            arguments: args.clone(),
        };
        let v = serde_json::to_value(&dto).unwrap();
        assert_eq!(v["type"], "tool_call", "ToolCall 变体 → tool_call tag");
        assert_eq!(v["id"], "tc-1", "tc-N id 配对");
        assert_eq!(v["name"], "read_file", "字段名 name(非 tool)");
        assert_eq!(v["arguments"]["path"], "/tmp/x", "字段名 arguments(非 args)");
        // stream_event_map 无损回读(extern 侧 Value → DTO)。
        match stream_event_map(Some(v)) {
            SseEventDto::ToolCall { id, name, arguments } => {
                assert_eq!(id.as_deref(), Some("tc-1"));
                assert_eq!(name, "read_file");
                assert_eq!(arguments["path"], "/tmp/x");
            }
            other => panic!("回读应为 ToolCall,got {other:?}"),
        }
        // tool_result:复用 id + result + status=success。
        let dto = SseEventDto::ToolResult {
            id: Some("tc-1".into()),
            name: "read_file".into(),
            arguments: args.clone(),
            result: "ok".into(),
            status: "success".into(),
        };
        let v = serde_json::to_value(&dto).unwrap();
        assert_eq!(v["type"], "tool_result");
        assert_eq!(v["id"], "tc-1");
        assert_eq!(v["status"], "success");
        assert!(matches!(stream_event_map(Some(v)), SseEventDto::ToolResult { .. }));
    }

    /// ag SseEventDto 的 thinking 变体与 hw `{"type":"thinking","thinking":...}`
    /// 一致(前端折叠思考区)。delta/warning/error 亦回读无损。
    #[test]
    fn ag_sse_dto_text_variants_match_hw_wire_shape() {
        use crate::auto_generated::server_stream::SseEventDto;
        use crate::auto_generated::extern_impl::stream_event_map;
        let cases: Vec<(SseEventDto, &str, &str)> = vec![
            (SseEventDto::Thinking { thinking: "reason".into() }, "thinking", "reason"),
            (SseEventDto::Delta { text: "hi".into() }, "delta", "hi"),
            (SseEventDto::Warning { text: "cap".into() }, "warning", "cap"),
            (SseEventDto::Error { message: "boom".into() }, "error", "boom"),
        ];
        for (dto, ty, payload) in cases {
            let v = serde_json::to_value(&dto).unwrap();
            assert_eq!(v["type"], ty, "{ty} tag");
            let rt = stream_event_map(Some(v));
            assert!(
                std::mem::discriminant(&dto) == std::mem::discriminant(&rt),
                "{ty} 回读同变体"
            );
            let _ = payload;
        }
        // Done 携带 turns + tool_calls(name/arguments 形状)。
        let dto = SseEventDto::Done {
            output: "answer".into(),
            turns: 3,
            tool_calls: vec![crate::auto_generated::server_stream::ToolCallOut {
                name: "read_file".into(),
                arguments: json!({"path": "/a"}),
                result: "r".into(),
            }],
        };
        let v = serde_json::to_value(&dto).unwrap();
        assert_eq!(v["type"], "done");
        assert_eq!(v["turns"], 3);
        assert_eq!(v["tool_calls"][0]["name"], "read_file", "tool_calls 用 name");
        assert!(matches!(stream_event_map(Some(v)), SseEventDto::Done { .. }));
    }
}
