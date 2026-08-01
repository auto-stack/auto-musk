//! extern_impl.rs — glue layer stubs for a2r-transpiled .at files.
#![allow(dead_code, unused_variables, non_upper_case_globals, clippy::too_many_arguments)]

use serde_json::Value;
use std::sync::Arc;
use axum::extract::{State, Query, Path};
use axum::response::{Response, IntoResponse};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;

pub use crate::specs::*;
pub use crate::auth::{AuthStore, UserInfo, Session};
pub use crate::mode::{AgentMode, ModeRegistry};
pub use crate::tool_safety::{CommandTier, classify_command};
pub use crate::conversation::*;
pub use crate::chats::*;
pub use crate::server::AppState;
pub use crate::tool_context::ToolContext;
pub use crate::workflow;
pub use crate::workspace::{WorkspaceRegistry, WorkspaceStores};
pub use auto_ai_agent::{
    Agent, Client, Tool, ToolError, Role, ModelTier, SkillRegistry, SkillTool,
    load_builtin, load_role, parse_at_workflow, Workflow,
};
pub use auto_ai_agent::orchestration::*;

pub fn parse_json(s: &str) -> Value { serde_json::from_str(s).unwrap_or(Value::Null) }
pub fn value_get_str(v: &Value, k: &str) -> String { v.get(k).and_then(|s| s.as_str()).unwrap_or("").to_string() }
pub fn value_get_bool(v: &Value, k: &str) -> bool { v.get(k).and_then(|b| b.as_bool()).unwrap_or(false) }
pub fn value_get_array(v: &Value, k: &str) -> Value { v.get(k).cloned().unwrap_or(Value::Array(vec![])) }
pub fn null_value() -> Value { Value::Null }
pub fn new_id(_: u32) -> String { format!("{:016x}", rand::random::<u64>()) }
pub fn random_hex(n: u32) -> String { format!("{:0width$x}", rand::random::<u64>(), width = (n as usize) * 2) }
pub fn hash_password(p: &str, s: &str) -> String {
    use sha2::Digest; let mut h = sha2::Sha256::new(); h.update(s.as_bytes()); h.update(p.as_bytes()); hex::encode(h.finalize())
}

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

