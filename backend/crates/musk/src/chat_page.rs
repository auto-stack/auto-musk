//! chat_page.rs — PLAN-083 T-02：会话详情分页端点 + 归一化直出。
//!
//! `GET /api/chats/session/{id}/page?workspace=&limit=&before=`
//!
//! 背景（PLAN-082 §10-7 实测 + 083 勘定）：VM 轨切 workspace 冻结 20.9s 的
//! 载荷面放大器——`chats_get_session` 全量单发（auto-edit 首会话 578KB，
//! 单条工具密集回复可达 938KB），且 tool 载荷在 `blocks[].tool` 与消息级
//! `tool_calls[]` **双份**（652KB 条里 tool_calls 重复数组独占 225KB）。
//! 前端 `normalizeToolBlocks` 再在 VM 解释器逐块循环拍平——解析+拍平+渲染
//! 同帧执行。
//!
//! 本端点三件事（缺省全量的旧端点 `/session/{id}` 字节不变，web 旧消费
//! 零影响）：
//! 1. **分页**：`limit=N`（缺省 50）取最近 N 条；`before=<message_id>`
//!   取该 id 之前（不含）的最近 N 条——历史向上滚动加载；`has_more` +
//!   `next_before`（下一页的 before 游标=本页首条 id）。
//! 2. **归一化直出**（`blocks_normalized: true`）：后端按前端
//!   `normalizeToolBlocks` r5b 契约直出拍平块——tool 块重建为**纯字符串
//!   字段**（tool_name/tool_id/tool_status/tool_result/tool_gate_id/
//!   tool_pending_cmd/tool_escape_paths_text/tool_args_json/summary 现算），
//!   thinking 块重建为 `{kind,text,state:"done"}`。消费侧（VM）读字符串
//!   字段全程可靠（渲染/计算上下文对存储可达嵌套对象的字段读产出 "0"，
//!   PLAN-081 r5 定罪），免二次归一。
//! 3. **块级瘦身**：分页数限制不住单条巨消息（552KB/938KB 条=84~109 个
//!   tool 块）——per-block 截断（result/thinking/args_json 上限 + 块携带
//!   `truncated:true` 标记）+ 每消息字节预算（尾部优先保真——用户注意力
//!   在最新内容，超出预算的更早 tool 块降为摘要桩）+ 消息级 `tool_calls`
//!   瘦身为 thin 数组（id/name/status，载荷只留 blocks 一份）。
//!
//! 等价性（U-2）：`summary` 端口与 forge_store.at `summaryTextOf` 逐分支
//! 对齐（segJoin 拼接/60→57/80→77 截断口径）；`tool_args_json` 为
//! `JSON.stringify(arguments)` 等价值（serde_json 无 preserve_order，键序
//! 与 JS 插入序不同——消费侧 parse 后使用，无字符串序语义）。

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::chats::{ChatBlock, ChatMessage, ChatSession, ToolCall};
use crate::server::AppState;

/// 分页上限与截断口径（chars——Rust char 语义 ≈ JS string char 语义差在
/// 代理对，验收样本含 CJK 无 emoji，等价面内）。
pub const DEFAULT_PAGE_LIMIT: usize = 50;
/// 单消息总预算（序列化字节数估计）——尾部优先，超出预算的更早块降桩。
const MSG_BUDGET_BYTES: usize = 48 * 1024;
/// 单块 tool result 截断上限。
const TOOL_RESULT_CAP: usize = 4096;
/// 单块 thinking 文本截断上限。
const THINKING_CAP: usize = 4096;
/// args_json 截断上限（截断后非合法 JSON——消费侧按串展示，不 parse）。
const ARGS_JSON_CAP: usize = 2048;
/// 消息级 content / text 块截断上限。
const TEXT_CAP: usize = 16 * 1024;

#[derive(Debug, Deserialize)]
pub struct SessionPageQuery {
    pub workspace: Option<String>,
    pub limit: Option<usize>,
    pub before: Option<String>,
}

