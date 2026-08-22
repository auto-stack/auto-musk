//! Report tools — `emit_report`（PLAN-032）。
//!
//! document 相位 agent 调用：生成自包含、无脚本、PPT 风格的 HTML 汇报报告，
//! 落盘 workspace `.autoos/reports/{run_id}/`（html 呈现 + markdown 源双产物，
//! 为未来 AutoDown 演示格式预留），并向 run 事件流追加 `report_emitted`
//! （持久化 + SSE 广播，前端 deck 层据此渲染）。

use async_trait::async_trait;
use auto_ai_agent::{Tool, ToolError};
use serde_json::{json, Value};

use crate::relay::store::ReportMeta;
use crate::tool_context::ToolContext;
use std::sync::Arc;

use crate::workspace::WorkspaceStores;

pub struct EmitReport {
    ctx: ToolContext,
}

impl EmitReport {
    pub fn from_ctx(ctx: &ToolContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    fn ws(&self) -> Arc<WorkspaceStores> {
        self.ctx.state.registry.get(&self.ctx.workspace_id)
    }
}

/// 粗粒度内容防线：报告必须自包含、无脚本、无外链（前端 iframe sandbox
/// 之外的第二道闸；模板同款措辞引导 agent 自检）。
fn guard_self_contained(html: &str) -> Result<(), String> {
    let lower = html.to_lowercase();
    for bad in ["<script", "javascript:", "<iframe", "<object", "<embed"] {
        if lower.contains(bad) {
            return Err(format!("报告 html 含禁止元素 `{bad}`——只允许纯 CSS 的自包含文档"));
        }
    }
    for pat in ["src=\"http", "src='http", "href=\"http", "href='http", "@import"] {
        if lower.contains(pat) {
            return Err(format!("报告 html 含外链资源 `{pat}`——所有样式/图片必须内联"));
        }
    }
    Ok(())
}

#[async_trait]
impl Tool for EmitReport {
    fn name(&self) -> &str {
        "emit_report"
    }

    fn description(&self) -> &str {
        "生成本 Run 的汇报报告（PPT 风格 HTML 单文件），作为独立 ReportBlock 展示给用户。\
         要求：自包含（内联 CSS）、无任何 <script>/iframe/外链资源；分节：封面（标题+日期+\
         run 概要）/需求与方案/各阶段成果/指标（步骤·工具调用·令牌·时长）/交付物清单/结尾；\
         视觉基调类 PPT 分节卡片（大标题、留白、16:9 心智）。同时提供同结构的 markdown 源\
         （未来 AutoDown 演示格式的直通源）。仅在 relay run 的 document 相位可用。"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "description": "报告格式。本期仅支持 'html'（'autodown' 为未来演示格式预留，勿传）。"
                },
                "title": { "type": "string", "description": "报告标题（封面主标题）" },
                "html": {
                    "type": "string",
                    "description": "完整自包含 HTML 文档全文（<!DOCTYPE html> 起）。无 <script>/iframe/外链。"
                },
                "markdown": { "type": "string", "description": "同结构 markdown 源（AutoDown 直通预留）" }
            },
            "required": ["format", "title", "html", "markdown"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let format = args["format"].as_str().unwrap_or("");
        let title = args["title"].as_str().unwrap_or("");
        let html = args["html"].as_str().unwrap_or("");
        let markdown = args["markdown"].as_str().unwrap_or("");
        if format != "html" {
            return Err(ToolError::Args(format!(
                "emit_report: format 本期仅支持 'html'（收到 '{format}'；'autodown' 为预留）"
            )));
        }
        if title.is_empty() || html.is_empty() || markdown.is_empty() {
            return Err(ToolError::Args(
                "emit_report: format/title/html/markdown 均必填（html 与 markdown 同结构）".into(),
            ));
        }
        if let Err(e) = guard_self_contained(html) {
            return Err(ToolError::Args(format!("emit_report: {e}")));
        }

        // relay step agent 语境：parent_conversation_id = run_id（会话唯一化）。
        let run_id = self.ctx.parent_conversation_id.clone();
        let ws = self.ws();
        if ws.relay.report_meta(&run_id).is_none() && ws.relay.status(&run_id).is_none() {
            return Err(ToolError::Exec(format!(
                "emit_report: 未找到 run '{run_id}'——本工具仅在 relay run 相位内可用"
            )));
        }

        let root = ws.root.join(".autoos").join("reports").join(&run_id);
        std::fs::create_dir_all(&root)
            .map_err(|e| ToolError::Exec(format!("create report dir: {e}")))?;
        let html_path = root.join("report.html");
        let md_path = root.join("report.md");
        std::fs::write(&html_path, html)
            .map_err(|e| ToolError::Exec(format!("write report.html: {e}")))?;
        std::fs::write(&md_path, markdown)
            .map_err(|e| ToolError::Exec(format!("write report.md: {e}")))?;

        let rel = format!(".autoos/reports/{run_id}/report.html");
        let meta = ReportMeta {
            format: format.to_string(),
            title: title.to_string(),
            path: rel.clone(),
        };
        ws.relay
            .append_report(&run_id, meta)
            .ok_or_else(|| ToolError::Exec("emit_report: 登记 report_emitted 失败（run 已失效）".into()))?;

        Ok(format!(
            "报告已生成并登记：{title}（html: {rel}，md 源同目录 report.md）"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_rejects_script_and_external_links() {
        assert!(guard_self_contained("<html><body>ok</body></html>").is_ok());
        assert!(guard_self_contained("<div onclick=alert(1)>").is_ok()); // v1 粗粒度：不查事件属性
        assert!(guard_self_contained("<SCRIPT>x</SCRIPT>").is_err());
        assert!(guard_self_contained("<img src=\"http://x/y.png\">").is_err());
        assert!(guard_self_contained("<style>@import 'a.css'</style>").is_err());
        assert!(guard_self_contained("<iframe srcdoc=x>").is_err());
    }
}
