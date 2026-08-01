//! extern_impl.rs — glue layer for a2r-transpiled .at files.
//!
//! Provides stub implementations for all extern functions declared in .at files.
//! These are called by the Auto-transpiled handlers; the real implementations
//! wrap the hand-written Rust store/registry/agent APIs.
//!
//! This file makes the transpiled output compile. Real implementations delegate
//! to the existing hand-written Rust code in the musk crate.

#![allow(dead_code, unused_variables, non_upper_case_globals)]

use serde_json::Value;
use std::sync::Arc;
use axum::extract::{Query, Path};

// ── Value helpers ───────────────────────────────────────────────────────────

pub fn parse_json(s: &str) -> Value { serde_json::from_str(s).unwrap_or(Value::Null) }
pub fn value_get_str(v: &Value, key: &str) -> String {
    v.get(key).and_then(|s| s.as_str()).unwrap_or("").to_string()
}
pub fn value_get_bool(v: &Value, key: &str) -> bool {
    v.get(key).and_then(|b| b.as_bool()).unwrap_or(false)
}
pub fn value_get_array(v: &Value, key: &str) -> Value {
    v.get(key).cloned().unwrap_or(Value::Array(vec![]))
}
pub fn null_value() -> Value { Value::Null }
pub fn new_id(_nbytes: u32) -> String { uuid_like_id() }
fn uuid_like_id() -> String {
    use rand::RngCore;
    let mut buf = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}
pub fn random_hex(nbytes: u32) -> String {
    use rand::RngCore;
    let mut buf = vec![0u8; nbytes as usize];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}
pub fn hash_password(password: &str, salt: &str) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

// ── JSON schema constants ──────────────────────────────────────────────────

pub const read_file_schema: &str = r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#;
pub const write_file_schema: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}"#;
pub const run_command_schema: &str = r#"{"type":"object","properties":{"cmd":{"type":"string"},"force":{"type":"boolean"}},"required":["cmd"]}"#;
pub const edit_file_schema: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"old_string":{"type":"string"},"new_string":{"type":"string"}},"required":["path","old_string","new_string"]}"#;
pub const batch_replace_schema: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"replacements":{"type":"array"}},"required":["path","replacements"]}"#;
pub const search_schema: &str = r#"{"type":"object","properties":{"pattern":{"type":"string"}},"required":["pattern"]}"#;
pub const list_dir_schema: &str = r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#;
pub const list_symbols_schema: &str = r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#;
pub const glob_schema: &str = r#"{"type":"object","properties":{"pattern":{"type":"string"}},"required":["pattern"]}"#;
pub const read_specs_schema: &str = r#"{"type":"object","properties":{"section":{"type":"string"}}}"#;
pub const list_specs_schema: &str = r#"{"type":"object"}"#;
pub const write_spec_schema: &str = r#"{"type":"object","properties":{"section":{"type":"string"},"content":{"type":"string"}},"required":["section","content"]}"#;
pub const update_spec_schema: &str = r#"{"type":"object","properties":{"action":{"type":"string"},"section":{"type":"string"},"item_id":{"type":"string"}},"required":["action","section"]}"#;
pub const write_goals_schema: &str = r#"{"type":"object","properties":{"goals":{"type":"string"}},"required":["goals"]}"#;
pub const spawn_relay_schema: &str = r#"{"type":"object","properties":{"task":{"type":"string"},"flow_id":{"type":"string"}},"required":["task"]}"#;
pub const dispatch_schema: &str = r#"{"type":"object","properties":{"task":{"type":"string"},"to":{"type":"string"}},"required":["task","to"]}"#;
pub const bring_in_schema: &str = r#"{"type":"object","properties":{"query":{"type":"string"}},"required":["query"]}"#;

// ── Tool safety / fs ────────────────────────────────────────────────────────

pub fn resolve_within_project(path: &str) -> String { path.to_string() }
pub fn write_file_do(path: &str, content: &str) { let _ = std::fs::write(path, content); }
pub fn command_needs_approval(cmd: &str) -> bool { !cmd.starts_with("echo") }
pub fn run_shell_command(cmd: &str) -> String { format!("(stub) {}", cmd) }
pub fn edit_file_do(path: &str, old_str: &str, new_str: &str) -> String {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let count = content.matches(old_str).count();
    if count == 0 { return "old_string not found".into(); }
    if count > 1 { return "old_string not unique".into(); }
    let new_content = content.replacen(old_str, new_str, 1);
    let _ = std::fs::write(path, new_content);
    "edited 1 replacement".into()
}
pub fn batch_replace_do(path: &str, _replacements: Value) -> String { format!("(stub) batch {}", path) }
pub fn search_files(pattern: &str) -> String { format!("(stub) search {}", pattern) }
pub fn list_directory(path: &str) -> String { format!("(stub) ls {}", path) }
pub fn list_symbols_in(path: &str) -> String { format!("(stub) symbols {}", path) }
pub fn glob_files(pattern: &str) -> String { format!("(stub) glob {}", pattern) }

