//! PLAN-087 T-06: agent 工具五件 —— canvas_run/stop/snapshot/act/state。
//!
//! 挂 `build_agent_with_context` 既有表（lib.rs），per-mode 白名单放行
//! （coding.at）。路径安全与 API 同口径：`tool_safety::resolve_multi`
//! fail-closed（越报文列全部根，AC-06 双面之一）。

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use auto_ai_agent::{Tool, ToolError, ToolOutput};
use serde_json::{json, Value};

use crate::tool_context::ToolContext;

/// PLAN-093 T-02：会话归属守卫。目标会话已归属其他会话时，本会话的
/// canvas 工具不得操作（后台会话不得操作后来替换的应用）。未绑定预览
///（无归属）保持旧语义：任何会话可用。
fn ensure_session_owner(ctx: &ToolContext) -> Result<(), ToolError> {
    if let Some(owner) = ctx.state.canvas.owner_conversation() {
        if !owner.is_empty() && owner != ctx.parent_conversation_id {
            return Err(ToolError::Exec(format!(
                "canvas session belongs to conversation {owner} — this conversation cannot operate it (canvas_run to replace, or continue in the owning session)"
            )));
        }
    }
    Ok(())
}

pub(crate) fn resolve_within_sandbox(ctx: &ToolContext, path: &str) -> Result<PathBuf, ToolError> {
    let ws = ctx.state.registry.get(&ctx.workspace_id);
    let mut roots = vec![ws.root.clone()];
    roots.extend(
        ctx.state
            .registry
            .extra_roots(&ctx.workspace_id)
            .into_iter()
            .map(PathBuf::from),
    );
    crate::tool_safety::resolve_multi(path, &roots).map_err(ToolError::Exec)
}

/// PLAN-093 T-09：工具结果结构化元数据盖章（§5.8——结果文本缺少预览身份
/// 标识时补充 details.canvas，旧文本原样保留；SSE tool_result details
/// 透传既有，前端进度投影按 kind/代次关联，不另开轮询引擎）。
pub(crate) fn canvas_details(kind: &str, extra: Value) -> Option<Value> {
    let mut canvas = json!({ "kind": kind });
    if let (Some(dst), Some(src)) = (canvas.as_object_mut(), extra.as_object()) {
        for (k, v) in src {
            dst.insert(k.clone(), v.clone());
        }
    }
    Some(json!({ "canvas": canvas }))
}

// ── canvas_run ──────────────────────────────────────────────────────────────

pub struct CanvasRun {
    ctx: ToolContext,
}