/// `GET /api/chats/session/{id}/page` — 分页 + 归一化直出的会话详情。
pub async fn chat_get_page(
    State(s): State<AppState>,
    Query(q): Query<SessionPageQuery>,
    Path(id): Path<String>,
) -> Response {
    let ws = s.registry.get(&q.workspace.clone().unwrap_or_default());
    let Some(session) = ws.chats.get(&id) else {
        return error_response("session not found");
    };
    let limit = q.limit.unwrap_or(DEFAULT_PAGE_LIMIT).clamp(1, 500);
    let page = paginate_and_normalize(&session, limit, q.before.as_deref());
    Json(page).into_response()
}

fn error_response(msg: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": msg })),
    )
        .into_response()
}

/// 分页窗口：消息按存储序（旧→新）。`before` 存在时上界=其索引（不含），
/// 否则=末尾；窗口=上界的最近 `limit` 条。`has_more`=上界内还有更早消息。
fn page_window(messages: &[ChatMessage], limit: usize, before: Option<&str>) -> (usize, usize, bool) {
    let upper = match before {
        Some(bid) => messages.iter().position(|m| m.id == bid).unwrap_or(messages.len()),
        None => messages.len(),
    };
    let start = upper.saturating_sub(limit);
    let has_more = start > 0;
    (start, upper, has_more)
}

/// 构建分页响应体：`{session, has_more, next_before, blocks_normalized}`。
/// session 保留全部会话字段（serde 直转），messages 换为归一化+瘦身的页。
pub fn paginate_and_normalize(
    session: &ChatSession,
    limit: usize,
    before: Option<&str>,
) -> serde_json::Value {
    let (start, upper, has_more) = page_window(&session.messages, limit, before);
    let page: Vec<serde_json::Value> = session.messages[start..upper]
        .iter()
        .map(normalize_message_for_page)
        .collect();
    let next_before = if has_more {
        session.messages[start].id.clone()
    } else {
        String::new()
    };
    let mut v = serde_json::to_value(session).unwrap_or(serde_json::Value::Null);
    if let serde_json::Value::Object(map) = &mut v {
        map.insert("messages".into(), serde_json::Value::Array(page));
    }
    serde_json::json!({
        "session": v,
        "has_more": has_more,
        "next_before": next_before,
        "blocks_normalized": true,
    })
}

/// 消息级归一化：thin tool_calls + 拍平 blocks（尾部优先预算）+ 大字段截断。
fn normalize_message_for_page(m: &ChatMessage) -> serde_json::Value {
    let mut v = serde_json::to_value(m).unwrap_or(serde_json::Value::Null);
    let Some(map) = v.as_object_mut() else { return v };

    // thin tool_calls：id/name/status（载荷只留 blocks 一份）。
    let thin: Vec<serde_json::Value> = m
        .tool_calls
        .iter()
        .map(|tc| {
            serde_json::json!({
                "id": tc.id,
                "name": tc.tool,
                "status": tc.status,
            })
        })
        .collect();
    if !thin.is_empty() {
        map.insert("tool_calls".into(), serde_json::Value::Array(thin));
    }

    // 大字段截断。
    if m.thinking.len() > THINKING_CAP {
        let (cut, _) = trunc_chars(&m.thinking, THINKING_CAP);
        map.insert("thinking".into(), serde_json::Value::String(cut));
    }
    if m.content.len() > TEXT_CAP {
        let (cut, _) = trunc_chars(&m.content, TEXT_CAP);
        map.insert("content".into(), serde_json::Value::String(cut));
    }

    // 拍平 blocks（尾部优先预算）。
    let blocks = flatten_blocks_budgeted(&m.blocks);
    map.insert("blocks".into(), serde_json::Value::Array(blocks));
    v
}