// ── HTTP / SSE helpers ──────────────────────────────────────────────────────

pub fn http_post_json(_url: String) -> impl std::future::Future<Output = Result<Value, String>> {
    async { Ok(serde_json::json!({"status": "ok"})) }
}
pub fn mpsc_channel() -> (std::sync::mpsc::Sender<Value>, std::sync::mpsc::Receiver<Value>) {
    std::sync::mpsc::channel()
}
pub fn mpsc_try_send(tx: &std::sync::mpsc::Sender<Value>, msg: Value) { let _ = tx.send(msg); }
pub async fn mpsc_recv(rx: &mut std::sync::mpsc::Receiver<Value>) -> Option<Value> { rx.recv().ok() }
pub fn msg_is_none(msg: &Option<Value>) -> bool { msg.is_none() }
pub fn msg_unwrap(msg: Option<Value>) -> Value { msg.unwrap_or(Value::Null) }
pub fn broadcast_recv(_rx: &()) -> impl std::future::Future<Output = Option<Value>> {
    async { None }
}
pub fn path_inner(p: &axum::extract::Path<String>) -> String { p.0.clone() }
pub fn json_response<T: serde::Serialize>(_data: T) -> axum::response::Response {
    axum::response::Response::default()
}
pub fn error_response<T: serde::Serialize>(_code: u16, _data: T) -> axum::response::Response {
    axum::response::Response::default()
}
pub fn atomic_bool_false() -> Arc<std::sync::atomic::AtomicBool> {
    Arc::new(std::sync::atomic::AtomicBool::new(false))
}

// ── Stub types for missing upstream types ──────────────────────────────────

pub struct Agent;
pub struct AppState;
pub struct AgentMode;
pub struct ModelTier;
pub struct ToolError;
pub struct RoleRegistry;
pub struct Workflow;

// Re-export axum types for handler signatures
pub use axum::extract::{Query as QueryType, Path as PathType};

// Stub traits for upstream auto_ai_agent
pub trait Tool {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn parameters(&self) -> Value;
    async fn execute(&self, args: Value) -> Result<String, ToolError>;
}
pub trait Client {
    async fn complete(&self, req: Value) -> Result<String, ToolError>;
}

// ── Auth helpers ────────────────────────────────────────────────────────────
pub fn auth_login_role(_s: &AppState, _u: String, _p: String) -> String { "admin".into() }
pub fn auth_login_token(_s: &AppState, _u: String, _p: String) -> String { "stub-token".into() }
pub fn auth_token_from_headers(_s: &AppState, _h: axum::http::HeaderMap) -> String { String::new() }
pub fn auth_username_from_token(_s: &AppState, _t: String) -> String { "admin".into() }
pub fn auth_role_from_token(_s: &AppState, _t: String) -> String { "admin".into() }
pub fn auth_logout_token(_s: &AppState, _h: axum::http::HeaderMap) {}
pub fn auth_header_token(_h: axum::http::HeaderMap) -> Option<String> { None }

// ── Specs helpers ───────────────────────────────────────────────────────────
pub fn specs_load(_s: &AppState, _q: Value) -> Value { Value::Null }
pub fn specs_overview_of(_s: &AppState, _q: Value) -> Value { Value::Null }
pub fn specs_drift(_s: &AppState, _q: Value) -> Value { Value::Null }
pub fn specs_rebuild(_s: &AppState, _q: Value) -> Value { Value::Null }
pub fn specs_related_of(_s: &AppState, _q: Value, _p: axum::extract::Path<String>) -> Value { Value::Null }
pub fn specs_upsert_of(_s: &AppState, _q: Value, _b: Value) -> Value { Value::Null }
pub fn specs_transition_of(_s: &AppState, _q: Value, _b: Value) -> String { "done".into() }
pub fn specs_delete_of(_s: &AppState, _q: Value, _p: axum::extract::Path<(String, String)>) -> String { String::new() }
pub fn specs_read(_section: String) -> String { String::new() }
pub fn specs_list() -> String { String::new() }
pub fn specs_write(_section: String, _content: String) -> String { "ok".into() }
pub fn specs_update(_action: String, _section: String, _args: Value) -> String { "ok".into() }
pub fn specs_write_goals(_goals: String) -> String { "ok".into() }

