//! extern_impl.rs — glue layer stubs for a2r-transpiled .at files.
#![allow(dead_code, unused_variables, non_upper_case_globals, clippy::too_many_arguments)]

use serde_json::Value;
use std::sync::Arc;
use std::any::Any;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, Ordering};
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
    WorkflowRunRequest, RunRequest, WorkspaceQuery as StreamWorkspaceQuery,
    ToolCallOut as StreamToolCallOut,
};

// ── Plan 019 Phase 2-4: side-table 基础设施 ────────────────────────────────
// 类型擦除墙:mpsc/broadcast 的 tx/rx 句柄以 i64 id 存进全局注册表,Value 只存
// id(数字或 {"pair": id})。run 结束后 extern 移除 pair 条目让唯一 Sender 析构
// → channel 关闭 → SSE 流侧 mpsc_recv 得 None → .at 的 break 终止流
// (与 hw 的 `while let Some(v) = rx.recv().await` 语义一致)。
static HANDLES: std::sync::LazyLock<Mutex<std::collections::HashMap<i64, Box<dyn Any + Send>>>> =
    std::sync::LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));
static NEXT_ID: AtomicI64 = AtomicI64::new(1);

fn next_handle_id() -> i64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

struct ChannelPair {
    tx: tokio::sync::mpsc::Sender<Value>,
    rx: Option<tokio::sync::mpsc::Receiver<Value>>,
}

/// 移除 channel pair(析构唯一的 Sender)→ channel 关闭 → 流侧 recv 得 None。
fn close_channel(tx: &Value) {
    let pair_id = match tx.get("pair").and_then(|v| v.as_i64()) {
        Some(i) => i,
        None => return,
    };
    HANDLES.lock().unwrap().remove(&pair_id);
}

pub fn parse_json(s: &str) -> Value { serde_json::from_str(s).unwrap_or(Value::Null) }

// Phase C(计划 018 §12 C3):ag server 的状态码模型 —— handler 返回 ~Response,
// 经这两个 helper 构建带状态码的响应(补齐"状态码在外壳层处理"的缺失)。
pub fn ok_response<T: serde::Serialize>(v: T) -> axum::response::Response {
    axum::Json(v).into_response()
}
pub fn err_response(msg: impl Into<String>, code: u16) -> axum::response::Response {
    let status = axum::http::StatusCode::from_u16(code)
        .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    (status, axum::Json(ApiError { error: msg.into() })).into_response()
}
/// Plan 019 Phase 2-4:非流式 handler 的 4xx/5xx 区分 —— 委托 extern 返回
/// `{"error":{"code":N,"message":...}}` 包络;handler 经这三个 helper 转
/// err_response / ok_response(hw run/workflow_run 的 400/500 等价)。
pub fn resp_is_err(v: &Value) -> bool {
    v.get("error").map_or(false, |e| e.is_object())
}
pub fn resp_err_code(v: &Value) -> u16 {
    v.get("error")
        .and_then(|e| e.get("code"))
        .and_then(|c| c.as_u64())
        .unwrap_or(500) as u16
}
pub fn resp_err_message(v: &Value) -> String {
    v.get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
        .unwrap_or("request failed")
        .to_string()
}
/// 复审 A3:委托函数约定 `Value::Null` = 错误。handler 统一经此 helper 转成
/// `err_response(msg, code)`;成功值直接 ok_response。修复"错误 → 200+null"回归。
pub fn to_response(v: Value, msg: &str, code: u16) -> axum::response::Response {
    if v.is_null() {
        err_response(msg, code)
    } else {
        ok_response(v)
    }
}
pub fn value_get_str(v: &Value, k: &str) -> String { v.get(k).and_then(|s| s.as_str()).unwrap_or("").to_string() }
pub fn value_get_bool(v: &Value, k: &str) -> bool { v.get(k).and_then(|b| b.as_bool()).unwrap_or(false) }
pub fn value_get_array(v: &Value, k: &str) -> Value { v.get(k).cloned().unwrap_or(Value::Array(vec![])) }
pub fn null_value() -> Value { Value::Null }
pub fn new_id(n: u32) -> String { random_hex(n) }
pub fn random_hex(n: u32) -> String {
    // 复审 A2: 旧实现用单个 u64 零填充(只有 64-bit 熵,高位可预测)。改为
    // fill_bytes 全随机,与 hw src/auth.rs::random_hex / src/chats.rs::new_id
    // (rand::thread_rng().fill_bytes) 语义一致。
    use rand::RngCore;
    let mut buf = vec![0u8; n as usize];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}
pub fn hash_password(p: &str, s: &str) -> String { use sha2::Digest; let mut h = sha2::Sha256::new(); h.update(s.as_bytes()); h.update(p.as_bytes()); hex::encode(h.finalize()) }