/// 拍平块族：与 forge_store.at normalizeToolBlocks r5b 契约逐字段对齐
/// （tool→纯字符串字段+summary；thinking→done 态全字段；其余透传+截断），
/// 外加预算降桩。
fn flatten_blocks_budgeted(blocks: &[ChatBlock]) -> Vec<serde_json::Value> {
    // 先各块归一化（含截断），再从尾部起分配预算：超预算的更早 tool 块
    // 降为摘要桩（result 置空 + truncated 标记），文本类保头截断不降桩
    // （占宽本来就小）。
    let normalized: Vec<serde_json::Value> = blocks.iter().map(normalize_block).collect();
    let mut budget = MSG_BUDGET_BYTES as i64;
    let mut out: Vec<serde_json::Value> = Vec::with_capacity(normalized.len());
    for b in normalized.iter().rev() {
        let cost = serde_json::to_string(b).map(|s| s.len() as i64).unwrap_or(0);
        let over = budget - cost < 0;
        let entry = if over && b.get("kind").and_then(|k| k.as_str()) == Some("tool") {
            // 降桩：字符串字段保留（含 summary），重载字段收缩——args_json
            // 契约恒合法 JSON（V-4 实测二次定罪：降桩清空串令消费侧
            // messageBlocks r5b 臂 JSON.parse("") 抛异常炸掉整条渲染
            // computed），占位用 "{}"；result 是字节大头，置空即可。
            let mut stub = b.clone();
            if let Some(map) = stub.as_object_mut() {
                map.insert("tool_result".into(), serde_json::Value::String(String::new()));
                map.insert("tool_args_json".into(), serde_json::Value::String("{}".into()));
                map.insert("tool_escape_paths_text".into(), serde_json::Value::String("[]".into()));
                map.insert("truncated".into(), serde_json::Value::Bool(true));
            }
            stub
        } else {
            b.clone()
        };
        budget -= serde_json::to_string(&entry).map(|s| s.len() as i64).unwrap_or(0);
        out.push(entry);
    }
    out.reverse();
    out
}

/// 单块归一化（无预算考量）：tool→拍平字符串字段；thinking→done 态；
/// text→透传+截断。无 tool 载荷的块透传（防御：流式双形态 `tc` 键不在
/// 持久化面出现，出现也不拍平——消费侧 live 块不经本端点）。
fn normalize_block(b: &ChatBlock) -> serde_json::Value {
    match b.kind.as_str() {
        "tool" => match &b.tool {
            Some(tc) => {
                let (result, result_cut) = trunc_chars(&tc.result, TOOL_RESULT_CAP);
                // args_json 契约：**恒为合法 JSON**——消费侧（messageBlocks
                // r5b 臂）对每 tool 块 JSON.parse 重建 arguments，截断串
                // （'"…' 收尾）会令 parse 抛异常炸掉整条渲染 computed
                // （V-4 实测：652KB 会话 AI 气泡整体空白）。超限时以合法
                // 占位对象替代，截断语义由 truncated 标记承载。
                let args_full = serde_json::to_string(&tc.args).unwrap_or_else(|_| "{}".into());
                let (args_json, args_cut) = if args_full.chars().count() > ARGS_JSON_CAP {
                    ("{\"_truncated\":true}".to_string(), true)
                } else {
                    (args_full, false)
                };
                // gate 面（tool_gate 越界暂停）在 ToolCall 结构外——持久化
                // 块载荷仅 name/args/result/status/id（chats.rs ToolCall），
                // gate_* 字段恒空串对齐消费侧缺省；escape_paths 缺席 = TS
                // `?? []` → "[]"。
                let mut v = serde_json::json!({
                    "kind": "tool",
                    "tool_name": tc.tool,
                    "tool_id": tc.id,
                    "tool_status": tc.status,
                    "tool_result": result,
                    "tool_gate_id": "",
                    "tool_pending_cmd": "",
                    "tool_escape_paths_text": "[]",
                    "tool_args_json": args_json,
                    "summary": summary_text_of(&tc.args),
                });
                if result_cut || args_cut {
                    v["truncated"] = serde_json::Value::Bool(true);
                }
                v
            }
            None => serde_json::to_value(b).unwrap_or(serde_json::Value::Null),
        },
        "thinking" => {
            let (text, cut) = trunc_chars(&b.text, THINKING_CAP);
            let mut v = serde_json::json!({
                "kind": "thinking",
                "text": text,
                "state": "done",
            });
            if cut {
                v["truncated"] = serde_json::Value::Bool(true);
            }
            v
        }
        _ => {
            if b.text.len() > TEXT_CAP {
                let (text, _) = trunc_chars(&b.text, TEXT_CAP);
                serde_json::json!({ "kind": b.kind, "text": text, "truncated": true })
            } else {
                serde_json::json!({ "kind": b.kind, "text": b.text })
            }
        }
    }
}

/// char 边界安全截断：`(截断串, 是否截断)`。
fn trunc_chars(s: &str, max_chars: usize) -> (String, bool) {
    if s.chars().count() <= max_chars {
        return (s.to_string(), false);
    }
    (
        s.chars().take(max_chars).collect::<String>() + "…",
        true,
    )
}

