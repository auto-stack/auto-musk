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

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

use auto_ai_agent::{builtin_names, load_builtin, load_role, Client, Role};

use crate::build_agent_from_mode;
use crate::workspace::WorkspaceQuery;

/// Shared server state: a client that talks to the daemon, the auth store,
/// and the workspace registry (which resolves per-workspace specs/chats/wiki/
/// relay stores via `?workspace=<id>`).
#[derive(Clone)]
pub struct AppState {
    pub client: Arc<dyn Client>,
    pub auth: Arc<crate::auth::AuthStore>,
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
        auth: Arc::new(crate::auth::AuthStore::new(users_path)),
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
        .route("/api/specs/overview", get(specs_overview))
        .route("/api/specs/drift-check", post(specs_drift_check))
        .route("/api/specs/rebuild-relations", post(specs_rebuild_relations))
        .route("/api/specs/related/{item_id}", get(specs_related))
        .route("/api/config", get(config_overview))
        .route("/api/modes", get(modes_list))
        .route("/api/skills", get(skills_list))
        // Plan 004: Agent Roles — list / detail / save / delete.
        .route("/api/roles", get(roles_list))
        .route("/api/roles/{name}", get(role_detail).put(role_save).delete(role_delete))
        // App runtime config (musk): how it connects to the daemon, default mode, etc.
        .route("/api/app-config", get(app_config_get).put(app_config_save))
        // Service registry proxy: musk → aaid → ensure os-config → return URL.
        .route("/api/settings-link", post(settings_link))
        // App harness (Design 005): merged view of OS-level (inherit) + app-level (custom).
        .route("/api/app-harness/{kind}", get(app_harness_list))
        .route("/api/app-harness/{kind}/{name}", axum::routing::put(app_harness_save).delete(app_harness_delete))
        // Chats (Plan 008): persistent multi-turn sessions.
        .route("/api/chats/sessions", get(chat_list).delete(chat_delete_all))
        .route("/api/chats/session", post(chat_create))
        .route(
            "/api/chats/session/{id}",
            get(chat_get).patch(chat_rename).delete(chat_delete),
        )
        .route("/api/chats/session/{id}/message", post(chat_message))
        .route("/api/chats/session/{id}/stream", get(chat_stream))
        .route("/api/chats/session/{id}/approve/{index}", post(chat_approve))
        .route("/api/chats/session/{id}/reject/{index}", post(chat_reject))
        .route("/api/chats/session/{id}/reject-all", post(chat_reject_all))
        // Conversations (unified chat + flow): direct ConversationStore reads
        // + an SSE stream of real-time turn/status events.
        .route("/api/conversations", get(conversation_list))
        .route(
            "/api/conversations/{id}",
            get(conversation_get).delete(conversation_delete),
        )
        .route("/api/conversations/{id}/title", patch(conversation_rename))
        .route("/api/conversations/{id}/stream", get(conversation_stream))
        // Workspace management (recent / open / status / browse) — operate on
        // the registry itself, not per-workspace stores.
        .route("/api/workspace/list", get(workspace_list))
        .route("/api/workspace/open", post(workspace_open))
        .route("/api/workspace/status", get(workspace_status))
        .route("/api/workspace/browse", get(workspace_browse))
        .route("/api/workspace/initialize", post(workspace_initialize))
        // Relay (Flows) orchestration engine (P2a + P2b.1): runs/flows/professions
        // + the pipeline state machine. Full background driver arrives in P2b.2.
        .merge(crate::relay::api::relay_routes())
        // Wiki knowledge base (Phase 4): markdown pages + raw resource tree.
        .merge(crate::wiki::wiki_routes())
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
async fn specs_list(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.specs.load() {
        Ok(doc) => Json(doc).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { error: format!("load specs: {e}") }),
        )
            .into_response(),
    }
}

/// `GET /api/specs/overview` — aggregate per-section summary (counts + status dist).
async fn specs_overview(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.specs.load() {
        Ok(doc) => {
            // relations + derived statuses reflect the freshest view
            let mut doc = doc;
            doc.rebuild_relations();
            doc.derive_statuses();
            Json(doc.overview()).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { error: format!("load specs: {e}") }),
        )
            .into_response(),
    }
}

/// `POST /api/specs/drift-check` — compare in-memory vs disk version.
async fn specs_drift_check(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.specs.load() {
        Ok(doc) => match ws.specs.drift_check(&doc) {
            Ok((disk_version, drifted)) => Json(serde_json::json!({
                "memory_version": doc.version,
                "disk_version": disk_version,
                "drifted": drifted,
            }))
            .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { error: e }),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { error: format!("load specs: {e}") }),
        )
            .into_response(),
    }
}

