//! Report tools — `emit_report`（PLAN-032；PLAN-035 v2 结构化）。
//!
//! document 相位 agent 调用：只交付**结构化数据**（目标/关联 Goals/各阶段
//! 成果/交付物），HTML 与 markdown 双产物由后端**机械渲染**（单页自包含、
//! 无脚本无外链），机械指标（步骤/工具调用/令牌/时长）由 `run_report`
//! 自动装配——Agent 不碰版面与指标，杜绝稀疏/冲突。产物落盘 workspace
//! `.autoos/reports/{run_id}/` 并登记 `report_emitted`（持久化 + SSE 广播）；
//! 结构化数据随 ReportMeta.structured 流向前端 block 渲染。

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
/// 之外的第二道闸；v2 机械渲染天然满足，仍保留以防未来手写路径回归）。
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

/// 校验 v2 结构化入参（字段级防线——kind/change 枚举、必填段）。
fn validate_structured(st: &Value) -> Result<(), String> {
    if st["objective"].as_str().unwrap_or("").trim().is_empty() {
        return Err("emit_report: objective（一句话目标）必填".into());
    }
    let stages = st["stages"].as_array().cloned().unwrap_or_default();
    if stages.is_empty() {
        return Err("emit_report: stages（各阶段成果）至少一项".into());
    }
    for s in &stages {
        if s["title"].as_str().unwrap_or("").trim().is_empty() {
            return Err("emit_report: 每个 stage 必须有 title".into());
        }
    }
    for d in st["deliverables"].as_array().cloned().unwrap_or_default() {
        match d["kind"].as_str().unwrap_or("") {
            "code" | "spec" | "doc" | "file" | "report" => {}
            other => {
                return Err(format!(
                    "emit_report: deliverable.kind 非法 '{other}'（允许 code/spec/doc/file/report）"
                ))
            }
        }
        match d["change"].as_str().unwrap_or("") {
            "+" | "-" | "M" => {}
            other => return Err(format!("emit_report: deliverable.change 非法 '{other}'（允许 +/-/M）")),
        }
        if d["name"].as_str().unwrap_or("").trim().is_empty() {
            return Err("emit_report: 每个 deliverable 必须有 name".into());
        }
    }
    Ok(())
}

