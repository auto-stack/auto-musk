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

use crate::build_agent;

/// Shared server state: a client that talks to the daemon.
#[derive(Clone)]
pub struct AppState {
    pub client: Arc<dyn Client>,
}

/// Run the HTTP server on the given address (default `127.0.0.1:8080`).
pub async fn serve(addr: &str, client: Arc<dyn Client>) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState { client };

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/professions", get(professions))
        .route("/api/run", post(run))
        .route("/api/run/stream", post(run_stream_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("musk server listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

async fn professions() -> impl IntoResponse {
    let list: Vec<serde_json::Value> = builtin_names()
        .iter()
        .filter_map(|name| {
            load_builtin(name).map(|p| {
                json!({
                    "name": name,
                    "model": p.model(),
                    "temperature": p.temperature(),
                    "max_turns": p.max_turns(),
                })
            })
        })
        .collect();
    Json(json!({"professions": list}))
}

/// `POST /api/run` request body.
#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub task: String,
    /// Built-in name (e.g. "coder") or path to a `.at` file. Defaults to "coder".
    #[serde(default = "default_profession")]
    pub profession: String,
}

fn default_profession() -> String {
    "coder".to_string()
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
    let profession: Arc<dyn Profession> = resolve(&req.profession).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: format!("invalid profession '{}': {e}", req.profession),
            }),
        )
    })?;

    let mut agent = build_agent(profession, state.client.clone());
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

    // Resolve the profession up front so we can fail fast on a bad spec.
    let profession: Arc<dyn Profession> = match resolve(&req.profession) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: format!("invalid profession '{}': {e}", req.profession),
                }),
            )
                .into_response();
        }
    };

    // Spawn the agent run, pushing StreamEvents into the channel as SSE JSON.
    let client = state.client.clone();
    tokio::spawn(async move {
        let mut agent = build_agent(profession, client);
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

    #[tokio::test]
    async fn run_endpoint_returns_result() {
        let state = AppState {
            client: Arc::new(MockClient) as Arc<dyn Client>,
        };
        let req = RunRequest {
            task: "say hello".into(),
            profession: "coder".into(),
        };
        let resp = run_inner(state, req).await.unwrap();
        assert_eq!(resp.output, "mock answer");
        assert_eq!(resp.turns, 1);
    }

    #[tokio::test]
    async fn run_endpoint_bad_profession_errors() {
        let state = AppState {
            client: Arc::new(MockClient) as Arc<dyn Client>,
        };
        let req = RunRequest {
            task: "x".into(),
            profession: "nonexistent".into(),
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