/// `POST /api/specs/rebuild-relations` — recompute all `related` reverse-links.
async fn specs_rebuild_relations(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let mut doc = match ws.specs.load() {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { error: format!("load: {e}") }),
            )
                .into_response()
        }
    };
    doc.rebuild_relations();
    doc.derive_statuses();
    if let Err(e) = ws.specs.save(&doc) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { error: format!("save: {e}") }),
        )
            .into_response();
    }
    Json(doc).into_response()
}

/// `GET /api/specs/related/{item_id}` — the related (reverse-link) ids of an item.
async fn specs_related(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    axum::extract::Path(item_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.specs.load() {
        Ok(mut doc) => {
            doc.rebuild_relations();
            for section in &doc.sections {
                if let Some(item) = section.items.iter().find(|i| i.id == item_id) {
                    return Json(serde_json::json!({
                        "item_id": item_id,
                        "depends_on": item.depends_on,
                        "related": item.related,
                    }))
                    .into_response();
                }
            }
            (
                StatusCode::NOT_FOUND,
                Json(ApiError { error: format!("item '{item_id}' not found") }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { error: format!("load: {e}") }),
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
    Query(q): Query<WorkspaceQuery>,
    Json(req): Json<SpecsUpsertRequest>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let mut doc = match ws.specs.load() {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { error: format!("load: {e}") }),
            )
                .into_response()
        }
    };
    match ws
        .specs
        .upsert_item(&mut doc, &req.section_id, req.item)
    {
        Ok(_) => match ws.specs.save(&doc) {
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
    Query(q): Query<WorkspaceQuery>,
    Json(req): Json<SpecsTransitionRequest>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let new_status = crate::specs::SpecStatus::from_str_lossy(&req.new_status);
    let mut doc = match ws.specs.load() {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { error: format!("load: {e}") }),
            )
                .into_response()
        }
    };
    match ws
        .specs
        .transition_item(&mut doc, &req.section_id, &req.item_id, new_status)
    {
        Ok(_) => match ws.specs.save(&doc) {
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
    Query(q): Query<WorkspaceQuery>,
    axum::extract::Path((section_id, item_id)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let mut doc = match ws.specs.load() {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { error: format!("load: {e}") }),
            )
                .into_response()
        }
    };
    match ws.specs.delete_item(&mut doc, &section_id, &item_id) {
        Ok(true) => match ws.specs.save(&doc) {
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

// ── Wiki (Flows) ── relay routes now live in `crate::relay::api`. ──────────

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
                    "role": m.role,
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
                    "role": m.role,
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

// ── Plan 004: Agent Roles endpoints ─────────────────────────────────────────

/// `GET /api/roles` — list all roles (built-in + user). Built-ins are flagged.
async fn roles_list() -> impl IntoResponse {
    let reg = auto_ai_agent::RoleRegistry::load();
    let roles: Vec<serde_json::Value> = reg
        .list()
        .iter()
        .map(|r| {
            json!({
                "name": r.name,
                "description": r.description,
                "tier": format!("{:?}", r.tier).to_lowercase(),
                "allowed_tiers": r.allowed_tiers.iter()
                    .map(|t| format!("{:?}", t).to_lowercase())
                    .collect::<Vec<_>>(),
                "skills": r.skills,
                "skill_count": r.skills.len(),
                "token_budget": r.token_budget,
                "is_builtin": r.is_builtin,
            })
        })
        .collect();
    Json(json!({ "roles": roles }))
}

/// `GET /api/roles/{name}` — full detail of one role, including the Soul md.
async fn role_detail(axum::extract::Path(name): axum::extract::Path<String>) -> impl IntoResponse {
    let reg = auto_ai_agent::RoleRegistry::load();
    match reg.get(&name) {
        Some(d) => {
            let cfg = &d.config;
            Json(json!({
                "name": d.summary.name,
                "description": d.summary.description,
                "tier": format!("{:?}", d.summary.tier).to_lowercase(),
                "allowed_tiers": d.summary.allowed_tiers.iter()
                    .map(|t| format!("{:?}", t).to_lowercase())
                    .collect::<Vec<_>>(),
                "skills": d.summary.skills,
                "token_budget": d.summary.token_budget,
                "is_builtin": d.summary.is_builtin,
                "soul": d.soul,
                "soul_from_file": d.soul_from_file,
                "temperature": cfg.temperature,
                "max_turns": cfg.max_turns,
                "inherit": cfg.inherit,
                "tools": cfg.tools.clone().unwrap_or_default(),
                "model": cfg.model,
                "soul_file": cfg.soul_file,
            }))
            .into_response()
        }
        None => (StatusCode::NOT_FOUND, format!("role '{name}' not found")).into_response(),
    }
}

/// `PUT /api/roles/{name}` body.
#[derive(Debug, Deserialize)]
struct RoleSaveBody {
    description: Option<String>,
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    allowed_tiers: Vec<String>,
    #[serde(default)]
    skills: Vec<String>,
    token_budget: Option<u64>,
    temperature: Option<f64>,
    max_turns: Option<usize>,
    inherit: Option<String>,
    #[serde(default)]
    tools: Vec<String>,
    model: Option<String>,
    /// The Soul markdown body. When Some, written to the sidecar .soul.md.
    #[serde(default)]
    soul: Option<String>,
}

/// `PUT /api/roles/{name}` — create or overwrite a user role.
async fn role_save(
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(body): Json<RoleSaveBody>,
) -> impl IntoResponse {
    use auto_ai_agent::{parse_tier_field, RoleConfig};
    let cfg = RoleConfig {
        name: Some(name.clone()),
        description: body.description,
        inherit: body.inherit,
        model: body.model,
        model_tier: body.tier.as_deref().and_then(parse_tier_field),
        temperature: body.temperature,
        max_turns: body.max_turns,
        allowed_tiers: if body.allowed_tiers.is_empty() {
            None
        } else {
            Some(body.allowed_tiers.iter().filter_map(|s| parse_tier_field(s)).collect())
        },
        skills: if body.skills.is_empty() { None } else { Some(body.skills) },
        token_budget: body.token_budget,
        tools: if body.tools.is_empty() { None } else { Some(body.tools) },
        soul_file: None, // set by save() when soul is provided
        system_prompt: None,
        system_prompt_append: None,
        tools_append: None,
        memory_limit: None,
    };
    let reg = auto_ai_agent::RoleRegistry::load();
    match reg.save(&name, cfg, body.soul.as_deref()) {
        Ok(_) => Json(json!({"status": "saved", "name": name})).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("failed to save role '{name}': {e}"),
        )
            .into_response(),
    }
}

/// `DELETE /api/roles/{name}` — delete a user role (built-ins: 403).
async fn role_delete(axum::extract::Path(name): axum::extract::Path<String>) -> impl IntoResponse {
    let reg = auto_ai_agent::RoleRegistry::load();
    match reg.delete(&name) {
        Ok(_) => Json(json!({"status": "deleted", "name": name})).into_response(),
        Err(e) => {
            // Built-in roles → 403; not-found → 404; else 400.
            let code = if e.to_string().contains("built-in") {
                StatusCode::FORBIDDEN
            } else if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            (code, e.to_string()).into_response()
        }
    }
}

// ── App runtime config (musk) ───────────────────────────────────────────────
//
// ── App runtime config (musk) ───────────────────────────────────────────────
//
// musk's runtime config lives in `crate::app_config` (shared with the CLI).
// The handlers here read/persist it; the CLI applies it to the environment.
// Per the unified-Harness design, app config is "how this app runs", not
// "which capabilities it inherits".

use crate::app_config::{musk_config_path, HarnessSelection, MuskAppConfig};

/// `GET /api/app-config` — the persisted config + the effective (merged) values.
async fn app_config_get() -> impl IntoResponse {
    let cfg = MuskAppConfig::load();
    Json(json!({
        "stored": cfg,
        "effective": cfg.effective(),
    }))
}

/// `PUT /api/app-config` body.
#[derive(Debug, Deserialize)]
struct AppConfigSaveBody {
    daemon_url: Option<String>,
    default_mode: Option<String>,
    context_file: Option<String>,
    serve_addr: Option<String>,
    auto_start_daemon: Option<bool>,
    #[serde(default)]
    harness: HarnessBody,
}

#[derive(Debug, Default, Deserialize)]
struct HarnessBody {
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    skills: Vec<String>,
    #[serde(default)]
    modes: Vec<String>,
}

/// `PUT /api/app-config` — persist the config to ~/.config/autoos/apps/musk/config.at.
async fn app_config_save(Json(body): Json<AppConfigSaveBody>) -> impl IntoResponse {
    let cfg = MuskAppConfig {
        daemon_url: body.daemon_url,
        default_mode: body.default_mode,
        context_file: body.context_file,
        serve_addr: body.serve_addr,
        auto_start_daemon: body.auto_start_daemon,
        harness: HarnessSelection {
            roles: body.harness.roles,
            skills: body.harness.skills,
            modes: body.harness.modes,
        },
    };
    let path = match musk_config_path() {
        Some(p) => p,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, "cannot determine home dir".to_string()).into_response(),
    };
    if let Err(e) = std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new("."))) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("mkdir: {e}")).into_response();
    }
    let src = cfg.to_at_source();
    if let Err(e) = std::fs::write(&path, &src) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("write: {e}")).into_response();
    }
    Json(json!({"status": "saved", "path": path.display().to_string(), "effective": cfg.effective()})).into_response()
}