/// HTML 文本转义。
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 机械渲染单页自包含 HTML（深色卡片基调；区块：头部/目标+chips/流程链/
/// 指标格/交付物表/脚注）。metrics 为 build_run_report 的机械指标 JSON。
fn render_report_html(title: &str, st: &Value, metrics: &Value, date: &str) -> String {
    let objective = esc(st["objective"].as_str().unwrap_or(""));
    let mut goal_chips = String::new();
    for g in st["goal_links"].as_array().cloned().unwrap_or_default() {
        let label = esc(g["label"].as_str().or_else(|| g["id"].as_str()).unwrap_or("?"));
        goal_chips.push_str(&format!(
            "<span style=\"display:inline-block;padding:4px 12px;margin:2px;border-radius:999px;background:#164e3e;color:#6ee7b9;font-size:13px\">{label}</span>"
        ));
    }
    let mut flow = String::new();
    let stages = st["stages"].as_array().cloned().unwrap_or_default();
    for (i, s) in stages.iter().enumerate() {
        let t = esc(s["title"].as_str().unwrap_or(""));
        let o = esc(s["outcome"].as_str().unwrap_or(""));
        if i > 0 {
            flow.push_str("<div style=\"color:#64748b;font-size:20px;align-self:center\">→</div>");
        }
        flow.push_str(&format!(
            "<div style=\"flex:1;min-width:120px;background:#1e293b;border:1px solid #334155;border-radius:12px;padding:12px 14px\"><div style=\"font-size:14px;font-weight:600;color:#93c5fd\">{t}</div><div style=\"font-size:12px;color:#94a3b8;margin-top:6px;line-height:1.5\">{o}</div></div>"
        ));
    }
    let m = |k: &str| esc(metrics[k].as_str().unwrap_or("—"));
    let deliverables = st["deliverables"].as_array().cloned().unwrap_or_default();
    let mut dl_rows = String::new();
    for d in deliverables {
        let kind = d["kind"].as_str().unwrap_or("file");
        let name = esc(d["name"].as_str().unwrap_or(""));
        let change = d["change"].as_str().unwrap_or("");
        let detail = esc(d["detail"].as_str().unwrap_or(""));
        let (mark, color) = match change {
            "+" => ("+", "#6ee7b9"),
            "-" => ("-", "#fca5a5"),
            _ => ("M", "#93c5fd"),
        };
        dl_rows.push_str(&format!(
            "<tr><td style=\"padding:6px 10px;border-top:1px solid #334155;color:#94a3b8;font-size:12px\">{kind}</td><td style=\"padding:6px 10px;border-top:1px solid #334155;color:#e2e8f0;font-size:13px\">{name}</td><td style=\"padding:6px 10px;border-top:1px solid #334155;color:{color};font-weight:700\">{mark}</td><td style=\"padding:6px 10px;border-top:1px solid #334155;color:#94a3b8;font-size:12px\">{detail}</td></tr>"
        ));
    }
    let dl_section = if dl_rows.is_empty() {
        String::new()
    } else {
        format!(
            "<div class=\"slide\"><h2>交付物</h2><table style=\"width:100%;border-collapse:collapse\"><tr><th style=\"text-align:left;padding:6px 10px;color:#64748b;font-size:12px\">类型</th><th style=\"text-align:left;padding:6px 10px;color:#64748b;font-size:12px\">名称</th><th style=\"text-align:left;padding:6px 10px;color:#64748b;font-size:12px\">变更</th><th style=\"text-align:left;padding:6px 10px;color:#64748b;font-size:12px\">说明</th></tr>{dl_rows}</table></div>"
        )
    };
    let goals_section = if goal_chips.is_empty() {
        String::new()
    } else {
        format!("<div style=\"margin-top:8px\">{goal_chips}</div>")
    };
    format!(
        "<!DOCTYPE html><html lang=\"zh-CN\"><head><meta charset=\"utf-8\"><title>{t}</title><style>body{{font-family:'PingFang SC','Microsoft YaHei',sans-serif;background:#0f172a;color:#e2e8f0;margin:0;padding:24px}}.slide{{max-width:960px;margin:16px auto;padding:32px 40px;background:#1e293b;border-radius:16px}}h1{{font-size:30px;margin:0 0 6px}}h2{{font-size:20px;margin:0 0 14px;color:#93c5fd}}.obj{{font-size:15px;line-height:1.7;color:#cbd5e1}}.metric{{display:flex;gap:12px}}.mbox{{flex:1;background:#0f172a;border:1px solid #334155;border-radius:12px;padding:14px;text-align:center}}.mnum{{font-size:24px;font-weight:700;color:#6ee7b9}}.mlab{{font-size:12px;color:#64748b;margin-top:4px}}.flow{{display:flex;gap:8px;flex-wrap:wrap}}.foot{{text-align:center;color:#475569;font-size:12px;margin:24px 0}}</style></head><body>\
<div class=\"slide\"><h1>{t}</h1><div style=\"color:#64748b;font-size:13px\">{date} · 机械渲染报告（数据与指标同源）</div><h2 style=\"margin-top:18px\">目标</h2><div class=\"obj\">{objective}</div>{goals_section}</div>\
<div class=\"slide\"><h2>实现流程 · 各阶段成果</h2><div class=\"flow\">{flow}</div></div>\
<div class=\"slide\"><h2>指标</h2><div class=\"metric\"><div class=\"mbox\"><div class=\"mnum\">{m0}</div><div class=\"mlab\">步骤完成</div></div><div class=\"mbox\"><div class=\"mnum\">{m1}</div><div class=\"mlab\">工具调用</div></div><div class=\"mbox\"><div class=\"mnum\">{m2}</div><div class=\"mlab\">令牌消耗</div></div><div class=\"mbox\"><div class=\"mnum\">{m3}</div><div class=\"mlab\">总用时</div></div></div></div>\
{dl_section}\
<div class=\"foot\">AutoMusk · 结构化报告 v2</div></body></html>",
        t = esc(title),
        objective = objective,
        goals_section = goals_section,
        flow = flow,
        m0 = m("goals_met"),
        m1 = m("tool_calls"),
        m2 = m("cost"),
        m3 = m("duration_s"),
        dl_section = dl_section,
        date = date,
    )
}