// ─── summary 端口（forge_store.at summaryTextOf 逐分支对齐，U-2 契约） ────

fn seg_join(a: &str, b: &str) -> String {
    if a.is_empty() {
        b.to_string()
    } else {
        format!("{} {}", a, b)
    }
}

/// 工具卡 header 单行摘要——与 forge_store.at `summaryTextOf`（TS 形）逐分
/// 支对齐：path[:limit:offset] + slug/section_id/pattern(60→57)/query(60→57)
/// /skill_name segJoin；task 仅在 out 空时（60→57）；command=args.command
/// ?? args.cmd 仅在 out 空时（80→77）。
fn summary_text_of(args: &serde_json::Value) -> String {
    let get_str = |k: &str| args.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
    let path = get_str("path");
    let slug = get_str("slug");
    let section_id = get_str("section_id");
    let pattern = get_str("pattern");
    let query = get_str("query");
    let task = get_str("task");
    let skill_name = get_str("skill_name");
    let mut command = get_str("command");
    if command.is_empty() {
        command = get_str("cmd");
    }
    let limit = args.get("limit");
    let offset = args.get("offset");

    let mut out = String::new();
    if !path.is_empty() {
        out = path.clone();
        if limit.is_some() || offset.is_some() {
            let l = scalar_to_str(limit);
            let o = scalar_to_str(offset);
            out = format!("{}:{}:{}", out, l, o);
        }
    }
    if !slug.is_empty() {
        out = seg_join(&out, &slug);
    }
    if !section_id.is_empty() {
        out = seg_join(&out, &section_id);
    }
    if !pattern.is_empty() {
        let s = trunc_elide(&pattern, 60, 57);
        out = seg_join(&out, &format!("\"{}\"", s));
    }
    if !query.is_empty() {
        let s = trunc_elide(&query, 60, 57);
        out = seg_join(&out, &format!("\"{}\"", s));
    }
    if !skill_name.is_empty() {
        out = seg_join(&out, &skill_name);
    }
    if !task.is_empty() && out.is_empty() {
        out = trunc_elide(&task, 60, 57);
    }
    if !command.is_empty() && out.is_empty() {
        out = trunc_elide(&command, 80, 77);
    }
    out
}

/// JS `s.length > over ? s.substring(0, keep) + "…" : s` 的 char 语义等价。
fn trunc_elide(s: &str, over: usize, keep: usize) -> String {
    if s.chars().count() > over {
        format!("{}…", s.chars().take(keep).collect::<String>())
    } else {
        s.to_string()
    }
}

