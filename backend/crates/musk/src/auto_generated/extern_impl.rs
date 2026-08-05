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
pub use crate::workspace::{WorkspaceRegistry, WorkspaceStores};
pub use auto_ai_agent::{
    Agent, Client, Tool, ToolError, Role, ModelTier, SkillRegistry, SkillTool,
    load_builtin, load_role,
};
pub use auto_ai_agent::orchestration::*;

// a2r-generated DTO structs (in sibling modules), used as precise stub return types.
use super::server::{
    DriftResult, RelatedInfo, ProfessionItem, ModeItem, RoleItem, RoleDetail,
    ConfigOverview, AppConfigResp, WorkspaceMeta, WorkspaceResp, WorkspaceStatusResp,
    BrowseEntry, SessionResp, SessionSummary, ApiError,
};
use super::server_stream::{
    WorkflowRunResponse, WorkflowEventDto, SseEventDto, RunResponse,
};

pub fn parse_json(s: &str) -> Value { serde_json::from_str(s).unwrap_or(Value::Null) }

// Phase C(计划 018 §12 C3):ag server 的状态码模型 —— handler 返回 ~Response,
// 经这两个 helper 构建带状态码的响应(补齐"状态码在外壳层处理"的缺失)。
pub fn ok_response<T: serde::Serialize>(v: T) -> axum::response::Response {
    axum::Json(v).into_response()
}
pub fn err_response(msg: &str, code: u16) -> axum::response::Response {
    let status = axum::http::StatusCode::from_u16(code)
        .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    (status, axum::Json(ApiError { error: msg.to_string() })).into_response()
}
pub fn value_get_str(v: &Value, k: &str) -> String { v.get(k).and_then(|s| s.as_str()).unwrap_or("").to_string() }
pub fn value_get_bool(v: &Value, k: &str) -> bool { v.get(k).and_then(|b| b.as_bool()).unwrap_or(false) }
pub fn value_get_array(v: &Value, k: &str) -> Value { v.get(k).cloned().unwrap_or(Value::Array(vec![])) }
pub fn null_value() -> Value { Value::Null }
pub fn new_id(_: u32) -> String { format!("{:016x}", rand::random::<u64>()) }
pub fn random_hex(n: u32) -> String { format!("{:0width$x}", rand::random::<u64>(), width = (n as usize) * 2) }
pub fn hash_password(p: &str, s: &str) -> String { use sha2::Digest; let mut h = sha2::Sha256::new(); h.update(s.as_bytes()); h.update(p.as_bytes()); hex::encode(h.finalize()) }

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

// C 阶段:auth 真实委托(走 AppState.auth —— a2r 转译 AuthStore, 见 ①)。
// 单次 login 返回 (token, role), 避免 split 版两次 login 产生两个 session。
pub fn auth_login_result(s: &State<AppState>, u: String, p: String) -> (String, String) {
    match s.0.auth.login(&u, &p) {
        Some(ses) => {
            let role = s
                .0
                .auth
                .session_user(&ses.token)
                .map(|u| u.role.to_string())
                .unwrap_or_default();
            (ses.token, role)
        }
        None => (String::new(), String::new()),
    }
}
pub fn auth_token_from_headers(_s: &State<AppState>, h: axum::http::HeaderMap) -> String {
    bearer_from(&h).unwrap_or_default()
}
pub fn auth_username_from_token(s: &State<AppState>, t: &str) -> String {
    s.0.auth.session_user(t).map(|u| u.username).unwrap_or_default()
}
pub fn auth_role_from_token(s: &State<AppState>, t: &str) -> String {
    s.0.auth.session_user(t).map(|u| u.role.to_string()).unwrap_or_default()
}
pub fn auth_logout_token(s: &State<AppState>, h: axum::http::HeaderMap) {
    if let Some(t) = bearer_from(&h) {
        s.0.auth.logout(&t);
    }
}
pub fn auth_header_token(h: axum::http::HeaderMap) -> Option<String> {
    bearer_from(&h)
}

/// Extract a bearer token from `Authorization: Bearer <token>`.
fn bearer_from(headers: &axum::http::HeaderMap) -> Option<String> {
    let h = headers.get("authorization")?.to_str().ok()?;
    let t = h.strip_prefix("Bearer ")?.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}
