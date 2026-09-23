//! PLAN-090 T-04: ui_lint 规则注册表 v1 + 工具 + 红绿样例集。
//!
//! 针对 AutoUI 视图坑与 codegen 坑的轻量静态 advisory 护栏（8 条规则 L001-L008）。
//! 报告头部声明 advisory 性质（报告≠失败，指引 agent 修）。

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use auto_ai_agent::{Tool, ToolError, ToolOutput};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::tool_context::ToolContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Warning,
    Advice,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Warning => write!(f, "WARNING"),
            Severity::Advice => write!(f, "ADVICE"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintDiagnostic {
    pub rule_id: &'static str,
    pub severity: Severity,
    pub line: usize,
    pub snippet: String,
    pub message: &'static str,
    pub prescription: &'static str,
}

/// 规则 L001-L008 定义与扫描器
pub fn lint_at_source(source: &str) -> Vec<LintDiagnostic> {
    let mut diags = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    // 括号跟踪（L008）
    let mut paren_stack: Vec<(char, usize)> = Vec::new();

    // 块上下文跟踪（L002 computed 块, L004 msg/on 块）
    let mut in_computed_block = false;
    let mut computed_brace_depth = 0;
    let mut in_msg_block = false;
    let mut msg_brace_depth = 0;
    let mut in_on_block = false;
    let mut on_brace_depth = 0;

    for (idx, line_raw) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let line = line_raw.trim();

        // 剔除单行注释
        let code_part = if let Some(pos) = line.find("//") {
            line[..pos].trim()
        } else {
            line
        };

        if code_part.is_empty() {
            continue;
        }

        let is_computed_line = in_computed_block
            || code_part.contains("computed ")
            || code_part.contains("computed{")
            || code_part == "computed";
        let is_msg_line = in_msg_block
            || code_part.contains("msg ")
            || code_part.contains("msg{")
            || code_part == "msg";
        let is_on_line = in_on_block
            || code_part.contains("on ")
            || code_part.contains("on{")
            || code_part == "on";

        // L001: span 挂 onclick
        // 匹配 `span` 出现且含有 `onclick:`
        if (code_part.contains("span ") || code_part.starts_with("span{") || code_part == "span")
            && code_part.contains("onclick:")
        {
            diags.push(LintDiagnostic {
                rule_id: "L001",
                severity: Severity::Warning,
                line: line_num,
                snippet: line.to_string(),
                message: "span 挂载 onclick：在升格为 button 时会丢失非文本子节点",
                prescription: "行内可点元素一律显式使用 `button`，不要在 `span` 上挂 onclick",
            });
        }

        // L002: computed 块内调用 use.web 声明的函数或副作用函数
        if is_computed_line
            && (code_part.contains("_web(")
                || code_part.contains("web.")
                || code_part.contains("fetch(")
                || code_part.contains("fetch_web(")
                || code_part.contains("window.")
                || code_part.contains("document."))
        {
            diags.push(LintDiagnostic {
                rule_id: "L002",
                severity: Severity::Warning,
                line: line_num,
                snippet: line.to_string(),
                message: "computed 块内调用 web port/副作用函数",
                prescription: "将副作用/外部函数调用移至 handler/msg 域，computed 只用于同步读取 state",
            });
        }

        // L003: t( 非字面量键
        if let Some(pos) = code_part.find("t(") {
            let after_call = code_part[pos + 2..].trim_start();
            if !after_call.starts_with('"')
                && !after_call.starts_with('\'')
                && !after_call.starts_with(')')
            {
                diags.push(LintDiagnostic {
                    rule_id: "L003",
                    severity: Severity::Warning,
                    line: line_num,
                    snippet: line.to_string(),
                    message: "t(...) 传入非字面量键：动态键在 VM 模式查表失明",
                    prescription: "将国际化键字面量化（例如 `t(\"key\")`），禁止传入动态变量",
                });
            }
        }

        // L004: handler/msg 名含下划线
        if is_msg_line || is_on_line {
            let tokens: Vec<&str> = code_part
                .split(|c: char| {
                    c.is_whitespace()
                        || c == '{'
                        || c == '}'
                        || c == '('
                        || c == ')'
                        || c == ','
                        || c == '-'
                        || c == '>'
                        || c == ':'
                })
                .filter(|s| !s.is_empty())
                .collect();
            for token in tokens {
                let id = token.trim_start_matches('.');
                if id != "msg"
                    && id != "on"
                    && id.contains('_')
                    && !id.starts_with("__")
                    && id.chars().next().map_or(false, |c| c.is_alphabetic())
                {
                    diags.push(LintDiagnostic {
                        rule_id: "L004",
                        severity: Severity::Advice,
                        line: line_num,
                        snippet: line.to_string(),
                        message: "msg 或 handler 名称中包含下划线",
                        prescription: "使用驼峰命名法（PascalCase 或 camelCase），避免下划线",
                    });
                    break;
                }
            }
        }

        // L005: .length 出现在表达式中（静默返空）
        if code_part.contains(".length") {
            diags.push(LintDiagnostic {
                rule_id: "L005",
                severity: Severity::Warning,
                line: line_num,
                snippet: line.to_string(),
                message: ".length 出现在表达式中：Auto 语言列表求长应为 .len()，.length 会静默返空",
                prescription: "改为 `.len()`，或在 handler 域计算好拍平标量",
            });
        }

        // L006: list.join( 链接期 Undefined symbol
        if code_part.contains(".join(") {
            diags.push(LintDiagnostic {
                rule_id: "L006",
                severity: Severity::Warning,
                line: line_num,
                snippet: line.to_string(),
                message: "list.join(...) 在某些后端链接期产生 Undefined symbol",
                prescription: "改用循环手拼字符串，或在 handler 域进行字符串拼接处理",
            });
        }

        // L007: style: 中出现 CSS 声明语法（带分号）或 style { 块
        if code_part.contains("style {") || code_part.contains("style{") {
            diags.push(LintDiagnostic {
                rule_id: "L007",
                severity: Severity::Warning,
                line: line_num,
                snippet: line.to_string(),
                message: "出现 `style {` 块语法：AutoUI 样式采用 `style: \"tailwind-classes\"`",
                prescription: "使用 Tailwind 类名串 `style: \"...\"`，移除 CSS 块声明",
            });
        } else if let Some(pos) = code_part.find("style:") {
            let style_val = code_part[pos + 6..].trim();
            if style_val.contains(';') {
                diags.push(LintDiagnostic {
                    rule_id: "L007",
                    severity: Severity::Warning,
                    line: line_num,
                    snippet: line.to_string(),
                    message: "style 字符串包含 CSS 分号声明（例如 `display: flex;`）",
                    prescription: "AutoUI style 属性接收 Tailwind 工具类串（例如 `style: \"flex items-center\"`），勿写原生 CSS 语法",
                });
            }
        }

        // L008: 括号未配对检查
        for c in code_part.chars() {
            match c {
                '(' | '{' | '[' => paren_stack.push((c, line_num)),
                ')' => {
                    if let Some((top, _)) = paren_stack.pop() {
                        if top != '(' {
                            diags.push(LintDiagnostic {
                                rule_id: "L008",
                                severity: Severity::Warning,
                                line: line_num,
                                snippet: line.to_string(),
                                message: "括号不匹配：遇到 ')' 但栈顶不为 '('",
                                prescription: "检查并闭合对应的表达式括号",
                            });
                        }
                    } else {
                        diags.push(LintDiagnostic {
                            rule_id: "L008",
                            severity: Severity::Warning,
                            line: line_num,
                            snippet: line.to_string(),
                            message: "多余的闭括号 ')'",
                            prescription: "检查表达式闭合",
                        });
                    }
                }
                '}' => {
                    if let Some((top, _)) = paren_stack.pop() {
                        if top != '{' {
                            diags.push(LintDiagnostic {
                                rule_id: "L008",
                                severity: Severity::Warning,
                                line: line_num,
                                snippet: line.to_string(),
                                message: "花括号不匹配：遇到 '}' 但栈顶不为 '{'",
                                prescription: "检查并闭合对应的块花括号",
                            });
                        }
                    } else {
                        diags.push(LintDiagnostic {
                            rule_id: "L008",
                            severity: Severity::Warning,
                            line: line_num,
                            snippet: line.to_string(),
                            message: "多余的闭花括号 '}'",
                            prescription: "检查块闭合",
                        });
                    }
                }
                ']' => {
                    if let Some((top, _)) = paren_stack.pop() {
                        if top != '[' {
                            diags.push(LintDiagnostic {
                                rule_id: "L008",
                                severity: Severity::Warning,
                                line: line_num,
                                snippet: line.to_string(),
                                message: "方括号不匹配：遇到 ']' 但栈顶不为 '['",
                                prescription: "检查并闭合对应的数组方括号",
                            });
                        }
                    } else {
                        diags.push(LintDiagnostic {
                            rule_id: "L008",
                            severity: Severity::Warning,
                            line: line_num,
                            snippet: line.to_string(),
                            message: "多余的闭方括号 ']'",
                            prescription: "检查列表方括号闭合",
                        });
                    }
                }
                _ => {}
            }
        }

        let open_count = code_part.chars().filter(|&c| c == '{').count() as i32;
        let close_count = code_part.chars().filter(|&c| c == '}').count() as i32;

        if code_part.contains("computed ") || code_part.contains("computed{") || code_part == "computed" {
            in_computed_block = true;
        }
        if in_computed_block {
            computed_brace_depth += open_count - close_count;
            if computed_brace_depth <= 0 {
                in_computed_block = false;
                computed_brace_depth = 0;
            }
        }

        if code_part.contains("msg ") || code_part.contains("msg{") || code_part == "msg" {
            in_msg_block = true;
        }
        if in_msg_block {
            msg_brace_depth += open_count - close_count;
            if msg_brace_depth <= 0 {
                in_msg_block = false;
                msg_brace_depth = 0;
            }
        }

        if code_part.contains("on ") || code_part.contains("on{") || code_part == "on" {
            in_on_block = true;
        }
        if in_on_block {
            on_brace_depth += open_count - close_count;
            if on_brace_depth <= 0 {
                in_on_block = false;
                on_brace_depth = 0;
            }
        }
    }

    // L008: 检查未闭合的左括号
    if let Some((unclosed, unclosed_line)) = paren_stack.pop() {
        diags.push(LintDiagnostic {
            rule_id: "L008",
            severity: Severity::Warning,
            line: unclosed_line,
            snippet: format!("unclosed '{unclosed}'"),
            message: "文件结束时存在未闭合的括号/花括号",
            prescription: "检查并补齐未闭合的括号或花括号",
        });
    }

    diags
}