pub const read_file_schema: &str = r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#;
pub const write_file_schema: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}"#;
pub const run_command_schema: &str = r#"{"type":"object","properties":{"cmd":{"type":"string"},"force":{"type":"boolean"}},"required":["cmd"]}"#;
pub const edit_file_schema: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"old_string":{"type":"string"},"new_string":{"type":"string"}},"required":["path","old_string","new_string"]}"#;
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
pub fn auth_register_result(s: &State<AppState>, u: String, p: String) -> (String, String) {
    match s.0.auth.register(&u, &p) {
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
/// 复审 A3:错误 → `Value::Null`(handler 经 to_response 转 500),不再返回
/// 全零 DriftResult(那会被当成"无漂移")。
pub fn specs_drift(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.specs.load() {
        Ok(doc) => match ws.specs.drift_check(&doc) {
            Ok((disk_version, drifted)) => serde_json::to_value(DriftResult {
                memory_version: doc.version,
                disk_version,
                drifted,
            })
            .unwrap_or(Value::Null),
            Err(_) => Value::Null,
        },
        Err(_) => Value::Null,
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
/// 复审 A3:错误 → `Value::Null`(handler 经 to_response 转 404)。
pub fn specs_related_of(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let item_id = p.0.clone();
    match ws.specs.load() {
        Ok(mut doc) => {
            doc.rebuild_relations();
            for section in &doc.sections {
                if let Some(item) = section.items.iter().find(|i| i.id == item_id) {
                    return serde_json::to_value(RelatedInfo {
                        item_id,
                        depends_on: item.depends_on.clone(),
                        related: item.related.clone(),
                    })
                    .unwrap_or(Value::Null);
                }
            }
            Value::Null
        }
        Err(_) => Value::Null,
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
/// ② 委托:transition = load + transition_item + save + 返回 TransitionOk wire 形状。
/// 复审 A3:错误 → `Value::Null`(handler 经 to_response 转 400)。
pub fn specs_transition_of(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, b: Json<crate::auto_generated::server::SpecsTransitionRequest>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let new_status = crate::specs::SpecStatus::from_str_lossy(&b.new_status);
    let mut doc = match ws.specs.load() {
        Ok(d) => d,
        Err(_) => return Value::Null,
    };
    match ws.specs.transition_item(&mut doc, &b.section, &b.item_id, new_status) {
        Ok(_) => {
            let _ = ws.specs.save(&doc);
            serde_json::to_value(crate::auto_generated::server::TransitionOk {
                status: "ok".to_string(),
                new_status: b.new_status.clone(),
            })
            .unwrap_or(Value::Null)
        }
        Err(_) => Value::Null,
    }
}
/// ② 委托:delete = load + delete_item + save + 返回 Deleted wire 形状。
/// 复审 A3:错误 → `Value::Null`(handler 经 to_response 转 404)。
pub fn specs_delete_of(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<(String, String)>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let (section_id, item_id) = p.0;
    let mut doc = match ws.specs.load() {
        Ok(d) => d,
        Err(_) => return Value::Null,
    };
    match ws.specs.delete_item(&mut doc, &section_id, &item_id) {
        Ok(true) => {
            let _ = ws.specs.save(&doc);
            serde_json::to_value(crate::auto_generated::server::Deleted {
                status: "deleted".to_string(),
                id: item_id,
            })
            .unwrap_or(Value::Null)
        }
        Ok(false) | Err(_) => Value::Null,
    }
}
pub fn specs_read(_s: String) -> String { String::new() }
pub fn specs_list() -> String { String::new() }
pub fn specs_write(_s: String, _c: String) -> String { "ok".into() }
pub fn specs_update(_a: String, _s: String, _v: Value) -> String { "ok".into() }
pub fn specs_write_goals(_g: String) -> String { "ok".into() }
/// ③ 委托(config 页):professions/config/modes/skills/roles 走真实 registry,
/// 返回 hw wire 形状的 Value。
pub fn professions_list() -> Value {
    let list: Vec<serde_json::Value> = auto_ai_agent::builtin_names()
        .iter()
        .filter_map(|name| {
            auto_ai_agent::load_builtin(name).map(|p| {
                serde_json::json!({
                    "name": name,
                    "tier": format!("{:?}", p.model_tier()).to_lowercase(),
                    "model": p.model(),
                    "temperature": p.temperature(),
                    "max_turns": p.max_turns(),
                })
            })
        })
        .collect();
    serde_json::json!({ "professions": list })
}
pub fn config_build() -> Value {
    let reg = crate::mode::ModeRegistry::load();
    let modes: Vec<serde_json::Value> = reg
        .names()
        .iter()
        .filter_map(|n| {
            reg.get(n).map(|m| {
                serde_json::json!({
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
                serde_json::json!({
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
        let sreg = std::sync::Arc::new(auto_ai_agent::SkillRegistry::scan(&dir));
        sreg.descriptions()
            .iter()
            .map(|(name, desc)| serde_json::json!({ "name": name, "description": desc }))
            .collect()
    } else {
        vec![]
    };
    serde_json::to_value(serde_json::json!({ "modes": modes, "professions": profs, "skills": skills }))
        .unwrap_or(Value::Null)
}
pub fn modes_all() -> Value {
    let reg = crate::mode::ModeRegistry::load();
    let modes: Vec<serde_json::Value> = reg
        .names()
        .iter()
        .filter_map(|n| {
            reg.get(n).map(|m| {
                serde_json::json!({
                    "name": m.name,
                    "description": m.description,
                    "role": m.role,
                    "skills": m.skills,
                    "tool_count": m.tools.len(),
                })
            })
        })
        .collect();
    serde_json::json!({ "modes": modes })
}
pub fn skills_all() -> Value {
    let skills_dir = dirs::home_dir().map(|h| h.join(".config/autoos/skills"));
    let skills: Vec<serde_json::Value> = if let Some(dir) = skills_dir {
        let reg = std::sync::Arc::new(auto_ai_agent::SkillRegistry::scan(&dir));
        reg.descriptions()
            .iter()
            .map(|(name, desc)| serde_json::json!({ "name": name, "description": desc }))
            .collect()
    } else {
        vec![]
    };
    serde_json::json!({ "skills": skills })
}
pub fn roles_all() -> Value {
    let reg = auto_ai_agent::RoleRegistry::load();
    let roles: Vec<serde_json::Value> = reg
        .list()
        .iter()
        .map(|r| {
            serde_json::json!({
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
    serde_json::json!({ "roles": roles })
}
pub fn role_get(p: &Path<String>) -> Value {
    let reg = auto_ai_agent::RoleRegistry::load();
    match reg.get(&p.0) {
        Some(d) => {
            let cfg = &d.config;
            serde_json::to_value(serde_json::json!({
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
            .unwrap_or(Value::Null)
        }
        None => Value::Null,
    }
}
pub fn role_save_of(p: &Path<String>, b: Json<crate::auto_generated::server::RoleSaveBody>) -> Value {
    use auto_ai_agent::{parse_tier_field, RoleConfig};
    let cfg = RoleConfig {
        name: Some(p.0.clone()),
        description: b.description.clone(),
        inherit: b.inherit.clone(),
        model: b.model.clone(),
        model_tier: b.tier.as_deref().and_then(parse_tier_field),
        temperature: b.temperature,
        max_turns: b.max_turns,
        allowed_tiers: if b.allowed_tiers.is_empty() {
            None
        } else {
            Some(b.allowed_tiers.iter().filter_map(|s| parse_tier_field(s)).collect())
        },
        skills: if b.skills.is_empty() { None } else { Some(b.skills.clone()) },
        token_budget: b.token_budget,
        tools: if b.tools.is_empty() { None } else { Some(b.tools.clone()) },
        soul_file: None,
        system_prompt: b.system_prompt.clone(),
        system_prompt_append: None,
        tools_append: None,
        memory_limit: None,
        thinking_level: None,
        models: None,
    };
    let reg = auto_ai_agent::RoleRegistry::load();
    match reg.save(&p.0, cfg, b.soul.as_deref()) {
        Ok(_) => serde_json::to_value(serde_json::json!({ "status": "saved", "name": p.0 }))
            .unwrap_or(Value::Null),
        Err(_) => Value::Null,
    }
}
pub fn role_delete_of(p: &Path<String>) -> Value {
    let reg = auto_ai_agent::RoleRegistry::load();
    match reg.delete(&p.0) {
        Ok(_) => serde_json::to_value(serde_json::json!({ "status": "deleted", "name": p.0 }))
            .unwrap_or(Value::Null),
        Err(_) => Value::Null,
    }
}
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
/// ③ 委托:app-config + harness 走真实 MuskAppConfig / RoleRegistry /
/// SkillRegistry / ModeRegistry + app harness 目录扫描,返回 hw wire 形状。
pub fn app_config_load() -> Value {
    let cfg = crate::app_config::MuskAppConfig::load();
    serde_json::json!({ "stored": cfg, "effective": cfg.effective() })
}
pub fn app_config_write(b: Json<crate::auto_generated::server::AppConfigSaveBody>) -> Value {
    use crate::app_config::{musk_config_path, HarnessSelection, MuskAppConfig};
    let cfg = MuskAppConfig {
        daemon_url: b.daemon_url.clone(),
        default_mode: b.default_mode.clone(),
        forge_mode: b.forge_mode.clone(),
        context_file: b.context_file.clone(),
        serve_addr: b.serve_addr.clone(),
        auto_start_daemon: b.auto_start_daemon,
        harness: HarnessSelection {
            roles: b.harness.roles.clone(),
            skills: b.harness.skills.clone(),
            modes: b.harness.modes.clone(),
        },
    };
    let path = match musk_config_path() {
        Some(p) => p,
        None => return Value::Null,
    };
    if let Err(_e) = std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new("."))) {
        return Value::Null;
    }
    let src = cfg.to_at_source();
    if let Err(_e) = std::fs::write(&path, &src) {
        return Value::Null;
    }
    serde_json::json!({
        "status": "saved",
        "path": path.display().to_string(),
        "effective": cfg.effective(),
    })
}
pub fn app_config_effective_daemon_url<T>(_c: T) -> String { "http://127.0.0.1:17654".into() }
/// Forge 执行模式（Plan 022 遗留）:读 MuskAppConfig.forge_mode 的 effective 值。
pub fn forge_mode_load() -> Value {
    let cfg = crate::app_config::MuskAppConfig::load();
    serde_json::json!({ "mode": cfg.effective_forge_mode() })
}
/// Forge 执行模式写:校验 gsd/check 后写入 config.at(仅覆盖 forge_mode 字段,
/// 其余字段保留——load 现有配置→改 forge_mode→to_at_source 整写)。
pub fn forge_mode_write(b: Json<crate::auto_generated::server::ForgeModeBody>) -> Value {
    use crate::app_config::{musk_config_path, MuskAppConfig};
    let mode = b.mode.clone();
    if mode != "gsd" && mode != "check" {
        return Value::Null; // 非法 mode → to_response 走 500 错误路径
    }
    let mut cfg = MuskAppConfig::load();
    cfg.forge_mode = Some(mode);
    let path = match musk_config_path() {
        Some(p) => p,
        None => return Value::Null,
    };
    if let Err(_e) = std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new("."))) {
        return Value::Null;
    }
    if let Err(_e) = std::fs::write(&path, cfg.to_at_source()) {
        return Value::Null;
    }
    serde_json::json!({
        "status": "saved",
        "mode": cfg.effective_forge_mode(),
    })
}
pub fn harness_list(p: &Path<String>) -> Value {
    let kind = p.0.clone();
    let cfg = crate::app_config::MuskAppConfig::load();
    let selected_list: &[String] = match kind.as_str() {
        "roles" => &cfg.harness.roles,
        "skills" => &cfg.harness.skills,
        "modes" => &cfg.harness.modes,
        _ => return Value::Null,
    };
    let os_available: Vec<serde_json::Value> = match kind.as_str() {
        "roles" => {
            let reg = auto_ai_agent::RoleRegistry::load();
            reg.list().iter().map(|r| {
                serde_json::json!({
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
                    serde_json::json!({
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
                reg.get(n).map(|m| {
                    serde_json::json!({
                        "name": m.name,
                        "description": m.description,
                        "is_builtin": false,
                        "selected": selected_list.contains(&m.name),
                    })
                })
            }).collect()
        }
        _ => vec![],
    };
    let app_custom: Vec<serde_json::Value> = if let Some(dir) = crate::server::app_harness_dir(&kind) {
        match kind.as_str() {
            "roles" => crate::server::scan_app_roles(&dir),
            "skills" => crate::server::scan_app_skills(&dir),
            "modes" => crate::server::scan_app_modes(&dir),
            _ => vec![],
        }
    } else {
        vec![]
    };
    serde_json::json!({ "os_available": os_available, "app_custom": app_custom })
}
pub fn harness_save(p: &Path<(String, String)>, b: Json<crate::auto_generated::server::AppHarnessSaveBody>) -> Value {
    let (kind, name) = p.0.clone();
    match kind.as_str() {
        "roles" => {
            let cfg = auto_ai_agent::RoleConfig {
                name: Some(name.clone()),
                description: b.description.clone(),
                inherit: b.inherit.clone(),
                model: b.model.clone(),
                model_tier: b.tier.as_deref().and_then(auto_ai_agent::parse_tier_field),
                temperature: b.temperature,
                max_turns: b.max_turns,
                allowed_tiers: if b.allowed_tiers.is_empty() {
                    None
                } else {
                    Some(b.allowed_tiers.iter().filter_map(|s| auto_ai_agent::parse_tier_field(s)).collect())
                },
                skills: if b.skills.is_empty() { None } else { Some(b.skills.clone()) },
                token_budget: b.token_budget,
                tools: if b.tools.is_empty() { None } else { Some(b.tools.clone()) },
                soul_file: None,
                system_prompt: None,
                system_prompt_append: None,
                tools_append: None,
                memory_limit: None,
                thinking_level: None,
                models: None,
            };
            let dir = match crate::server::app_harness_dir("roles") {
                Some(d) => d,
                None => return Value::Null,
            };
            if std::fs::create_dir_all(&dir).is_err() {
                return Value::Null;
            }
            if let Some(md) = &b.soul {
                let soul_path = dir.join(format!("{name}.soul.md"));
                if std::fs::write(&soul_path, md).is_err() {
                    return Value::Null;
                }
            }
            let src = auto_ai_agent::serialize_at_role(&cfg);
            let at_path = dir.join(format!("{name}.at"));
            if std::fs::write(&at_path, &src).is_err() {
                return Value::Null;
            }
            serde_json::json!({
                "status": "saved",
                "kind": kind,
                "name": name,
                "path": at_path.display().to_string(),
            })
        }
        _ => Value::Null,
    }
}
pub fn harness_delete(p: &Path<(String, String)>) -> Value {
    let (kind, name) = p.0.clone();
    let dir = match crate::server::app_harness_dir(&kind) {
        Some(d) => d,
        None => return Value::Null,
    };
    let at_path = dir.join(format!("{name}.at"));
    let soul_path = dir.join(format!("{name}.soul.md"));
    let existed = at_path.exists();
    if existed {
        let _ = std::fs::remove_file(&at_path);
    }
    let _ = std::fs::remove_file(&soul_path);
    if !existed {
        return Value::Null;
    }
    serde_json::json!({ "status": "deleted", "kind": kind, "name": name })
}
pub fn harness_name_from_path(p: &Path<(String, String)>) -> String {
    p.0 .1.clone()
}
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
            serde_json::json!({ "session": session })
        }
        Err(_) => Value::Null,
    }
}
pub fn chats_list(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    serde_json::json!({ "sessions": ws.chats.list() })
}
// PLAN-084 T-05: 已归档会话列表（默认列表已滤除归档态的对偶面）。
pub fn chats_list_archived(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    serde_json::json!({ "sessions": ws.chats.list_archived() })
}
// PLAN-084 T-05: 会话归档 toggle（归档=列表级隐藏；详见 ChatStore::archive）。
pub fn chats_archive(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>, b: Json<crate::auto_generated::server::ChatArchiveBody>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.chats.archive(&p.0, b.archived) {
        Ok(Some(session)) => serde_json::json!({ "session": session }),
        _ => Value::Null,
    }
}
pub fn chats_get(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.chats.get(&p.0) {
        Some(session) => serde_json::json!({ "session": session }),
        None => Value::Null,
    }
}
pub fn chats_rename(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>, b: Json<crate::auto_generated::server::ChatRenameBody>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.chats.rename(&p.0, &b.name) {
        Ok(Some(session)) => {
            let _ = ws.conversations.rename(&p.0, &b.name);
            serde_json::json!({ "session": session })
        }
        _ => Value::Null,
    }
}
/// PLAN-064: set/clear a session's thinking level (see ChatStore::set_thinking_level).
pub fn chats_thinking(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>, b: Json<crate::auto_generated::server::ChatThinkingBody>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.chats.set_thinking_level(&p.0, b.thinking_level.clone()) {
        Ok(Some(session)) => serde_json::json!({ "session": session }),
        _ => Value::Null,
    }
}
// PLAN-067 T-05: 会话审批模式 PATCH（human|auto;非法值后端归一 human）。
pub fn chats_approval(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>, b: Json<crate::auto_generated::server::ChatApprovalBody>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.chats.set_approval_mode(&p.0, b.approval_mode.as_str()) {
        Ok(Some(session)) => serde_json::json!({ "session": session }),
        _ => Value::Null,
    }
}
pub fn chats_delete(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: &Path<String>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    if ws.chats.delete(&p.0).unwrap_or(false) {
        let _ = ws.conversations.delete(&p.0);
        serde_json::to_value(crate::auto_generated::server::Deleted {
            status: "deleted".to_string(),
            id: p.0.clone(),
        })
        .unwrap_or(Value::Null)
    } else {
        Value::Null
    }
}
pub fn chats_delete_all(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    if ws.chats.delete_all().is_ok() {
        ws.conversations.delete_all();
        serde_json::json!({ "status": "deleted_all" })
    } else {
        Value::Null
    }
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
            // PLAN-055 ⑧(D1): VM 轨显式 run 触发——VM 的 Sse.open 是 no-op，
            // 「订阅即运行」不可达，发送时带 run=true 由后端直接 spawn 运行
            // 内核（T1 证毕：持久化在 run 循环体内，tx 未注册即无订阅者安全
            // 运行）。per-session 守卫防双跑；run 缺省（web 轨）不触发。
            if b.run.unwrap_or(false) {
                let run_key = format!("{}:{}", q.workspace.clone().unwrap_or_default(), p.0);
                if s.0.chat_run_try_start(&run_key) {
                    let st = State(s.0.clone());
                    let wsq = crate::auto_generated::server_stream::WorkspaceQuery {
                        workspace: q.workspace.clone(),
                    };
                    let sid = p.0.clone();
                    // PLAN-071 T-02：spawn 运行主体 chat_run_owner（SSE 订阅
                    // 走 chat_run_stream 恒附加，两者角色显式分离）。
                    tokio::spawn(async move {
                        chat_run_owner(&st, Query(wsq), Path(sid), serde_json::Value::Null).await;
                    });
                } else {
                    // PLAN-071 需求⑤ T-24：守卫占用不再静默——落一条可见的
                    // busy 提示（持久化，与超时/失败落盘同构），响应体带 busy
                    // 字段供 API 消费方判断。
                    // PLAN-071 需求⑮：b.queued=true 为前端队列自动重发——
                    // 收束竞态窗口内的重试不落提示（否则每拍重试都会写入
                    // 一条 ⚠），只回 busy:true 由前端稍后再试。
                    tracing::warn!(
                        "chat run spawn skipped (guard held): session={} ws={}",
                        p.0,
                        q.workspace.clone().unwrap_or_default()
                    );
                    if !b.queued {
                        let notice = crate::chats::ChatMessage::assistant(
                            "⚠ 已有运行在途，本条消息未执行；待当前运行结束后请重新发送。"
                                .to_string(),
                        );
                        let _ = ws.chats.append_message(&p.0, notice.clone());
                        let seq_base = ws
                            .conversations
                            .get(&p.0)
                            .map(|c| c.turns.len())
                            .unwrap_or(0);
                        for turn in crate::conversation::chat_message_to_turns(&notice, seq_base) {
                            let _ = ws.conversations.append_turn(&p.0, turn);
                        }
                    }
                    return serde_json::to_value(serde_json::json!({
                        "session": session, "queued": msg, "busy": true
                    }))
                    .unwrap_or(Value::Null);
                }
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
        Ok(Some(session)) => serde_json::json!({ "session": session }),
        _ => Value::Null,
    }
}
pub fn chats_reject_all(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.chats.reject_all_spec_changes(&p.0) {
        Ok(Some(session)) => serde_json::json!({ "session": session }),
        _ => Value::Null,
    }
}
pub fn conversations_list(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    serde_json::json!({ "conversations": ws.conversations.list() })
}
pub fn conversations_get(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.conversations.get(&p.0) {
        Some(conv) => serde_json::to_value(conv).unwrap_or(Value::Null),
        None => Value::Null,
    }
}
pub fn conversations_delete(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: &Path<String>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    if ws.conversations.delete(&p.0) {
        serde_json::to_value(crate::auto_generated::server::Deleted {
            status: "deleted".to_string(),
            id: p.0.clone(),
        })
        .unwrap_or(Value::Null)
    } else {
        Value::Null
    }
}
pub fn conversations_rename(s: &State<AppState>, q: Query<crate::auto_generated::server::WorkspaceQuery>, p: Path<String>, b: Json<crate::auto_generated::server::ConversationTitleBody>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    match ws.conversations.rename(&p.0, &b.title) {
        Some(conv) => serde_json::to_value(conv).unwrap_or(Value::Null),
        None => Value::Null,
    }
}
/// Plan 019 Phase 2:conversation_stream 真实化 —— broadcast::Receiver 存
/// Plan 019 §6.2:被 `conv_event_stream` owned 的 broadcast 订阅句柄。
///
/// **根治连接泄漏** —— 此前 broadcast::Receiver 存 side-table(Value 存 i64 id),
/// 客户端在事件间隙断开时 conv_event_stream 被 drop、不再调 broadcast_recv →
/// registry 条目永不回收(每连接一条,累积)。
///
/// 现改为:rx 包进 `Arc<Mutex<Option<Receiver>>>`,`BroadcastSub` 持有 Arc。
/// `conv_event_stream` 把 `BroadcastSub`(clone)move 进 `async_stream::stream!` 块
/// → stream drop(正常关闭或客户端断开)→ BroadcastSub clone drop → Arc 引用归零
/// → rx 析构。**不再用 registry 存 broadcast receiver**,rx 所有权完全由
/// BroadcastSub 的 Arc 引用计数管理(与 hw `BroadcastStream::new(rx)` 同语义)。
pub struct BroadcastSub {
    inner: Arc<Mutex<Option<tokio::sync::broadcast::Receiver<crate::conversation::ConversationEvent>>>>,
    /// 此订阅关注的 conversation_id(过滤用,来自请求 path)。存 owned String
    /// 避免把 &str 借用带进 ~Stream 函数参数(触发 impl Stream 生命周期捕获 E0700)。
    conversation_id: String,
}
// Clone:a2r 对非 Copy 类型传参会自动加 .clone()(见 conv_event_stream 调用点)。
// clone 只是 Arc 计数 +1 —— Arc 归零时 rx 析构,无需手动清理 registry。
impl Clone for BroadcastSub {
    fn clone(&self) -> Self {
        BroadcastSub { inner: self.inner.clone(), conversation_id: self.conversation_id.clone() }
    }
}

/// side-table(Value 存 i64 id),字段提取走 ConversationEvent 序列化 Value。
pub fn conversations_subscribe(s: &State<AppState>, q: Query<StreamWorkspaceQuery>, conversation_id: &str) -> BroadcastSub {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let rx = ws.conversations.subscribe();
    BroadcastSub {
        inner: Arc::new(Mutex::new(Some(rx))),
        conversation_id: conversation_id.to_string(),
    }
}
/// §6.2:用 sub 内的 conversation_id 过滤事件(等价 hw filter_map 的
/// `ev.conversation_id == id`,但 id owned 在 sub 里,避免 &str 流参数)。
pub fn sub_matches_conv(sub: &BroadcastSub, ev: &Value) -> bool {
    ev.get("conversation_id").and_then(|v| v.as_str()) == Some(sub.conversation_id.as_str())
}
pub fn conv_event_matches(ev: &Value, id: &str) -> bool {
    ev.get("conversation_id").and_then(|v| v.as_str()) == Some(id)
}
pub fn conv_event_id(ev: &Value) -> String {
    ev.get("conversation_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}
pub fn conv_event_turn(ev: &Value) -> Option<Value> {
    ev.get("turn").cloned()
}
pub fn conv_event_status(ev: &Value) -> Option<String> {
    ev.get("status")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}
/// ② workspace 委托(与 specs/chats 同模式):走 s.0.registry 真实逻辑,返回
/// 完整 metas / Value 供 ag handler 包装,wire 形状与 hw 一致。
pub fn workspace_list_all(s: &State<AppState>) -> Value {
    serde_json::json!({ "workspaces": s.0.registry.list() })
}
pub fn workspace_open_of(s: &State<AppState>, b: Json<crate::auto_generated::server::OpenWorkspaceBody>) -> Value {
    let meta = s.0.registry.open(&b.path);
    s.0.registry.touch(&meta.id);
    serde_json::json!({ "workspace": meta })
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
/// Plan 019 Phase 1b+2-4:真实化 wf_run —— 与 hw `workflow_run`(src/server.rs:841)
/// 同路径:require_builtin 校验 + feature_dev::run + DTO 映射。
///
/// Plan 019 Phase 2-4 状态码模型:返回 Value + 错误包络
/// (`{"error":{"code","message"}}`),handler 经 resp_is_err/err_response 转
/// 400(坏 workflow) / 500(run 失败),与 hw 的 400/500 等价。
pub async fn wf_run(
    s: &State<AppState>,
    q: Query<StreamWorkspaceQuery>,
    b: Json<WorkflowRunRequest>,
) -> Value {
    if let Err(e) = crate::auto_generated::feature_dev::require_builtin(&b.workflow) {
        return serde_json::json!({"error": {"code": 400, "message": format!("invalid workflow '{}': {e}", b.workflow)}});
    }
    let hw_q = crate::workspace::WorkspaceQuery { workspace: q.workspace.clone() };
    let ws_id = hw_q.id_or_default(&s.0.registry);
    let ws = s.0.registry.get(&ws_id);
    // Plan 020 Phase F:引擎切到 ag feature_dev(parity_feature_dev 端到端 9 项绿)。
    match crate::auto_generated::feature_dev::run(s.0.clone(), ws.clone(), &b.task).await {
        Ok(r) => serde_json::to_value(WorkflowRunResponse {
            steps: r.steps,
            outputs: r.outputs,
        })
        .unwrap_or(Value::Null),
        Err(e) => serde_json::json!({"error": {"code": 500, "message": format!("workflow failed: {e}")}}),
    }
}
/// Plan 019 §6.1:流式 handler 前置校验 —— mode/workflow 在建 mpsc channel 前
/// 校验,坏 spec 直接 400(与 hw run_stream_handler / workflow_run_stream 等价),
/// 避免提交 SSE 响应后才发 error 帧(HTTP 200)的回归。
///
/// 约定:校验通过返回 `Value::Null`(resp_is_err=false,非错误);失败返回
/// `{"error":{"code":400,"message":...}}`(复用 resp_is_err/resp_err_* helper)。
pub fn workflow_exists(name: &str) -> Value {
    if let Err(e) = crate::auto_generated::feature_dev::require_builtin(name) {
        return serde_json::json!({"error": {"code": 400, "message": format!("invalid workflow '{}': {e}", name)}});
    }
    Value::Null
}
pub fn mode_exists(name: &str) -> Value {
    let reg = crate::mode::ModeRegistry::load();
    if reg.get(name).is_none() {
        return serde_json::json!({"error": {"code": 400, "message": format!("unknown mode '{}'; available: {}", name, reg.names().join(", "))}});
    }
    Value::Null
}
/// Plan 019 Phase 3:workflow_run_stream 真实化 —— sink 桥接:把 ag feature_dev
/// 的 stream 事件喂给 mpsc(tx 句柄),run 结束关闭 channel 让 SSE 流终止。
/// Plan 020 Phase F:引擎切到 ag feature_dev(run_with_emit 内部负责 tool-root
/// 限定 + Finished 事件 + 清理,与 hw run_stream 同序)。
pub async fn wf_run_with_progress(
    s: &State<AppState>,
    q: Query<StreamWorkspaceQuery>,
    b: Json<WorkflowRunRequest>,
    tx: Value,
) {
    if let Err(e) = crate::auto_generated::feature_dev::require_builtin(&b.workflow) {
        mpsc_try_send(&tx, serde_json::json!({"type":"error","message": format!("invalid workflow '{}': {e}", b.workflow)}));
        close_channel(&tx);
        return;
    }
    let hw_q = crate::workspace::WorkspaceQuery { workspace: q.workspace.clone() };
    let ws_id = hw_q.id_or_default(&s.0.registry);
    let ws = s.0.registry.get(&ws_id);
    let state = s.0.clone();
    let task = b.task.clone();
    let tx2 = tx.clone();
    tokio::spawn(async move {
        // run_with_emit 内部 feature_dev_set_root(ag extern)→ 同 tool_safety 线程
        // 局部;结束前 clear(与 hw drive_run 的 set/clear 对称)。
        if let Err(e) =
            crate::auto_generated::feature_dev::run_with_emit(state, ws, &task, tx2.clone()).await
        {
            tracing::error!("workflow stream failed: {e}");
        }
        // run 结束 → 关闭 channel,让 SSE 流侧 mpsc_recv 得 None → break 终止。
        close_channel(&tx2);
    });
}
pub fn orch_spawn_relay(_t: String, _a: Value) -> String { "(stub)".into() }
pub fn orch_dispatch(_t: String, _to: String) -> String { "(stub)".into() }
pub fn orch_bring_in(_q: String) -> String { "(stub)".into() }
/// Per-(workspace,run) accumulated Delta text — the ag equivalent of hw
/// run_step's local `Arc<Mutex<String>>` (which the on_event closure fills and
/// run_step drains). The ag sink crosses an extern boundary, so it can't share a
/// local; this side-table mirrors the same lifetime (a run is single-driver).
static RELAY_ACC: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<(String, String), String>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

pub fn drive_set_root(s: &Arc<AppState>, w: &str) {
    // hw drive_run: set tool root to the workspace root before driving, so file
    // tools are confined (same tool_safety thread-local as feature_dev).
    let ws = s.registry.get(w);
    crate::tool_safety::set_current_root(ws.root.clone());
}
pub fn drive_clear_root() {
}
/// Plan 020 Phase G (relay_driver.at): serialize the `Option<AdvanceResult>`
/// from `ws.relay.advance(run_id)`. None → Null (advance_is_none true);
/// Some(ar) → serde Value of the AdvanceResult (externally-tagged enum, so the
/// tag string — "ExecuteStep"/"WaitForHuman"/"Completed"/"Failed"/"Paused" — is
/// the object key, used by advance_kind/advance_role_id).
pub fn relay_advance(s: &Arc<AppState>, w: &str, r: &str) -> Value {
    // PLAN-030 试用诊断线：曾出现 spawn_relay 派生的 driver 任务静默未推进
    // run（同路径另一次却正常）——serve 此前无日志无法定位，此线保证复发可诊。
    tracing::info!("relay_advance ws={w} run={r}");
    let ws = s.registry.get(w);
    match ws.relay.advance(r) {
        None => Value::Null,
        Some((ar, _state)) => serde_json::to_value(&ar).unwrap_or(Value::Null),
    }
}
/// hw `relay::api::publish_advance_result`: maps the AdvanceResult to a RunEvent
/// (StepStarted/GateWaiting/RunCompleted/RunFailed/RelayUpdate) and broadcasts it
/// on the relay SSE bus. `v` is the serde Value produced by relay_advance.
/// PLAN-031 T5: completion frames carry the run report (re-read from the store —
/// relay_advance has already appended the RunCompleted event with the payload).
pub fn relay_publish(state: &Arc<AppState>, ws_id: &str, r: &str, v: &Value) {
    if let Ok(ar) = serde_json::from_value::<crate::relay::AdvanceResult>(v.clone()) {
        let ws = state.registry.get(ws_id);
        let report = ws.relay.run_report(r);
        crate::relay::api::publish_advance_result_with_report(r, &ar, report);
    }
}
/// None when relay_advance returned Null (run vanished mid-drive).
pub fn advance_is_none(r: &Value) -> bool { r.is_null() }
/// Tag of the externally-tagged AdvanceResult enum (lowercased to the drive_loop
/// match arms: "execute"/"wait"/"completed"/"failed"/"paused"). Unknown → "".
pub fn advance_kind(r: &Value) -> String {
    match r.as_object().and_then(|o| o.keys().next()) {
        Some(k) => match k.as_str() {
            "ExecuteStep" => "execute",
            "WaitForHuman" => "wait",
            "Completed" => "completed",
            "Failed" => "failed",
            "Paused" => "paused",
            _ => "",
        }.to_string(),
        None => String::new(),
    }
}
/// role_id from ExecuteStep variant (empty for other variants — drive_loop only
/// reads it in the "execute" branch).
pub fn advance_role_id(r: &Value) -> String {
    r.get("ExecuteStep")
        .and_then(|v| v.get("role_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}
/// hw driver.rs ExecuteStep error path: wrap the agent error in a HandoffDocument
/// whose summary is `[agent error] {e}` and submit_handoff (the engine routes to
/// the next step / fails the run). `e` is the Err side of run_step's Result.
pub fn relay_submit_error(s: &Arc<AppState>, w: &str, r: &str, _role_id: &str, e: &Result<String, String>) {
    // PLAN-030 试用修复：agent 错误 → 显式置败（原 submit_handoff 会级联到
    // 后续相位空转出假完成），与 hw driver 的 fail_run 路径一致。
    if let Err(msg) = e {
        let ws = s.registry.get(w);
        let _ = ws.relay.fail_run(r, &format!("[agent error] {msg}"));
    }
}
/// PLAN-034 T9：run 完成钩子——把报告作为助手消息写回发起它的 chat 会话
///（`chat_session_id` 由 plan-merge 短路 / spawn_relay 写入 run context）。
/// 报告以 tool call `report` 携带全量 RunReportPayload，前端在对话流内
/// 内联渲染报告卡（刷新持久），与 Run 卡片互链。hw/ag 两个驱动共用。
pub fn relay_append_report_message_to(
    ws: &std::sync::Arc<crate::workspace::WorkspaceStores>,
    run_id: &str,
) -> () {
    let Some(session_id) = ws.relay.context_var(run_id, "chat_session_id") else {
        return;
    };
    let Some(report) = ws.relay.run_report(run_id) else {
        return;
    };
    let title = if report.title.is_empty() {
        run_id.to_string()
    } else {
        report.title.clone()
    };
    // PLAN-034 T10：一句话 + 标题为锚点链接（前端点击回滚到 Run 卡片）。
    let mut msg = crate::chats::ChatMessage::assistant(format!(
        "✅ Run 已完成：[{title}](#run-card-{run_id})，完整报告见下方卡片。"
    ));
    msg.tool_calls = vec![crate::chats::ToolCall {
        tool: "report".into(),
        args: serde_json::to_value(&report).unwrap_or_default(),
        result: String::new(),
        status: "success".into(),
        id: "report-1".into(),
    }];
    // PLAN-071 r3：报告回写身份 = 发起会话 mode 的 role（未注册回退 mode 名）。
    if let Some(sess) = ws.chats.get(&session_id) {
        msg.profession_id = Some(
            crate::mode::ModeRegistry::load()
                .get(&sess.mode)
                .map(|m| m.role.clone())
                .unwrap_or_else(|| sess.mode.clone()),
        );
    }
    let _ = ws.chats.append_message(&session_id, msg.clone());
    let seq_base = ws
        .conversations
        .get(&session_id)
        .map(|c| c.turns.len())
        .unwrap_or(0);
    for turn in crate::conversation::chat_message_to_turns(&msg, seq_base) {
        let _ = ws.conversations.append_turn(&session_id, turn);
    }
    tracing::info!("run {run_id}: report message appended to session {session_id}");
}
/// hw `ws.relay.step_context(run_id)` → the task string for the agent (fallback
/// "Continue the relay pipeline." when no initial task, matching hw driver.rs:165).
pub fn relay_step_context(s: &Arc<AppState>, w: &str, r: &str) -> String {
    let ws = s.registry.get(w);
    ws.relay.step_context(r)
        .map(|(task, _prior_md)| task)
        .unwrap_or_else(|| "Continue the relay pipeline.".to_string())
}
/// ③ 委托:factory_build_agent — 构造 hw `relay::driver::MuskAgentFactory` 并
/// build_agent(role_id, last_handoff 注入 — 与 hw run_step:173-174 一致:把上一
/// 步 handoff render 成 prior_md 注入 agent history)。失败回退 StubRole + 日志。
pub async fn factory_build_agent(s: &Arc<AppState>, w: &str, r: &str, r2: &str) -> Agent {
    let factory = crate::relay::driver::MuskAgentFactory {
        state: s.clone(),
        workspace_id: w.to_string(),
        run_id: r.to_string(),
    };
    let ws = s.registry.get(w);
    let prior_handoff = ws.relay.last_handoff(r);
    match factory.build_agent(r2, prior_handoff.as_ref()) {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!("factory_build_agent delegation failed: {e}");
            Agent::new(StubRole, s.client.clone())
        }
    }
}
/// Drain the accumulated Delta output for this run/turn (hw run_step:253 takes it
/// out of the Arc<Mutex<String>> the on_event closure filled). Clears the slot.
pub fn drive_accumulated(_s: &Arc<AppState>, w: &str, r: &str) -> String {
    let mut acc = RELAY_ACC.lock().unwrap();
    acc.remove(&(w.to_string(), r.to_string())).unwrap_or_default()
}
/// hw run_step:254-258: if accumulated output is blank, fall back to the agent's
/// final AgentResult.output. `v` is the serde Value of AgentResult from
/// agent_run_stream_with_sink.
pub fn drive_finalize_output(o: &str, v: &Value) -> String {
    if o.trim().is_empty() {
        v.get("output").and_then(|x| x.as_str()).unwrap_or("").to_string()
    } else {
        o.to_string()
    }
}
/// hw run_step:270-282: wrap final_output in a HandoffDocument (from=role_id,
/// to=next_profession, summary=final_output, step_tokens = total_tokens/2) and
/// submit_handoff (engine routes to next step + publishes StepCompleted/TokenSpend).
/// `v` is the AgentResult Value (carries total_tokens).
/// PLAN-086 T-03: ag 驱动此前缺 hw run_step 的 PLAN_FILE 标记提取（仅 hw
/// 独有）；此处对齐——create_plan 绑定主通道已写入则不覆盖（守门函数
/// plan_file_marker_write 双轨单源），无绑定时标记回退生效。
pub fn drive_submit_handoff(s: &Arc<AppState>, w: &str, r: &str, role_id: &str, output: &str, v: &Value) {
    let ws = s.registry.get(w);
    // hw run_step:261-267: TurnComplete event before the handoff submit.
    ws.relay.push_event(r, crate::relay::store::RunEvent::TurnComplete {
        timestamp: now_secs(),
        role_id: role_id.to_string(),
    });
    // hw run_step:322-327 (PLAN-030/086): guarded marker extraction before submit.
    if let Some(plan_file) = crate::relay::plan_flow::plan_file_marker_write(
        ws.relay.context_var(r, "plan_file"),
        output,
    ) {
        ws.relay.set_context_var(r, "plan_file", &plan_file);
    }
    let next_profession = ws.relay.next_profession(r).unwrap_or_default();
    let mut handoff = auto_ai_agent::orchestration::HandoffDocument::new(role_id, &next_profession);
    handoff.summary = output.to_string();
    let total = v.get("total_tokens").and_then(|x| x.as_u64()).unwrap_or(0);
    handoff.token_usage.step_tokens = total / 2;
    let _ = ws.relay.submit_handoff(r, handoff);
}
/// hw run_step on_event closure (driver.rs:183-244): match the 8 StreamEvent
/// variants and push corresponding RunEvents into the store + accumulate Delta
/// text. `e` is the serde Value of StreamEvent (by value — a2r `Value` param).
/// Delta/Cancelled accumulate into the side-table read by drive_accumulated;
/// Tool pushes TurnToolCall+TurnToolResult.
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn drive_handle_stream_event(s: &Arc<AppState>, w: &str, r: &str, role_id: &str, e: Value) {
    use auto_ai_agent::StreamEvent;
    let ws = s.registry.get(w);
    let now = now_secs();
    match serde_json::from_value::<StreamEvent>(e) {
        Ok(StreamEvent::Delta { text }) => {
            ws.relay.push_event(r, crate::relay::store::RunEvent::TurnDelta {
                timestamp: now,
                role_id: role_id.to_string(),
                text: text.clone(),
            });
            let mut acc = RELAY_ACC.lock().unwrap();
            acc.entry((w.to_string(), r.to_string())).or_default().push_str(&text);
        }
        Ok(StreamEvent::ToolStart { .. }) => {
            // Nothing to persist yet (the Tool event follows with the result);
            // kept for state-tracking parity with hw driver.rs:196-199.
        }
        // auto-ai PLAN-026 turn 边界事件:store 不消费,覆盖以保穷尽。
        Ok(StreamEvent::TurnStart { .. }) | Ok(StreamEvent::TurnEnd { .. }) => {}
        Ok(StreamEvent::Tool { tool, args, result, details }) => {
            ws.relay.push_event(r, crate::relay::store::RunEvent::TurnToolCall {
                timestamp: now,
                role_id: role_id.to_string(),
                tool_id: String::new(),
                tool_name: tool.clone(),
                arguments: args.clone(),
            });
            ws.relay.push_event(r, crate::relay::store::RunEvent::TurnToolResult {
                timestamp: now,
                role_id: role_id.to_string(),
                tool_id: String::new(),
                result: result.clone(),
                // PLAN-042:结构化载荷透传(run events SSE + 会话回放)。
                details: details.clone(),
            });
        }
        Ok(StreamEvent::Warning { text }) => {
            tracing::warn!("relay turn warning: {text}");
        }
        Ok(StreamEvent::Thinking { text }) => {
            tracing::debug!("relay thinking: {}…", &text[..text.len().min(60)]);
        }
        Ok(StreamEvent::Done { .. }) | Ok(StreamEvent::Error { .. }) => {
            // Handled via the return value of agent_run_stream_with_sink.
        }
        Ok(StreamEvent::Cancelled { result }) => {
            // The driver never sets the cancel flag; belt-and-braces so partial
            // output still lands (hw driver.rs:237-242).
            let mut acc = RELAY_ACC.lock().unwrap();
            acc.entry((w.to_string(), r.to_string())).or_default().push_str(&result.output);
            tracing::warn!("relay turn cancelled (unexpected)");
        }
        Err(err) => {
            tracing::warn!("drive_handle_stream_event: malformed StreamEvent: {err}");
        }
    }
}
/// Plan 020 Phase A (feature_dev.at): tool-root confinement for a feature-dev
/// run — delegates to the same `tool_safety` thread-local hw module.
pub fn feature_dev_set_root(root: std::path::PathBuf) {
    crate::tool_safety::set_current_root(root);
}
pub fn feature_dev_clear_root() {
}
/// Plan 020 Phase A (feature_dev.at): AgentError → message string (the drive
/// loop's `agent '{step}' failed: {e}` error path; AgentError impls Display).
pub fn agent_error_msg(e: &auto_ai_agent::AgentError) -> String {
    e.to_string()
}
/// Plan 020 Phase B (task_plan_engine.at): cheap unique-ish id (hw uuidish —
/// subsec-nanos in hex, `format!("{:x}", nanos)`; a2r can't hex-format).
pub fn task_plan_uuidish() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}
/// Plan 020 Phase B (task_plan_engine.at): broadcast a task-plan event on the
/// shared relay SSE bus (hw `relay::api::publish_task_plan_event`).
pub fn task_plan_broadcast(instance_id: &str, event_type: &str, payload: Value) {
    crate::relay::api::publish_task_plan_event(instance_id, event_type, payload);
}
/// Plan 020 Phase B (task_plan_engine.at): resolve a `handoff.path.to.value`
/// reference from the HandoffStore; stringified with serde_json Display
/// (compact JSON) — identical to hw `format!("{}", value)`.
pub fn handoff_resolve_path(store: &crate::relay::handoff_store::HandoffStore, path: &str) -> Option<String> {
    store.resolve_path(path).map(|v| v.to_string())
}
/// Plan 020 Phase H: ag drive_run 跨模块调用包装(task_plan_engine.at → relay_driver)。
/// 委托 ag relay_driver::drive_run(Phase G 产物);Result 丢弃(ag executor 不检查
/// drive_run 的 bool 返回,与 hw drive_task_plan_run 不检查 drive_run 的 () 一致)。
pub async fn drive_run(s: &Arc<AppState>, w: &str, r: &str) -> bool {
    crate::auto_generated::relay_driver::drive_run(s.clone(), w, r)
        .await
        .is_ok()
}
/// Plan 020 Phase H: ws.relay.start_run(extern 内部构造 StartRunRequest 字面量,
/// a2r 不能跨 crate 构造 hw struct)。等价 hw drive_task_plan_run:478-484。
pub fn task_plan_start_run(s: &Arc<AppState>, w: &str, r: &str, f: &str, t: &str) {
    let ws = s.registry.get(w);
    let req = crate::relay::store::StartRunRequest {
        run_id: Some(r.to_string()),
        flow_id: Some(f.to_string()),
        steps: Vec::new(),
        task: Some(t.to_string()),
    };
    ws.relay.start_run(&req, Some(w.to_string()));
}
/// Plan 020 Phase H: ws.relay.get(run_id).map(|s| s.status)(等价 hw:494-495)。
pub fn task_plan_run_status(s: &Arc<AppState>, w: &str, r: &str) -> Option<String> {
    let ws = s.registry.get(w);
    ws.relay.get(r).map(|s| s.status)
}
/// Plan 020 Phase H: ws.relay.last_handoff(run_id)(等价 hw:505)。
pub fn task_plan_last_handoff(s: &Arc<AppState>, w: &str, r: &str) -> Option<auto_ai_agent::orchestration::HandoffDocument> {
    let ws = s.registry.get(w);
    ws.relay.last_handoff(r)
}
/// Plan 020 Phase B (task_plan_parser.at): `Atom::to_value()` 的 `{:?}` 调试
/// 文本——组装 hw 的非 Node InvalidType 报错("found {:?}")。
pub fn atom_debug_value(atom: &auto_atom::Atom) -> String {
    format!("{:?}", atom.clone().to_value())
}
/// Plan 020 Phase B (task_plan_registry.at): 内建 deferred-decompose atom
/// (hw include_str!)。
pub fn task_plan_builtin_atom() -> String {
    include_str!("../relay/task_plans/builtin/deferred-decompose.atom").to_string()
}
/// Plan 020 Phase B (task_plan_registry.at): 删除用户 plan 的 .atom 文件。
pub fn task_plan_delete_file(path: std::path::PathBuf) {
    let _ = std::fs::remove_file(path);
}
/// ③ 委托(agent/ctx 簇):auto_lib 镜像的真实 agent 构建 —— 与 hw
/// `build_agent_from_mode/build_agent_with_context`(src/lib.rs) 同路径。
pub fn agent_register_shared(a: &mut Agent, t: Arc<dyn Tool>) {
    a.register_shared(t);
}
pub fn agent_register_skill_tool(a: &mut Agent) {
    if let Some(skills_dir) = dirs::home_dir().map(|h| h.join(".config/autoos/skills")) {
        let registry = auto_ai_agent::SkillRegistry::scan(&skills_dir);
        if !registry.is_empty() {
            a.register_skill_tool(auto_ai_agent::SkillTool::new(std::sync::Arc::new(registry)));
        }
    }
}
pub fn agent_with_context_file(a: Agent, p: &str) -> Agent {
    a.with_context_file(std::path::Path::new(p))
}
pub fn agent_with_history(a: Agent, h: &str) -> Agent {
    a.with_history(vec![("user".to_string(), h.to_string())])
}
pub fn build_agent_with_context(m: AgentMode, c: Arc<dyn Client>, ctx: Option<ToolContext>) -> Agent {
    match crate::build_agent_with_context(&m, c.clone(), ctx) {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!("build_agent_with_context delegation failed: {e}");
            Agent::new(StubRole, c)
        }
    }
}
pub fn mode_tools_contains(m: &AgentMode, n: &str) -> bool {
    m.tools.is_empty() || m.tools.iter().any(|t| t == n)
}
pub fn resolve_role(s: String) -> Result<Arc<dyn Role>, String> {
    crate::resolve_role(&s)
}
pub fn registry_resolve(r: auto_ai_agent::RoleRegistry, s: &str) -> Option<Arc<dyn Role>> {
    r.resolve_role(s)
}
pub fn load_builtin_role(s: &str) -> Option<Arc<dyn Role>> {
    auto_ai_agent::load_builtin(s)
}
pub fn read_at_file(s: &str) -> String {
    std::fs::read_to_string(s).unwrap_or_default()
}
pub fn find_context_file() -> Option<String> {
    crate::find_context_file().map(|p| p.to_string_lossy().into_owned())
}
pub fn find_ctx_upward(c: &str) -> Option<String> {
    crate::find_ctx_upward(std::path::Path::new(c)).map(|p| p.to_string_lossy().into_owned())
}
pub fn current_dir() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| ".".into())
}
pub fn ctx_is_some(c: &Option<String>) -> bool {
    c.is_some()
}
pub fn ctx_unwrap(c: Option<String>) -> String {
    c.unwrap_or_default()
}
pub fn handoff_render(h: String) -> String {
    serde_json::from_str::<crate::relay::HandoffDocument>(&h)
        .map(|d| d.render())
        .unwrap_or_default()
}
/// Plan 019 Phase 1b+2-4:真实化 agent_run —— 与 hw `run_inner`(src/server.rs:280)
/// 同路径:ModeRegistry::load + get(mode) + build_agent_from_mode + agent.run +
/// AgentResult → RunResponse 映射(含 turns + tool_calls)。
///
/// Plan 019 Phase 2-4 状态码模型:返回 Value + 错误包络,handler 经
/// resp_is_err/err_response 转 400(未知 mode) / 500(build/run 失败),与 hw 等价。
/// ag 的 RunRequest.mode 是 Option<String>(hw 是 String + serde default),
/// None 时回退 "superpowers"(与 hw default_mode 一致)。
pub async fn agent_run(
    s: &State<AppState>,
    q: Query<StreamWorkspaceQuery>,
    b: Json<RunRequest>,
) -> Value {
    let mode_name = b.mode.clone().unwrap_or_else(|| "superpowers".into());
    let reg = crate::mode::ModeRegistry::load();
    let mode = match reg.get(&mode_name).cloned() {
        Some(m) => m,
        None => {
            return serde_json::json!({"error": {"code": 400, "message": format!("unknown mode '{}'; available: {}", mode_name, reg.names().join(", "))}});
        }
    };
    // PLAN-069 W1：沙箱工作区 fail-closed 解析（缺失/未知 → 400 包络）。
    let ws = match s.0.registry.get_exact(q.workspace.as_deref().unwrap_or("")) {
        Some(w) => w,
        None => {
            return serde_json::json!({"error": {"code": 400, "message": "unknown or missing workspace; pass ?workspace=".to_string()}});
        }
    };
    // PLAN-070 T-02：多根 = [workspace 根, *白名单]。
    let ws_roots = s
        .0
        .registry
        .sandbox_roots(q.workspace.as_deref().unwrap_or(""));
    let mut agent = match crate::build_agent_from_mode(&mode, s.0.client.clone(), Some(&ws_roots)) {
        Ok(a) => a,
        Err(e) => {
            return serde_json::json!({"error": {"code": 500, "message": format!("build agent: {e}")}});
        }
    };
    match agent.run(&b.task).await {
        Ok(result) => {
            let tool_calls = result
                .tool_calls
                .iter()
                .map(|tc| StreamToolCallOut {
                    name: tc.tool.clone(),
                    arguments: tc.args.clone(),
                    result: tc.result.clone(),
                })
                .collect();
            serde_json::to_value(RunResponse {
                output: result.output,
                turns: result.turns as i32,
                tool_calls,
            })
            .unwrap_or(Value::Null)
        }
        Err(e) => {
            serde_json::json!({"error": {"code": 500, "message": format!("agent failed: {e}")}})
        }
    }
}
/// Plan 019 Phase 4:run_stream_handler 真实化 —— 与 hw `run_stream_handler`
/// (server.rs:358)同路径:build_agent + agent.run_stream + on_event 闭包复刻
/// tc_counter/tc_stack id 配对 + StreamEvent → SseEventDto(经 stream_event_map
/// 无损回读)→ 喂给 mpsc。run 结束关闭 channel 让 SSE 流终止。
pub async fn agent_run_stream(
    s: &State<AppState>,
    q: Query<StreamWorkspaceQuery>,
    b: Json<RunRequest>,
    tx: Value,
) {
    let mode_name = b.mode.clone().unwrap_or_else(|| "superpowers".into());
    let reg = crate::mode::ModeRegistry::load();
    let mode = match reg.get(&mode_name).cloned() {
        Some(m) => m,
        None => {
            mpsc_try_send(&tx, serde_json::json!({"type":"error","message": format!("unknown mode '{}'; available: {}", mode_name, reg.names().join(", "))}));
            close_channel(&tx);
            return;
        }
    };
    // PLAN-069 W1：沙箱工作区 fail-closed 解析（缺失/未知 → error 事件）。
    let ws = match s.0.registry.get_exact(q.workspace.as_deref().unwrap_or("")) {
        Some(w) => w,
        None => {
            mpsc_try_send(&tx, serde_json::json!({"type":"error","message":"unknown or missing workspace; pass ?workspace="}));
            close_channel(&tx);
            return;
        }
    };
    // PLAN-070 T-02：多根 = [workspace 根, *白名单]。
    let ws_roots = s
        .0
        .registry
        .sandbox_roots(q.workspace.as_deref().unwrap_or(""));
    let client = s.0.client.clone();
    let task = b.task.clone();
    let tx2 = tx.clone();
    tokio::spawn(async move {
        // root 经 build_agent 注入（thread-local 退役）。
        let mut agent = match crate::build_agent_from_mode(&mode, client, Some(&ws_roots)) {
            Ok(a) => a,
            Err(e) => {
                mpsc_try_send(&tx2, serde_json::json!({"type":"error","message": format!("build agent: {e}")}));
                close_channel(&tx2);
                return;
            }
        };
        let tx3 = tx2.clone();
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
                let dto: SseEventDto = match &ev {
                    // auto-ai PLAN-026 turn 边界事件:透传为 SSE DTO。
                    StreamEvent::TurnStart { turn } => SseEventDto::TurnStart { turn: *turn },
                    StreamEvent::TurnEnd { turn, tool_count, .. } => SseEventDto::TurnEnd {
                        turn: *turn,
                        tool_count: *tool_count,
                    },
                    StreamEvent::Delta { text } => SseEventDto::Delta { text: text.clone() },
                    StreamEvent::Thinking { text } => {
                        SseEventDto::Thinking { thinking: text.clone() }
                    }
                    StreamEvent::ToolStart { tool, args } => SseEventDto::ToolCall {
                        id: id.clone(),
                        name: tool.clone(),
                        arguments: args.clone(),
                    },
                    StreamEvent::Tool { tool, args, result, details: _ } => SseEventDto::ToolResult {
                        id: id.clone(),
                        name: tool.clone(),
                        arguments: args.clone(),
                        result: result.clone(),
                        status: "success".into(),
                    },
                    StreamEvent::Warning { text } => SseEventDto::Warning { text: text.clone() },
                    StreamEvent::Done { result } => SseEventDto::Done {
                        output: result.output.clone(),
                        turns: result.turns as i32,
                        tool_calls: result
                            .tool_calls
                            .iter()
                            .map(|tc| StreamToolCallOut {
                                name: tc.tool.clone(),
                                arguments: tc.args.clone(),
                                result: tc.result.clone(),
                            })
                            .collect(),
                    },
                    StreamEvent::Cancelled { result } => SseEventDto::Cancelled {
                        output: result.output.clone(),
                        turns: result.turns as i32,
                        tool_calls: result
                            .tool_calls
                            .iter()
                            .map(|tc| StreamToolCallOut {
                                name: tc.tool.clone(),
                                arguments: tc.args.clone(),
                                result: tc.result.clone(),
                            })
                            .collect(),
                    },
                    StreamEvent::Error { message } => {
                        SseEventDto::Error { message: message.clone() }
                    }
                };
                let value = serde_json::to_value(&dto).unwrap_or(Value::Null);
                mpsc_try_send(&tx3, value);
            });
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        if let Err(e) = agent.run_stream(&task, on_event, cancel).await {
            mpsc_try_send(&tx2, serde_json::json!({"type":"error","message": format!("{e}")}));
        }
        close_channel(&tx2);
    });
}
/// PLAN-071 T-02：chat SSE 订阅（GET /api/chats/session/{id}/stream 与 VM 桥
/// host 共用）——恒为**附加/空闲订阅**：在途运行 → 附加转发 relay_bus 事件
/// 直至 done；不在途 → 空闲流（挂起等下一次运行）。绝不孵化运行、不触碰
/// per-session 守卫。运行孵化唯一入口 = chats_message run=true → spawn
/// chat_run_owner（会话 18683b29 双回复实证：F-03 以共享守卫窥探分流主体/
/// 订阅者，在途订阅会并行再跑一轮——结构性缺陷，改显式角色分离）。
pub async fn chat_run_stream(
    s: &State<AppState>,
    q: Query<StreamWorkspaceQuery>,
    p: Path<String>,
    tx: Value,
) {
    let session_id = p.0.clone();
    let mut attach_rx = crate::relay::api::relay_bus().subscribe();
    // PLAN-073 P2-T4：附加窥探——run 守卫空闲时立即回 idle 帧收流（前端
    // 据此知道无在途 run，不再悬挂空流；server.rs chat_run_active 只读
    // 窥探原注释自述"订阅路径据此决定附加转发或空闲等待"但未接线，本处
    // 补上）。先订阅后窥探：订阅 → 收束广播 → 窥探 的窗口内 done 已在
    // 队列里，try_recv 捞到匹配事件则照常进收流循环（不丢尾巴）。
    let probe_key = format!(
        "{}:{}",
        q.workspace.clone().unwrap_or_default(),
        session_id
    );
    if !s.0.chat_run_active(&probe_key) {
        // 窥探瞬间恰逢收束：done 可能已广播在队列里（守卫在 done 之后、
        // 终版落盘之后才清）——捞到本会话 done 就转发尾巴，否则 idle。
        let tail_done = match attach_rx.try_recv() {
            Ok(ev)
                if ev.run_id == session_id
                    && ev.payload.get("type").and_then(|t| t.as_str()) == Some("done") =>
            {
                Some(ev.payload)
            }
            _ => None,
        };
        match tail_done {
            Some(payload) => mpsc_try_send(&tx, payload),
            None => mpsc_try_send(&tx, serde_json::json!({"type": "idle"})),
        }
        // PLAN-071 T-02：收束必须析构 channel pair——桥接 Sender 常驻 HANDLES 表，
        // tx Value 掉落不关闭通道（旧附加分支漏调，SSE 永不收束）。
        close_channel(&tx);
        return;
    }
    chat_run_stream_loop(&mut attach_rx, &session_id, &tx).await;
    // PLAN-071 T-02：收束必须析构 channel pair——桥接 Sender 常驻 HANDLES 表，
    // tx Value 掉落不关闭通道（旧附加分支漏调，SSE 永不收束）。
    close_channel(&tx);
}

/// PLAN-073 P2-T4：附加收流循环（chat_run_stream 主体拆出，供窥探两路
/// 复用）——转发本 session 的 chat_event/tool_update/gate 事件，done 即止。
async fn chat_run_stream_loop(
    attach_rx: &mut tokio::sync::broadcast::Receiver<crate::relay::api::BusEvent>,
    session_id: &str,
    tx: &Value,
) {
    loop {
        match attach_rx.recv().await {
            Ok(ev) => {
                if ev.run_id != session_id {
                    continue;
                }
                let ty = ev.event_type.as_str();
                if ty != "chat_event"
                    && ty != "tool_update"
                    && ty != "tool_gate_waiting"
                    && ty != "relay_gate_waiting"
                {
                    continue;
                }
                let is_done = ty == "chat_event"
                    && ev.payload.get("type").and_then(|t| t.as_str()) == Some("done");
                mpsc_try_send(tx, ev.payload.clone());
                if is_done {
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(_) => break,
        }
    }
}

/// PLAN-073 P2-T2/P2-T3：chat run 消息组装（超时/失败/成功收束三臂与
/// turn_end 增量快照共用）。clone 语义——不消耗累积器（快照与终版可多次
/// 组装同一来源）。结构对齐 r4 T-04 超时臂样板：封口叙述块 → 思考首块 →
/// 可选尾块（超时/失败的 ⚠ 文案）。
fn assemble_chat_run_msg(
    role: &str,
    content: String,
    thinking: String,
    tcs: Vec<crate::chats::ToolCall>,
    pending_text: &str,
    blocks: &[crate::chats::ChatBlock],
    tail: Option<&str>,
) -> crate::chats::ChatMessage {
    let mut msg = crate::chats::ChatMessage::assistant(content);
    msg.thinking = thinking;
    msg.tool_calls = tcs;
    msg.profession_id = Some(role.to_string());
    let mut bl = blocks.to_vec();
    if !pending_text.is_empty() {
        bl.push(crate::chats::ChatBlock {
            kind: "text".into(),
            text: pending_text.to_string(),
            tool: None,
        });
    }
    if !msg.thinking.is_empty() {
        bl.insert(
            0,
            crate::chats::ChatBlock {
                kind: "thinking".into(),
                text: msg.thinking.clone(),
                tool: None,
            },
        );
    }
    if let Some(t) = tail {
        bl.push(crate::chats::ChatBlock {
            kind: "text".into(),
            text: t.to_string(),
            tool: None,
        });
    }
    msg.blocks = bl;
    msg
}

/// PLAN-073 P2-T2/P2-T3：收束/快照落盘统一口——upsert（终版同 id 原位替换
/// turn 快照），失败与 session 缺失均显式 warn（原三臂 `let _ =` 静默吞错）。
fn persist_chat_run_msg(
    chats: &crate::chats::ChatStore,
    session_id: &str,
    msg: &crate::chats::ChatMessage,
    phase: &str,
) {
    match chats.upsert_message(session_id, msg.clone()) {
        Ok(Some(_)) => {}
        Ok(None) => tracing::warn!(
            "chat run persist skipped (session gone): session={} phase={}",
            session_id,
            phase
        ),
        Err(e) => tracing::warn!(
            "chat run persist failed: session={} phase={} error={}",
            session_id,
            phase,
            e
        ),
    }
}

/// PLAN-071 T-02：chat 运行主体（原 chat_run_stream 运行躯干；PLAN-055 ⑧ D1
/// 守卫语义不变）——session + history + build_agent_with_context + run_stream +
/// 完成后持久化(append_message + 双写 conversation turns)。唯一孵化入口 =
/// chats_message run=true → chat_run_try_start 抢到守卫后 spawn 本函数；守卫
/// 由本函数全部出口清除。SSE 订阅路径（chat_run_stream）不进入本函数。
pub async fn chat_run_owner(
    s: &State<AppState>,
    q: Query<StreamWorkspaceQuery>,
    p: Path<String>,
    tx: Value,
) {
    let ws_id = q.workspace.clone().unwrap_or_default();
    let ws = s.0.registry.get(&ws_id);
    let session_id = p.0.clone();
    // PLAN-055 ⑧(D1): per-session run 守卫键——由 chats_message 置位，
    // 本函数全部出口清除。
    let run_key = format!("{ws_id}:{session_id}");
    // PLAN-071 T-02：事件双发补全——运行主体把全部 SSE 事件镜像上 relay_bus
    // （run_id=session_id，event_type="chat_event"），SSE 订阅方
    // （chat_run_stream 附加/空闲流）据此转发。PLAN-069 T-06 的 emit_bus 闭包
    // 此前只有定义无调用点（死代码）：流式事件实际只进本运行 tx，而
    // chats_message spawn 路径 tx 为 Null——订阅者永远收不到 delta/done，
    // "订阅在途可见流"此前由 F-03 缺陷（订阅方自行再跑一轮）意外顶替。
    let emit_bus = {
        let bus_sid = session_id.clone();
        move |v: &serde_json::Value| {
            let _ = crate::relay::api::relay_bus().send(crate::relay::api::BusEvent {
                run_id: bus_sid.clone(),
                event_type: "chat_event".into(),
                payload: v.clone(),
            });
        }
    };
    let session = match ws.chats.get(&session_id) {
        Some(sess) => sess,
        None => {
            s.0.chat_run_finish(&run_key);
            mpsc_try_send(&tx, serde_json::json!({"type":"error","message": format!("session '{session_id}' not found")}));
            close_channel(&tx);
            return;
        }
    };
    let mode = session.mode.clone();
    // The user message to run = the last user turn in history.
    let user_msg =
        match session.messages.iter().rev().find(|m| m.role == crate::chats::Role::User) {
            Some(m) => m.content.clone(),
            None => {
                s.0.chat_run_finish(&run_key);
                mpsc_try_send(&tx, serde_json::json!({"type":"error","message":"no user message to run"}));
                close_channel(&tx);
                return;
            }
        };

    // PLAN-034 修正：`/auto-plan:merge <PLAN-NNN>` 斜杠指令短路——不经 LLM，
    // 按原生 spawn_relay 语义（orch_tools.rs）启动 plan-merge run 并挂到本
    // 会话：助手消息含 spawn_relay tool call（Run 卡片渲染来源）持久化 +
    // 双写 conversation turns，刷新不丢；后台起 run 驱动；SSE 回
    // delta/relay_spawned/done。解析器复用 hw server.rs 的 pub 版。
    // PLAN-071 需求⑤ T-25/T-26：取消旗标注册 + 生命周期日志。
    let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    s.0.chat_cancel_register(&run_key, cancel_flag.clone());
    let run_started = std::time::Instant::now();
    tracing::info!(
        "chat run spawned: session={} ws={} mode={} msg_len={}",
        session_id, ws_id, mode, user_msg.len()
    );
    // PLAN-071 需求⑭：@plan/<seq> / @spec/<relpath> 引用解析——附加注记块
    // 给 agent（指向文件路径 + 计划摘要，agent 用既有 read_plan/文件工具
    // 自取全文）；用户可见/持久化消息保持原文不变。
    let agent_refs_block =
        crate::server::resolve_chat_mention_refs(&user_msg, &ws.plans);

    if let Some(plan_id) = crate::server::parse_plan_merge_command(&user_msg) {
        let task = format!("沉淀 {plan_id} 到 Spec 知识库");
        let req = crate::relay::store::StartRunRequest {
            run_id: None,
            flow_id: Some("plan-merge".into()),
            steps: Vec::new(),
            task: Some(task.clone()),
        };
        let (run_id, _initial) = ws.relay.start_run(&req, Some(ws_id.clone()));
        // PLAN-034 T9：登记发起会话——driver 完成时把报告消息写回这里。
        ws.relay.set_context_var(&run_id, "chat_session_id", &session_id);
        // PLAN-067 T-05：登记审批模式（plan-merge 流无 gate,登记为一致性
        // 语义;后续带门 flow 复用本短路时即生效）。
        ws.relay
            .set_context_var(&run_id, "approval_mode", &session.approval_mode);
        let tc_args = serde_json::json!({
            "flow_id": "plan-merge",
            "task": task,
            "run_id": run_id,
        });
        let tc_result = format!("{{\"run_id\":\"{run_id}\",\"status\":\"started\"}}");
        let tc = crate::chats::ToolCall {
            tool: "spawn_relay".into(),
            args: tc_args.clone(),
            result: tc_result.clone(),
            status: "success".into(),
            id: "tc-1".into(),
        };
        let summary = format!(
            "📦 **智能沉淀已启动**（{plan_id}）\n\nRun `{run_id}`（flow: plan-merge）：\
`merge_plan` 机械沉淀（幂等）→ 按 spec-impact 更新 `docs/specs/` 模块树 → \
`emit_report` 生成 HTML 报告。"
        );
        let mut msg = crate::chats::ChatMessage::assistant(summary.clone());
        // PLAN-071 r3：职业身份 = 会话 mode 的 role（未注册回退 mode 名）。
        msg.profession_id = Some(
            crate::mode::ModeRegistry::load()
                .get(&mode)
                .map(|m| m.role.clone())
                .unwrap_or_else(|| mode.clone()),
        );
        msg.tool_calls = vec![tc];
        let _ = ws.chats.append_message(&session_id, msg.clone());
        let seq_base = ws
            .conversations
            .get(&session_id)
            .map(|c| c.turns.len())
            .unwrap_or(0);
        for turn in crate::conversation::chat_message_to_turns(&msg, seq_base) {
            let _ = ws.conversations.append_turn(&session_id, turn);
        }
        let state2 = std::sync::Arc::new(s.0.clone());
        let ws_id2 = ws_id.clone();
        let rid = run_id.clone();
        tracing::info!(
            "plan-merge shortcut: driver for {} (session={})",
            rid,
            session_id
        );
        tokio::spawn(async move {
            let _ = crate::auto_generated::relay_driver::drive_run(state2, &ws_id2, &rid).await;
        });
        // SSE（SseEventDto 严格枚举——无 relay_spawned 变体，改用原生
        // tool_call/tool_result 形状携带 run_id，前端据此实时渲染 Run 卡片）。
        // PLAN-071 T-02：同步镜像上总线（本路径 tx 恒为 Null，订阅方靠总线收）。
        let ev_tc = serde_json::json!({"type": "tool_call", "id": "tc-1", "name": "spawn_relay", "arguments": tc_args});
        emit_bus(&ev_tc);
        mpsc_try_send(&tx, ev_tc);
        let ev_tr = serde_json::json!({
            "type": "tool_result", "id": "tc-1", "name": "spawn_relay",
            "arguments": tc_args, "result": tc_result, "status": "success",
        });
        emit_bus(&ev_tr);
        mpsc_try_send(&tx, ev_tr);
        let ev_delta = serde_json::json!({"type": "delta", "text": summary});
        emit_bus(&ev_delta);
        mpsc_try_send(&tx, ev_delta);
        let ev_done = serde_json::json!({
            "type": "done", "output": summary, "turns": 1,
            "tool_calls": [{"name": "spawn_relay", "arguments": tc_args, "result": tc_result}],
        });
        emit_bus(&ev_done);
        mpsc_try_send(&tx, ev_done);
        // PLAN-055 ⑧(D1): plan-merge 短路的 run 由 relay driver 独立驱动，
        // chat 层 run 到此为止——清守卫。
        s.0.chat_cancel_remove(&run_key);
        tracing::info!(
            "chat run finished (plan-merge shortcut): session={} elapsed={}s",
            session_id,
            run_started.elapsed().as_secs()
        );
        s.0.chat_run_finish(&run_key);
        close_channel(&tx);
        return;
    }

    // PLAN-043: 历史按活跃路径投影(分叉后只喂当前分支;旧数据线性退化)。
    let history: Vec<(String, String)> = session.history_pairs();

    // Resolve the session's mode to an AgentMode (built-in or user .at).
    let mode_reg = crate::mode::ModeRegistry::load();
    let agent_mode = match mode_reg.get(&mode).cloned() {
        Some(m) => m,
        None => mode_reg.get("superpowers").cloned().unwrap_or_else(|| {
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

    let client = s.0.client.clone();
    let chats = ws.chats.clone();
    let conversations = ws.conversations.clone();
    let ws_root = ws.root.clone();
    let state_for_ctx = std::sync::Arc::new(s.0.clone());
    // PLAN-071 需求⑤ T-22：空闲看门狗窗口（env AUTO_RUN_IDLE_TIMEOUT_SECS，默认 300s）。
    let idle_timeout = s.0.run_idle_timeout;
    let tx2 = tx.clone();
    let run_key2 = run_key.clone();
    // PLAN-064: 会话思考档位（spawn 任务内用克隆）。
    // PLAN-071 需求⑩：默认档位改为 low——未设置（None/旧会话）一律按
    // "low" 生效；显式 "off"（需求⑩起由前端持久化）仍可完全关闭。
    let session_thinking2 = session
        .thinking_level
        .clone()
        .or_else(|| Some("low".to_string()));
    tokio::spawn(async move {
        // PLAN-069 W1：root 注入由 build_agent_with_context 完成（thread-local 退役）。
        // Build agent with orchestration tool context (spawn_relay, dispatch).
        let tool_ctx = crate::tool_context::ToolContext {
            state: state_for_ctx.clone(),
            workspace_id: ws_id.clone(),
            parent_conversation_id: session_id.clone(),
            // PLAN-040 T5：工具进度挂 session_id（chat 场景的 run_id）——
            // 下方 bridge 任务把总线上的 ToolUpdate 桥接进本 SSE 流。
            progress: Some(crate::tool_context::ProgressSink::for_run(&session_id)),
            // PLAN-069 W3：审批模式透传（human → run_command 越界首触挂门）。
            approval_mode: Some(session.approval_mode.clone()),
        };
        let mut agent = match crate::build_agent_with_context(&agent_mode, client, Some(tool_ctx)) {
            Ok(a) => a,
            Err(e) => {
                state_for_ctx.chat_run_finish(&run_key2);
                mpsc_try_send(&tx2, serde_json::json!({"type":"error","message": format!("build agent: {e}")}));
                close_channel(&tx2);
                return;
            }
        };
        // PLAN-064: 会话思考档位 → agent override（None = 跟随 role 默认）。
        agent.set_thinking_level_override(session_thinking2.clone());
        // Pre-load the conversation history so the agent has context.
        agent = agent.with_history(history);

        // Accumulate the streamed text + thinking + tool calls to persist on completion.
        let accumulated = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        // PLAN-069 T-06 + PLAN-071 T-02：事件双发总线——emit_bus 自
        // chat_run_owner 作用域捕获进本任务（原闭包死代码已上移接线）；
        // 订阅者按 run_id==session_id 过滤接收，任意数量订阅者、断线重连
        // 只附加不重跑。
        let thinking_acc = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let tool_calls: std::sync::Arc<std::sync::Mutex<Vec<crate::chats::ToolCall>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        // PLAN-069 W2：活动时间线块（执行序）——Delta 追加当前叙述块，
        // TurnStart/工具边界翻新叙述块，工具成对入块。
        let blocks: std::sync::Arc<std::sync::Mutex<Vec<crate::chats::ChatBlock>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let cur_text: std::sync::Arc<std::sync::Mutex<String>> =
            std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let tool_idx: std::sync::Arc<
            std::sync::Mutex<std::collections::HashMap<String, usize>>,
        > = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
        let tx3 = tx2.clone();
        // PLAN-071 需求⑤ T-22：空闲看门狗通道——on_event 每帧 touch。
        let (wd_tx, mut wd_rx) = tokio::sync::watch::channel(std::time::Instant::now());
        let wd_tx2 = wd_tx.clone();
        let acc2 = accumulated.clone();
        let think2 = thinking_acc.clone();
        let tc2 = tool_calls.clone();
        let tc_counter = std::sync::Arc::new(std::sync::Mutex::new(0usize));
        let tc_stack: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let blocks2 = blocks.clone();
        let cur_text2 = cur_text.clone();
        let tool_idx2 = tool_idx.clone();
        // PLAN-073 P2-T3：run 消息 id——turn 快照与收束终版共用（upsert 原位
        // 替换的键）。快照闭包：turn_end 边界把当前积累（clone，不消耗累积
        // 器）组装成 pending 消息落盘——刷新页面即可见已完成轮次的现场，
        // 收束终版以同 id 换入并清 pending。
        let run_msg_id = crate::chats::new_message_id();
        let persist_snapshot = {
            let chats_snap = chats.clone();
            let sid_snap = session_id.clone();
            let rid_snap = run_msg_id.clone();
            let role_snap = agent_mode.role.clone();
            let acc_s = accumulated.clone();
            let think_s = thinking_acc.clone();
            let tcs_s = tool_calls.clone();
            let bl_s = blocks.clone();
            let ct_s = cur_text.clone();
            move || {
                let content = acc_s.lock().unwrap().clone();
                let thinking = think_s.lock().unwrap().clone();
                let tcs_v = tcs_s.lock().unwrap().clone();
                let pending_text = ct_s.lock().unwrap().clone();
                let blocks_v = bl_s.lock().unwrap().clone();
                if content.is_empty()
                    && thinking.is_empty()
                    && tcs_v.is_empty()
                    && blocks_v.is_empty()
                    && pending_text.is_empty()
                {
                    return;
                }
                let mut msg = assemble_chat_run_msg(
                    &role_snap,
                    content,
                    thinking,
                    tcs_v,
                    &pending_text,
                    &blocks_v,
                    None,
                );
                msg.id = rid_snap.clone();
                msg.pending = Some(true);
                persist_chat_run_msg(&chats_snap, &sid_snap, &msg, "turn_snapshot");
            }
        };
        // PLAN-071 需求⑤：超时/失败落盘臂专用的总线发射器（emit_bus 本体
        // 已被 on_event 捕获）。
        let emit_bus_tail = emit_bus.clone();
        let on_event: Arc<dyn Fn(auto_ai_agent::StreamEvent) + Send + Sync> =
            Arc::new(move |ev| {
                use auto_ai_agent::StreamEvent;
                let _ = wd_tx2.send(std::time::Instant::now());
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
                let dto: SseEventDto = match &ev {
                    // auto-ai PLAN-026 turn 边界事件:透传为 SSE DTO。
                    StreamEvent::TurnStart { turn } => SseEventDto::TurnStart { turn: *turn },
                    StreamEvent::TurnEnd { turn, tool_count, .. } => SseEventDto::TurnEnd {
                        turn: *turn,
                        tool_count: *tool_count,
                    },
                    StreamEvent::Delta { text } => SseEventDto::Delta { text: text.clone() },
                    StreamEvent::Thinking { text } => {
                        SseEventDto::Thinking { thinking: text.clone() }
                    }
                    StreamEvent::ToolStart { tool, args } => SseEventDto::ToolCall {
                        id: id.clone(),
                        name: tool.clone(),
                        arguments: args.clone(),
                    },
                    StreamEvent::Tool { tool, args, result, details: _ } => SseEventDto::ToolResult {
                        id: id.clone(),
                        name: tool.clone(),
                        arguments: args.clone(),
                        result: result.clone(),
                        status: "success".into(),
                    },
                    StreamEvent::Warning { text } => SseEventDto::Warning { text: text.clone() },
                    StreamEvent::Done { result } => SseEventDto::Done {
                        output: result.output.clone(),
                        turns: result.turns as i32,
                        tool_calls: result
                            .tool_calls
                            .iter()
                            .map(|tc| StreamToolCallOut {
                                name: tc.tool.clone(),
                                arguments: tc.args.clone(),
                                result: tc.result.clone(),
                            })
                            .collect(),
                    },
                    StreamEvent::Cancelled { result } => SseEventDto::Cancelled {
                        output: result.output.clone(),
                        turns: result.turns as i32,
                        tool_calls: result
                            .tool_calls
                            .iter()
                            .map(|tc| StreamToolCallOut {
                                name: tc.tool.clone(),
                                arguments: tc.args.clone(),
                                result: tc.result.clone(),
                            })
                            .collect(),
                    },
                    StreamEvent::Error { message } => {
                        SseEventDto::Error { message: message.clone() }
                    }
                };
                let value = serde_json::to_value(&dto).unwrap_or(Value::Null);
                // PLAN-071 T-02：双发——镜像上总线供订阅方（chat_run_stream）
                // 转发；本运行 tx 在 chats_message spawn 路径为 Null，总线是
                // 唯一活通道。
                emit_bus(&value);
                // PLAN-069 W2：块维护（先于 capture）。
                {
                    let ev_type = value.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    match ev_type {
                        "turn_start" => {
                            // 新迭代：封口当前叙述块（下个 delta 另起新块）。
                            cur_text2.lock().unwrap().clear();
                        }
                        "delta" => {
                            if let Some(text) = value.get("text").and_then(|t| t.as_str()) {
                                cur_text2.lock().unwrap().push_str(text);
                            }
                        }
                        "tool_call" => {
                            let pending = cur_text2.lock().unwrap().clone();
                            if !pending.is_empty() {
                                blocks2.lock().unwrap().push(crate::chats::ChatBlock {
                                    kind: "text".into(),
                                    text: pending,
                                    tool: None,
                                });
                                cur_text2.lock().unwrap().clear();
                            }
                            let id = value
                                .get("id")
                                .and_then(|t| t.as_str())
                                .unwrap_or("")
                                .to_string();
                            let tc = crate::chats::ToolCall {
                                tool: value
                                    .get("name")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                args: value.get("arguments").cloned().unwrap_or(Value::Null),
                                result: String::new(),
                                status: "running".into(),
                                id: id.clone(),
                            };
                            let mut bl = blocks2.lock().unwrap();
                            bl.push(crate::chats::ChatBlock {
                                kind: "tool".into(),
                                text: String::new(),
                                tool: Some(tc),
                            });
                            tool_idx2.lock().unwrap().insert(id, bl.len() - 1);
                        }
                        "tool_result" => {
                            let id = value
                                .get("id")
                                .and_then(|t| t.as_str())
                                .unwrap_or("")
                                .to_string();
                            let idx = tool_idx2.lock().unwrap().get(&id).copied();
                            if let Some(idx) = idx {
                                let mut bl = blocks2.lock().unwrap();
                                if let Some(block) = bl.get_mut(idx) {
                                    if let Some(tc) = block.tool.as_mut() {
                                        tc.result = value
                                            .get("result")
                                            .and_then(|t| t.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        tc.status = value
                                            .get("status")
                                            .and_then(|t| t.as_str())
                                            .unwrap_or("success")
                                            .to_string();
                                    }
                                }
                            }
                        }
                        "done" => {
                            let pending = cur_text2.lock().unwrap().clone();
                            if !pending.is_empty() {
                                blocks2.lock().unwrap().push(crate::chats::ChatBlock {
                                    kind: "text".into(),
                                    text: pending,
                                    tool: None,
                                });
                                cur_text2.lock().unwrap().clear();
                            }
                        }
                        // PLAN-073 P2-T3：turn 边界增量落盘——快照闭包内
                        // clone 组装（不动累积器），刷新页面即可见已完成
                        // 轮次的现场。
                        "turn_end" => {
                            persist_snapshot();
                        }
                        // PLAN-073 T-03/T-04：daemon 降级告警（如工具流式参数
                        // 解析失败被替换为 {}）→ ⚠️ 文本块入时间线，用户可见、
                        // 持久化随块走（先封口当前叙述块避免粘连）。
                        "warning" => {
                            let warn_text = value
                                .get("text")
                                .and_then(|t| t.as_str())
                                .unwrap_or("");
                            if !warn_text.is_empty() {
                                let pending = cur_text2.lock().unwrap().clone();
                                let mut bl = blocks2.lock().unwrap();
                                if !pending.is_empty() {
                                    bl.push(crate::chats::ChatBlock {
                                        kind: "text".into(),
                                        text: pending,
                                        tool: None,
                                    });
                                    cur_text2.lock().unwrap().clear();
                                }
                                bl.push(crate::chats::ChatBlock {
                                    kind: "text".into(),
                                    text: format!("⚠️ {}", warn_text),
                                    tool: None,
                                });
                            }
                        }
                        _ => {}
                    }
                }
                // capture for persistence
                if let Some(text) = value.get("text").and_then(|t| t.as_str()) {
                    acc2.lock().unwrap().push_str(text);
                }
                if let Some(text) = value.get("thinking").and_then(|t| t.as_str()) {
                    think2.lock().unwrap().push_str(text);
                }
                if value.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                    let tool = value.get("name").and_then(|t| t.as_str()).unwrap_or("").to_string();
                    let args = value.get("arguments").cloned().unwrap_or(Value::Null);
                    let result = value.get("result").and_then(|t| t.as_str()).unwrap_or("").to_string();
                    tc2.lock().unwrap().push(crate::chats::ToolCall {
                        tool, args, result,
                        status: String::from("success"),
                        id: id.unwrap_or_default(),
                    });
                }
                mpsc_try_send(&tx3, value);
            });
        // PLAN-071 需求⑤ T-25：取消旗标来自注册表（cancel 端点置位，
        // agent 于迭代边界检查；流内 stall 由下方空闲看门狗 drop 兜底）。
        let cancel = cancel_flag.clone();
        // PLAN-040 T2：工具流式进度（ToolUpdate）桥接——工具经
        // ToolContext.progress 推上进程级 broadcast 总线，这里订阅并过滤本
        // session 的 tool_update 事件转进 chat SSE（SseEventDto 严格枚举之外
        // 的透传 JSON，前端 useForge 按 type 分发）。run 结束后 abort。
        // PLAN-073 T-01：桥上事件同时喂空闲看门狗——命令执行期没有 agent
        // 流事件（ToolStart 已发、Tool 结果等命令退出），进度是唯一活性
        // 信号；不喂狗则任何 ≥300s 的命令都会被看门狗误杀
        //（会话 90ed3ae0 诊断）。
        let mut bridge_rx = crate::relay::api::relay_bus().subscribe();
        let tx_bridge = tx2.clone();
        let wd_tx_bridge = wd_tx.clone();
        let bridge_sid = session_id.clone();
        let bridge = tokio::spawn(async move {
            loop {
                match bridge_rx.recv().await {
                    Ok(ev) => {
                        // PLAN-067 T-04(a)：放行 relay_gate_waiting——发起
                        // 会话镜像事件（relay/driver.rs WaitForHuman 分支
                        // 以 chat_session_id 为 bus run_id 发射），前端
                        // OnStreamEvent 据此把 Run 卡置 gate_waiting。
                        if ev.run_id == bridge_sid
                            && (ev.event_type == "tool_update"
                                || ev.event_type == "tool_gate_waiting"
                                || ev.event_type == "relay_gate_waiting")
                        {
                            // PLAN-073 r3：gate 等待事件把空闲窗推到门超时
                            // （1800s）之外再留一个完整窗口（+300s 余量）——
                            // 喂 now+1500s 会让看门狗与门超时同时到期再次
                            // 竞速（实测看门狗又胜，运行被误杀）。门超时
                            // 先自然发生 → deny 结果回灌模型，运行继续；
                            // 看门狗只兜底真正的管道级停顿。
                            if ev.event_type == "tool_gate_waiting"
                                || ev.event_type == "relay_gate_waiting"
                            {
                                let _ = wd_tx_bridge.send(
                                    std::time::Instant::now()
                                        + std::time::Duration::from_secs(1800),
                                );
                            } else {
                                let _ = wd_tx_bridge.send(std::time::Instant::now());
                            }
                            mpsc_try_send(&tx_bridge, ev.payload);
                        }
                    }
                    // partial 是易态，Lagged（背压丢帧）可接受——继续收。
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
        });
        // PLAN-071 需求⑤ T-22：空闲看门狗 select——窗口内无任何流式事件即
        // drop run_stream future（中止在途 LLM 流），按超时收束并落盘。
        // PLAN-071 需求⑭：agent 输入 = 原文 + @引用解析块（有引用才追加）。
        let agent_input = if agent_refs_block.is_empty() {
            user_msg.clone()
        } else {
            format!("{}{}", user_msg, agent_refs_block)
        };
        let mut idle_timed_out = false;
        let run_out = tokio::select! {
            r = agent.run_stream(&agent_input, on_event, cancel) => Some(r),
            _ = async {
                loop {
                    let last = *wd_rx.borrow();
                    tokio::time::sleep_until(tokio::time::Instant::from_std(last + idle_timeout)).await;
                    if *wd_rx.borrow() <= last { break; }
                }
            } => { idle_timed_out = true; None }
        };
        if idle_timed_out {
            // 需求⑤：超时收束——错误事件上总线/SSE，且错误消息**持久化**。
            // PLAN-073 T-04：中止前的部分产物（叙述/思考/工具卡/时间线）按
            // 成功路径同款组装落盘，超时通知作尾块追加——会话 90ed3ae0 里
            // 用户看到的执行痕迹曾随超时整体丢失，只剩一行超时文案。
            let text = format!(
                "⏱ 运行空闲超时：{}s 内无任何流式事件，已中止本轮。建议把任务拆小后重发。",
                idle_timeout.as_secs()
            );
            tracing::warn!(
                "chat run idle-timeout: session={} idle={}s elapsed={}s",
                session_id, idle_timeout.as_secs(), run_started.elapsed().as_secs()
            );
            let partial = std::mem::take(&mut *accumulated.lock().unwrap());
            let thinking = std::mem::take(&mut *thinking_acc.lock().unwrap());
            let tcs = std::mem::take(&mut *tool_calls.lock().unwrap());
            let content = if partial.is_empty() {
                text.clone()
            } else {
                format!("{}\n\n{}", partial, text)
            };
            // PLAN-073 P2-T2/P2-T3：组装收口 assemble_chat_run_msg（W2 同款
            // 结构：封口叙述块 → 思考首块 → ⚠ 尾块）；终版与 turn 快照同 id
            // 原位替换，落盘失败显式 warn（不再 `let _ =` 静默）。
            let pending_text = cur_text.lock().unwrap().clone();
            let blocks_v = std::mem::take(&mut *blocks.lock().unwrap());
            let mut msg = assemble_chat_run_msg(
                &agent_mode.role,
                content,
                thinking,
                tcs,
                &pending_text,
                &blocks_v,
                Some(&text),
            );
            msg.id = run_msg_id.clone();
            persist_chat_run_msg(&chats, &session_id, &msg, "idle_timeout");
            let seq_base = conversations
                .get(&session_id)
                .map(|c| c.turns.len())
                .unwrap_or(0);
            for turn in crate::conversation::chat_message_to_turns(&msg, seq_base) {
                let _ = conversations.append_turn(&session_id, turn);
            }
            emit_bus_tail(&serde_json::json!({"type": "error", "message": text}));
            emit_bus_tail(&serde_json::json!({"type": "done", "output": text, "turns": 0}));
            mpsc_try_send(&tx2, serde_json::json!({"type": "error", "message": text}));
        } else {
            match run_out.unwrap() {
            Ok(_) => {
                // Persist the assistant reply + thinking + tool calls.
                let text = std::mem::take(&mut *accumulated.lock().unwrap());
                let thinking = std::mem::take(&mut *thinking_acc.lock().unwrap());
                let tcs = std::mem::take(&mut *tool_calls.lock().unwrap());
                // PLAN-073 P2-T2/P2-T3：组装收口 assemble_chat_run_msg——
                // PLAN-069 W2 封口叙述块挂时间线 + PLAN-071 需求⑦ T-31 思考
                // 沉淀为首块（blocks 非空时前端只渲染 blocks）；终版与 turn
                // 快照同 id 原位替换（run_msg_id），落盘失败显式 warn。
                let pending_text = cur_text.lock().unwrap().clone();
                let blocks_v = std::mem::take(&mut *blocks.lock().unwrap());
                let mut msg = assemble_chat_run_msg(
                    &agent_mode.role,
                    text,
                    thinking,
                    tcs,
                    &pending_text,
                    &blocks_v,
                    None,
                );
                msg.id = run_msg_id.clone();
                persist_chat_run_msg(&chats, &session_id, &msg, "done");
                // Dual-write: mirror the assistant message (+ tool calls) into
                // the conversation as turns.
                let seq_base = conversations
                    .get(&session_id)
                    .map(|c| c.turns.len())
                    .unwrap_or(0);
                for turn in crate::conversation::chat_message_to_turns(&msg, seq_base) {
                    let _ = conversations.append_turn(&session_id, turn);
                }
                tracing::info!(
                    "chat run finished: session={} elapsed={}s",
                    session_id,
                    run_started.elapsed().as_secs()
                );
            }
            Err(e) => {
                // PLAN-071 需求⑤ T-22：失败**落盘**（原只发瞬态 SSE——失败对
                // 用户不可见），双写 turns + 总线错误事件。
                tracing::warn!("chat run failed: session={} error={}", session_id, e);
                // PLAN-073 P2-T2：失败臂保现场——对齐超时臂 T-04 形态组装已
                // 积累的 thinking/工具卡/叙述（此前只落一行 ⚠ 文本，24a43d74
                // 跑满 100 轮失败后全部现场丢失），⚠ 文案作尾块追加；终版与
                // turn 快照同 id 原位替换。
                let partial = std::mem::take(&mut *accumulated.lock().unwrap());
                let thinking = std::mem::take(&mut *thinking_acc.lock().unwrap());
                let tcs = std::mem::take(&mut *tool_calls.lock().unwrap());
                let text = format!("⚠ 运行失败：{e}");
                let content = if partial.is_empty() {
                    text.clone()
                } else {
                    format!("{}\n\n{}", partial, text)
                };
                let pending_text = cur_text.lock().unwrap().clone();
                let blocks_v = std::mem::take(&mut *blocks.lock().unwrap());
                let mut msg = assemble_chat_run_msg(
                    &agent_mode.role,
                    content,
                    thinking,
                    tcs,
                    &pending_text,
                    &blocks_v,
                    Some(&text),
                );
                msg.id = run_msg_id.clone();
                persist_chat_run_msg(&chats, &session_id, &msg, "failed");
                let seq_base = conversations
                    .get(&session_id)
                    .map(|c| c.turns.len())
                    .unwrap_or(0);
                for turn in crate::conversation::chat_message_to_turns(&msg, seq_base) {
                    let _ = conversations.append_turn(&session_id, turn);
                }
                emit_bus_tail(&serde_json::json!({"type": "error", "message": text}));
                emit_bus_tail(&serde_json::json!({"type": "done", "output": text, "turns": 0}));
                mpsc_try_send(&tx2, serde_json::json!({"type": "error", "message": text}));
            }
            }
        }
        bridge.abort();
        // PLAN-055 ⑧(D1): run 结束（Ok/Err 两路汇合点）——清 per-session 守卫。
        state_for_ctx.chat_run_finish(&run_key2);
        close_channel(&tx2);
    });
}
/// Plan 020 Phase G (relay_driver.at run_step): bridge agent.run_stream's
/// `Arc<dyn Fn(StreamEvent)>` callback to the ag `DriveStreamSink` (which forwards
/// each event to drive_handle_stream_event via its on_event(Value)). Serializes
/// each StreamEvent to a serde Value, calls sink.on_event, then runs the agent.
/// Returns the AgentResult serialized to Value (output/turns/tool_calls/total_tokens).
pub fn agent_run_stream_with_sink(mut a: Agent, t: String, sink: Arc<crate::auto_generated::relay_driver::DriveStreamSink>, c: Arc<std::sync::atomic::AtomicBool>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, String>> + Send>> {
    Box::pin(async move {
        let on_event: Arc<dyn Fn(auto_ai_agent::StreamEvent) + Send + Sync> = Arc::new(move |ev| {
            // Serialize the StreamEvent and forward to the sink (which calls
            // drive_handle_stream_event → 8-branch match → push_event + accumulate).
            if let Ok(v) = serde_json::to_value(&ev) {
                // sink is Arc<DriveStreamSink>; deref to the DriveStreamSink which
                // implements DriveSink.
                crate::auto_generated::relay_driver::DriveSink::on_event(&*sink, v);
            }
        });
        match a.run_stream(&t, on_event, c).await {
            Ok(result) => serde_json::to_value(&result).map_err(|e| format!("agent: {e}")),
            Err(e) => Err(format!("agent: {e}")),
        }
    })
}
pub fn serve_init_state(_c: Arc<dyn Client>) -> AppState { unimplemented!("serve_init_state") }
pub fn serve_build_static() -> () {}
pub fn serve_build_cors() -> () {}
pub fn serve_build_app(_s: AppState, _st: (), _c: ()) -> () {}
pub async fn serve_listen(_a: &str, _app: ()) {}
pub fn stream_event_map(e: Option<Value>) -> SseEventDto {
    match e {
        Some(v) => serde_json::from_value(v)
            .unwrap_or_else(|_| SseEventDto::Error { message: "malformed stream event".into() }),
        None => SseEventDto::Error { message: String::new() },
    }
}
pub fn workflow_event_map(e: Option<Value>) -> WorkflowEventDto {
    match e {
        Some(v) => serde_json::from_value(v).unwrap_or_else(|_| WorkflowEventDto::StepSkipped {
            step_id: String::new(),
        }),
        None => WorkflowEventDto::StepSkipped { step_id: String::new() },
    }
}
pub fn step_err_is_err(e: &Result<String, String>) -> bool { e.is_err() }
pub fn resolve_within_project(p: &str) -> String { p.to_string() }
pub fn write_file_do(p: &str, c: &str) { let _ = std::fs::write(p, c); }
pub fn command_needs_approval(c: &str) -> bool { !c.starts_with("echo") }
pub fn run_shell_command(c: &str) -> String { format!("(stub) {}", c) }
pub fn edit_file_do(p: &str, o: &str, n: &str) -> String {
    let c = std::fs::read_to_string(p).unwrap_or_default();
    let n2 = c.replacen(o, n, 1); let _ = std::fs::write(p, n2); "ok".into()
}
pub fn search_files(p: &str) -> String { format!("(stub) {}", p) }
pub fn list_directory(p: &str) -> String { format!("(stub) {}", p) }
pub fn list_symbols_in(p: &str) -> String { format!("(stub) {}", p) }
pub fn glob_files(p: &str) -> String { format!("(stub) {}", p) }
pub fn http_post_json(_u: &str) -> impl std::future::Future<Output = Result<Value, String>> { async { Ok(Value::Null) } }
/// Plan 019 Phase 3/4: mpsc side-table —— channel pair 存注册表(Value 存
/// i64 id),tx 句柄是 `{"pair": id}` 指针(不 clone Sender,保证 run 结束后
/// close_channel 移除 pair 即可让 channel 关闭 → 流 break 终止)。
pub fn mpsc_channel() -> Value {
    let (tx, rx) = tokio::sync::mpsc::channel::<Value>(64);
    let id = next_handle_id();
    HANDLES.lock().unwrap().insert(id, Box::new(ChannelPair { tx, rx: Some(rx) }));
    serde_json::json!(id)
}
pub fn mpsc_sender(ch: &Value) -> Value {
    match ch.as_i64() {
        Some(pair_id) => serde_json::json!({ "pair": pair_id }),
        None => Value::Null,
    }
}
pub fn mpsc_receiver(ch: &Value) -> Value {
    let pair_id = match ch.as_i64() {
        Some(i) => i,
        None => return Value::Null,
    };
    let mut handles = HANDLES.lock().unwrap();
    let rx = match handles
        .get_mut(&pair_id)
        .and_then(|b| b.downcast_mut::<ChannelPair>())
        .and_then(|p| p.rx.take())
    {
        Some(rx) => rx,
        None => return Value::Null,
    };
    let id = next_handle_id();
    handles.insert(id, Box::new(rx));
    serde_json::json!(id)
}
pub fn mpsc_try_send(t: &Value, m: Value) {
    let pair_id = match t.get("pair").and_then(|v| v.as_i64()) {
        Some(i) => i,
        None => return,
    };
    let handles = HANDLES.lock().unwrap();
    if let Some(pair) = handles.get(&pair_id).and_then(|b| b.downcast_ref::<ChannelPair>()) {
        let _ = pair.tx.try_send(m);
    }
}
pub async fn mpsc_recv(r: &Value) -> Option<Value> {
    let id = match r.as_i64() {
        Some(i) => i,
        None => return None,
    };
    let mut rx = {
        let mut handles = HANDLES.lock().unwrap();
        match handles.remove(&id) {
            Some(b) => match b.downcast::<tokio::sync::mpsc::Receiver<Value>>() {
                Ok(rx) => *rx,
                Err(b) => {
                    handles.insert(id, b);
                    return None;
                }
            },
            None => return None,
        }
    };
    let result = rx.recv().await;
    if result.is_some() {
        HANDLES.lock().unwrap().insert(id, Box::new(rx));
    }
    // None → channel closed:不重新入表,stream 侧 break 后句柄即回收。
    result
}
pub fn msg_is_none(m: &Option<Value>) -> bool { m.is_none() }
pub fn msg_unwrap(m: Option<Value>) -> Value { m.unwrap_or(Value::Null) }
/// Plan 019 Phase 2 + §6.2: broadcast_recv —— 从 `BroadcastSub` 借出 Receiver
/// 收一条 ConversationEvent(序列化为 hw wire 形状 Value)。Lagged 跳过积压继续
/// 流(hw BroadcastStream 语义),Closed 返回 None 让 .at 的 break 终止流。
///
/// rx 不再走 side-table 的 remove/insert 来回搬动,而是锁 `BroadcastSub` 的
/// Arc<Mutex<Option<Receiver>>>:take 出来 recv,Ok 后 put 回;Closed 后不 put 回
/// (rx 析构)。`BroadcastSub` 随 stream drop 时统一清理 lease 条目(见 Drop)。
pub async fn broadcast_recv(sub: &BroadcastSub) -> Option<Value> {
    let mut rx = sub.inner.lock().unwrap().take()?;
    loop {
        match rx.recv().await {
            Ok(ev) => {
                let value = serde_json::json!({
                    "conversation_id": ev.conversation_id,
                    "turn": ev.turn,
                    "status": ev.status,
                });
                sub.inner.lock().unwrap().replace(rx);
                return Some(value);
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            // Closed:不 put 回(让 rx 析构);sub drop 时移除 lease 条目。
            Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
        }
    }
}
/// Plan 384 S1: build an axum SSE Event from a serializable DTO + event name,
/// unwrapping the inner json_data Result so callers can `yield event` directly.
///
/// Plan 019 Phase 0a: **根因修复** —— 不调用 `.event(name)`。前端 `EventSource`
/// 只挂 `onmessage`,而按 SSE 协议**带 `event:` 行的消息不进 `onmessage`**(被
/// 路由到 `addEventListener("<name>")` 通道,前端未注册)。hw 的 run/chat/workflow
/// stream 是 raw `data:` 格式(无 event 行,server.rs:437/741/929),ag 必须对齐。
/// 经 axum 0.8.9 `sse.rs` 源码验证:`Event::default().json_data(v)` 产出纯
/// `data: {json}\n\n` 帧;`.event(name)` 才会追加 `event:` 行。保留 `name` 参数
/// 以维持签名稳定(.at 调用点 `sse_event("run", ...)` 与 a2r 透传均零改动)。
pub fn sse_event(_name: &str, dto: Value) -> Event {
    Event::default().json_data(dto).unwrap_or_else(|_| Event::default())
}
pub fn path_inner(p: &Path<String>) -> String { p.0.clone() }
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
/// Plan 020 Phase B (task_plan_registry.at): 写用户 plan 的 .atom 文件
/// (std::fs::write 的 bool 包装;hw map_err 的错误文本经 a2r 桥丢失,
/// 保留 Err/Ok 行为)。
pub fn task_plan_write_atom(path: std::path::PathBuf, content: &str) -> bool {
    std::fs::write(path, content).is_ok()
}
/// Plan 020 Phase B (task_plan_registry.at): 文件扩展名(hw
/// `path.extension().and_then(|e| e.to_str()).unwrap_or("")`)。
pub fn path_extension_str(path: std::path::PathBuf) -> String {
    path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string()
}
/// Plan 020 Phase B (task_plan_registry.at): 读用户 plan 文件
/// (std::fs::read_to_string 的 Result 包装)。
pub fn task_plan_read_file(path: std::path::PathBuf) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}
/// Plan 020 Phase C (wiki.at): 写 wiki 页面 .md / _manifest.json(hw
/// `std::fs::write(...).map_err(|e| format!("Failed to write page: {}", e))`)。
pub fn wiki_write_page(path: std::path::PathBuf, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| format!("Failed to write page: {}", e))
}
/// Plan 020 Phase C (wiki.at): 确保页面父目录存在(hw `if let Some(parent) =
/// page_path.parent() { let _ = fs::create_dir_all(parent); }`)。
pub fn wiki_ensure_parent(path: std::path::PathBuf) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
}
/// Plan 020 Phase C (wiki.at): 删除页面 .md 文件。
pub fn wiki_delete_file(path: std::path::PathBuf) {
    let _ = std::fs::remove_file(path);
}

// ── Plan 020 Phase D: relay_api.at HTTP 层委托 ─────────────────────────────
// relay 的 store/driver/bus 访问经 extern 委托到 hw(crate::relay::*);成功值
// 返回序列化 Value,未找到返回 Value::Null,业务错误返回 {"error":{code,message}}
// 包络(.at handler 经 resp_is_err / value_is_null 转 400/404)。

/// 解析 ?workspace= 并取对应 workspace 的 stores(hw `q.id_or_default` 等价)。
fn relay_ws(s: &State<AppState>, q: &Query<crate::auto_generated::relay_api::WorkspaceQuery>) -> Arc<WorkspaceStores> {
    let hw_q = crate::workspace::WorkspaceQuery { workspace: q.workspace.clone() };
    let ws_id = hw_q.id_or_default(&s.0.registry);
    s.0.registry.get(&ws_id)
}

pub fn relay_runs_list(s: &State<AppState>, q: Query<crate::auto_generated::relay_api::WorkspaceQuery>) -> Value {
    let ws = relay_ws(s, &q);
    serde_json::json!({ "runs": ws.relay.list() })
}

pub fn relay_start_run(
    s: &State<AppState>,
    q: Query<crate::auto_generated::relay_api::WorkspaceQuery>,
    b: Json<crate::auto_generated::relay_store::StartRunRequest>,
) -> Value {
    let hw_q = crate::workspace::WorkspaceQuery { workspace: q.workspace.clone() };
    let ws_id = hw_q.id_or_default(&s.0.registry);
    let ws = s.0.registry.get(&ws_id);
    // ag StartRunRequest → hw StartRunRequest(字段一一映射;GateType 枚举互转)。
    let hw_req = crate::relay::store::StartRunRequest {
        run_id: b.0.run_id.clone(),
        flow_id: b.0.flow_id.clone(),
        steps: b
            .0
            .steps
            .iter()
            .map(|s| crate::relay::store::StartRunStep {
                id: s.id.clone(),
                role_id: s.role_id.clone(),
                gate: s.gate.map(|g| match g {
                    crate::auto_generated::relay_store::GateType::Auto => crate::relay::GateType::Auto,
                    crate::auto_generated::relay_store::GateType::Human => crate::relay::GateType::Human,
                }),
            })
            .collect(),
        task: b.0.task.clone(),
    };
    let (run_id, run_state) = ws.relay.start_run(&hw_req, Some(ws_id.clone()));
    // 合成 relay_update,让任何存活订阅者刷新(hw api.rs start_run 同款)。
    crate::relay::api::publish(
        &run_id,
        &crate::relay::store::RunEvent::RelayUpdate {
            timestamp: relay_now_secs(),
            step_id: String::new(),
            role_id: String::new(),
            status: "idle".into(),
        },
    );
    serde_json::json!({ "run_id": run_id, "state": run_state })
}

fn relay_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn relay_run_get(s: &State<AppState>, q: Query<crate::auto_generated::relay_api::WorkspaceQuery>, r: &str) -> Value {
    let ws = relay_ws(s, &q);
    match ws.relay.get(r) {
        Some(state) => serde_json::to_value(&state).unwrap_or(Value::Null),
        None => Value::Null,
    }
}

pub fn relay_run_delete(s: &State<AppState>, q: Query<crate::auto_generated::relay_api::WorkspaceQuery>, r: &str) -> bool {
    relay_ws(s, &q).relay.delete(r)
}

pub fn relay_run_set_title(s: &State<AppState>, q: Query<crate::auto_generated::relay_api::WorkspaceQuery>, r: &str, t: &str) -> Value {
    match relay_ws(s, &q).relay.set_title(r, t) {
        Some(state) => serde_json::to_value(&state).unwrap_or(Value::Null),
        None => Value::Null,
    }
}

pub fn relay_run_advance(s: &State<AppState>, q: Query<crate::auto_generated::relay_api::WorkspaceQuery>, r: &str) -> Value {
    let hw_q = crate::workspace::WorkspaceQuery { workspace: q.workspace.clone() };
    let ws_id = hw_q.id_or_default(&s.0.registry);
    let ws = s.0.registry.get(&ws_id);
    // Guard:已 running 的 run 不启动第二个 driver(hw api.rs:191-196)。
    if ws.relay.is_running(r) {
        return match ws.relay.get(r) {
            Some(sn) => serde_json::to_value(&sn).unwrap_or(Value::Null),
            None => Value::Null,
        };
    }
    // spawn 后台 driver(hw api.rs:199-203):推进 + 每步 agent 流式运行 +
    // 停在 human gate / 终态。返回立即 snapshot。
    let state_arc = Arc::new(s.0.clone());
    let run_id_clone = r.to_string();
    tokio::spawn(async move {
        let _ = crate::auto_generated::relay_driver::drive_run(state_arc, &ws_id, &run_id_clone).await;
    });
    match ws.relay.get(r) {
        Some(sn) => serde_json::to_value(&sn).unwrap_or(Value::Null),
        None => Value::Null,
    }
}

pub fn relay_submit_handoff(
    s: &State<AppState>,
    q: Query<crate::auto_generated::relay_api::WorkspaceQuery>,
    r: &str,
    h: &Value,
) -> Value {
    let handoff: crate::relay::HandoffDocument = match serde_json::from_value(h.clone()) {
        Ok(hd) => hd,
        Err(e) => {
            return serde_json::json!({"error": {"code": 400, "message": format!("invalid handoff: {e}")}})
        }
    };
    let ws = relay_ws(s, &q);
    match ws.relay.submit_handoff(r, handoff) {
        Some((result, state)) => {
            crate::relay::api::publish_advance_result_with_report(r, &result, ws.relay.run_report(r));
            serde_json::to_value(&state).unwrap_or(Value::Null)
        }
        None => Value::Null,
    }
}

pub fn relay_resolve_gate(
    s: &State<AppState>,
    q: Query<crate::auto_generated::relay_api::WorkspaceQuery>,
    r: &str,
    d: &str,
    f: &str,
) -> Value {
    let hw_q = crate::workspace::WorkspaceQuery { workspace: q.workspace.clone() };
    let ws_id = hw_q.id_or_default(&s.0.registry);
    let ws = s.0.registry.get(&ws_id);
    // .at 侧已校验 decision ∈ {approve, edit, reject};此处只做映射 + reject 反馈。
    let decision = match d {
        "approve" | "edit" => crate::relay::GateDecision::Approve,
        "reject" => crate::relay::GateDecision::Reject { feedback: f.to_string() },
        _ => {
            return serde_json::json!({"error": {"code": 400, "message": format!("unknown gate decision '{d}' (want approve|reject|edit)")}})
        }
    };
    // PLAN-094 T-03（UAT K5）human 臂：plan 流 execute 门上的 approve 在
    // 计划文件未落实时拒绝且不消费门（409，用 reject+feedback 回答
    // advisor）。（与 hw relay/api.rs 同规则，判定核单源 plan_flow。）
    if matches!(decision, crate::relay::GateDecision::Approve) {
        let action = crate::relay::plan_flow::execute_gate_action(
            ws.relay.flow_of(r).as_deref(),
            ws.relay.pending_gate_step(r).as_deref(),
            ws.relay.context_var(r, "plan_file").as_deref(),
        );
        if action == Some(crate::relay::plan_flow::ExecuteGateAction::Fail) {
            return serde_json::json!({"error": {"code": 409, "message": crate::relay::plan_flow::PLAN_GATE_FAIL_ERROR}});
        }
    }
    match ws.relay.resolve_gate(r, decision) {
        Some((result, run_state)) => {
            crate::relay::api::publish_advance_result_with_report(r, &result, ws.relay.run_report(r));
            // ExecuteStep → resume 后台 driver(继续到下一 gate / 终态)。
            if matches!(result, crate::relay::AdvanceResult::ExecuteStep { .. }) {
                let state_arc = Arc::new(s.0.clone());
                let run_id_clone = r.to_string();
                tokio::spawn(async move {
                    let _ = crate::auto_generated::relay_driver::drive_run(state_arc, &ws_id, &run_id_clone).await;
                });
            }
            serde_json::to_value(&run_state).unwrap_or(Value::Null)
        }
        None => Value::Null,
    }
}

pub fn relay_rerun(s: &State<AppState>, q: Query<crate::auto_generated::relay_api::WorkspaceQuery>, r: &str) -> Value {
    match relay_ws(s, &q).relay.rerun(r) {
        Some(state) => serde_json::to_value(&state).unwrap_or(Value::Null),
        None => Value::Null,
    }
}

pub fn relay_professions_list() -> Value {
    let reg = crate::relay::profession::ProfessionRegistry::load();
    serde_json::json!({ "professions": reg.list() })
}

pub fn relay_flows_list() -> Value {
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
    serde_json::json!({ "flows": flows })
}

// ── relay SSE bus (RelaySub) ────────────────────────────────────────────────
// 与 BroadcastSub 同 pattern:rx 包进 Arc<Mutex<Option<Receiver>>>;stream drop
// → Arc 归零 → rx 析构。relay_sub_recv 在 extern 内按 run_id 过滤(hw
// BroadcastStream::filter_map 语义),Closed 返回 None 让 .at 的 break 终止流。

pub struct RelaySub {
    inner: Arc<Mutex<Option<tokio::sync::broadcast::Receiver<crate::relay::api::BusEvent>>>>,
    run_id: String,
}
impl Clone for RelaySub {
    fn clone(&self) -> Self {
        RelaySub { inner: self.inner.clone(), run_id: self.run_id.clone() }
    }
}

pub fn relay_bus_subscribe(run_id: &str) -> RelaySub {
    let rx = crate::relay::api::relay_bus().subscribe();
    RelaySub { inner: Arc::new(Mutex::new(Some(rx))), run_id: run_id.to_string() }
}

pub async fn relay_sub_recv(sub: &RelaySub) -> Option<Value> {
    let mut rx = sub.inner.lock().unwrap().take()?;
    loop {
        match rx.recv().await {
            Ok(ev) => {
                if ev.run_id != sub.run_id {
                    continue;
                }
                let value = serde_json::json!({
                    "run_id": ev.run_id,
                    "event_type": ev.event_type,
                    "payload": ev.payload,
                });
                sub.inner.lock().unwrap().replace(rx);
                return Some(value);
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            // Closed:不 put 回(让 rx 析构);sub drop 时 Arc 归零回收。
            Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
        }
    }
}

/// 带 event 名的 SSE 帧(hw relay api.rs 的 `.event(...)`)。run_events /
/// task_plan_events 的 event 名供前端 EventSource addEventListener 路由。
/// 与 server_stream 的 sse_event(无 event 名,019 根因修复后的 raw data 帧)
/// 各司其职 —— hw relay 两个 SSE handler 确实带 event 名。
pub fn sse_named_event(name: &str, dto: Value) -> Event {
    Event::default().event(name).json_data(dto).unwrap_or_else(|_| Event::default())
}
/// 未命名 SSE 帧（默认 message 事件，EventSource.onmessage 兼容）。
/// PLAN-030 试用修复：relay 两个 SSE（run/task_plan）原用具名帧——
/// onmessage 收不到，RunBox 实时更新全链路失效（与 019 chat 根因同款）。
pub fn sse_plain_event(dto: Value) -> Event {
    Event::default().json_data(dto).unwrap_or_else(|_| Event::default())
}


// ── TaskPlan registry/engine HTTP 层委托 ───────────────────────────────────

pub fn relay_task_plans_list(s: &State<AppState>, q: Query<crate::auto_generated::relay_api::WorkspaceQuery>) -> Value {
    let ws = relay_ws(s, &q);
    let summaries = ws.task_plans.lock().unwrap().list();
    serde_json::json!({ "task_plans": summaries })
}

pub fn relay_task_plan_get(s: &State<AppState>, q: Query<crate::auto_generated::relay_api::WorkspaceQuery>, id: &str) -> Value {
    let ws = relay_ws(s, &q);
    let reg = ws.task_plans.lock().unwrap();
    match reg.get(id) {
        Some(plan) => serde_json::to_value(&plan).unwrap_or(Value::Null),
        None => Value::Null,
    }
}

pub fn relay_task_plan_register(s: &State<AppState>, q: Query<crate::auto_generated::relay_api::WorkspaceQuery>, atom: &str) -> Value {
    let ws = relay_ws(s, &q);
    let mut reg = ws.task_plans.lock().unwrap();
    match reg.register(atom) {
        Ok(plan) => {
            let phase_count = plan.phases.len();
            let run_count = plan.phases.iter().map(|p| p.runs.len()).sum::<usize>();
            serde_json::json!({
                "task_plan_registered": true,
                "id": plan.id,
                "phase_count": phase_count,
                "run_count": run_count,
            })
        }
        Err(e) => {
            serde_json::json!({"error": {"code": 400, "message": format!("register failed: {e}")}})
        }
    }
}

pub fn relay_task_plan_delete(s: &State<AppState>, q: Query<crate::auto_generated::relay_api::WorkspaceQuery>, id: &str) -> bool {
    let ws = relay_ws(s, &q);
    let mut reg = ws.task_plans.lock().unwrap();
    reg.remove(id).is_some()
}

/// 启动 TaskPlan 实例:registry get(404) → engine new/validate(400) →
/// tokio::spawn 背景执行(DriveTaskPlanExecutor 桥接 hw drive_task_plan_run)。
pub fn relay_task_plan_start(
    s: &State<AppState>,
    q: Query<crate::auto_generated::relay_api::WorkspaceQuery>,
    id: &str,
    input: &str,
) -> Value {
    let hw_q = crate::workspace::WorkspaceQuery { workspace: q.workspace.clone() };
    let ws_id = hw_q.id_or_default(&s.0.registry);
    let plan = {
        let ws = s.0.registry.get(&ws_id);
        let reg = ws.task_plans.lock().unwrap();
        match reg.get(id) {
            Some(p) => p,
            None => return Value::Null,
        }
    };
    // hw TaskPlan → ag TaskPlan(serde 往返;wire 一致已由 parity_task_plan 验证)。
    let ag_plan: crate::auto_generated::task_plan::TaskPlan =
        match serde_json::from_value(serde_json::to_value(&plan).unwrap_or(Value::Null)) {
            Ok(p) => p,
            Err(e) => {
                return serde_json::json!({"error": {"code": 400, "message": format!("plan invalid: {e}")}})
            }
        };
    let mut engine = crate::auto_generated::task_plan_engine::TaskPlanEngine::new(ag_plan, input);
    if let Err(e) = engine.validate() {
        return serde_json::json!({"error": {"code": 400, "message": format!("plan invalid: {e}")}});
    }
    let instance_id = engine.instance_id.clone();
    let handoffs = s.0.registry.get(&ws_id).handoffs.clone();
    let state_clone = s.0.clone();
    // ag `TaskPlanExecutor` trait 无 Send+Sync bound(a2r spec 不支持 supertrait)
    // → 生成的 execute future 非 Send,不能直接 tokio::spawn。改用独立线程 +
    // current-thread runtime:future 在同一线程创建并 block_on,全程不跨线程,
    // 闭包捕获的 engine/handoffs/state 均为 Send。行为与 hw 的 tokio::spawn
    // 等价(后台执行,HTTP 立即返回)。
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async move {
            let ctx = crate::auto_generated::task_plan_engine::TaskPlanContext {
                state: state_clone,
                workspace_id: ws_id,
            };
            // Plan 020 Phase H: executor 从透传 hw drive_task_plan_run 的壳
            // (DriveTaskPlanExecutor) 切换为 ag 版 RelayTaskPlanExecutor
            // (start_run + ag drive_run + 读 status/handoff,全 Auto 表达)。
            let executor = crate::auto_generated::task_plan_engine::RelayTaskPlanExecutor { ctx: ctx.clone() };
            // ag execute 按值收 HandoffStore(hw 是 &);clone 共享 data_dir,磁盘读取等价。
            if let Err(e) = engine.execute((*handoffs).clone(), Arc::new(executor)).await {
                tracing::error!("TaskPlan instance failed: {e}");
            }
        });
    });
    serde_json::json!({ "instance_id": instance_id, "task_plan_id": id, "status": "started" })
}

// ── Phase D 通用 helpers ────────────────────────────────────────────────────
pub fn value_get(v: &Value, k: &str) -> Value {
    v.get(k).cloned().unwrap_or(Value::Null)
}
pub fn value_is_null(v: &Value) -> bool {
    v.is_null()
}
/// hw relay 404/400 的纯文本响应(`(StatusCode, String).into_response()`)。
/// `impl Into<String>` 兼容两类调用点:a2r 对字符串字面拼接注入 `.as_str()`(传
/// `&str`),对 fn 调用返回 String 直接传值。
pub fn text_response(msg: impl Into<String>, code: u16) -> axum::response::Response {
    let status = axum::http::StatusCode::from_u16(code)
        .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    (status, msg.into()).into_response()
}

/// 无 body 的状态码响应(hw `StatusCode::NO_CONTENT` 等)。
pub fn empty_response(code: u16) -> axum::response::Response {
    axum::http::StatusCode::from_u16(code)
        .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        .into_response()
}

// ── Plan 020 Phase D: wiki.at HTTP 层委托 ──────────────────────────────────
// ws.wiki 访问经 extern 委托;成功值返回序列化 Value,未找到/错误返回
// {"error":{code,message}} 包络(.at handler 转 text_response / empty_response)。
// `load()` 折叠进各读/写 extern(hw 每个 handler 先 load 再操作,语义等价)。

impl Default for crate::auto_generated::wiki::WikiSource {
    /// 对齐 hw `impl Default for WikiSource { Custom }`(CreatePageRequest 的
    /// `#[serde(default)] source_type` 需要;a2r 不 derive Default)。
    fn default() -> Self {
        crate::auto_generated::wiki::WikiSource::Custom
    }
}

fn wiki_ws(s: &State<AppState>, q: &Query<crate::auto_generated::wiki::WorkspaceQuery>) -> Arc<WorkspaceStores> {
    let hw_q = crate::workspace::WorkspaceQuery { workspace: q.workspace.clone() };
    let ws_id = hw_q.id_or_default(&s.0.registry);
    s.0.registry.get(&ws_id)
}

pub fn ws_wiki_dir(s: &State<AppState>, q: Query<crate::auto_generated::wiki::WorkspaceQuery>) -> std::path::PathBuf {
    wiki_ws(s, &q).wiki.wiki_dir.clone()
}
pub fn ws_raw_dir(s: &State<AppState>, q: Query<crate::auto_generated::wiki::WorkspaceQuery>) -> std::path::PathBuf {
    wiki_ws(s, &q).wiki.raw_dir.clone()
}

pub fn ws_wiki_list(s: &State<AppState>, q: Query<crate::auto_generated::wiki::WorkspaceQuery>) -> Value {
    let ws = wiki_ws(s, &q);
    ws.wiki.load();
    serde_json::json!({ "pages": ws.wiki.list_pages() })
}

pub fn ws_wiki_get(s: &State<AppState>, q: Query<crate::auto_generated::wiki::WorkspaceQuery>, slug: &str) -> Value {
    let ws = wiki_ws(s, &q);
    ws.wiki.load();
    match ws.wiki.get_page(slug) {
        Some(page) => serde_json::to_value(&page).unwrap_or(Value::Null),
        None => Value::Null,
    }
}

pub fn ws_wiki_create(
    s: &State<AppState>,
    q: Query<crate::auto_generated::wiki::WorkspaceQuery>,
    b: Json<crate::auto_generated::wiki::CreatePageRequest>,
) -> Value {
    let ws = wiki_ws(s, &q);
    ws.wiki.load();
    let page = crate::wiki::WikiPage {
        slug: b.0.slug,
        title: b.0.title,
        content: b.0.content,
        source_type: to_hw_wiki_source(b.0.source_type),
        tags: b.0.tags,
        version: 0,
        created_at: 0,
        updated_at: 0,
    };
    match ws.wiki.create_page(page) {
        Ok(p) => serde_json::to_value(&p).unwrap_or(Value::Null),
        Err(e) => serde_json::json!({"error": {"code": 409, "message": e}}),
    }
}

pub fn ws_wiki_update(
    s: &State<AppState>,
    q: Query<crate::auto_generated::wiki::WorkspaceQuery>,
    slug: &str,
    content: &str,
    title: Option<String>,
    tags: Option<Vec<String>>,
) -> Value {
    let ws = wiki_ws(s, &q);
    ws.wiki.load();
    match ws.wiki.update_page(slug, content.to_string(), title, tags) {
        Ok(p) => serde_json::to_value(&p).unwrap_or(Value::Null),
        Err(e) => serde_json::json!({"error": {"code": 404, "message": e}}),
    }
}

pub fn ws_wiki_delete(s: &State<AppState>, q: Query<crate::auto_generated::wiki::WorkspaceQuery>, slug: &str) -> Value {
    let ws = wiki_ws(s, &q);
    ws.wiki.load();
    match ws.wiki.delete_page(slug) {
        Ok(()) => Value::Null,
        Err(e) => serde_json::json!({"error": {"code": 404, "message": e}}),
    }
}

pub fn ws_wiki_search(s: &State<AppState>, q: Query<crate::auto_generated::wiki::WorkspaceQuery>, query: &str) -> Value {
    let ws = wiki_ws(s, &q);
    ws.wiki.load();
    serde_json::json!({ "results": ws.wiki.search(query) })
}

/// raw_upload:Multipart 透传(hw raw_upload 逐字段处理)。
pub async fn wiki_raw_upload(
    s: &State<AppState>,
    q: Query<crate::auto_generated::wiki::UploadQuery>,
    prefix: &str,
    mut multipart: axum::extract::Multipart,
) -> Value {
    // hw:workspace query(默认 default workspace)+ raw_dir。
    let hw_q = crate::workspace::WorkspaceQuery { workspace: q.workspace.clone() };
    let ws_id = hw_q.id_or_default(&s.0.registry);
    let raw_dir = s.0.registry.get(&ws_id).wiki.raw_dir.clone();

    let mut uploaded: Vec<String> = Vec::new();
    loop {
        let next = multipart.next_field().await;
        match next {
            Ok(Some(field)) => {
                let filename = field.file_name().unwrap_or("unnamed").to_string();
                if crate::wiki::validate_path_pub(&filename).is_err() {
                    return serde_json::json!({"error": {"code": 400, "message": "Invalid path"}});
                }
                let data = match field.bytes().await {
                    Ok(d) => d,
                    Err(e) => {
                        return serde_json::json!({"error": {"code": 400, "message": e.to_string()}})
                    }
                };
                let target_dir = if prefix.is_empty() {
                    raw_dir.clone()
                } else {
                    raw_dir.join(prefix)
                };
                let _ = std::fs::create_dir_all(&target_dir);
                let file_path = target_dir.join(&filename);
                if let Err(e) = std::fs::write(&file_path, &data) {
                    return serde_json::json!({"error": {"code": 500, "message": e.to_string()}});
                }
                let relative = if prefix.is_empty() {
                    filename.clone()
                } else {
                    format!("{}/{}", prefix, filename)
                };
                uploaded.push(relative);
            }
            Ok(None) => break,
            Err(e) => {
                return serde_json::json!({"error": {"code": 400, "message": e.to_string()}})
            }
        }
    }
    serde_json::json!({ "uploaded": uploaded })
}

/// raw_file:直接构造 `(Content-Type, bytes)` 响应(hw raw_file 同款)。
pub fn wiki_raw_file(s: &State<AppState>, q: Query<crate::auto_generated::wiki::WorkspaceQuery>, path: &str) -> axum::response::Response {
    let ws = wiki_ws(s, &q);
    let file_path = ws.wiki.raw_dir.join(path);
    let data = match std::fs::read(&file_path) {
        Ok(d) => d,
        Err(_) => return axum::http::StatusCode::NOT_FOUND.into_response(),
    };
    let mime = crate::wiki::guess_mime(&file_path);
    ([(axum::http::header::CONTENT_TYPE, mime)], data).into_response()
}

/// Plan 021 Phase A: /api/files/{workspace_id}/{*path} — serve a file from the
/// workspace root (display_image tool's inline-URL target). Confines the path to
/// the workspace root via canonicalize + starts_with (FORBIDDEN on escape),
/// reads bytes (NOT_FOUND on err), returns with Content-Type from
/// wiki::guess_mime. Replaces hw server.rs:718 workspace_file.
pub fn workspace_file_do(s: &State<AppState>, p: axum::extract::Path<(String, String)>) -> axum::response::Response {
    let (workspace_id, path) = p.0;
    let ws = s.registry.get(&workspace_id);
    let candidate = ws.root.join(&path);
    let canonical = match std::fs::canonicalize(&candidate) {
        Ok(c) => c,
        Err(_) => return axum::http::StatusCode::NOT_FOUND.into_response(),
    };
    if canonical != ws.root && !canonical.starts_with(&ws.root) {
        return axum::http::StatusCode::FORBIDDEN.into_response();
    }
    let data = match std::fs::read(&canonical) {
        Ok(d) => d,
        Err(_) => return axum::http::StatusCode::NOT_FOUND.into_response(),
    };
    let mime = crate::wiki::guess_mime(&canonical);
    ([(axum::http::header::CONTENT_TYPE, mime)], data).into_response()
}

pub fn wiki_raw_delete(s: &State<AppState>, q: Query<crate::auto_generated::wiki::WorkspaceQuery>, path: &str) -> Value {
    let ws = wiki_ws(s, &q);
    let file_path = ws.wiki.raw_dir.join(path);
    if !file_path.exists() {
        return serde_json::json!({"error": {"code": 404, "message": "Not found"}});
    }
    let res = if file_path.is_dir() {
        std::fs::remove_dir_all(&file_path)
    } else {
        std::fs::remove_file(&file_path)
    };
    match res {
        Ok(_) => Value::Null,
        Err(e) => serde_json::json!({"error": {"code": 500, "message": e.to_string()}}),
    }
}

pub fn wiki_raw_mkdir(s: &State<AppState>, q: Query<crate::auto_generated::wiki::WorkspaceQuery>, path: &str) -> Value {
    let ws = wiki_ws(s, &q);
    let target = ws.wiki.raw_dir.join(path);
    match std::fs::create_dir_all(&target) {
        Ok(_) => Value::Null,
        Err(e) => serde_json::json!({"error": {"code": 500, "message": e.to_string()}}),
    }
}

/// Path<(String, String)> 的第二段(hw `Path((_project, slug))` 的 slug)。
pub fn path_second(p: &Path<(String, String)>) -> String {
    p.0 .1.clone()
}

fn to_hw_wiki_source(s: crate::auto_generated::wiki::WikiSource) -> crate::wiki::WikiSource {
    match s {
        crate::auto_generated::wiki::WikiSource::Manual => crate::wiki::WikiSource::Manual,
        crate::auto_generated::wiki::WikiSource::Guide => crate::wiki::WikiSource::Guide,
        crate::auto_generated::wiki::WikiSource::ApiRef => crate::wiki::WikiSource::ApiRef,
        crate::auto_generated::wiki::WikiSource::Custom => crate::wiki::WikiSource::Custom,
    }
}

// ── Plan 020 Phase E: settings_link Auto 化 ────────────────────────────────
// reqwest::blocking + spawn_blocking 封装(hw server.rs settings_link 同路径)。
// 返回完整 Value:.at handler 依 status 字段决定 200/500。

pub async fn settings_link_do() -> Value {
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
                serde_json::json!({ "status": "running", "url": url })
            } else {
                let err = val.get("error").and_then(|e| e.as_str()).unwrap_or("unknown");
                serde_json::json!({ "status": "error", "error": err })
            }
        }
        Ok(Err(e)) => serde_json::json!({ "status": "error", "error": e }),
        Err(e) => serde_json::json!({ "status": "error", "error": format!("internal: {e}") }),
    }
}

/// hw `(StatusCode, Json({...}))` 的 JSON 错误响应(settings_link 的
/// `{"status":"error","error":…}` 形状;区别于 ApiError 的 err_response)。
pub fn err_json_response<T: serde::Serialize>(v: T, code: u16) -> axum::response::Response {
    let status = axum::http::StatusCode::from_u16(code)
        .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    (status, axum::Json(v)).into_response()
}

#[cfg(test)]
mod chat_run_assemble_tests {
    // PLAN-073 P2-T2/P2-T3：assemble_chat_run_msg 组装契约——三臂共用，
    // 结构对齐 r4 T-04 超时臂样板（封口叙述块 → 思考首块 → 尾块）。
    use super::assemble_chat_run_msg;
    use crate::chats::{ChatBlock, ToolCall};

    fn tc(name: &str) -> ToolCall {
        ToolCall {
            tool: name.into(),
            args: serde_json::json!({"path": "specs"}),
            result: "ok".into(),
            status: "success".into(),
            id: "tc-1".into(),
        }
    }

    #[test]
    fn assembles_blocks_thinking_head_and_tail() {
        let blocks = vec![ChatBlock {
            kind: "tool".into(),
            text: String::new(),
            tool: Some(tc("list_dir")),
        }];
        let msg = assemble_chat_run_msg(
            "coder",
            "叙述\n\n⚠ 运行失败：boom".into(),
            "我在思考".into(),
            vec![tc("list_dir")],
            "未封口的叙述",
            &blocks,
            Some("⚠ 运行失败：boom"),
        );
        assert_eq!(msg.profession_id.as_deref(), Some("coder"));
        assert_eq!(msg.thinking, "我在思考");
        assert_eq!(msg.tool_calls.len(), 1);
        // 块序：思考首块 → 已积累工具块 → 封口叙述块 → ⚠ 尾块。
        let kinds: Vec<&str> = msg.blocks.iter().map(|b| b.kind.as_str()).collect();
        assert_eq!(kinds, vec!["thinking", "tool", "text", "text"]);
        assert_eq!(msg.blocks[0].text, "我在思考");
        assert_eq!(msg.blocks[2].text, "未封口的叙述");
        assert_eq!(msg.blocks[3].text, "⚠ 运行失败：boom");
        // 默认终态（pending 由调用方按需覆写）。
        assert_eq!(msg.pending, None);
    }

    #[test]
    fn empty_inputs_degrade_to_plain_message() {
        // 失败臂零现场（如秒败）：只剩尾块，仍可落盘。
        let msg = assemble_chat_run_msg(
            "assistant",
            "⚠ 运行失败：boom".into(),
            String::new(),
            Vec::new(),
            "",
            &[],
            Some("⚠ 运行失败：boom"),
        );
        let kinds: Vec<&str> = msg.blocks.iter().map(|b| b.kind.as_str()).collect();
        assert_eq!(kinds, vec!["text"]);
        assert_eq!(msg.blocks[0].text, "⚠ 运行失败：boom");
    }

    #[test]
    fn clone_semantics_source_untouched() {
        // 增量快照路径反复组装同一来源——不得消耗（blocks 仍可再组装）。
        let blocks = vec![ChatBlock {
            kind: "text".into(),
            text: "turn1".into(),
            tool: None,
        }];
        let _first = assemble_chat_run_msg(
            "coder",
            "c".into(),
            String::new(),
            Vec::new(),
            "",
            &blocks,
            None,
        );
        let second = assemble_chat_run_msg(
            "coder",
            "c2".into(),
            String::new(),
            Vec::new(),
            "",
            &blocks,
            None,
        );
        assert_eq!(second.blocks.len(), 1);
        assert_eq!(blocks[0].text, "turn1");
    }
}