impl CanvasRun {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for CanvasRun {
    fn name(&self) -> &str {
        "canvas_run"
    }
    fn description(&self) -> &str {
        "Launch the target Auto app in the live canvas (isolated VM window + \
         screenshot stream). Pass the app directory path (must contain pac.at), \
         relative to the workspace root. Starting a new app replaces the \
         current canvas session. The app dir must be inside the workspace."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "app_path": { "type": "string", "description": "app directory (contains pac.at), relative to workspace root" }
            },
            "required": ["app_path"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let path = args["app_path"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'app_path' argument".into()))?;
        let resolved = resolve_within_sandbox(&self.ctx, path)?;
        super::session::validate_app_dir(&resolved).map_err(ToolError::Exec)?;
        let ws = self.ctx.state.registry.get(&self.ctx.workspace_id);
        // PLAN-093 T-02：工具启动登记归属（workspace + 所属会话），后续
        // 其他会话的工具/停止按归属拒操作。
        self.ctx
            .state
            .canvas
            .start_owned(
                resolved,
                &ws.root,
                &self.ctx.workspace_id,
                Some(&self.ctx.parent_conversation_id),
                None,
            )
            .await
            .map_err(|e| match e {
                super::StartError::Spawn(msg) => ToolError::Exec(msg),
                super::StartError::GenerationConflict(_) => {
                    ToolError::Exec("canvas: generation conflict on tool start".to_string())
                }
            })?;
        let status = self.ctx.state.canvas.status();
        Ok(ToolOutput {
            content: format!(
                "canvas session starting for '{}' (state: {:?}). Frames stream to the \
                 canvas panel; use canvas_snapshot / canvas_state to verify, canvas_act \
                 to drive the UI.",
                status.app_path, status.state
            ),
            // PLAN-093 T-09：进度投影身份章（start 返回 ≠ 已可见——前端
            // 以 CanvasStore 实际状态为权威，代次章供迟到事件去伪）。
            details: canvas_details(
                "run",
                json!({ "generation_id": self.ctx.state.canvas.generation() }),
            ),
        })
    }
}

// ── canvas_stop ─────────────────────────────────────────────────────────────

pub struct CanvasStop {
    ctx: ToolContext,
}

impl CanvasStop {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for CanvasStop {
    fn name(&self) -> &str {
        "canvas_stop"
    }
    fn description(&self) -> &str {
        "Stop the current canvas session and close the live app window. Use \
         when the user asks to stop the preview or before wrapping up."
    }
    fn parameters(&self) -> Value {
        json!({ "type": "object", "properties": {} })
    }
    async fn execute(&self, _args: &Value) -> Result<ToolOutput, ToolError> {
        // PLAN-093 T-02：按归属守卫——其他会话的工具不得停掉当前目标。
        self.ctx
            .state
            .canvas
            .stop_guarded(None, Some(&self.ctx.parent_conversation_id))
            .await
            .map_err(|e| match e {
                super::StopRefused::OwnedByOther { owner } => ToolError::Exec(format!(
                    "canvas session belongs to conversation {owner} — not stopped from this conversation"
                )),
                super::StopRefused::GenerationConflict { .. } => {
                    ToolError::Exec("canvas: generation conflict on tool stop".to_string())
                }
            })?;
        Ok(ToolOutput {
            content: "canvas session stopped.".to_string(),
            details: canvas_details("stop", json!({})),
        })
    }
}

// ── canvas_snapshot ─────────────────────────────────────────────────────────

pub struct CanvasSnapshot {
    ctx: ToolContext,
}

impl CanvasSnapshot {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for CanvasSnapshot {
    fn name(&self) -> &str {
        "canvas_snapshot"
    }
    fn description(&self) -> &str {
        "Capture the current canvas state: saves the latest live frame as an \
         image (shown inline) plus the UI structure tree (vtree) as text. Use \
         after canvas_run or after canvas_act to visually verify the result."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "include_bounds": { "type": "boolean", "description": "include element bounds annotations in the vtree (default false)" }
            }
        })
    }
    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let include_bounds = args["include_bounds"].as_bool().unwrap_or(false);
        ensure_session_owner(&self.ctx)?;
        let manager = &self.ctx.state.canvas;
        let frame = manager.frame().ok_or_else(|| {
            ToolError::Exec("canvas: no frame yet (session starting or not running)".to_string())
        })?;
        let seq = manager.seq();

        // vtree 文本（独立短连，快）。
        let vtree = manager
            .with_client(|client| client)
            .await
            .map_err(ToolError::Exec)?
            .snapshot(include_bounds)
            .await
            .map_err(ToolError::Exec)?;

        // PNG 落 workspace/.canvas/snap-{seq}.png → /api/files 贴图。
        let ws = self.ctx.state.registry.get(&self.ctx.workspace_id);
        let dir = ws.root.join(".canvas");
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| ToolError::Exec(format!("canvas: cannot create .canvas dir: {e}")))?;
        let file = dir.join(format!("snap-{seq}.png"));
        tokio::fs::write(&file, frame.as_ref())
            .await
            .map_err(|e| ToolError::Exec(format!("canvas: cannot write snapshot: {e}")))?;
        let url = format!("/api/files/{}/.canvas/snap-{seq}.png", self.ctx.workspace_id);

        // vtree 摘要截断（防大载荷刷屏；头部 2000 字符足够定位）。
        let mut summary = vtree;
        if summary.chars().count() > 2000 {
            let cut: String = summary.chars().take(2000).collect();
            summary = format!("{cut}\n… (truncated)");
        }
        Ok(ToolOutput {
            content: format!(
                "![canvas snapshot]({url})\n\nUI structure:\n```\n{summary}\n```"
            ),
            details: canvas_details("verify", json!({})),
        })
    }
}

// ── canvas_act ──────────────────────────────────────────────────────────────

pub struct CanvasAct {
    ctx: ToolContext,
}

