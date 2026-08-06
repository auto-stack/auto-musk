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
/// side-table(Value 存 i64 id),字段提取走 ConversationEvent 序列化 Value。
pub fn conversations_subscribe(s: &State<AppState>, q: Query<StreamWorkspaceQuery>) -> Value {
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let rx = ws.conversations.subscribe();
    let id = next_handle_id();
    HANDLES.lock().unwrap().insert(id, Box::new(rx));
    serde_json::json!(id)
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
    if let Err(e) = crate::relay::feature_dev::require_builtin(&b.workflow) {
        return serde_json::json!({"error": {"code": 400, "message": format!("invalid workflow '{}': {e}", b.workflow)}});
    }
    let hw_q = crate::workspace::WorkspaceQuery { workspace: q.workspace.clone() };
    let ws_id = hw_q.id_or_default(&s.0.registry);
    let ws = s.0.registry.get(&ws_id);
    match crate::relay::feature_dev::run(&s.0, &ws, &b.task).await {
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
    if let Err(e) = crate::relay::feature_dev::require_builtin(name) {
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
/// Plan 019 Phase 3:workflow_run_stream 真实化 —— sink 桥接:把 hw 强类型
/// WorkflowStreamEvent 序列化成 Value 喂给 mpsc(tx 句柄),run 结束关闭 channel
/// 让 SSE 流终止。与 hw `workflow_run_stream`(server.rs:885)同路径。
pub async fn wf_run_with_progress(
    s: &State<AppState>,
    q: Query<StreamWorkspaceQuery>,
    b: Json<WorkflowRunRequest>,
    tx: Value,
) {
    if let Err(e) = crate::relay::feature_dev::require_builtin(&b.workflow) {
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
        crate::tool_safety::set_current_root(ws.root.clone());
        let tx3 = tx2.clone();
        let on_event: Arc<dyn Fn(crate::relay::feature_dev::WorkflowStreamEvent) + Send + Sync> =
            Arc::new(move |ev| {
                let v = serde_json::to_value(&ev).unwrap_or(Value::Null);
                mpsc_try_send(&tx3, v);
            });
        if let Err(e) =
            crate::relay::feature_dev::run_stream(&state, &ws, &task, on_event).await
        {
            tracing::error!("workflow stream failed: {e}");
        }
        crate::tool_safety::clear_current_root();
        close_channel(&tx2);
    });
}
pub fn orch_spawn_relay(_t: String, _a: Value) -> String { "(stub)".into() }
pub fn orch_dispatch(_t: String, _to: String) -> String { "(stub)".into() }
pub fn orch_bring_in(_q: String) -> String { "(stub)".into() }
pub fn drive_set_root(_w: &str) {}
pub fn drive_clear_root() {
    crate::tool_safety::clear_current_root();
}
pub fn relay_advance(_w: &str, _r: &str) -> Value { Value::Null }
pub fn relay_publish(_r: &str, _v: &Value) {}
pub fn advance_is_none(_r: &Value) -> bool { true }
pub fn advance_kind(_r: &Value) -> String { "completed".into() }
pub fn advance_role_id(_r: &Value) -> String { String::new() }
pub fn relay_submit_error(_r: &str, _r2: &str, _e: &Result<String, String>) {}
pub fn relay_step_context(_w: &str, _r: &str) -> String { String::new() }
/// ③ 委托:factory_build_agent — 构造 hw `relay::driver::MuskAgentFactory` 并
/// build_agent(role_id, 无 handoff;handoff 注入在转译 run_step 里由
/// handoff_render/agent_with_history 承担)。失败回退 StubRole + 日志。
pub async fn factory_build_agent(s: &Arc<AppState>, w: &str, r: &str, r2: &str) -> Agent {
    let factory = crate::relay::driver::MuskAgentFactory {
        state: s.clone(),
        workspace_id: w.to_string(),
        run_id: r.to_string(),
    };
    match factory.build_agent(r2, None) {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!("factory_build_agent delegation failed: {e}");
            Agent::new(StubRole, s.client.clone())
        }
    }
}
pub fn drive_accumulated(_w: &str, _r: &str) -> String { String::new() }
pub fn drive_finalize_output(_o: String, _r: &Value) -> String { _o }
pub fn drive_submit_handoff(_w: &str, _r: &str, _r2: &str, _o: &str, _v: &Value) {}
pub fn drive_handle_stream_event(_w: &str, _r: &str, _r2: &str, _e: i32) {}
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
    _q: Query<StreamWorkspaceQuery>,
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
    let mut agent = match crate::build_agent_from_mode(&mode, s.0.client.clone()) {
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
    let ws = s.0.registry.get(&q.workspace.clone().unwrap_or_default());
    let ws_root = ws.root.clone();
    let client = s.0.client.clone();
    let task = b.task.clone();
    let tx2 = tx.clone();
    tokio::spawn(async move {
        crate::tool_safety::set_current_root(ws_root.clone());
        let mut agent = match crate::build_agent_from_mode(&mode, client) {
            Ok(a) => a,
            Err(e) => {
                crate::tool_safety::clear_current_root();
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
                    StreamEvent::Delta { text } => SseEventDto::Delta { text: text.clone() },
                    StreamEvent::Thinking { text } => {
                        SseEventDto::Thinking { thinking: text.clone() }
                    }
                    StreamEvent::ToolStart { tool, args } => SseEventDto::ToolCall {
                        id: id.clone(),
                        name: tool.clone(),
                        arguments: args.clone(),
                    },
                    StreamEvent::Tool { tool, args, result } => SseEventDto::ToolResult {
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
        crate::tool_safety::clear_current_root();
        close_channel(&tx2);
    });
}
/// Plan 019 Phase 4:chat_stream 真实化 —— 与 hw `chat_stream`(server.rs:575)
/// 同路径:session + history + build_agent_with_context + run_stream + 完成后
/// 持久化(append_message + 双写 conversation turns)。持久化在 extern 内直接做
/// (它知道 session_id),不依赖 sink 回调。
pub async fn chat_run_stream(
    s: &State<AppState>,
    q: Query<StreamWorkspaceQuery>,
    p: Path<String>,
    tx: Value,
) {
    let ws_id = q.workspace.clone().unwrap_or_default();
    let ws = s.0.registry.get(&ws_id);
    let session_id = p.0.clone();
    let session = match ws.chats.get(&session_id) {
        Some(sess) => sess,
        None => {
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
                mpsc_try_send(&tx, serde_json::json!({"type":"error","message":"no user message to run"}));
                close_channel(&tx);
                return;
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
    let tx2 = tx.clone();
    tokio::spawn(async move {
        crate::tool_safety::set_current_root(ws_root.clone());
        // Build agent with orchestration tool context (spawn_relay, dispatch).
        let tool_ctx = crate::tool_context::ToolContext {
            state: state_for_ctx.clone(),
            workspace_id: ws_id.clone(),
            parent_conversation_id: session_id.clone(),
        };
        let mut agent = match crate::build_agent_with_context(&agent_mode, client, Some(tool_ctx)) {
            Ok(a) => a,
            Err(e) => {
                crate::tool_safety::clear_current_root();
                mpsc_try_send(&tx2, serde_json::json!({"type":"error","message": format!("build agent: {e}")}));
                close_channel(&tx2);
                return;
            }
        };
        // Pre-load the conversation history so the agent has context.
        agent = agent.with_history(history);

        // Accumulate the streamed text + tool calls to persist on completion.
        let accumulated = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let tool_calls: std::sync::Arc<std::sync::Mutex<Vec<crate::chats::ToolCall>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let tx3 = tx2.clone();
        let acc2 = accumulated.clone();
        let tc2 = tool_calls.clone();
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
                    StreamEvent::Delta { text } => SseEventDto::Delta { text: text.clone() },
                    StreamEvent::Thinking { text } => {
                        SseEventDto::Thinking { thinking: text.clone() }
                    }
                    StreamEvent::ToolStart { tool, args } => SseEventDto::ToolCall {
                        id: id.clone(),
                        name: tool.clone(),
                        arguments: args.clone(),
                    },
                    StreamEvent::Tool { tool, args, result } => SseEventDto::ToolResult {
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
                // capture for persistence
                if let Some(text) = value.get("text").and_then(|t| t.as_str()) {
                    acc2.lock().unwrap().push_str(text);
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
        // No cancellation endpoint yet — the run flag is never set.
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
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
                for turn in crate::conversation::chat_message_to_turns(&msg, seq_base) {
                    let _ = conversations.append_turn(&session_id, turn);
                }
            }
            Err(e) => {
                mpsc_try_send(&tx2, serde_json::json!({"type":"error","message": format!("{e}")}));
            }
        }
        crate::tool_safety::clear_current_root();
        close_channel(&tx2);
    });
}
/// server_serve(休眠镜像)仍在调用 —— 保持 stub(该模块未接线)。
pub fn agent_run_stream_with_sink<W: Send + Sync + 'static>(_a: Agent, _t: String, _sink: Arc<W>, _c: Arc<std::sync::atomic::AtomicBool>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, String>> + Send>> {
    Box::pin(async { Ok(Value::Null) })
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
pub fn batch_replace_do(p: &str, _r: Value) -> String { format!("(stub) {}", p) }
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
/// Plan 019 Phase 2: broadcast_recv —— 从 side-table 取 Receiver 收一条
/// ConversationEvent(序列化为 hw wire 形状 Value)。Lagged 跳过积压继续流
/// (hw BroadcastStream 语义),Closed 返回 None 让 .at 的 break 终止流。
pub async fn broadcast_recv(r: &Value) -> Option<Value> {
    let id = match r.as_i64() {
        Some(i) => i,
        None => return None,
    };
    let mut rx = {
        let mut handles = HANDLES.lock().unwrap();
        match handles.remove(&id) {
            Some(b) => match b.downcast::<tokio::sync::broadcast::Receiver<crate::conversation::ConversationEvent>>() {
                Ok(rx) => *rx,
                Err(b) => {
                    handles.insert(id, b);
                    return None;
                }
            },
            None => return None,
        }
    };
    loop {
        match rx.recv().await {
            Ok(ev) => {
                let value = serde_json::json!({
                    "conversation_id": ev.conversation_id,
                    "turn": ev.turn,
                    "status": ev.status,
                });
                HANDLES.lock().unwrap().insert(id, Box::new(rx));
                return Some(value);
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
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