// C1 重新评估(plan 018 §11 ②):specs/chats 委托路径 PoC —— extern_impl 从
// 泛型 fake stub 改为走 `s.0.registry` 的真实 workspace stores(与 auth 委托
// 同模式),证明"换 store 类型(41 处级联)"非必需。调用点仍由 extern_sigs.at
// 的 `@T` 驱动 `&s`;此处签名改为具体类型(手写 impl,可任意定)。
pub fn specs_load(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.specs.load() {
        Ok(doc) => serde_json::to_value(doc).unwrap_or(Value::Null),
        Err(_) => Value::Null,
    }
}
/// ② 委托:overview = load + rebuild + derive + doc.overview()(与 hw specs_overview 一致)。
pub fn specs_overview_of(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.specs.load() {
        Ok(mut doc) => {
            doc.rebuild_relations();
            doc.derive_statuses();
            serde_json::to_value(doc.overview()).unwrap_or(Value::Null)
        }
        Err(_) => Value::Null,
    }
}
/// ② 委托:drift = load + drift_check(与 hw specs_drift_check 一致)。
pub fn specs_drift(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>) -> DriftResult {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.specs.load() {
        Ok(doc) => match ws.specs.drift_check(&doc) {
            Ok((disk_version, drifted)) => DriftResult {
                memory_version: doc.version,
                disk_version,
                drifted,
            },
            Err(_) => DriftResult { memory_version: doc.version, disk_version: 0, drifted: false },
        },
        Err(_) => DriftResult { memory_version: 0, disk_version: 0, drifted: false },
    }
}
/// ② 委托:rebuild = load + rebuild + derive + save + 返回 doc(与 hw 一致)。
pub fn specs_rebuild(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let mut doc = match ws.specs.load() {
        Ok(d) => d,
        Err(_) => return Value::Null,
    };
    doc.rebuild_relations();
    doc.derive_statuses();
    if ws.specs.save(&doc).is_err() {
        return Value::Null;
    }
    serde_json::to_value(doc).unwrap_or(Value::Null)
}
/// ② 委托:related = load + rebuild + 找 item 返回 depends_on/related(与 hw 一致)。
pub fn specs_related_of(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>) -> RelatedInfo {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let item_id = p.0.clone();
    match ws.specs.load() {
        Ok(mut doc) => {
            doc.rebuild_relations();
            for section in &doc.sections {
                if let Some(item) = section.items.iter().find(|i| i.id == item_id) {
                    return RelatedInfo {
                        item_id,
                        depends_on: item.depends_on.clone(),
                        related: item.related.clone(),
                    };
                }
            }
            RelatedInfo { item_id, depends_on: vec![], related: vec![] }
        }
        Err(_) => RelatedInfo { item_id, depends_on: vec![], related: vec![] },
    }
}
/// ② 委托:upsert = load + upsert_item + save + 返回 doc。ag 请求体字段是
/// section / item{id,title,content,status}(简化版),转换为 hw SpecItem。
pub fn specs_upsert_of(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, b: Json<crate::auto_generated::server::SpecsUpsertRequest>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let mut doc = match ws.specs.load() {
        Ok(d) => d,
        Err(_) => return Value::Null,
    };
    let mut item = crate::specs::SpecItem::new(b.item.id.clone(), b.item.title.clone());
    item.content = b.item.content.clone();
    item.status = crate::specs::SpecStatus::from_str_lossy(&b.item.status);
    match ws.specs.upsert_item(&mut doc, &b.section, item) {
        Ok(_) => {
            if ws.specs.save(&doc).is_err() {
                return Value::Null;
            }
            serde_json::to_value(doc).unwrap_or(Value::Null)
        }
        Err(_) => Value::Null,
    }
}
/// ② 委托:transition = load + transition_item + save + 返回 new_status 字符串。
pub fn specs_transition_of(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, b: Json<crate::auto_generated::server::SpecsTransitionRequest>) -> String {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let new_status = crate::specs::SpecStatus::from_str_lossy(&b.new_status);
    let mut doc = match ws.specs.load() {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    match ws.specs.transition_item(&mut doc, &b.section, &b.item_id, new_status) {
        Ok(_) => {
            let _ = ws.specs.save(&doc);
            b.new_status.clone()
        }
        Err(_) => String::new(),
    }
}
/// ② 委托:delete = load + delete_item + save + 返回 item_id 字符串。
pub fn specs_delete_of(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<(String, String)>) -> String {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let (section_id, item_id) = p.0;
    let mut doc = match ws.specs.load() {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    match ws.specs.delete_item(&mut doc, &section_id, &item_id) {
        Ok(true) => {
            let _ = ws.specs.save(&doc);
            item_id
        }
        Ok(false) | Err(_) => String::new(),
    }
}
pub fn specs_read(_s: String) -> String { String::new() }
pub fn specs_list() -> String { String::new() }
pub fn specs_write(_s: String, _c: String) -> String { "ok".into() }
pub fn specs_update(_a: String, _s: String, _v: Value) -> String { "ok".into() }
pub fn specs_write_goals(_g: String) -> String { "ok".into() }
pub fn professions_list() -> Vec<ProfessionItem> { vec![] }
pub fn config_build() -> ConfigOverview { ConfigOverview { modes: vec![], professions: vec![], skills: vec![] } }
pub fn modes_all() -> Vec<ModeItem> { vec![] }
pub fn skills_all() -> Vec<String> { vec![] }
pub fn roles_all() -> Vec<RoleItem> { vec![] }
pub fn role_get<T>(_p: &T) -> RoleDetail { RoleDetail { name: String::new(), tier: String::new(), model: String::new(), temperature: 0.7, system_prompt: String::new() } }
pub fn role_save_of<T,U>(_p: &T, _b: U) {}
pub fn role_delete_of<T>(_p: &T) {}
pub fn role_name<T>(_r: T) -> String { String::new() }
pub fn role_system_prompt<T>(_r: &T) -> String { String::new() }
pub fn role_model<T>(_r: T) -> String { String::new() }
pub fn role_model_tier<T>(_r: T) -> ModelTier { ModelTier::Mid }
pub fn role_temperature<T>(_r: T) -> f64 { 0.7 }
pub fn role_max_turns<T>(_r: T) -> u32 { 10 }
pub fn role_allowed_tools<T>(_r: T) -> Vec<String> { vec![] }
pub fn role_memory_limit<T>(_r: T) -> Option<u32> { None }
pub fn role_allowed_tiers<T>(_r: T) -> Vec<ModelTier> { vec![] }
pub fn role_token_budget<T>(_r: T) -> Option<u64> { None }
pub fn role_skills<T>(_r: T) -> Vec<String> { vec![] }
pub fn app_config_load() -> AppConfigResp { AppConfigResp { daemon_url: String::new(), default_mode: String::new() } }
pub fn app_config_write<T>(_b: T) -> AppConfigResp { AppConfigResp { daemon_url: String::new(), default_mode: String::new() } }
pub fn app_config_effective_daemon_url<T>(_c: T) -> String { "http://127.0.0.1:17654".into() }
pub fn harness_list<T>(_p: &T) -> Value { Value::Null }
pub fn harness_save<T,U>(_p: &T, _b: U) {}
pub fn harness_delete<T>(_p: &T) {}
pub fn harness_name_from_path<T>(_p: &T) -> String { String::new() }
// C1 重新评估(plan 018 §11 ②):specs/chats 委托路径 —— extern_impl 从泛型
// fake stub 改为走 `s.0.registry` 的真实 workspace stores(与 auth 委托同模式)。
// chats 返回完整 session 的 Value(与 hw 的 `{"session": {...}}` wire 形状一致);
// ag server 的 handler 负责包装(session/sessions 键)。conversation 双写与 hw
// 一致(create/rename/delete/message),保证两个 store 保持链接。
pub fn chats_create(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, b: Json<crate::auto_generated::server::ChatCreateBody>) -> Value {
    let ws_id = q.workspace.clone().unwrap_or_default();
    let ws = s.0.registry.get(&ws_id);
    let mode = b.mode.clone().unwrap_or_else(|| "superpowers".into());
    match ws.chats.create(&mode, Some(ws_id.clone())) {
        Ok(session) => {
            let _ = ws.conversations.create_with_id(
                session.id.clone(),
                crate::conversation::ConversationKind::Chat,
                ws_id.clone(),
                crate::conversation::Driver::Human,
                Some(mode),
                Some(session.name.clone()),
            );
            serde_json::to_value(session).unwrap_or(Value::Null)
        }
        Err(_) => Value::Null,
    }
}
pub fn chats_list(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    serde_json::to_value(ws.chats.list()).unwrap_or(Value::Null)
}
pub fn chats_get(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.chats.get(&p.0) {
        Some(session) => serde_json::to_value(session).unwrap_or(Value::Null),
        None => Value::Null,
    }
}
pub fn chats_rename(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>, b: Json<crate::auto_generated::server::ChatRenameBody>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.chats.rename(&p.0, &b.name) {
        Ok(Some(session)) => {
            let _ = ws.conversations.rename(&p.0, &b.name);
            serde_json::to_value(session).unwrap_or(Value::Null)
        }
        _ => Value::Null,
    }
}
pub fn chats_delete(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: &Path<String>) {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    if ws.chats.delete(&p.0).unwrap_or(false) {
        let _ = ws.conversations.delete(&p.0);
    }
}
pub fn chats_delete_all(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>) -> u32 {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let count = ws.chats.list().len() as u32;
    if ws.chats.delete_all().is_ok() {
        ws.conversations.delete_all();
    }
    count
}
pub fn chats_message(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>, b: Json<crate::auto_generated::server::ChatMessageBody>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let msg = crate::chats::ChatMessage::user(b.content.clone());
    match ws.chats.append_message(&p.0, msg.clone()) {
        Ok(Some(session)) => {
            let seq_base = ws
                .conversations
                .get(&p.0)
                .map(|c| c.turns.len())
                .unwrap_or(0);
            for turn in crate::conversation::chat_message_to_turns(&msg, seq_base) {
                let _ = ws.conversations.append_turn(&p.0, turn);
            }
            serde_json::to_value(serde_json::json!({ "session": session, "queued": msg }))
                .unwrap_or(Value::Null)
        }
        _ => Value::Null,
    }
}
pub fn chats_approve(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<(String, u32)>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let (id, index) = p.0;
    match ws.chats.approve_spec_change(&id, index as usize, &ws.specs) {
        Ok(Some((change, session))) => serde_json::to_value(serde_json::json!({
            "applied": change,
            "session": session,
        }))
        .unwrap_or(Value::Null),
        _ => Value::Null,
    }
}
pub fn chats_reject(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<(String, u32)>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let (id, index) = p.0;
    match ws.chats.reject_spec_change(&id, index as usize) {
        Ok(Some(session)) => serde_json::to_value(session).unwrap_or(Value::Null),
        _ => Value::Null,
    }
}
pub fn chats_reject_all(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.chats.reject_all_spec_changes(&p.0) {
        Ok(Some(session)) => serde_json::to_value(session).unwrap_or(Value::Null),
        _ => Value::Null,
    }
}
pub fn conversations_list<T,U>(_s: &T, _q: U) -> Value { Value::Null }
pub fn conversations_get<T,U,V>(_s: &T, _q: U, _p: V) -> Value { Value::Null }
pub fn conversations_delete<T,U,V>(_s: &T, _q: U, _p: &V) {}
pub fn conversations_rename<T,U,V,W>(_s: &T, _q: U, _p: V, _b: W) -> Value { Value::Null }
pub fn conversations_subscribe<T,U>(_s: &T, _q: U) -> Value { Value::Null }
pub fn conv_event_matches(_ev: &Value, _id: &str) -> bool { false }
pub fn conv_event_id(_ev: &Value) -> String { String::new() }
pub fn conv_event_turn(_ev: &Value) -> Option<String> { None }
pub fn conv_event_status(_ev: &Value) -> Option<String> { None }
/// ② workspace 委托(与 specs/chats 同模式):走 s.0.registry 真实逻辑,返回
/// 完整 metas / Value 供 ag handler 包装,wire 形状与 hw 一致。
pub fn workspace_list_all(s: &State<AppState>) -> Value {
    serde_json::to_value(s.0.registry.list()).unwrap_or(Value::Null)
}
pub fn workspace_open_of(s: &State<AppState>, b: Json<crate::auto_generated::server::OpenWorkspaceBody>) -> Value {
    let meta = s.0.registry.open(&b.path);
    s.0.registry.touch(&meta.id);
    serde_json::to_value(meta).unwrap_or(Value::Null)
}
pub fn workspace_status_of(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>) -> Value {
    let hw_q = crate::workspace::WorkspaceQuery { workspace: q.workspace.clone() };
    let ws_id = hw_q.id_or_default(&s.0.registry);
    let mut meta = s.0.registry.list().into_iter().find(|m| m.id == ws_id);
    match meta.as_mut() {
        Some(m) => {
            m.is_empty = crate::workspace::is_workspace_empty(std::path::Path::new(&m.path));
            serde_json::to_value(serde_json::json!({
                "workspace": m,
                "root_exists": std::path::Path::new(&m.path).exists(),
            }))
            .unwrap_or(Value::Null)
        }
        None => Value::Null,
    }
}
pub fn workspace_browse_of(q: &Query<crate::auto_generated::server::BrowseQuery>) -> Value {
    let base = match &q.path {
        Some(p) if !p.is_empty() => p.clone(),
        _ => ".".to_string(),
    };
    let mut entries: Vec<serde_json::Value> = Vec::new();
    if let Ok(dir) = std::fs::read_dir(&base) {
        for e in dir.flatten() {
            if e.path().is_dir() {
                let name = e.file_name().to_string_lossy().to_string();
                if !name.starts_with('.') {
                    entries.push(serde_json::json!({
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
    serde_json::to_value(serde_json::json!({ "entries": entries, "parent": parent }))
        .unwrap_or(Value::Null)
}
pub fn workspace_initialize_of(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>) -> Value {
    let hw_q = crate::workspace::WorkspaceQuery { workspace: q.workspace.clone() };
    let ws_id = hw_q.id_or_default(&s.0.registry);
    let ws = s.0.registry.get(&ws_id);
    let marker = ws.root.join(".autoos").join("initialized");
    match std::fs::write(&marker, b"1") {
        Ok(_) => serde_json::to_value(serde_json::json!({
            "status": "initialized",
            "workspace": ws_id,
        }))
        .unwrap_or(Value::Null),
        Err(_) => Value::Null,
    }
}
pub fn workflows_builtin_names() -> Vec<String> { vec!["feature-dev".into()] }
pub async fn wf_run<T,U,V>(_s: &T, _q: U, _b: V) -> WorkflowRunResponse { WorkflowRunResponse { steps: std::collections::HashMap::new(), outputs: std::collections::HashMap::new() } }
pub async fn wf_run_with_progress<T,U,V,W>(_s: &T, _q: U, _b: V, _sink: W) {}
pub fn orch_spawn_relay(_t: String, _a: Value) -> String { "(stub)".into() }
pub fn orch_dispatch(_t: String, _to: String) -> String { "(stub)".into() }
pub fn orch_bring_in(_q: String) -> String { "(stub)".into() }
pub fn drive_set_root(_w: &str) {}
pub fn drive_clear_root() {}
pub fn relay_advance(_w: &str, _r: &str) -> Value { Value::Null }
pub fn relay_publish(_r: &str, _v: &Value) {}
pub fn advance_is_none(_r: &Value) -> bool { true }
pub fn advance_kind(_r: &Value) -> String { "completed".into() }
pub fn advance_role_id(_r: &Value) -> String { String::new() }
pub fn relay_submit_error(_r: &str, _r2: &str, _e: &Result<String, String>) {}
pub fn relay_step_context(_w: &str, _r: &str) -> String { String::new() }
pub async fn factory_build_agent<T>(_s: &T, _w: &str, _r: &str, _r2: &str) -> Agent {
    Agent::new(StubRole, Arc::new(NoDaemonClient) as Arc<dyn Client>)
}
pub fn drive_accumulated(_w: &str, _r: &str) -> String { String::new() }
pub fn drive_finalize_output(_o: String, _r: &Value) -> String { _o }
pub fn drive_submit_handoff(_w: &str, _r: &str, _r2: &str, _o: &str, _v: &Value) {}
pub fn drive_handle_stream_event(_w: &str, _r: &str, _r2: &str, _e: i32) {}
pub fn agent_register_shared(_a: &Agent, _t: Arc<dyn Tool>) {}
pub fn agent_register_skill_tool(_a: &Agent) {}
pub fn agent_with_context_file(_a: Agent, _p: &str) -> Agent { _a }
pub fn agent_with_history(_a: Agent, _h: &str) -> Agent { _a }
pub fn build_agent_with_context(_m: AgentMode, _c: Arc<dyn Client>, _ctx: Option<ToolContext>) -> Agent { Agent::new(StubRole, _c) }
pub fn mode_tools_contains(_m: &AgentMode, _n: &str) -> bool { true }
pub fn resolve_role(_s: String) -> Result<Arc<dyn Role>, String> { Err("stub".into()) }
pub fn registry_resolve<T>(_r: T, _s: &str) -> Option<Arc<dyn Role>> { None }
pub fn load_builtin_role(_s: &str) -> Option<Arc<dyn Role>> { None }
pub fn read_at_file(_s: &str) -> String { String::new() }
pub fn find_context_file() -> Option<String> { None }
pub fn find_ctx_upward(_c: &str) -> Option<String> { None }
pub fn current_dir() -> String { ".".into() }
pub fn ctx_is_some<T>(_c: &T) -> bool { false }
pub fn ctx_unwrap(_c: Option<String>) -> String { String::new() }
pub fn handoff_render(_h: String) -> String { String::new() }
pub async fn agent_run<T,U,V>(_s: &T, _q: U, _b: V) -> RunResponse { RunResponse { output: String::new(), tool_calls: vec![] } }
pub async fn chat_run_stream<T,U,V,W>(_s: &T, _q: U, _p: V, _sink: W) {}
pub async fn agent_run_stream<T,U,V,W>(_s: &T, _q: U, _b: V, _sink: W) {}
pub fn agent_run_stream_with_sink<W: Send + Sync + 'static>(_a: Agent, _t: String, _sink: Arc<W>, _c: Arc<std::sync::atomic::AtomicBool>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, String>> + Send>> {
    Box::pin(async { Ok(Value::Null) })
}
pub fn serve_init_state(_c: Arc<dyn Client>) -> AppState { unimplemented!("serve_init_state") }
pub fn serve_build_static() -> () {}
pub fn serve_build_cors() -> () {}
pub fn serve_build_app(_s: AppState, _st: (), _c: ()) -> () {}
pub async fn serve_listen(_a: &str, _app: ()) {}
pub fn stream_event_map(_e: Option<Value>) -> SseEventDto { SseEventDto::Cancelled }
pub fn workflow_event_map(_e: Option<Value>) -> WorkflowEventDto { WorkflowEventDto::StepSkipped { step_id: String::new() } }
pub fn step_err_is_err(e: &Result<String, String>) -> bool { e.is_err() }
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
pub fn http_post_json(_u: &str) -> impl std::future::Future<Output = Result<Value, String>> { async { Ok(Value::Null) } }
pub fn mpsc_channel() -> Value { Value::Null }
pub fn mpsc_sender(_ch: &Value) -> Value { Value::Null }
pub fn mpsc_receiver(_ch: &Value) -> Value { Value::Null }
pub fn mpsc_try_send(_t: &Value, _m: Value) {}
pub async fn mpsc_recv(_r: &Value) -> Option<Value> { None }
pub fn msg_is_none(m: &Option<Value>) -> bool { m.is_none() }
pub fn msg_unwrap(m: Option<Value>) -> Value { m.unwrap_or(Value::Null) }
pub fn broadcast_recv(_r: &Value) -> impl std::future::Future<Output = Option<Value>> { async { None } }
/// Plan 384 S1: build an axum SSE Event from a serializable DTO + event name,
/// unwrapping the inner json_data Result so callers can `yield event` directly.
pub fn sse_event(name: &str, dto: Value) -> Event {
    Event::default().event(name).json_data(dto).unwrap_or_else(|_| Event::default())
}
pub fn path_inner<T>(p: &T) -> String { String::new() }
pub fn json_response<T: serde::Serialize>(_d: T) -> Response { Response::default() }
pub fn error_response<T: serde::Serialize>(_c: u16, _d: T) -> Response { Response::default() }
pub fn atomic_bool_false() -> Arc<std::sync::atomic::AtomicBool> { Arc::new(std::sync::atomic::AtomicBool::new(false)) }

struct NoDaemonClient;
#[async_trait::async_trait]
impl Client for NoDaemonClient {
    async fn complete(&self, _req: &auto_ai_client::CompletionRequest) -> Result<auto_ai_client::CompletionResponse, auto_ai_client::ClientError> {
        Err(auto_ai_client::ClientError::DaemonUnavailable)
    }
}

/// Minimal `Role` implementor so stubs can construct `Agent::new(role, client)`.
struct StubRole;
impl Role for StubRole {
    fn name(&self) -> &str { "stub" }
    fn system_prompt(&self) -> &str { "" }
}