/// `POST /api/settings-link` — proxy to aaid's service registry to ensure
/// auto-os-config is running, then return its URL. The frontend uses this to
/// implement the "Open System Settings" deep-link (musk → aaid → os-config).
async fn settings_link() -> axum::response::Response {
    let cfg = crate::app_config::MuskAppConfig::load();
    let daemon_url = cfg.effective_daemon_url();
    let ensure_url = format!("{}/v1/services/os-config/ensure", daemon_url);

    let result = tokio::task::spawn_blocking(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(|e| format!("http client: {e}"))?;
        client
            .post(&ensure_url)
            .send()
            .map_err(|e| format!("aaid unreachable: {e}"))?
            .json::<serde_json::Value>()
            .map_err(|e| format!("parse aaid response: {e}"))
    })
    .await;

    match result {
        Ok(Ok(val)) => {
            let status = val.get("status").and_then(|s| s.as_str()).unwrap_or("error");
            let url = val.get("url").and_then(|u| u.as_str()).unwrap_or("");
            if status == "running" && !url.is_empty() {
                Json(json!({"status": "running", "url": url})).into_response()
            } else {
                let err = val.get("error").and_then(|e| e.as_str()).unwrap_or("unknown");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"status": "error", "error": err})),
                )
                    .into_response()
            }
        }
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error", "error": e})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error", "error": format!("internal: {e}")})),
        )
            .into_response(),
    }
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