// ── Meta helpers ────────────────────────────────────────────────────────────
pub fn professions_list() -> Vec<Value> { vec![] }
pub fn config_build() -> Value { Value::Null }
pub fn modes_all() -> Vec<Value> { vec![] }
pub fn skills_all() -> Vec<String> { vec![] }
pub fn roles_all() -> Vec<Value> { vec![] }
pub fn role_get(_p: axum::extract::Path<String>) -> Value { Value::Null }
pub fn role_save_of(_p: axum::extract::Path<String>, _b: Value) {}
pub fn role_delete_of(_p: axum::extract::Path<String>) {}
pub fn role_name(_r: Arc<()>) -> String { String::new() }
pub fn role_system_prompt(_r: Arc<()>) -> String { String::new() }
pub fn role_model(_r: Arc<()>) -> String { String::new() }
pub fn role_model_tier(_r: Arc<()>) -> String { "Mid".into() }
pub fn role_temperature(_r: Arc<()>) -> f64 { 0.7 }
pub fn role_max_turns(_r: Arc<()>) -> u32 { 10 }
pub fn role_allowed_tools(_r: Arc<()>) -> Vec<String> { vec![] }
pub fn role_memory_limit(_r: Arc<()>) -> Option<u32> { None }
pub fn role_allowed_tiers(_r: Arc<()>) -> Vec<String> { vec![] }
pub fn role_token_budget(_r: Arc<()>) -> Option<u64> { None }
pub fn role_skills(_r: Arc<()>) -> Vec<String> { vec![] }

// ── App config / harness ────────────────────────────────────────────────────
pub fn app_config_load() -> Value { Value::Null }
pub fn app_config_write(_b: Value) -> Value { Value::Null }
pub fn app_config_effective_daemon_url(_cfg: Value) -> String { "http://127.0.0.1:17654".into() }
pub fn harness_list(_p: axum::extract::Path<String>) -> Value { Value::Null }
pub fn harness_save(_p: axum::extract::Path<(String, String)>, _b: Value) {}
pub fn harness_delete(_p: axum::extract::Path<(String, String)>) {}
pub fn harness_name_from_path(_p: axum::extract::Path<(String, String)>) -> String { String::new() }

// ── Chats helpers ───────────────────────────────────────────────────────────
pub fn chats_create(_s: &AppState, _q: Value, _b: Value) -> Value { Value::Null }
pub fn chats_list(_s: &AppState, _q: Value) -> Vec<Value> { vec![] }
pub fn chats_get(_s: &AppState, _q: Value, _p: axum::extract::Path<String>) -> Value { Value::Null }
pub fn chats_rename(_s: &AppState, _q: Value, _p: axum::extract::Path<String>, _b: Value) -> Value { Value::Null }
pub fn chats_delete(_s: &AppState, _q: Value, _p: axum::extract::Path<String>) {}
pub fn chats_delete_all(_s: &AppState, _q: Value) -> u32 { 0 }
pub fn chats_message(_s: &AppState, _q: Value, _p: axum::extract::Path<String>, _b: Value) {}
pub fn chats_approve(_s: &AppState, _q: Value, _p: axum::extract::Path<(String, u32)>) -> bool { true }
pub fn chats_reject(_s: &AppState, _q: Value, _p: axum::extract::Path<(String, u32)>) -> Value { Value::Null }
pub fn chats_reject_all(_s: &AppState, _q: Value, _p: axum::extract::Path<String>) -> Value { Value::Null }

// ── Conversations helpers ───────────────────────────────────────────────────
pub fn conversations_list(_s: &AppState, _q: Value) -> Value { Value::Null }
pub fn conversations_get(_s: &AppState, _q: Value, _p: axum::extract::Path<String>) -> Value { Value::Null }
pub fn conversations_delete(_s: &AppState, _q: Value, _p: axum::extract::Path<String>) {}
pub fn conversations_rename(_s: &AppState, _q: Value, _p: axum::extract::Path<String>, _b: Value) -> Value { Value::Null }
pub fn conversations_subscribe(_s: &AppState, _q: Value) {}
pub fn conv_event_matches(_ev: Value, _id: String) -> bool { false }
pub fn conv_event_id(_ev: Value) -> String { String::new() }
pub fn conv_event_turn(_ev: Value) -> Option<String> { None }
pub fn conv_event_status(_ev: Value) -> Option<String> { None }

// ── Workspace helpers ───────────────────────────────────────────────────────
pub fn workspace_list_all(_s: &AppState) -> Vec<Value> { vec![] }
pub fn workspace_open_of(_s: &AppState, _b: Value) -> Value { Value::Null }
pub fn workspace_status_of(_s: &AppState, _q: Value) -> Value { Value::Null }
pub fn workspace_browse_of(_q: Value) -> Vec<Value> { vec![] }
pub fn workspace_initialize_of(_s: &AppState, _q: Value) {}