impl CanvasAct {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for CanvasAct {
    fn name(&self) -> &str {
        "canvas_act"
    }
    fn description(&self) -> &str {
        "Drive the canvas app UI: perform an action on an element. action is \
         one of press / type_text / submit / toggle / select_option / set_value \
         / clear / scroll / drag / key_press. element_id comes from \
         canvas_snapshot's structure tree. Returns the action result and any \
         state changes."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "element_id": { "type": "string", "description": "target element id (from canvas_snapshot vtree)" },
                "action": { "type": "string", "description": "press | type_text | submit | toggle | select_option | set_value | clear | scroll | drag | key_press" },
                "value": { "description": "value for type_text / set_value / select_option / key_press" }
            },
            "required": ["element_id", "action"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let element_id = args["element_id"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'element_id' argument".into()))?;
        let action = args["action"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'action' argument".into()))?;
        let value = if args.get("value").map(|v| !v.is_null()).unwrap_or(false) {
            Some(args["value"].clone())
        } else {
            None
        };
        ensure_session_owner(&self.ctx)?;
        let manager = &self.ctx.state.canvas;
        let result = manager
            .with_client(|client| client)
            .await
            .map_err(ToolError::Exec)?
            .action(element_id, action, value)
            .await
            .map_err(ToolError::Exec)?;
        Ok(ToolOutput {
            content: result,
            details: canvas_details("verify", json!({})),
        })
    }
}

// ── canvas_state ────────────────────────────────────────────────────────────

pub struct CanvasState {
    ctx: ToolContext,
}

impl CanvasState {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for CanvasState {
    fn name(&self) -> &str {
        "canvas_state"
    }
    fn description(&self) -> &str {
        "Read the live app's evaluated state (e.g. counter values, list \
         lengths). Pass optional field names to filter (suffix match); omit \
         to read everything. Use this to ASSERT state after canvas_act (e.g. \
         count incremented)."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "fields": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "optional field name filter (exact or .field suffix match)"
                }
            }
        })
    }
    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let fields: Option<Vec<String>> = args
            .get("fields")
            .and_then(|f| f.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            });
        ensure_session_owner(&self.ctx)?;
        let manager = &self.ctx.state.canvas;
        let result = manager
            .with_client(|client| client)
            .await
            .map_err(ToolError::Exec)?
            .state(fields)
            .await
            .map_err(ToolError::Exec)?;
        Ok(ToolOutput {
            content: result,
            details: canvas_details("verify", json!({})),
        })
    }
}

// ── canvas_pick ─────────────────────────────────────────────────────────────

pub struct CanvasPick {
    ctx: ToolContext,
}

impl CanvasPick {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for CanvasPick {
    fn name(&self) -> &str {
        "canvas_pick"
    }
    fn description(&self) -> &str {
        "Resolve an element's source anchor on the live canvas: returns the \
         vnode id, kind, label, bounding box, source file:line and ancestor \
         chain (same structure as the user's click-selection). Use this to \
         say precisely WHICH element you are about to change. element_id \
         comes from canvas_snapshot's structure tree (vnode_N)."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "element_id": { "type": "string", "description": "target element id (vnode_N, from canvas_snapshot vtree)" }
            },
            "required": ["element_id"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let raw = args["element_id"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'element_id' argument".into()))?;
        let vnode = parse_vnode_id(raw).ok_or_else(|| {
            ToolError::Args(format!("invalid element_id '{raw}' — expected 'vnode_N'"))
        })?;
        ensure_session_owner(&self.ctx)?;
        let manager = &self.ctx.state.canvas;
        let payload = manager.pick_vnode(vnode).ok_or_else(|| {
            ToolError::Exec(format!(
                "canvas_pick: vnode_{vnode} not in current frame — re-read current ids via canvas_snapshot / autoui_find (hot reload may have renumbered the tree)"
            ))
        })?;
        Ok(ToolOutput::text(
            serde_json::to_string_pretty(&payload)
                .unwrap_or_else(|_| payload.to_string()),
        ))
    }
}

// ── canvas_overlay ──────────────────────────────────────────────────────────

pub struct CanvasOverlay {
    ctx: ToolContext,
}

impl CanvasOverlay {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for CanvasOverlay {
    fn name(&self) -> &str {
        "canvas_overlay"
    }
    fn description(&self) -> &str {
        "Highlight elements on the live canvas for the user: draws boxes on \
         the frame over the given elements (they stay highlighted until the \
         next canvas_overlay or a clear). Use it to point at WHAT you are \
         talking about, e.g. before proposing an edit. Pass clear=true to \
         remove all highlights."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "element_ids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "elements to highlight (vnode_N ids from canvas_snapshot)"
                },
                "clear": { "type": "boolean", "description": "clear all highlights instead" }
            }
        })
    }
    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        ensure_session_owner(&self.ctx)?;
        let manager = &self.ctx.state.canvas;
        let clear = args["clear"].as_bool().unwrap_or(false);
        if clear {
            manager.set_overlay(Vec::new(), true).map_err(ToolError::Exec)?;
            return Ok(ToolOutput::text("canvas overlay cleared.".to_string()));
        }
        let raw_ids = args["element_ids"]
            .as_array()
            .ok_or_else(|| ToolError::Args("missing 'element_ids' argument (or set clear=true)".into()))?;
        let mut ids = Vec::new();
        for v in raw_ids {
            let s = v
                .as_str()
                .ok_or_else(|| ToolError::Args("element_ids items must be strings (vnode_N)".into()))?;
            let id = parse_vnode_id(s)
                .ok_or_else(|| ToolError::Args(format!("invalid element_id '{s}' — expected 'vnode_N'")))?;
            ids.push(id);
        }
        manager.set_overlay(ids.clone(), false).map_err(ToolError::Exec)?;
        let list = ids
            .iter()
            .map(|v| format!("vnode_{v}"))
            .collect::<Vec<_>>()
            .join(", ");
        Ok(ToolOutput::text(format!(
            "canvas overlay set: [{list}] — the user sees the highlight on the next frame."
        )))
    }
}