// ── App Harness endpoints (Design 005) ──────────────────────────────────────
//
// Merged view: for each kind (roles/skills/modes), show OS-level harnesses
// (with `selected` flag from the app's reference list) + app-level custom
// harnesses (scanned from apps/musk/harness/<kind>/).

/// The app-level harness dir: `~/.config/autoos/apps/musk/harness/<kind>/`.
fn app_harness_dir(kind: &str) -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(format!(".config/autoos/apps/musk/harness/{kind}")))
}

/// `GET /api/app-harness/{kind}` — merged OS-available + app-custom view.
///
/// Returns `{ os_available: [...], app_custom: [...] }` where each item has
/// `name`, `description`, `is_builtin`, and `selected` (for os_available, based
/// on the app config's harness reference list).
async fn app_harness_list(axum::extract::Path(kind): axum::extract::Path<String>) -> Response {
    let cfg = crate::app_config::MuskAppConfig::load();
    let selected_list: &[String] = match kind.as_str() {
        "roles" => &cfg.harness.roles,
        "skills" => &cfg.harness.skills,
        "modes" => &cfg.harness.modes,
        _ => return (StatusCode::BAD_REQUEST, format!("unknown kind '{kind}'")).into_response(),
    };

    // OS-level list (reuse existing endpoints' data sources).
    let os_available: Vec<serde_json::Value> = match kind.as_str() {
        "roles" => {
            let reg = auto_ai_agent::RoleRegistry::load();
            reg.list().iter().map(|r| {
                json!({
                    "name": r.name,
                    "description": r.description,
                    "tier": format!("{:?}", r.tier).to_lowercase(),
                    "is_builtin": r.is_builtin,
                    "selected": selected_list.contains(&r.name),
                })
            }).collect()
        }
        "skills" => {
            let skills_dir = dirs::home_dir().map(|h| h.join(".config/autoos/skills"));
            if let Some(dir) = skills_dir {
                let reg = auto_ai_agent::SkillRegistry::scan(&dir);
                reg.descriptions().iter().map(|(name, desc)| {
                    json!({
                        "name": name,
                        "description": desc,
                        "is_builtin": false,
                        "selected": selected_list.contains(name),
                    })
                }).collect()
            } else {
                vec![]
            }
        }
        "modes" => {
            let reg = crate::mode::ModeRegistry::load();
            reg.names().iter().filter_map(|n| {
                reg.get(n).map(|m| json!({
                    "name": m.name,
                    "description": m.description,
                    "is_builtin": false,
                    "selected": selected_list.contains(&m.name),
                }))
            }).collect()
        }
        _ => vec![],
    };

    // App-level custom: scan apps/musk/harness/<kind>/.
    let app_custom: Vec<serde_json::Value> = if let Some(dir) = app_harness_dir(&kind) {
        match kind.as_str() {
            "roles" => scan_app_roles(&dir),
            "skills" => scan_app_skills(&dir),
            "modes" => scan_app_modes(&dir),
            _ => vec![],
        }
    } else {
        vec![]
    };

    Json(json!({
        "os_available": os_available,
        "app_custom": app_custom,
    })).into_response()
}