/// 机械渲染同构 markdown 源。
fn render_report_markdown(title: &str, st: &Value, metrics: &Value) -> String {
    let mut md = format!("# {}\n\n**目标**：{}\n\n", title, st["objective"].as_str().unwrap_or(""));
    let goals: Vec<String> = st["goal_links"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|g| g["label"].as_str().or_else(|| g["id"].as_str()).unwrap_or("?").to_string())
        .collect();
    if !goals.is_empty() {
        md.push_str(&format!("**关联 Goals**：{}\n\n", goals.join("、")));
    }
    md.push_str("## 实现流程 · 各阶段成果\n\n");
    for s in st["stages"].as_array().cloned().unwrap_or_default() {
        md.push_str(&format!(
            "- **{}**：{}\n",
            s["title"].as_str().unwrap_or(""),
            s["outcome"].as_str().unwrap_or("")
        ));
    }
    md.push_str(&format!(
        "\n## 指标\n\n| 步骤 | 工具调用 | 令牌 | 用时 |\n|---|---|---|---|\n| {} | {} | {} | {}s |\n",
        metrics["goals_met"].as_str().unwrap_or("—"),
        metrics["tool_calls"].as_str().unwrap_or("—"),
        metrics["cost"].as_str().unwrap_or("—"),
        metrics["duration_s"].as_str().unwrap_or("—"),
    ));
    let dls = st["deliverables"].as_array().cloned().unwrap_or_default();
    if !dls.is_empty() {
        md.push_str("\n## 交付物\n\n| 类型 | 名称 | 变更 | 说明 |\n|---|---|---|---|\n");
        for d in dls {
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                d["kind"].as_str().unwrap_or(""),
                d["name"].as_str().unwrap_or(""),
                d["change"].as_str().unwrap_or(""),
                d["detail"].as_str().unwrap_or("")
            ));
        }
    }
    md
}

#[async_trait]
impl Tool for EmitReport {
    fn name(&self) -> &str {
        "emit_report"
    }