/// `vnode_N` / 裸数字 → u64（agent 面与路由共用口径）。
pub(crate) fn parse_vnode_id(raw: &str) -> Option<u64> {
    raw.strip_prefix("vnode_")
        .unwrap_or(raw)
        .parse::<u64>()
        .ok()
}

/// 登记表（lib.rs build_agent_with_context 消费；名字须与白名单一致）。
pub fn canvas_tool_registry(ctx: &ToolContext) -> Vec<(&'static str, Arc<dyn Tool>)> {
    vec![
        ("canvas_run", Arc::new(CanvasRun::new(ctx.clone()))),
        ("canvas_stop", Arc::new(CanvasStop::new(ctx.clone()))),
        ("canvas_snapshot", Arc::new(CanvasSnapshot::new(ctx.clone()))),
        ("canvas_act", Arc::new(CanvasAct::new(ctx.clone()))),
        ("canvas_state", Arc::new(CanvasState::new(ctx.clone()))),
        ("canvas_pick", Arc::new(CanvasPick::new(ctx.clone()))),
        ("canvas_overlay", Arc::new(CanvasOverlay::new(ctx.clone()))),
        ("bp_list", Arc::new(super::bp_tools::BpList::new(ctx.clone()))),
        ("bp_show", Arc::new(super::bp_tools::BpShow::new(ctx.clone()))),
        ("bp_check", Arc::new(super::bp_tools::BpCheck::new(ctx.clone()))),
        ("ui_lint", Arc::new(super::ui_lint::UiLint::new(ctx.clone()))),
        ("app_examples_list", Arc::new(super::examples_pool::AppExamplesList::new(ctx.clone()))),
        ("app_example_read", Arc::new(super::examples_pool::AppExampleRead::new(ctx.clone()))),
    ]
}

#[cfg(test)]
mod details_tests {
    use super::*;

    /// PLAN-093 T-09：details.canvas 盖章契约——kind 落位、extra 合并、
    /// 旧文本面不受影响（盖章只加 details，content 原样）。
    #[test]
    fn canvas_details_kind_and_extra_merge() {
        let d = canvas_details("run", json!({ "generation_id": 7u64 })).unwrap();
        assert_eq!(d["canvas"]["kind"], "run");
        assert_eq!(d["canvas"]["generation_id"], 7);

        let d2 = canvas_details("bp_check", json!({ "ok": false })).unwrap();
        assert_eq!(d2["canvas"]["kind"], "bp_check");
        assert_eq!(d2["canvas"]["ok"], false);

        // 空 extra：只 kind，无杂物。
        let d3 = canvas_details("verify", json!({})).unwrap();
        assert_eq!(d3["canvas"]["kind"], "verify");
        assert_eq!(d3["canvas"].as_object().unwrap().len(), 1);
    }
}