/// Scan `dir` for app-level custom roles (*.at files).
fn scan_app_roles(dir: &std::path::Path) -> Vec<serde_json::Value> {
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
fn scan_app_skills(dir: &std::path::Path) -> Vec<serde_json::Value> {
    let reg = auto_ai_agent::SkillRegistry::scan(dir);
    reg.descriptions().iter().map(|(name, desc)| {
        json!({ "name": name, "description": desc, "is_builtin": false })
    }).collect()
}

/// Scan `dir` for app-level custom modes (*.at files).
fn scan_app_modes(dir: &std::path::Path) -> Vec<serde_json::Value> {
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

/// `PUT /api/app-harness/{kind}/{name}` body — app-level custom role/mode creation.
#[derive(Debug, Deserialize)]
struct AppHarnessSaveBody {
    description: Option<String>,
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    allowed_tiers: Vec<String>,
    #[serde(default)]
    skills: Vec<String>,
    token_budget: Option<u64>,
    temperature: Option<f64>,
    max_turns: Option<usize>,
    inherit: Option<String>,
    #[serde(default)]
    tools: Vec<String>,
    model: Option<String>,
    #[serde(default)]
    soul: Option<String>,
}

/// `PUT /api/app-harness/{kind}/{name}` — create/update an app-level custom harness.
async fn app_harness_save(
    axum::extract::Path((kind, name)): axum::extract::Path<(String, String)>,
    Json(body): Json<AppHarnessSaveBody>,
) -> Response {
    match kind.as_str() {
        "roles" => {
            let cfg = auto_ai_agent::RoleConfig {
                name: Some(name.clone()),
                description: body.description,
                inherit: body.inherit,
                model: body.model,
                model_tier: body.tier.as_deref().and_then(auto_ai_agent::parse_tier_field),
                temperature: body.temperature,
                max_turns: body.max_turns,
                allowed_tiers: if body.allowed_tiers.is_empty() {
                    None
                } else {
                    Some(body.allowed_tiers.iter().filter_map(|s| auto_ai_agent::parse_tier_field(s)).collect())
                },
                skills: if body.skills.is_empty() { None } else { Some(body.skills) },
                token_budget: body.token_budget,
                tools: if body.tools.is_empty() { None } else { Some(body.tools) },
                soul_file: None,
                system_prompt: None,
                system_prompt_append: None,
                tools_append: None,
                memory_limit: None,
            };
            let dir = match app_harness_dir("roles") {
                Some(d) => d,
                None => return (StatusCode::INTERNAL_SERVER_ERROR, "no home dir").into_response(),
            };
            if let Err(e) = std::fs::create_dir_all(&dir) {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("mkdir: {e}")).into_response();
            }
            // Write sidecar soul if provided.
            if let Some(md) = &body.soul {
                let soul_path = dir.join(format!("{name}.soul.md"));
                if let Err(e) = std::fs::write(&soul_path, md) {
                    return (StatusCode::INTERNAL_SERVER_ERROR, format!("write soul: {e}")).into_response();
                }
            }
            let src = auto_ai_agent::serialize_at_role(&cfg);
            let at_path = dir.join(format!("{name}.at"));
            if let Err(e) = std::fs::write(&at_path, &src) {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("write: {e}")).into_response();
            }
            Json(json!({"status": "saved", "kind": kind, "name": name, "path": at_path.display().to_string()})).into_response()
        }
        _ => (StatusCode::BAD_REQUEST, format!("kind '{kind}' not yet supported for app custom")).into_response(),
    }
}

/// `DELETE /api/app-harness/{kind}/{name}` — delete an app-level custom harness.
async fn app_harness_delete(
    axum::extract::Path((kind, name)): axum::extract::Path<(String, String)>,
) -> Response {
    let dir = match app_harness_dir(&kind) {
        Some(d) => d,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, "no home dir").into_response(),
    };
    let at_path = dir.join(format!("{name}.at"));
    let soul_path = dir.join(format!("{name}.soul.md"));
    let existed = at_path.exists();
    if existed {
        let _ = std::fs::remove_file(&at_path);
    }
    let _ = std::fs::remove_file(&soul_path);
    if !existed {
        return (StatusCode::NOT_FOUND, format!("{kind} '{name}' not found")).into_response();
    }
    Json(json!({"status": "deleted", "kind": kind, "name": name})).into_response()
}