pub fn auth_login_role<T>(_s: T, _u: String, _p: String) -> String { "admin".into() }
pub fn auth_login_token<T>(_s: T, _u: String, _p: String) -> String { "stub".into() }
pub fn auth_token_from_headers<T>(_s: T, _h: axum::http::HeaderMap) -> String { String::new() }
pub fn auth_username_from_token<T>(_s: T, _t: String) -> String { "admin".into() }
pub fn auth_role_from_token<T>(_s: T, _t: String) -> String { "admin".into() }
pub fn auth_logout_token<T>(_s: T, _h: axum::http::HeaderMap) {}
pub fn auth_header_token(_h: axum::http::HeaderMap) -> Option<String> { None }
pub fn specs_load<T,U>(_s: T, _q: U) -> Value { Value::Null }
pub fn specs_overview_of<T,U>(_s: T, _q: U) -> Value { Value::Null }
pub fn specs_drift<T,U>(_s: T, _q: U) -> Value { Value::Null }
pub fn specs_rebuild<T,U>(_s: T, _q: U) -> Value { Value::Null }
pub fn specs_related_of<T,U,V>(_s: T, _q: U, _p: V) -> Value { Value::Null }
pub fn specs_upsert_of<T,U,V>(_s: T, _q: U, _b: V) -> Value { Value::Null }
pub fn specs_transition_of<T,U,V>(_s: T, _q: U, _b: V) -> String { "done".into() }
pub fn specs_delete_of<T,U,V>(_s: T, _q: U, _p: V) -> String { String::new() }
pub fn specs_read(_s: String) -> String { String::new() }
pub fn specs_list() -> String { String::new() }
pub fn specs_write(_s: String, _c: String) -> String { "ok".into() }
pub fn specs_update(_a: String, _s: String, _v: Value) -> String { "ok".into() }
pub fn specs_write_goals(_g: String) -> String { "ok".into() }
pub fn professions_list() -> Vec<Value> { vec![] }
pub fn config_build() -> Value { Value::Null }
pub fn modes_all() -> Vec<Value> { vec![] }
pub fn skills_all() -> Vec<String> { vec![] }
pub fn roles_all() -> Vec<Value> { vec![] }
pub fn role_get<T>(_p: T) -> Value { Value::Null }
pub fn role_save_of<T,U>(_p: T, _b: U) {}
pub fn role_delete_of<T>(_p: T) {}
pub fn role_name<T>(_r: T) -> String { String::new() }
pub fn role_system_prompt<T>(_r: T) -> String { String::new() }
pub fn role_model<T>(_r: T) -> String { String::new() }
pub fn role_model_tier<T>(_r: T) -> ModelTier { ModelTier::Mid }
pub fn role_temperature<T>(_r: T) -> f64 { 0.7 }
pub fn role_max_turns<T>(_r: T) -> u32 { 10 }
pub fn role_allowed_tools<T>(_r: T) -> Vec<String> { vec![] }
pub fn role_memory_limit<T>(_r: T) -> Option<u32> { None }
pub fn role_allowed_tiers<T>(_r: T) -> Vec<ModelTier> { vec![] }
pub fn role_token_budget<T>(_r: T) -> Option<u64> { None }
pub fn role_skills<T>(_r: T) -> Vec<String> { vec![] }
pub fn app_config_load() -> Value { Value::Null }
pub fn app_config_write<T>(_b: T) -> Value { Value::Null }
pub fn app_config_effective_daemon_url<T>(_c: T) -> String { "http://127.0.0.1:17654".into() }
pub fn harness_list<T>(_p: T) -> Value { Value::Null }
pub fn harness_save<T,U>(_p: T, _b: U) {}
pub fn harness_delete<T>(_p: T) {}
pub fn harness_name_from_path<T>(_p: T) -> String { String::new() }
pub fn chats_create<T,U,V>(_s: T, _q: U, _b: V) -> Value { Value::Null }
pub fn chats_list<T,U>(_s: T, _q: U) -> Vec<Value> { vec![] }
pub fn chats_get<T,U,V>(_s: T, _q: U, _p: V) -> Value { Value::Null }
pub fn chats_rename<T,U,V,W>(_s: T, _q: U, _p: V, _b: W) -> Value { Value::Null }
pub fn chats_delete<T,U,V>(_s: T, _q: U, _p: V) {}
pub fn chats_delete_all<T,U>(_s: T, _q: U) -> u32 { 0 }
pub fn chats_message<T,U,V,W>(_s: T, _q: U, _p: V, _b: W) {}
pub fn chats_approve<T,U,V>(_s: T, _q: U, _p: V) -> bool { true }
pub fn chats_reject<T,U,V>(_s: T, _q: U, _p: V) -> Value { Value::Null }
pub fn chats_reject_all<T,U,V>(_s: T, _q: U, _p: V) -> Value { Value::Null }
pub fn conversations_list<T,U>(_s: T, _q: U) -> Value { Value::Null }
pub fn conversations_get<T,U,V>(_s: T, _q: U, _p: V) -> Value { Value::Null }
pub fn conversations_delete<T,U,V>(_s: T, _q: U, _p: V) {}
pub fn conversations_rename<T,U,V,W>(_s: T, _q: U, _p: V, _b: W) -> Value { Value::Null }
pub fn conversations_subscribe<T,U>(_s: T, _q: U) {}
pub fn conv_event_matches(_ev: Value, _id: String) -> bool { false }
pub fn conv_event_id(_ev: Value) -> String { String::new() }
pub fn conv_event_turn(_ev: Value) -> Option<String> { None }
pub fn conv_event_status(_ev: Value) -> Option<String> { None }
pub fn workspace_list_all<T>(_s: T) -> Vec<Value> { vec![] }
pub fn workspace_open_of<T,U>(_s: T, _b: U) -> Value { Value::Null }
pub fn workspace_status_of<T,U>(_s: T, _q: U) -> Value { Value::Null }
pub fn workspace_browse_of<T>(_q: T) -> Vec<Value> { vec![] }
pub fn workspace_initialize_of<T,U>(_s: T, _q: U) {}
pub fn workflows_builtin_names() -> Vec<String> { vec!["feature-dev".into()] }
pub fn wf_run<T,U,V>(_s: T, _q: U, _b: V) -> Value { Value::Null }
pub fn wf_run_with_progress<T,U,V,W>(_s: T, _q: U, _b: V, _sink: W) {}
pub fn orch_spawn_relay(_t: String, _a: Value) -> String { "(stub)".into() }
pub fn orch_dispatch(_t: String, _to: String) -> String { "(stub)".into() }
pub fn orch_bring_in(_q: String) -> String { "(stub)".into() }
pub fn drive_set_root(_w: String) {}
pub fn drive_clear_root() {}
pub fn relay_advance(_w: String, _r: String) -> Value { Value::Null }
pub fn relay_publish(_r: String, _v: Value) {}
pub fn advance_is_none(_r: Value) -> bool { true }
pub fn advance_kind(_r: Value) -> String { "completed".into() }
pub fn advance_role_id(_r: Value) -> String { String::new() }
pub fn relay_submit_error(_r: String, _r2: String, _e: String) {}
pub fn relay_step_context(_w: String, _r: String) -> String { String::new() }
pub fn drive_accumulated(_w: String, _r: String) -> String { String::new() }
pub fn drive_finalize_output(_o: String, _r: Value) -> String { _o }
pub fn drive_submit_handoff(_w: String, _r: String, _r2: String, _o: String, _v: Value) {}
pub fn drive_handle_stream_event(_w: String, _r: String, _r2: String, _e: Value) {}
pub fn agent_register_shared(_a: Agent, _t: Arc<dyn Tool>) {}
pub fn agent_register_skill_tool(_a: Agent) {}
pub fn agent_with_context_file(_a: Agent, _p: String) -> Agent { _a }
pub fn agent_with_history(_a: Agent, _h: String) -> Agent { _a }
pub fn build_agent_with_context(_m: AgentMode, _c: Arc<dyn Client>, _ctx: Option<ToolContext>) -> Agent { Agent::new(_c, "") }
pub fn mode_tools_contains(_m: AgentMode, _n: String) -> bool { true }
pub fn resolve_role(_s: String) -> Result<Arc<dyn Role>, String> { Err("stub".into()) }
pub fn registry_resolve<T>(_r: T, _s: String) -> Option<Arc<dyn Role>> { None }
pub fn load_builtin_role(_s: String) -> Option<Arc<dyn Role>> { None }
pub fn read_at_file(_s: String) -> String { String::new() }
pub fn find_context_file() -> Option<String> { None }
pub fn find_ctx_upward(_c: String) -> Option<String> { None }
pub fn current_dir() -> String { ".".into() }
pub fn ctx_is_some<T>(_c: T) -> bool { false }
pub fn ctx_unwrap(_c: Option<String>) -> String { String::new() }
pub fn handoff_render(_h: String) -> String { String::new() }
pub fn factory_build_agent<T>(_s: T, _w: String, _r: String, _r2: String) -> Agent { Agent::new(Arc::new(NoDaemonClient), "") }
pub async fn agent_run<T,U,V>(_s: T, _q: U, _b: V) -> Value { Value::Null }
pub fn chat_run_stream<T,U,V,W>(_s: T, _q: U, _p: V, _sink: W) {}
pub fn agent_run_stream<T,U,V,W>(_s: T, _q: U, _b: V, _sink: W) {}
pub fn agent_run_stream_with_sink(_a: Agent, _t: String, _sink: Arc<()>, _c: Arc<std::sync::atomic::AtomicBool>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, String>> + Send>> {
    Box::pin(async { Ok(Value::Null) })
}
pub fn serve_init_state(_c: Arc<dyn Client>) -> AppState { unimplemented!("serve_init_state") }
pub fn serve_build_static() -> () {}
pub fn serve_build_cors() -> () {}
pub fn serve_build_app(_s: AppState, _st: (), _c: ()) -> () {}
pub async fn serve_listen(_a: String, _app: ()) {}
pub fn stream_event_map(_e: Value) -> Value { Value::Null }
pub fn workflow_event_map(_e: Value) -> Value { Value::Null }
pub fn resolve_within_project(p: &str) -> String { p.to_string() }
pub fn write_file_do(p: &str, c: &str) { let _ = std::fs::write(p, c); }
pub fn command_needs_approval(c: &str) -> bool { !c.starts_with("echo") }
pub fn run_shell_command(c: &str) -> String { format!("(stub) {}", c) }
pub fn edit_file_do(p: &str, o: &str, n: &str) -> String {
    let c = std::fs::read_to_string(p).unwrap_or_default();
    let n2 = c.replacen(o, n, 1); let _ = std::fs::write(p, n2); "ok".into()
}
pub fn batch_replace_do(p: &str, _r: Value) -> String { format!("(stub) {}", p) }
pub fn search_files(p: &str) -> String { format!("(stub) {}", p) }
pub fn list_directory(p: &str) -> String { format!("(stub) {}", p) }
pub fn list_symbols_in(p: &str) -> String { format!("(stub) {}", p) }
pub fn glob_files(p: &str) -> String { format!("(stub) {}", p) }
pub fn http_post_json(_u: String) -> impl std::future::Future<Output = Result<Value, String>> { async { Ok(Value::Null) } }
pub fn mpsc_channel() -> (std::sync::mpsc::Sender<Value>, std::sync::mpsc::Receiver<Value>) { std::sync::mpsc::channel() }
pub fn mpsc_try_send(t: &std::sync::mpsc::Sender<Value>, m: Value) { let _ = t.send(m); }
pub async fn mpsc_recv(r: &mut std::sync::mpsc::Receiver<Value>) -> Option<Value> { r.recv().ok() }
pub fn msg_is_none(m: &Option<Value>) -> bool { m.is_none() }
pub fn msg_unwrap(m: Option<Value>) -> Value { m.unwrap_or(Value::Null) }
pub fn broadcast_recv(_r: &()) -> impl std::future::Future<Output = Option<Value>> { async { None } }
pub fn path_inner(p: &axum::extract::Path<String>) -> String { p.0.clone() }
pub fn json_response<T: serde::Serialize>(_d: T) -> Response { Response::default() }
pub fn error_response<T: serde::Serialize>(_c: u16, _d: T) -> Response { Response::default() }
pub fn atomic_bool_false() -> Arc<std::sync::atomic::AtomicBool> { Arc::new(std::sync::atomic::AtomicBool::new(false)) }

struct NoDaemonClient;
#[async_trait::async_trait]
impl Client for NoDaemonClient {
    async fn complete(&self, _req: auto_ai_agent::CompletionRequest) -> Result<auto_ai_agent::CompletionResponse, auto_ai_agent::ClientError> {
        Err(auto_ai_agent::ClientError::DaemonUnavailable("stub".into()))
    }
}