    fn description(&self) -> &str {
        "登记本 Run 的结构化汇报报告（v2：只交数据，不写 HTML——版面与指标由系统\
         机械渲染，杜绝稀疏与冲突）。传：title（报告标题）、objective（一句话\
         目标）、goal_links（关联的 Spec Goal/SubGoal 列表，[{id,label}]，可空）、\
         stages（各阶段成果，[{title,outcome}]，必填≥1）、deliverables（交付物\
         [{kind:code|spec|doc|file|report, name, change:+|-|M, detail}]，可空）、\
         summary（可选补充）。仅在 relay run 的 document 相位可用。"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "title": { "type": "string", "description": "报告标题" },
                "objective": { "type": "string", "description": "一句话目标" },
                "goal_links": {
                    "type": "array",
                    "items": { "type": "object", "properties": { "id": { "type": "string" }, "label": { "type": "string" } } },
                    "description": "关联的 Spec Goal/SubGoal（可空数组）"
                },
                "stages": {
                    "type": "array",
                    "items": { "type": "object", "properties": { "title": { "type": "string" }, "outcome": { "type": "string" } }, "required": ["title"] },
                    "description": "各阶段成果（流程方框，至少一项）"
                },
                "deliverables": {
                    "type": "array",
                    "items": { "type": "object", "properties": { "kind": { "type": "string", "enum": ["code","spec","doc","file","report"] }, "name": { "type": "string" }, "change": { "type": "string", "enum": ["+","-","M"] }, "detail": { "type": "string" } }, "required": ["kind","name","change"] },
                    "description": "交付物清单（可空数组）"
                },
                "summary": { "type": "string", "description": "可选补充说明" }
            },
            "required": ["title", "objective", "stages"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let title = args["title"].as_str().unwrap_or("").trim().to_string();
        if title.is_empty() {
            return Err(ToolError::Args("emit_report: title 必填".into()));
        }
        if let Err(e) = validate_structured(args) {
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

        // 机械指标（同源装配，Agent 不可触碰）。
        let metrics = ws
            .relay
            .run_report(&run_id)
            .map(|r| serde_json::to_value(&r).unwrap_or_default())
            .unwrap_or_else(|| json!({}));
        let date = crate::plans::now_iso();

        // 机械渲染双产物（HTML 过自包含闸——代码生成天然满足）。
        let html = render_report_html(&title, args, &metrics, &date);
        if let Err(e) = guard_self_contained(&html) {
            return Err(ToolError::Exec(format!("emit_report: 机械渲染异常：{e}")));
        }
        let markdown = render_report_markdown(&title, args, &metrics);

        let root = ws.root.join(".autoos").join("reports").join(&run_id);
        std::fs::create_dir_all(&root)
            .map_err(|e| ToolError::Exec(format!("create report dir: {e}")))?;
        let html_path = root.join("report.html");
        let md_path = root.join("report.md");
        std::fs::write(&html_path, &html)
            .map_err(|e| ToolError::Exec(format!("write report.html: {e}")))?;
        std::fs::write(&md_path, &markdown)
            .map_err(|e| ToolError::Exec(format!("write report.md: {e}")))?;

        let rel = format!(".autoos/reports/{run_id}/report.html");
        let meta = ReportMeta {
            format: "html".to_string(),
            title: title.clone(),
            path: rel.clone(),
            structured: Some(args.clone()),
        };
        ws.relay
            .append_report(&run_id, meta)
            .ok_or_else(|| ToolError::Exec("emit_report: 登记 report_emitted 失败（run 已失效）".into()))?;

        Ok(format!(
            "报告已生成并登记（机械渲染）：{title}（html: {rel}，md 源同目录 report.md）"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Value {
        json!({
            "title": "PLAN-001 沉淀报告",
            "objective": "把计划沉淀进 Spec 知识库",
            "goal_links": [{"id": "G1", "label": "知识库"}],
            "stages": [
                {"title": "门禁校验", "outcome": "reviewed 通过"},
                {"title": "沉淀", "outcome": "4 条目入 3 区"},
                {"title": "报告", "outcome": "本报告"}
            ],
            "deliverables": [
                {"kind": "spec", "name": "docs/specs/README.md", "change": "M", "detail": "新增模块条目"}
            ]
        })
    }

    fn metrics() -> Value {
        json!({"goals_met": "1/1", "tool_calls": "3", "cost": "745", "duration_s": "47"})
    }

    #[test]
    fn guard_rejects_script_and_external_links() {
        assert!(guard_self_contained("<html><body>ok</body></html>").is_ok());
        assert!(guard_self_contained("<div onclick=alert(1)>").is_ok()); // v1 粗粒度：不查事件属性
        assert!(guard_self_contained("<SCRIPT>x</SCRIPT>").is_err());
        assert!(guard_self_contained("<img src=\"http://x/y.png\">").is_err());
        assert!(guard_self_contained("<style>@import 'a.css'</style>").is_err());
        assert!(guard_self_contained("<iframe srcdoc=x>").is_err());
    }

    #[test]
    fn validate_rejects_missing_fields_and_bad_enums() {
        assert!(validate_structured(&sample()).is_ok());
        let mut no_obj = sample();
        no_obj["objective"] = json!("");
        assert!(validate_structured(&no_obj).is_err());
        let mut no_stages = sample();
        no_stages["stages"] = json!([]);
        assert!(validate_structured(&no_stages).is_err());
        let mut bad_kind = sample();
        bad_kind["deliverables"][0]["kind"] = json!("movie");
        assert!(validate_structured(&bad_kind).is_err());
        let mut bad_change = sample();
        bad_change["deliverables"][0]["change"] = json!("~");
        assert!(validate_structured(&bad_change).is_err());
    }

    #[test]
    fn rendered_html_contains_all_sections_and_passes_guard() {
        let html = render_report_html("T", &sample(), &metrics(), "2026-08-22");
        for marker in [
            "目标", "实现流程", "指标", "交付物", "门禁校验", "docs/specs/README.md",
            "1/1", "745",
        ] {
            assert!(html.contains(marker), "missing section: {marker}");
        }
        assert!(!html.contains("<script"), "no script tags");
        assert!(guard_self_contained(&html).is_ok(), "mechanical html must pass guard");
    }

    #[test]
    fn rendered_markdown_mirrors_sections() {
        let md = render_report_markdown("T", &sample(), &metrics());
        for marker in ["# T", "目标", "门禁校验", "| 1/1 | 3 | 745 | 47s |", "docs/specs/README.md"] {
            assert!(md.contains(marker), "missing: {marker}");
        }
    }
}