// ── Chats endpoints (Plan 008) ──────────────────────────────────────────────

/// `POST /api/chats/session` body.
#[derive(Debug, Deserialize)]
struct ChatCreateBody {
    /// Mode for the session (defaults to "superpowers").
    #[serde(default = "default_mode")]
    mode: String,
}

/// `POST /api/chats/session` — create a new (empty) chat session.
async fn chat_create(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Json(body): Json<ChatCreateBody>,
) -> impl IntoResponse {
    let ws_id = q.id_or_default(&state.registry);
    let ws = state.registry.get(&ws_id);
    match ws.chats.create(&body.mode, Some(ws_id.clone())) {
        Ok(session) => {
            // Dual-write: create a matching conversation with the SAME id so the
            // two stores stay linked. Failure here is non-fatal — the ChatStore
            // is the source of truth for the /api/chats surface.
            let _ = ws.conversations.create_with_id(
                session.id.clone(),
                crate::conversation::ConversationKind::Chat,
                ws_id.clone(),
                crate::conversation::Driver::Human,
                Some(body.mode.clone()),
                Some(session.name.clone()),
            );
            Json(json!({"session": session})).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("create: {e}")).into_response(),
    }
}

/// `GET /api/chats/sessions` — list all sessions (summaries).
async fn chat_list(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    Json(json!({"sessions": ws.chats.list()}))
}

/// `GET /api/chats/session/{id}` — full session with all messages.
async fn chat_get(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.chats.get(&id) {
        Some(session) => Json(json!({"session": session})).into_response(),
        None => (StatusCode::NOT_FOUND, format!("session '{id}' not found")).into_response(),
    }
}

/// `PATCH /api/chats/session/{id}` body.
#[derive(Debug, Deserialize)]
struct ChatRenameBody {
    name: String,
}

/// `PATCH /api/chats/session/{id}` — rename a session.
async fn chat_rename(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(body): Json<ChatRenameBody>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.chats.rename(&id, &body.name) {
        Ok(Some(session)) => {
            // Dual-write: keep the conversation title in sync.
            let _ = ws.conversations.rename(&id, &body.name);
            Json(json!({"session": session})).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, format!("session '{id}' not found")).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("rename: {e}")).into_response(),
    }
}

/// `DELETE /api/chats/session/{id}` — delete one session.
async fn chat_delete(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.chats.delete(&id) {
        Ok(true) => {
            // Dual-write: delete the matching conversation (ignore missing).
            let _ = ws.conversations.delete(&id);
            Json(json!({"status": "deleted", "id": id})).into_response()
        }
        Ok(false) => (StatusCode::NOT_FOUND, format!("session '{id}' not found")).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("delete: {e}")).into_response(),
    }
}

/// `DELETE /api/chats/sessions` — delete all sessions.
async fn chat_delete_all(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.chats.delete_all() {
        Ok(_) => {
            // Dual-write: clear all conversations too.
            ws.conversations.delete_all();
            Json(json!({"status": "deleted_all"})).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("delete_all: {e}")).into_response(),
    }
}

/// `POST /api/chats/session/{id}/message` body.
#[derive(Debug, Deserialize)]
struct ChatMessageBody {
    content: String,
}