/// JS 模板串拼 `(limit ?? "")`：null/undefined → 空串；数字/字符串原样。
fn scalar_to_str(v: Option<&serde_json::Value>) -> String {
    match v {
        None | Some(serde_json::Value::Null) => String::new(),
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(other) => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool_block(name: &str, args: serde_json::Value, result: &str) -> ChatBlock {
        ChatBlock {
            kind: "tool".into(),
            text: String::new(),
            tool: Some(ToolCall {
                tool: name.into(),
                args,
                result: result.into(),
                status: "success".into(),
                id: format!("tc-{}", name),
            }),
        }
    }

    fn msg(id: &str, role: crate::chats::Role, blocks: Vec<ChatBlock>) -> ChatMessage {
        ChatMessage {
            id: id.into(),
            role,
            content: "c".into(),
            thinking: String::new(),
            tool_calls: Vec::new(),
            created_at: 1,
            parent_id: None,
            blocks,
            profession_id: None,
            pending: None,
            // PLAN-093 T-08 新增字段（serde default）；测试夹具同步补位。
            design_context: None,
        }
    }

    fn session(msgs: Vec<ChatMessage>) -> ChatSession {
        ChatSession {
            id: "s1".into(),
            name: "n".into(),
            mode: "superpowers".into(),
            messages: msgs,
            created_at: 1,
            updated_at: 2,
            pending_spec_changes: Vec::new(),
            workspace_id: None,
            active_leaf: None,
            thinking_level: None,
            approval_mode: "human".into(),
            archived: false,
        }
    }

    /// U-1：limit/before/has_more/next_before 语义 + 缺省 limit=50。
    #[test]
    fn u1_pagination_semantics() {
        let msgs: Vec<ChatMessage> = (0..7)
            .map(|i| msg(&format!("m{}", i), crate::chats::Role::User, vec![]))
            .collect();
        let s = session(msgs);

        // 首屏：最近 3 条。
        let p = paginate_and_normalize(&s, 3, None);
        assert_eq!(p["has_more"], serde_json::json!(true));
        assert_eq!(p["next_before"], serde_json::json!("m4"));
        let ids: Vec<&str> = p["session"]["messages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, vec!["m4", "m5", "m6"]);

        // 滚动加载：before=m4 → 再往前 3 条。
        let p2 = paginate_and_normalize(&s, 3, Some("m4"));
        assert_eq!(p2["has_more"], serde_json::json!(true));
        assert_eq!(p2["next_before"], serde_json::json!("m1"));
        let ids2: Vec<&str> = p2["session"]["messages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids2, vec!["m1", "m2", "m3"]);

        // 最后一页：before=m1 → 只剩 m0。
        let p3 = paginate_and_normalize(&s, 3, Some("m1"));
        assert_eq!(p3["has_more"], serde_json::json!(false));
        assert_eq!(p3["next_before"], serde_json::json!(""));

        // 缺省 limit=50：少于 50 条时全量、无更多。
        let p4 = paginate_and_normalize(&s, DEFAULT_PAGE_LIMIT, None);
        assert_eq!(p4["has_more"], serde_json::json!(false));
        assert_eq!(p4["session"]["messages"].as_array().unwrap().len(), 7);

        // before 指向不存在 id：视为无上界（防呆，最近 N 条）。
        let p5 = paginate_and_normalize(&s, 2, Some("nope"));
        assert_eq!(p5["session"]["messages"].as_array().unwrap().len(), 2);
    }

    /// U-2：归一化直出与 TS normalizeToolBlocks r5b 契约等价（真实载荷
    /// 形态：tool 块 + thinking 块 + text 块混合；args 含 path/pattern/
    /// limit 等 summary 输入）。
    #[test]
    fn u2_normalized_shape_matches_ts_contract() {
        let blocks = vec![
            ChatBlock { kind: "thinking".into(), text: "深思考".into(), tool: None },
            tool_block(
                "read_file",
                serde_json::json!({"path": "src/main.rs", "limit": 10, "offset": 2}),
                "fn main() {}",
            ),
            tool_block(
                "run_command",
                serde_json::json!({"cmd": "cargo test"}),
                "ok",
            ),
            ChatBlock { kind: "text".into(), text: "done".into(), tool: None },
        ];
        let m = msg("a1", crate::chats::Role::Assistant, blocks);
        let p = paginate_and_normalize(&session(vec![m]), 50, None);
        let out = &p["session"]["messages"][0]["blocks"];

        assert_eq!(p["blocks_normalized"], serde_json::json!(true));
        // thinking → done 态全字段形态。
        assert_eq!(out[0]["kind"], serde_json::json!("thinking"));
        assert_eq!(out[0]["text"], serde_json::json!("深思考"));
        assert_eq!(out[0]["state"], serde_json::json!("done"));
        // tool → 纯字符串字段（无嵌套对象）+ summary 现算（path:limit:offset）。
        assert_eq!(out[1]["tool_name"], serde_json::json!("read_file"));
        assert_eq!(out[1]["tool_id"], serde_json::json!("tc-read_file"));
        assert_eq!(out[1]["tool_status"], serde_json::json!("success"));
        assert_eq!(out[1]["tool_result"], serde_json::json!("fn main() {}"));
        assert_eq!(out[1]["summary"], serde_json::json!("src/main.rs:10:2"));
        // args_json 等价（parse 对拍——键序无语义）。
        let args: serde_json::Value =
            serde_json::from_str(out[1]["tool_args_json"].as_str().unwrap()).unwrap();
        assert_eq!(args, serde_json::json!({"path": "src/main.rs", "limit": 10, "offset": 2}));
        // command 臂：out 空时取 cmd（80→77 口径在此样本不触发截断）。
        assert_eq!(out[2]["summary"], serde_json::json!("cargo test"));
        // text 透传。
        assert_eq!(out[3]["kind"], serde_json::json!("text"));
        assert_eq!(out[3]["text"], serde_json::json!("done"));
        // 无截断时不携带 truncated 键。
        assert!(out[1].get("truncated").is_none());
    }

    /// U-2 续：summary 截断口径（pattern>60→57+…）与 TS 一致。
    #[test]
    fn u2_summary_truncation_elision() {
        let long_pattern = "p".repeat(61);
        let args = serde_json::json!({ "pattern": long_pattern });
        assert_eq!(
            summary_text_of(&args),
            format!("\"{}…\"", "p".repeat(57))
        );
        // 恰 60 不截。
        let exact = serde_json::json!({ "pattern": "p".repeat(60) });
        assert_eq!(summary_text_of(&exact), format!("\"{}\"", "p".repeat(60)));
    }

    /// args_json 契约回归：超限截断不产生非法 JSON（V-4 实测定罪——
    /// 消费侧 messageBlocks r5b 臂逐块 JSON.parse，非法串炸掉整条渲染
    /// computed=AI 气泡整空）。
    #[test]
    fn args_json_over_cap_stays_valid_json() {
        let big_args = serde_json::json!({ "content": "x".repeat(5000) });
        let b = ChatBlock { kind: "tool".into(), text: String::new(), tool: Some(ToolCall {
            tool: "edit_file".into(), args: big_args, result: "r".into(),
            status: "success".into(), id: "tc-1".into(),
        })};
        let out = normalize_block(&b);
        let raw = out["tool_args_json"].as_str().unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(raw).is_ok(),
            "args_json must stay valid JSON, got: {}...", &raw[..40.min(raw.len())]);
        assert_eq!(out["truncated"], serde_json::json!(true));
    }

    /// 降桩臂回归：预算降桩后 args_json 仍合法（V-4 二次定罪——
    /// 降桩清空串 JSON.parse("") 同样炸渲染 computed）。
    #[test]
    fn stubbed_blocks_keep_args_json_valid() {
        // 堆 8 个大 result 块令预算必然降桩头部块。
        let big = "x".repeat(TOOL_RESULT_CAP + 100);
        let blocks: Vec<ChatBlock> = (0..8)
            .map(|i| tool_block(&format!("t{i}"), serde_json::json!({"path": "a.rs"}), &big))
            .collect();
        let msgs = vec![msg("a1", crate::chats::Role::Assistant, blocks)];
        let p = paginate_and_normalize(&session(msgs), 50, None);
        let out = p["session"]["messages"][0]["blocks"].as_array().unwrap();
        let mut checked_stub = 0;
        for b in out {
            if b["truncated"] == serde_json::json!(true) && b["kind"] == serde_json::json!("tool") {
                let raw = b["tool_args_json"].as_str().unwrap_or("");
                assert!(
                    serde_json::from_str::<serde_json::Value>(raw).is_ok(),
                    "stub args_json must stay valid JSON"
                );
                checked_stub += 1;
            }
        }
        assert!(checked_stub > 0, "fixture should have triggered stubbing");
    }

    /// 块级瘦身：超预算消息的更早 tool 块降桩（result 清空 + truncated），
    /// 尾部块保真。
    #[test]
    fn budget_stubs_older_tool_blocks_keeps_tail() {
        // 尾块小 + 头部堆大 result 块（单块截断后仍超预算 → 降桩）。
        let big = "x".repeat(TOOL_RESULT_CAP + 100);
        let mut blocks = vec![tool_block("read_file", serde_json::json!({"path": "a.rs"}), &big); 8];
        blocks.push(ChatBlock { kind: "text".into(), text: "tail".into(), tool: None });
        blocks.push(tool_block("run_command", serde_json::json!({"cmd": "echo hi"}), "hi"));
        let m = msg("a1", crate::chats::Role::Assistant, blocks);
        let p = paginate_and_normalize(&session(vec![m]), 50, None);
        let out = p["session"]["messages"][0]["blocks"].as_array().unwrap();

        // 尾部两个块保真。
        let last = out.last().unwrap();
        assert_eq!(last["tool_result"], serde_json::json!("hi"));
        assert!(last.get("truncated").is_none());
        assert_eq!(out[out.len() - 2]["text"], serde_json::json!("tail"));
        // 头部超预算块：截断或降桩——两者必居其一，且都带 truncated 标记。
        let head = &out[0];
        assert_eq!(head["truncated"], serde_json::json!(true));
        // 无论截断(≤cap+1)还是降桩(空)，单块 result 都被钳住。
        let rlen = head["tool_result"].as_str().unwrap().chars().count();
        assert!(rlen <= TOOL_RESULT_CAP + 1);
        // 总载荷受预算约束（48KB×2 容差——文本块不降桩只截断）。
        let total: usize = out
            .iter()
            .map(|b| serde_json::to_string(b).unwrap().len())
            .sum();
        assert!(total < MSG_BUDGET_BYTES * 2, "total={} bytes", total);
    }

    /// thin tool_calls：载荷字段（arguments/result）不再随消息级数组双发。
    #[test]
    fn tool_calls_thinned() {
        let mut m = msg(
            "a1",
            crate::chats::Role::Assistant,
            vec![tool_block("read_file", serde_json::json!({"path": "a"}), "r")],
        );
        m.tool_calls.push(ToolCall {
            tool: "read_file".into(),
            args: serde_json::json!({"path": "a"}),
            result: "r".into(),
            status: "success".into(),
            id: "tc-read_file".into(),
        });
        let p = paginate_and_normalize(&session(vec![m]), 50, None);
        let tc = &p["session"]["messages"][0]["tool_calls"][0];
        assert_eq!(tc["name"], serde_json::json!("read_file"));
        assert!(tc.get("arguments").is_none());
        assert!(tc.get("result").is_none());
    }

    /// U-2 实测样本：auto-edit workspace 真实 chats.json 的工具密集巨条
    /// （PLAN-082 §10-7 的 652KB 会话）——分页+直出后单页载荷受预算约束、
    /// tool 块全为纯字符串字段（无嵌套 tool/tc 对象）。文件缺席（非本机
    /// /已迁移）时跳过——合成形态等价性由前两测锁定。
    #[test]
    fn u2_real_auto_edit_payload_bounds() {
        let path = std::path::Path::new("D:/autostack/auto-edit/.autoos/chats.json");
        if !path.exists() {
            eprintln!("u2_real: SKIPPED — auto-edit chats.json not present");
            return;
        }
        let bytes = std::fs::read(path).expect("read auto-edit chats.json");
        let map: std::collections::HashMap<String, ChatSession> =
            serde_json::from_slice(&bytes).expect("parse as ChatStore map");
        // 巨条会话：按最大消息字节数定位（历史样本 9063dfd4*，勿硬编码）。
        let target = map
            .values()
            .max_by_key(|s| {
                s.messages
                    .iter()
                    .map(|m| serde_json::to_string(m).map(|x| x.len()).unwrap_or(0))
                    .max()
                    .unwrap_or(0)
            })
            .expect("non-empty store");
        let giant = target
            .messages
            .iter()
            .max_by_key(|m| serde_json::to_string(m).map(|x| x.len()).unwrap_or(0))
            .unwrap();

        let p = paginate_and_normalize(target, DEFAULT_PAGE_LIMIT, None);
        let msgs = p["session"]["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), target.messages.len(), "小会话全量一页");

        // 每消息（尤其巨条）blocks 拍平且无嵌套 tool 载荷。
        for m in msgs {
            for b in m["blocks"].as_array().unwrap() {
                if b["kind"] == serde_json::json!("tool") {
                    assert!(b.get("tool").is_none(), "tool payload must be flattened");
                    assert!(b.get("tc").is_none(), "no live-stream shape in paged view");
                    assert!(b["tool_name"].is_string());
                    assert!(b["summary"].is_string());
                }
            }
        }
        // 巨条瘦身：整页序列化尺寸较原巨条显著收缩（预算钳制 + thin
        // tool_calls 去重——原 652KB 条实测 < 200KB 级）。
        let page_bytes = serde_json::to_string(&p).unwrap().len();
        let giant_bytes =
            serde_json::to_string(giant).map(|x| x.len()).unwrap_or(0);
        assert!(
            page_bytes < giant_bytes,
            "page {} vs giant {} — must shrink",
            page_bytes,
            giant_bytes
        );
        eprintln!(
            "u2_real: session={} msgs={} giant={}B page={}B",
            target.id,
            target.messages.len(),
            giant_bytes,
            page_bytes
        );
    }
}