// ── Workflow helpers ────────────────────────────────────────────────────────
pub fn workflows_builtin_names() -> Vec<String> { vec!["feature-dev".into()] }
pub fn wf_run(_s: &AppState, _q: Value, _b: Value) -> Value { Value::Null }
pub fn wf_run_with_progress(_s: &AppState, _q: Value, _b: Value, _sink: Arc<()>) {}

// ── Orch helpers ────────────────────────────────────────────────────────────
pub fn orch_spawn_relay(_task: String, _args: Value) -> String { "(stub) relay".into() }
pub fn orch_dispatch(_task: String, _to: String) -> String { "(stub) dispatch".into() }
pub fn orch_bring_in(_query: String) -> String { "(stub) bring_in".into() }

// ── Drive helpers ───────────────────────────────────────────────────────────
pub fn drive_set_root(_ws_id: String) {}
pub fn drive_clear_root() {}
pub fn relay_advance(_ws_id: String, _run_id: String) -> Value { Value::Null }
pub fn relay_publish(_run_id: String, _result: Value) {}
pub fn advance_is_none(_r: Value) -> bool { true }
pub fn advance_kind(_r: Value) -> String { "completed".into() }
pub fn advance_role_id(_r: Value) -> String { String::new() }
pub fn relay_submit_error(_run_id: String, _role: String, _err: String) {}
pub fn relay_step_context(_ws_id: String, _run_id: String) -> String { String::new() }
pub fn factory_build_agent(_s: Arc<AppState>, _ws_id: String, _run_id: String, _role: String) -> Agent { Agent }
pub fn agent_run_stream_with_sink(_agent: Agent, _task: String, _sink: Arc<()>, _cancel: Arc<std::sync::atomic::AtomicBool>) -> impl std::future::Future<Output = Result<Value, String>> {
    async { Ok(Value::Null) }
}
pub fn agent_run(_s: &AppState, _q: Value, _b: Value) -> impl std::future::Future<Output = Value> {
    async { Value::Null }
}
pub fn drive_accumulated(_ws_id: String, _run_id: String) -> String { String::new() }
pub fn drive_finalize_output(_output: String, _r: Value) -> String { _output }
pub fn drive_submit_handoff(_ws_id: String, _run_id: String, _role: String, _output: String, _r2: Value) {}
pub fn drive_handle_stream_event(_ws_id: String, _run_id: String, _role: String, _ev: Value) {}

// ── Lib helpers ─────────────────────────────────────────────────────────────
pub fn agent_register_shared(_agent: Agent, _tool: Arc<()>) {}
pub fn agent_register_skill_tool(_agent: Agent) {}
pub fn agent_with_context_file(_agent: Agent, _path: String) {}
pub fn agent_with_history(_agent: Agent, _history: String) {}
pub fn build_agent_with_context(_mode: AgentMode, _client: Arc<()>, _ctx: Option<()>) -> Agent { Agent }
pub fn mode_tools_contains(_mode: AgentMode, _name: String) -> bool { true }
pub fn resolve_role(_spec: String) -> Result<Arc<()>, String> { Ok(Arc::new(())) }
pub fn registry_resolve(_reg: RoleRegistry, _spec: String) -> Option<Arc<()>> { None }
pub fn load_builtin(_spec: String) -> Option<Arc<()>> { None }
pub fn load_role(_content: String) -> Result<Arc<()>, String> { Err("stub".into()) }
pub fn read_at_file(_spec: String) -> String { String::new() }
pub fn find_context_file() -> Option<String> { None }
pub fn find_ctx_upward(_cwd: String) -> Option<String> { None }
pub fn current_dir() -> String { ".".into() }
pub fn ctx_is_some(_ctx: Option<String>) -> bool { false }
pub fn ctx_unwrap(_ctx: Option<String>) -> String { String::new() }
pub fn handoff_render(_h: String) -> String { String::new() }

// ── Serve helpers ───────────────────────────────────────────────────────────
pub fn serve_init_state(_client: Arc<()>) -> AppState { AppState }
pub fn serve_build_static() -> () {}
pub fn serve_build_cors() -> () {}
pub fn serve_build_app(_state: AppState, _static: (), _cors: ()) -> () {}
pub fn serve_listen(_addr: String, _app: ()) -> impl std::future::Future<Output = ()> {
    async {}
}

// ── Stream event helpers ────────────────────────────────────────────────────
pub fn stream_event_map(_ev: Value) -> Value { Value::Null }
pub fn workflow_event_map(_ev: Value) -> Value { Value::Null }

// ── chat_run_stream / agent_run_stream ──────────────────────────────────────
pub fn chat_run_stream(_s: Arc<AppState>, _q: Value, _p: axum::extract::Path<String>, _sink: Arc<()>) {}
pub fn agent_run_stream(_s: &AppState, _q: Value, _b: Value, _sink: Arc<()>) {}
pub fn step_err_is_err(e: String) -> bool { e.len() > 0 }
