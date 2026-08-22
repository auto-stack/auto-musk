//! Report tools — `emit_report`（PLAN-032 → 035 v2 → PLAN-036 `.ad`）。
//!
//! document 相位 agent 调用：交付一份 **`.ad` 文档**（YAML frontmatter +
//! Markdown 超集正文——`@autodown` 生态格式，LLM 最自然的文档形态）。
//! 转化链全部复用现成件：后端拆 frontmatter（标量 + 内联数组子集）→
//! `ReportMeta.structured`（frontmatter 数据 + 正文 body）；机械指标由
//! `run_report` 自动装配注入（**不采信文档中的数字**）；HTML/markdown 导出
//! 产物由后端机械渲染（`md_to_html` 最小 Markdown 子集）；前端报告卡正文
//! 直接喂 StreamingRenderer（`@autodown/vue` 同源副本），结构化 blocks
//! （Goal chips/交付物 badges）由 frontmatter 数据驱动。产物落盘 workspace
//! `.autoos/reports/{run_id}/` 并登记 `report_emitted`。

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

/// 粗粒度内容防线：报告必须自包含、无脚本、无外链（机械渲染天然满足，
/// 保留以防未来手写路径回归）。
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

/// HTML 文本转义。
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 去成对包裹引号 + 行尾注释。
fn unquote_scalar(s: &str) -> String {
    let s = s.trim();
    let s = if let Some(idx) = s.find(" #") { &s[..idx] } else { s };
    let s = s.trim();
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// 按顶层逗号切分（尊重花括号/引号嵌套）。
fn split_top_level(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut in_str: Option<char> = None;
    let mut cur = String::new();
    for c in s.chars() {
        if let Some(q) = in_str {
            cur.push(c);
            if c == q {
                in_str = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                in_str = Some(c);
                cur.push(c);
            }
            '{' | '[' => {
                depth += 1;
                cur.push(c);
            }
            '}' | ']' => {
                depth -= 1;
                cur.push(c);
            }
            ',' if depth == 0 => {
                out.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// 解析内联数组元素：`{k: v, k2: v2}` → object；裸标量 → string。
fn parse_inline_item(item: &str) -> Value {
    let t = item.trim();
    if t.starts_with('{') && t.ends_with('}') {
        let mut obj = serde_json::Map::new();
        for pair in split_top_level(&t[1..t.len() - 1]) {
            if let Some((k, v)) = pair.split_once(':') {
                let key = unquote_scalar(k);
                if !key.is_empty() {
                    let val = unquote_scalar(v);
                    obj.insert(key, json!(val));
                }
            }
        }
        Value::Object(obj)
    } else {
        json!(unquote_scalar(t))
    }
}

/// 解析内联数组值：`[{id: G1, label: x}, {...}]` / `[a, b]`。
fn parse_inline_list(s: &str) -> Value {
    let inner = s.trim().trim_start_matches('[').trim_end_matches(']');
    if inner.trim().is_empty() {
        return json!([]);
    }
    Value::Array(split_top_level(inner).iter().map(|i| parse_inline_item(i)).collect())
}

/// 拆 `.ad` 文档：frontmatter（标量 + 内联数组子集）与正文 body。
/// 无 frontmatter → 空 object + 全文为 body。
fn parse_ad_frontmatter(content: &str) -> (Value, String) {
    let mut lines = content.lines();
    if lines.next().map(|l| l.trim()) != Some("---") {
        return (json!({}), content.to_string());
    }
    let mut fm = serde_json::Map::new();
    let mut body_lines: Vec<&str> = Vec::new();
    let mut in_fm = true;
    for line in content.lines().skip(1) {
        if in_fm {
            if line.trim() == "---" {
                in_fm = false;
                continue;
            }
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue; // 空行/注释
            }
            if let Some((k, v)) = line.split_once(':') {
                let key = k.trim().to_string();
                let val = v.trim();
                if !key.is_empty() && !val.is_empty() {
                    let parsed = if val.starts_with('[') && val.ends_with(']') {
                        parse_inline_list(val)
                    } else {
                        json!(unquote_scalar(val))
                    };
                    fm.insert(key, parsed);
                }
            }
        } else {
            body_lines.push(line);
        }
    }
    // 去掉 body 开头空行
    let body = body_lines
        .iter()
        .skip_while(|l| l.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    (Value::Object(fm), body)
}

/// 校验 `.ad` 报告：title 与正文必填；deliverables 若给出走枚举校验。
fn validate_ad(fm: &Value, body: &str) -> Result<String, String> {
    let title = fm["title"].as_str().unwrap_or("").trim().to_string();
    if title.is_empty() {
        return Err("emit_report: frontmatter 必须含 title".into());
    }
    if body.trim().is_empty() {
        return Err("emit_report: 正文（body）不能为空".into());
    }
    for d in fm["deliverables"].as_array().cloned().unwrap_or_default() {
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
    Ok(title)
}

/// 行内标记（转义后应用）：**bold** / `code`。
fn inline_html(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(pos) = rest.find("**") {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + 2..];
        if let Some(end) = after.find("**") {
            out.push_str("<strong>");
            out.push_str(after[..end].trim());
            out.push_str("</strong>");
            rest = &after[end + 2..];
        } else {
            out.push_str("**");
            rest = after;
        }
    }
    out.push_str(rest);
    // 行内码
    let mut out2 = String::new();
    let mut rest = out.as_str();
    while let Some(pos) = rest.find('`') {
        out2.push_str(&rest[..pos]);
        let after = &rest[pos + 1..];
        if let Some(end) = after.find('`') {
            out2.push_str("<code>");
            out2.push_str(after[..end].trim());
            out2.push_str("</code>");
            rest = &after[end + 1..];
        } else {
            out2.push('`');
            rest = after;
        }
    }
    out2.push_str(rest);
    out2
}

/// 最小 Markdown 子集 → HTML（标题/段落/粗体/行内码/有序无序列表/表格/
/// 代码栏/hr/引用）。行式状态机；全部文本先转义。
fn md_to_html(md: &str) -> String {
    let mut out = String::new();
    let mut in_code = false;
    let mut list_kind: Option<&str> = None; // "ul" | "ol"
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut para: Vec<String> = Vec::new();

    fn close_list(out: &mut String, kind: &mut Option<&str>) {
        if let Some(k) = kind.take() {
            out.push_str(&format!("</{k}>\n"));
        }
    }
    fn close_table(out: &mut String, rows: &mut Vec<Vec<String>>) {
        if !rows.is_empty() {
            out.push_str("<table style=\"width:100%;border-collapse:collapse\">\n");
            let (head, body) = rows.split_first().unwrap();
            out.push_str("<tr>");
            for c in head {
                out.push_str(&format!(
                    "<th style=\"text-align:left;padding:6px 10px;color:#64748b;font-size:12px\">{c}</th>"
                ));
            }
            out.push_str("</tr>\n");
            for r in body {
                out.push_str("<tr>");
                for c in r {
                    out.push_str(&format!(
                        "<td style=\"padding:6px 10px;border-top:1px solid #334155;color:#e2e8f0;font-size:13px\">{c}</td>"
                    ));
                }
                out.push_str("</tr>\n");
            }
            out.push_str("</table>\n");
            rows.clear();
        }
    }
    fn close_para(out: &mut String, para: &mut Vec<String>) {
        if !para.is_empty() {
            out.push_str(&format!("<p>{}</p>\n", inline_html(&para.join(" "))));
            para.clear();
        }
    }

    for line in md.lines() {
        let raw = line;
        let t = raw.trim();
        if t.starts_with("```") {
            close_para(&mut out, &mut para);
            close_list(&mut out, &mut list_kind);
            close_table(&mut out, &mut table_rows);
            if in_code {
                out.push_str("</code></pre>\n");
                in_code = false;
            } else {
                out.push_str("<pre><code style=\"color:#93c5fd\">");
                in_code = true;
            }
            continue;
        }
        if in_code {
            out.push_str(&esc(raw));
            out.push('\n');
            continue;
        }
        if t.is_empty() {
            close_para(&mut out, &mut para);
            close_list(&mut out, &mut list_kind);
            close_table(&mut out, &mut table_rows);
            continue;
        }
        // 表格行
        if t.starts_with('|') && t.ends_with('|') {
            close_para(&mut out, &mut para);
            close_list(&mut out, &mut list_kind);
            let cells: Vec<String> = t
                .trim_matches('|')
                .split('|')
                .map(|c| inline_html(&esc(c.trim())))
                .collect();
            // 分隔行（---）跳过
            if !cells.iter().all(|c| c.trim().is_empty() || c.starts_with('-')) {
                table_rows.push(cells);
            }
            continue;
        }
        close_table(&mut out, &mut table_rows);
        // hr
        if t == "---" || t == "***" {
            close_para(&mut out, &mut para);
            out.push_str("<hr>\n");
            continue;
        }
        // 标题
        let level = t.chars().take_while(|c| *c == '#').count();
        if level > 0 && level <= 6 && t[level..].starts_with(' ') {
            close_para(&mut out, &mut para);
            close_list(&mut out, &mut list_kind);
            let text = inline_html(&esc(t[level + 1..].trim()));
            let size = 26 - (level as i32 * 3);
            out.push_str(&format!(
                "<h{level} style=\"font-size:{size}px;margin:14px 0 8px;color:#e2e8f0\">{text}</h{level}>\n"
            ));
            continue;
        }
        // 引用
        if t.starts_with('>') {
            close_para(&mut out, &mut para);
            out.push_str(&format!(
                "<blockquote style=\"border-left:3px solid #475569;margin:8px 0;padding:2px 12px;color:#94a3b8\">{}</blockquote>\n",
                inline_html(&esc(t.trim_start_matches('>').trim()))
            ));
            continue;
        }
        // 无序列表
        if t.starts_with("- ") || t.starts_with("* ") {
            close_para(&mut out, &mut para);
            if list_kind.as_deref() != Some("ul") {
                close_list(&mut out, &mut list_kind);
                out.push_str("<ul style=\"margin:6px 0;padding-left:22px\">\n");
                list_kind = Some("ul");
            }
            out.push_str(&format!(
                "<li style=\"margin:3px 0\">{}</li>\n",
                inline_html(&esc(t[2..].trim()))
            ));
            continue;
        }
        // 有序列表
        if let Some((num, rest)) = t.split_once(". ") {
            if !num.is_empty() && num.chars().all(|c| c.is_ascii_digit()) {
                close_para(&mut out, &mut para);
                if list_kind.as_deref() != Some("ol") {
                    close_list(&mut out, &mut list_kind);
                    out.push_str("<ol style=\"margin:6px 0;padding-left:22px\">\n");
                    list_kind = Some("ol");
                }
                out.push_str(&format!(
                    "<li style=\"margin:3px 0\">{}</li>\n",
                    inline_html(&esc(rest.trim()))
                ));
                continue;
            }
        }
        // 段落
        para.push(esc(t));
    }
    close_para(&mut out, &mut para);
    close_list(&mut out, &mut list_kind);
    close_table(&mut out, &mut table_rows);
    if in_code {
        out.push_str("</code></pre>\n");
    }
    out
}

/// v3 机械渲染 HTML：标题头 + 机械指标四格 + 正文（md_to_html）。
fn render_report_html_v3(title: &str, st: &Value, metrics: &Value, date: &str) -> String {
    let m = |k: &str| esc(metrics[k].as_str().unwrap_or("—"));
    let body = md_to_html(st["body"].as_str().unwrap_or(""));
    format!(
        "<!DOCTYPE html><html lang=\"zh-CN\"><head><meta charset=\"utf-8\"><title>{}</title><style>body{{font-family:'PingFang SC','Microsoft YaHei',sans-serif;background:#0f172a;color:#e2e8f0;margin:0;padding:24px}}.slide{{max-width:960px;margin:16px auto;padding:32px 40px;background:#1e293b;border-radius:16px}}.metric{{display:flex;gap:12px;margin-top:16px}}.mbox{{flex:1;background:#0f172a;border:1px solid #334155;border-radius:12px;padding:14px;text-align:center}}.mnum{{font-size:24px;font-weight:700;color:#6ee7b9}}.mlab{{font-size:12px;color:#64748b;margin-top:4px}}p{{font-size:15px;line-height:1.7;color:#cbd5e1}}.foot{{text-align:center;color:#475569;font-size:12px;margin:24px 0}}</style></head><body>\
<div class=\"slide\"><h1 style=\"font-size:30px;margin:0\">{}</h1><div style=\"color:#64748b;font-size:13px\">{}</div>\
<div class=\"metric\"><div class=\"mbox\"><div class=\"mnum\">{}</div><div class=\"mlab\">步骤完成</div></div><div class=\"mbox\"><div class=\"mnum\">{}</div><div class=\"mlab\">工具调用</div></div><div class=\"mbox\"><div class=\"mnum\">{}</div><div class=\"mlab\">令牌消耗</div></div><div class=\"mbox\"><div class=\"mnum\">{}</div><div class=\"mlab\">总用时</div></div></div></div>\
<div class=\"slide\">{}</div>\
<div class=\"foot\">AutoMusk · AutoDown 报告（机械渲染，指标自动采集）</div></body></html>",
        esc(title),
        esc(title),
        format!("{date} · 机械渲染"),
        m("goals_met"),
        m("tool_calls"),
        m("cost"),
        m("duration_s"),
        body,
    )
}

/// v3 机械渲染 markdown 导出（frontmatter 元信息 + 正文 + 指标表）。
fn render_report_markdown_v3(title: &str, st: &Value, metrics: &Value) -> String {
    let mut md = format!("# {title}\n\n");
    if let Some(s) = st["summary"].as_str() {
        if !s.is_empty() {
            md.push_str(&format!("**摘要**：{s}\n\n"));
        }
    }
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
    md.push_str(st["body"].as_str().unwrap_or(""));
    md.push_str(&format!(
        "\n\n## 指标（自动采集）\n\n| 步骤 | 工具调用 | 令牌 | 用时 |\n|---|---|---|---|\n| {} | {} | {} | {}s |\n",
        metrics["goals_met"].as_str().unwrap_or("—"),
        metrics["tool_calls"].as_str().unwrap_or("—"),
        metrics["cost"].as_str().unwrap_or("—"),
        metrics["duration_s"].as_str().unwrap_or("—"),
    ));
    md
}

#[async_trait]
impl Tool for EmitReport {
    fn name(&self) -> &str {
        "emit_report"
    }

    fn description(&self) -> &str {
        "登记本 Run 的汇报报告（PLAN-036：交付 `.ad` 文档——YAML frontmatter + \
         Markdown 超集正文；版面与指标由系统机械渲染，正文中的指标数字不会被\
         采信）。frontmatter 键：`title`（必填）、`summary`（一句话摘要）、\
         `goal_links`（`[{id,label}]` 内联数组，可空）、`deliverables`\
        （`[{kind:code|spec|doc|file|report, name, change:+|-|M, detail}]` 内联\
         数组，可空）。正文建议：`## 目标` 一段；`## 实现流程 · 各阶段成果`\
         有序列表；交付物表格。仅在 relay run 的 document 相位可用。"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "ad": {
                    "type": "string",
                    "description": "完整 .ad 文档全文：`---` 包裹的 frontmatter + Markdown 超集正文"
                }
            },
            "required": ["ad"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let ad = args["ad"].as_str().unwrap_or("");
        if ad.trim().is_empty() {
            return Err(ToolError::Args("emit_report: ad（.ad 文档全文）必填".into()));
        }
        let (fm, body) = parse_ad_frontmatter(ad);
        let title = validate_ad(&fm, &body).map_err(ToolError::Args)?;

        // relay step agent 语境：parent_conversation_id = run_id（会话唯一化）。
        let run_id = self.ctx.parent_conversation_id.clone();
        let ws = self.ws();
        if ws.relay.report_meta(&run_id).is_none() && ws.relay.status(&run_id).is_none() {
            return Err(ToolError::Exec(format!(
                "emit_report: 未找到 run '{run_id}'——本工具仅在 relay run 相位内可用"
            )));
        }

        // 机械指标（同源装配；文档中的数字不采信）。
        let metrics = ws
            .relay
            .run_report(&run_id)
            .map(|r| serde_json::to_value(&r).unwrap_or_default())
            .unwrap_or_else(|| json!({}));
        let date = crate::plans::now_iso();

        // structured：frontmatter 数据 + 正文（前端 blocks 与 StreamingRenderer 共用）。
        let mut structured = fm.clone();
        structured["body"] = json!(body);

        // 机械渲染双产物。
        let html = render_report_html_v3(&title, &structured, &metrics, &date);
        if let Err(e) = guard_self_contained(&html) {
            return Err(ToolError::Exec(format!("emit_report: 机械渲染异常：{e}")));
        }
        let markdown = render_report_markdown_v3(&title, &structured, &metrics);

        let root = ws.root.join(".autoos").join("reports").join(&run_id);
        std::fs::create_dir_all(&root)
            .map_err(|e| ToolError::Exec(format!("create report dir: {e}")))?;
        std::fs::write(root.join("report.html"), &html)
            .map_err(|e| ToolError::Exec(format!("write report.html: {e}")))?;
        std::fs::write(root.join("report.md"), &markdown)
            .map_err(|e| ToolError::Exec(format!("write report.md: {e}")))?;
        std::fs::write(root.join("report.ad"), ad)
            .map_err(|e| ToolError::Exec(format!("write report.ad: {e}")))?;

        let rel = format!(".autoos/reports/{run_id}/report.html");
        let meta = ReportMeta {
            format: "autodown".to_string(),
            title: title.clone(),
            path: rel.clone(),
            structured: Some(structured),
        };
        ws.relay
            .append_report(&run_id, meta)
            .ok_or_else(|| ToolError::Exec("emit_report: 登记 report_emitted 失败（run 已失效）".into()))?;

        Ok(format!(
            "报告已生成并登记（.ad 机械渲染）：{title}（html/md/ad: {rel} 同目录）"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_AD: &str = "---\ntitle: PLAN-001 沉淀报告\nsummary: 沉淀完成\ngoal_links: [{id: G1, label: 知识库}]\ndeliverables: [{kind: spec, name: \"docs/specs/README.md\", change: M, detail: 新增模块条目}]\n---\n\n## 目标\n\n把计划沉淀进 **Spec 知识库**\n\n## 实现流程\n\n1. **门禁校验** — reviewed 通过\n2. **机械沉淀** — 4 条目入 3 区\n\n| 类型 | 名称 | 变更 |\n|---|---|---|\n| spec | docs/specs/README.md | M |\n";

    fn metrics() -> Value {
        json!({"goals_met": "1/1", "tool_calls": "3", "cost": "745", "duration_s": "47"})
    }

    #[test]
    fn guard_rejects_script_and_external_links() {
        assert!(guard_self_contained("<html><body>ok</body></html>").is_ok());
        assert!(guard_self_contained("<SCRIPT>x</SCRIPT>").is_err());
        assert!(guard_self_contained("<img src=\"http://x/y.png\">").is_err());
    }

    #[test]
    fn parse_ad_frontmatter_scalars_and_inline_lists() {
        let (fm, body) = parse_ad_frontmatter(SAMPLE_AD);
        assert_eq!(fm["title"], "PLAN-001 沉淀报告");
        assert_eq!(fm["summary"], "沉淀完成");
        let goals = fm["goal_links"].as_array().unwrap();
        assert_eq!(goals[0]["id"], "G1");
        assert_eq!(goals[0]["label"], "知识库");
        let dls = fm["deliverables"].as_array().unwrap();
        assert_eq!(dls[0]["kind"], "spec");
        assert_eq!(dls[0]["change"], "M");
        assert!(body.starts_with("## 目标"), "body strips leading blanks; got: {body:?}");
    }

    #[test]
    fn parse_ad_without_frontmatter_yields_empty_fm() {
        let (fm, body) = parse_ad_frontmatter("# 只是正文\n\n内容");
        assert!(fm.as_object().unwrap().is_empty());
        assert!(body.contains("只是正文"));
    }

    #[test]
    fn validate_ad_requires_title_and_body() {
        let (fm, body) = parse_ad_frontmatter(SAMPLE_AD);
        assert_eq!(validate_ad(&fm, &body).unwrap(), "PLAN-001 沉淀报告");
        let (bad_fm, _) = parse_ad_frontmatter("---\nsummary: x\n---\n\n正文");
        assert!(validate_ad(&bad_fm, "正文").is_err());
        assert!(validate_ad(&fm, "  ").is_err());
        let mut bad_enum = fm.clone();
        bad_enum["deliverables"][0]["kind"] = json!("movie");
        assert!(validate_ad(&bad_enum, "正文").is_err());
    }

    #[test]
    fn md_to_html_covers_block_subset() {
        let md = "## 标题\n\n段落有 **粗体** 与 `code`。\n\n- 项 A\n- 项 B\n\n1. 第一\n2. 第二\n\n> 引用\n\n---\n\n| a | b |\n|---|---|\n| 1 | 2 |\n";
        let html = md_to_html(md);
        for marker in [
            "<h2", "粗体", "<strong>", "<code>code</code>", "<ul", "<li", "<ol",
            "<blockquote", "<hr", "<table", "<th", ">1</td>",
        ] {
            assert!(html.contains(marker), "missing {marker}");
        }
        assert!(!html.contains("<script"), "escaped/no scripts");
    }

    #[test]
    fn rendered_v3_contains_metrics_and_body() {
        let (fm, body) = parse_ad_frontmatter(SAMPLE_AD);
        let mut st = fm.clone();
        st["body"] = json!(body);
        let html = render_report_html_v3("T", &st, &metrics(), "2026-08-22");
        for marker in ["步骤完成", "1/1", "745", "门禁校验", "<table", "知识库"] {
            assert!(html.contains(marker), "missing {marker}");
        }
        assert!(guard_self_contained(&html).is_ok());
        let md = render_report_markdown_v3("T", &st, &metrics());
        assert!(md.contains("| 1/1 | 3 | 745 | 47s |"));
        assert!(md.contains("门禁校验"));
    }
}