/// `POST /api/chats/session/{id}/message` — append the user's message and run
/// one agent turn (non-streaming). Returns the assistant reply + tool calls.
/// For streaming, the client calls `GET /api/chats/session/{id}/stream` after.
///
/// NOTE: in the streaming flow the client POSTs the message here (persisting
/// the user turn) then opens the SSE stream, which runs the turn and appends
/// the assistant message on completion. To avoid double-runs, this endpoint
/// only persists the user message + returns it; the actual agent run happens
/// in `chat_stream`. (Kept as a distinct endpoint so persistence and streaming
/// are cleanly separated.)
async fn chat_message(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(body): Json<ChatMessageBody>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let msg = crate::chats::ChatMessage::user(body.content);
    match ws.chats.append_message(&id, msg.clone()) {
        Ok(Some(session)) => {
            // Dual-write: mirror the user message as a turn in the conversation.
            let seq_base = ws
                .conversations
                .get(&id)
                .map(|c| c.turns.len())
                .unwrap_or(0);
            for turn in crate::conversation::chat_message_to_turns(&msg, seq_base) {
                let _ = ws.conversations.append_turn(&id, turn);
            }
            Json(json!({"session": session, "queued": msg})).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, format!("session '{id}' not found")).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("append: {e}")).into_response(),
    }
}

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
        let on_event: Arc<dyn Fn(auto_ai_agent::StreamEvent) + Send + Sync> =
            Arc::new(move |ev| {
                let value = stream_event_to_json(&ev);
                // capture for persistence
                if let Some(text) = value.get("text").and_then(|t| t.as_str()) {
                    acc2.lock().unwrap().push_str(text);
                }
                if value.get("type").and_then(|t| t.as_str()) == Some("tool") {
                    let tool = value.get("tool").and_then(|t| t.as_str()).unwrap_or("").to_string();
                    let args = value.get("args").cloned().unwrap_or(json!(null));
                    let result = value.get("result").and_then(|t| t.as_str()).unwrap_or("").to_string();
                    tc2.lock().unwrap().push(crate::chats::ToolCall { tool, args, result });
                }
                let _ = tx.try_send(value);
            });
        match agent.run_stream(&user_msg, on_event).await {
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

/// `POST /api/chats/session/{id}/approve/{index}` — apply the pending spec
/// change at `index` to the Spec Ledger, then remove it from the queue.
async fn chat_approve(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    axum::extract::Path((id, index)): axum::extract::Path<(String, usize)>,
) -> Response {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.chats.approve_spec_change(&id, index, &ws.specs) {
        Ok(Some((change, session))) => Json(json!({
            "applied": change,
            "session": session,
        }))
        .into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "session not found").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

/// `POST /api/chats/session/{id}/reject/{index}` — discard the pending spec
/// change at `index` without applying it.
async fn chat_reject(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    axum::extract::Path((id, index)): axum::extract::Path<(String, usize)>,
) -> Response {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.chats.reject_spec_change(&id, index) {
        Ok(Some(session)) => Json(json!({ "session": session })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "session not found").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

/// `POST /api/chats/session/{id}/reject-all` — discard all pending spec changes.
async fn chat_reject_all(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.chats.reject_all_spec_changes(&id) {
        Ok(Some(session)) => Json(json!({ "session": session })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "session not found").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

// ── Workspace management endpoints ──────────────────────────────────────────

/// `GET /api/workspace/list` — recent workspaces.
async fn workspace_list(State(state): State<AppState>) -> impl IntoResponse {
    Json(json!({ "workspaces": state.registry.list() }))
}

/// `POST /api/workspace/open` body.
#[derive(Deserialize)]
struct OpenWorkspaceBody {
    path: String,
}

/// `POST /api/workspace/open` — open/reuse a workspace by path.
async fn workspace_open(
    State(state): State<AppState>,
    Json(body): Json<OpenWorkspaceBody>,
) -> impl IntoResponse {
    let meta = state.registry.open(&body.path);
    state.registry.touch(&meta.id);
    Json(json!({ "workspace": meta }))
}

/// `GET /api/workspace/status?workspace=<id>` — current workspace meta + whether
/// its root still exists on disk.
async fn workspace_status(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
) -> Response {
    let ws_id = q.id_or_default(&state.registry);
    let mut meta = state.registry.list().into_iter().find(|m| m.id == ws_id);
    match meta.as_mut() {
        Some(m) => {
            // Re-check emptiness live (the dir may have gained files since open).
            m.is_empty = crate::workspace::is_workspace_empty(std::path::Path::new(&m.path));
            Json(json!({
                "workspace": m,
                "root_exists": std::path::Path::new(&m.path).exists(),
            }))
            .into_response()
        }
        None => (StatusCode::NOT_FOUND, "workspace not found").into_response(),
    }
}

/// `GET /api/workspace/browse?path=<dir>` — list child directories (for the
/// picker). Hides dotfiles. Returns `parent` for the "up one level" affordance.
#[derive(Deserialize)]
struct BrowseQuery {
    #[serde(default)]
    path: String,
}

async fn workspace_browse(Query(q): Query<BrowseQuery>) -> impl IntoResponse {
    let base = if q.path.is_empty() {
        ".".to_string()
    } else {
        q.path.clone()
    };
    let mut entries: Vec<serde_json::Value> = Vec::new();
    if let Ok(dir) = std::fs::read_dir(&base) {
        for e in dir.flatten() {
            if e.path().is_dir() {
                let name = e.file_name().to_string_lossy().to_string();
                if !name.starts_with('.') {
                    entries.push(json!({
                        "name": name,
                        "path": e.path().to_string_lossy().to_string(),
                    }));
                }
            }
        }
    }
    let parent = std::path::Path::new(&base)
        .parent()
        .map(|p| p.to_string_lossy().to_string());
    Json(json!({ "entries": entries, "parent": parent }))
}

/// `POST /api/workspace/initialize?workspace=<id>` — mark a workspace as
/// initialized (drops `.autoos/initialized`). Called by the frontend after
/// the onboarding dialog completes, so the dialog won't re-appear on re-open.
async fn workspace_initialize(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
) -> Response {
    let ws_id = q.id_or_default(&state.registry);
    let ws = state.registry.get(&ws_id);
    let marker = ws.root.join(".autoos").join("initialized");
    match std::fs::write(&marker, b"1") {
        Ok(_) => Json(json!({ "status": "initialized", "workspace": ws_id })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("write marker: {e}")).into_response(),
    }
}

// ── Conversation endpoints (unified chat + flow) ────────────────────────────

/// `GET /api/conversations?workspace=<id>` — list all conversations.
async fn conversation_list(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
) -> impl IntoResponse {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    Json(json!({ "conversations": ws.conversations.list() }))
}

/// `GET /api/conversations/{id}?workspace=<id>` — get conversation with all turns.
async fn conversation_get(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Query(q): Query<WorkspaceQuery>,
) -> Response {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.conversations.get(&id) {
        Some(conv) => Json(conv).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            format!("conversation '{id}' not found"),
        )
            .into_response(),
    }
}

/// `DELETE /api/conversations/{id}?workspace=<id>`
async fn conversation_delete(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Query(q): Query<WorkspaceQuery>,
) -> Response {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    if ws.conversations.delete(&id) {
        Json(json!({"status": "deleted", "id": id})).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            format!("conversation '{id}' not found"),
        )
            .into_response()
    }
}

/// `PATCH /api/conversations/{id}/title?workspace=<id>` body.
#[derive(Deserialize)]
struct ConversationTitleBody {
    title: String,
}

/// `PATCH /api/conversations/{id}/title?workspace=<id>` — rename a conversation.
async fn conversation_rename(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Query(q): Query<WorkspaceQuery>,
    Json(body): Json<ConversationTitleBody>,
) -> Response {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    match ws.conversations.rename(&id, &body.title) {
        Some(conv) => Json(conv).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            format!("conversation '{id}' not found"),
        )
            .into_response(),
    }
}

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
    Query(q): Query<WorkspaceQuery>,
    Json(req): Json<WorkflowRunRequest>,
) -> Result<Json<WorkflowRunResponse>, (StatusCode, Json<ApiError>)> {
    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let wf = crate::workflow::load(&req.workflow).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: format!("invalid workflow '{}': {e}", req.workflow),
            }),
        )
    })?;

    crate::tool_safety::set_current_root(ws.root.clone());
    let result = wf.run(&shared_tools(), state.client.clone(), &req.task).await;
    crate::tool_safety::clear_current_root();

    result
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
/// - `{"type":"step_start","step_id":"architect","role":"architect","input":"…"}`
/// - `{"type":"step_done","step_id":"architect","output":"…"}`
/// - `{"type":"step_skipped","step_id":"reviewer"}`
/// - `{"type":"finished",…}` (or `{"type":"error","message":"…"}`)
async fn workflow_run_stream(
    State(state): State<AppState>,
    Query(q): Query<WorkspaceQuery>,
    Json(req): Json<WorkflowRunRequest>,
) -> axum::response::Response {
    use axum::body::Body;
    use axum::response::Response;
    use tokio::sync::mpsc;

    let (tx, mut rx) = mpsc::channel::<serde_json::Value>(64);

    let ws = state.registry.get(&q.id_or_default(&state.registry));
    let ws_root = ws.root.clone();

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
        // Confine this task's file-tool operations to the workspace root.
        crate::tool_safety::set_current_root(ws_root.clone());
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

/// Serialize a [`auto_ai_agent::WorkflowEvent`] to the SSE JSON shape.
fn workflow_event_to_json(ev: &auto_ai_agent::WorkflowEvent) -> serde_json::Value {
    use auto_ai_agent::WorkflowEvent;
    match ev {
        WorkflowEvent::StepStart {
            step_id,
            role,
            input,
        } => json!({
            "type": "step_start",
            "step_id": step_id,
            "role": role,
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

    fn tmp_auth() -> Arc<crate::auth::AuthStore> {
        let path = std::env::temp_dir().join(format!(
            "musk_server_auth_test_{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        Arc::new(crate::auth::AuthStore::new(path))
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
}
