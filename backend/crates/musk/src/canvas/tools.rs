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

fn resolve_within_sandbox(ctx: &ToolContext, path: &str) -> Result<PathBuf, ToolError> {
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
        let status = self
            .ctx
            .state
            .canvas
            .start(resolved)
            .await
            .map_err(ToolError::Exec)?;
        Ok(ToolOutput::text(format!(
            "canvas session starting for '{}' (state: {:?}). Frames stream to the \
             canvas panel; use canvas_snapshot / canvas_state to verify, canvas_act \
             to drive the UI.",
            status.app_path, status.state
        )))
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
        self.ctx.state.canvas.stop().await;
        Ok(ToolOutput::text("canvas session stopped.".to_string()))
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
        Ok(ToolOutput::text(format!(
            "![canvas snapshot]({url})\n\nUI structure:\n```\n{summary}\n```"
        )))
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
        let manager = &self.ctx.state.canvas;
        let result = manager
            .with_client(|client| client)
            .await
            .map_err(ToolError::Exec)?
            .action(element_id, action, value)
            .await
            .map_err(ToolError::Exec)?;
        Ok(ToolOutput::text(result))
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
        let manager = &self.ctx.state.canvas;
        let result = manager
            .with_client(|client| client)
            .await
            .map_err(ToolError::Exec)?
            .state(fields)
            .await
            .map_err(ToolError::Exec)?;
        Ok(ToolOutput::text(result))
    }
}

/// 登记表（lib.rs build_agent_with_context 消费；名字须与白名单一致）。
pub fn canvas_tool_registry(ctx: &ToolContext) -> Vec<(&'static str, Arc<dyn Tool>)> {
    vec![
        ("canvas_run", Arc::new(CanvasRun::new(ctx.clone()))),
        ("canvas_stop", Arc::new(CanvasStop::new(ctx.clone()))),
        ("canvas_snapshot", Arc::new(CanvasSnapshot::new(ctx.clone()))),
        ("canvas_act", Arc::new(CanvasAct::new(ctx.clone()))),
        ("canvas_state", Arc::new(CanvasState::new(ctx.clone()))),
    ]
}