/// 格式化为 advisory 诊断报告
pub fn format_lint_report(path_str: &str, diags: &[LintDiagnostic]) -> String {
    if diags.is_empty() {
        return format!(
            "# UI Lint Advisory Report for '{path_str}'\n\
             Status: CLEAN (0 findings)\n\
             No known pitfalls detected across 8 lint rules (L001-L008).\n"
        );
    }

    let mut out = format!(
        "# UI Lint Advisory Report for '{path_str}' ({} finding{})\n\
         > [!NOTE]\n\
         > This is an advisory lint fast-gate. Warnings highlight known quirks and traps.\n\
         > Review and fix these recommendations before running bp_check / canvas_run.\n\n",
        diags.len(),
        if diags.len() > 1 { "s" } else { "" }
    );

    for d in diags {
        out.push_str(&format!(
            "- **[{}] [{}]** Line {}: `{}`\n  - Issue: {}\n  - Prescription: {}\n",
            d.rule_id, d.severity, d.line, d.snippet.trim(), d.message, d.prescription
        ));
    }

    out
}

// ── ui_lint Tool ────────────────────────────────────────────────────────────

pub struct UiLint {
    ctx: ToolContext,
}

impl UiLint {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for UiLint {
    fn name(&self) -> &str {
        "ui_lint"
    }

    fn description(&self) -> &str {
        "Advisory static linter for AutoUI .at files. Checks for 8 known traps (L001-L008) \
         such as span onclick, .length, CSS syntax in style, dynamic translation keys, \
         and bracket mismatches. Returns actionable prescriptions for fast self-repair."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "path to the .at file to check, relative to workspace root"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let path_str = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'path' argument".into()))?;

        let resolved = super::tools::resolve_within_sandbox(&self.ctx, path_str)?;
        if !resolved.exists() {
            return Err(ToolError::Args(format!(
                "target file does not exist: {path_str} (resolved: {})",
                resolved.display()
            )));
        }

        let content = std::fs::read_to_string(&resolved).map_err(|e| {
            ToolError::Exec(format!("failed to read file '{}': {e}", resolved.display()))
        })?;

        let diags = lint_at_source(&content);
        let report = format_lint_report(path_str, &diags);

        Ok(ToolOutput::text(report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 干净样例：002-counter 源内容
    const CLEAN_COUNTER: &str = r#"widget App {
    model {
        var count int = 0
    }
    view {
        col {
            text `Counter: ${.count}`
            row {
                button "-" { onclick: () => {.count -= 1} }
                button "Reset" { onclick: () => {.count = 0} }
                button "+" { onclick: () => {.count += 1} }
            }
            style: "items-center gap-4 p-6"
        }
    }
}
"#;

    #[test]
    fn test_clean_counter_has_zero_findings() {
        let diags = lint_at_source(CLEAN_COUNTER);
        assert!(diags.is_empty(), "clean counter should have 0 diags, got: {:?}", diags);
        let report = format_lint_report("002-counter/app.at", &diags);
        assert!(report.contains("Status: CLEAN (0 findings)"));
    }

    #[test]
    fn test_rule_l001_span_onclick() {
        let red = "widget A { view { span { text: \"click\"; onclick: .Click } } }";
        let green = "widget A { view { button \"click\" { onclick: .Click } } }";
        let d_red = lint_at_source(red);
        assert!(d_red.iter().any(|d| d.rule_id == "L001"));
        let d_green = lint_at_source(green);
        assert!(!d_green.iter().any(|d| d.rule_id == "L001"));
    }

    #[test]
    fn test_rule_l002_computed_web_call() {
        let red = "widget A { computed { var title str = fetch_web(\"url\") } view { col {} } }";
        let green = "widget A { computed { var title str = .state_title } view { col {} } }";
        let d_red = lint_at_source(red);
        assert!(d_red.iter().any(|d| d.rule_id == "L002"));
        let d_green = lint_at_source(green);
        assert!(!d_green.iter().any(|d| d.rule_id == "L002"));
    }

    #[test]
    fn test_rule_l003_dynamic_t_key() {
        let red = "widget A { view { text t(.dynamic_key) } }";
        let green = "widget A { view { text t(\"static.key\") } }";
        let d_red = lint_at_source(red);
        assert!(d_red.iter().any(|d| d.rule_id == "L003"));
        let d_green = lint_at_source(green);
        assert!(!d_green.iter().any(|d| d.rule_id == "L003"));
    }

    #[test]
    fn test_rule_l004_underscore_msg() {
        let red = "widget A { msg { Click_Button } view { col {} } }";
        let green = "widget A { msg { ClickButton } view { col {} } }";
        let d_red = lint_at_source(red);
        assert!(d_red.iter().any(|d| d.rule_id == "L004"));
        let d_green = lint_at_source(green);
        assert!(!d_green.iter().any(|d| d.rule_id == "L004"));
    }

    #[test]
    fn test_rule_l005_dot_length() {
        let red = "widget A { view { text f\"count: ${.items.length}\" } }";
        let green = "widget A { view { text f\"count: ${.items.len()}\" } }";
        let d_red = lint_at_source(red);
        assert!(d_red.iter().any(|d| d.rule_id == "L005"));
        let d_green = lint_at_source(green);
        assert!(!d_green.iter().any(|d| d.rule_id == "L005"));
    }

    #[test]
    fn test_rule_l006_list_join() {
        let red = "widget A { view { text .tags.join(\", \") } }";
        let green = "widget A { view { for t in .tags { text t } } }";
        let d_red = lint_at_source(red);
        assert!(d_red.iter().any(|d| d.rule_id == "L006"));
        let d_green = lint_at_source(green);
        assert!(!d_green.iter().any(|d| d.rule_id == "L006"));
    }

    #[test]
    fn test_rule_l007_css_style_syntax() {
        let red1 = "widget A { view { col { style: \"display: flex; color: red;\" } } }";
        let red2 = "widget A { view { col { style { color: red } } } }";
        let green = "widget A { view { col { style: \"flex text-red-500\" } } }";
        assert!(lint_at_source(red1).iter().any(|d| d.rule_id == "L007"));
        assert!(lint_at_source(red2).iter().any(|d| d.rule_id == "L007"));
        assert!(!lint_at_source(green).iter().any(|d| d.rule_id == "L007"));
    }

    #[test]
    fn test_rule_l008_unbalanced_brackets() {
        let red = "widget A { view { col { text \"hi\" } }"; // 缺少一个 }
        let green = "widget A { view { col { text \"hi\" } } }";
        assert!(lint_at_source(red).iter().any(|d| d.rule_id == "L008"));
        assert!(!lint_at_source(green).iter().any(|d| d.rule_id == "L008"));
    }
}
