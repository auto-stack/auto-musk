//! PLAN-090 T-05: examples 扩展池（后端只读面，兄弟位解析 + 截断护栏）。
//!
//! 提供 `app_examples_list` 与 `app_example_read` 两个工具，
//! 供 agent 查阅 auto-lang 既有 34 个真实 UI 应用示例（pac.at + src/front/app.at）。
//! 只读消费，不向沙箱暴露写权限；不可达时返回 [DEGRADED]。

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use auto_ai_agent::{Tool, ToolError, ToolOutput};
use serde_json::{json, Value};

use crate::tool_context::ToolContext;

pub const EXAMPLE_READ_MAX_BYTES: usize = 32 * 1024; // 32KB 截断护栏

/// 解析 auto-lang 的 examples/ui 目录位置
pub fn resolve_examples_ui_dir() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("AUTO_EXAMPLES_ROOT") {
        if !p.trim().is_empty() {
            let path = PathBuf::from(p);
            if path.is_dir() {
                return Some(path);
            }
            // 显式设置但目录不存在时，不静默回退，由调用方显式降级
            return None;
        }
    }
    if let Some(root) = super::vocabulary::resolve_auto_lang_root() {
        let candidate = root.join("examples/ui");
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    let main_checkout = PathBuf::from("D:/autostack/auto-lang/examples/ui");
    if main_checkout.is_dir() {
        return Some(main_checkout);
    }
    None
}

#[derive(Debug, Clone)]
pub struct ExampleEntry {
    pub id: String,
    pub name: String,
    pub dir_name: String,
    pub summary: String,
}

/// 解析 examples/ui/README.md 中的 Markdown 表格或扫描目录
pub fn load_examples_catalog(examples_dir: &Path) -> Vec<ExampleEntry> {
    let mut entries = Vec::new();
    let readme_path = examples_dir.join("README.md");

    if let Ok(content) = std::fs::read_to_string(&readme_path) {
        let mut in_table = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("| 编号 |") || trimmed.starts_with("|编号|") {
                in_table = true;
                continue;
            }
            if in_table {
                if !trimmed.starts_with('|') || trimmed.starts_with("|---") {
                    if !trimmed.starts_with('|') {
                        in_table = false;
                    }
                    continue;
                }
                let cols: Vec<&str> = trimmed
                    .split('|')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                if cols.len() >= 3 {
                    let id = cols[0].to_string();
                    let name = cols[1].to_string();
                    let summary = cols[2].to_string();
                    let dir_name = format!("{}-{}", id, name.split('（').next().unwrap_or(&name).trim());
                    entries.push(ExampleEntry {
                        id,
                        name,
                        dir_name,
                        summary,
                    });
                }
            }
        }
    }

    // 若 README 表格为空，兜底直接扫描子目录
    if entries.is_empty() {
        if let Ok(read_dir) = std::fs::read_dir(examples_dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                        let id = name.chars().take_while(|c| c.is_ascii_digit()).collect();
                        entries.push(ExampleEntry {
                            id,
                            name: name.clone(),
                            dir_name: name,
                            summary: "AutoUI example app".to_string(),
                        });
                    }
                }
            }
            entries.sort_by(|a, b| a.id.cmp(&b.id));
        }
    }

    entries
}

// ── app_examples_list ───────────────────────────────────────────────────────

pub struct AppExamplesList {
    #[allow(dead_code)]
    ctx: ToolContext,
}

impl AppExamplesList {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for AppExamplesList {
    fn name(&self) -> &str {
        "app_examples_list"
    }

    fn description(&self) -> &str {
        "List curated AutoUI reference example apps from auto-lang (examples/ui), \
         including demo id, name, directory name, and brief summary. \
         Useful for finding production-tested reference implementations."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: &Value) -> Result<ToolOutput, ToolError> {
        let examples_dir = match resolve_examples_ui_dir() {
            Some(d) => d,
            None => {
                return Ok(ToolOutput::text(
                    "[DEGRADED] AutoUI examples pool not found. \
                     Verify auto-lang checkout or set AUTO_EXAMPLES_ROOT to auto-lang/examples/ui."
                        .to_string(),
                ));
            }
        };

        let entries = load_examples_catalog(&examples_dir);
        if entries.is_empty() {
            return Ok(ToolOutput::text(
                "[DEGRADED] No examples found in examples/ui. Check directory structure.".to_string(),
            ));
        }

        let mut out = format!(
            "# AutoUI Examples Pool ({} curated applications)\n\n\
             | ID | Name | Summary | Directory |\n\
             |---|---|---|---|\n",
            entries.len()
        );

        for e in &entries {
            out.push_str(&format!(
                "| **{}** | {} | {} | `{}` |\n",
                e.id, e.name, e.summary, e.dir_name
            ));
        }

        out.push_str(
            "\n> Use `app_example_read` with `{ \"name\": \"<id-or-name>\" }` to read `pac.at` and `src/front/app.at`.\n"
        );

        Ok(ToolOutput::text(out))
    }
}

// ── app_example_read ────────────────────────────────────────────────────────

pub struct AppExampleRead {
    #[allow(dead_code)]
    ctx: ToolContext,
}

impl AppExampleRead {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

pub fn find_example_dir(base: &Path, name_query: &str) -> Option<PathBuf> {
    let q = name_query.trim().to_lowercase();
    if let Ok(entries) = std::fs::read_dir(base) {
        let mut candidates = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dirname = entry.file_name().to_string_lossy().to_string();
                let lower = dirname.to_lowercase();
                if lower == q || lower.starts_with(&format!("{q}-")) || lower.ends_with(&format!("-{q}")) || lower.contains(&q) {
                    candidates.push((dirname, path));
                }
            }
        }
        // 优先精确匹配
        if let Some((_, p)) = candidates.iter().find(|(name, _)| name.to_lowercase() == q) {
            return Some(p.clone());
        }
        // 其次匹配前缀（例如 "002" -> "002-counter"）
        if let Some((_, p)) = candidates.iter().find(|(name, _)| name.to_lowercase().starts_with(&format!("{q}-"))) {
            return Some(p.clone());
        }
        // 返回第一个候选
        if let Some((_, p)) = candidates.first() {
            return Some(p.clone());
        }
    }
    None
}

#[async_trait]
impl Tool for AppExampleRead {
    fn name(&self) -> &str {
        "app_example_read"
    }

    fn description(&self) -> &str {
        "Read source files from an AutoUI example application in examples/ui \
         (e.g. name: '002-counter', '002', or 'counter'). Defaults to returning \
         both pac.at and src/front/app.at, or a specific file if specified."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "demo name or prefix (e.g. '002-counter', '002', 'counter', '015-notes')"
                },
                "file": {
                    "type": "string",
                    "description": "optional specific file path within demo (e.g. 'src/front/app.at', 'pac.at')"
                }
            },
            "required": ["name"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let name_query = args["name"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'name' argument".into()))?;
        let file_opt = args["file"].as_str();

        let examples_dir = match resolve_examples_ui_dir() {
            Some(d) => d,
            None => {
                return Ok(ToolOutput::text(
                    "[DEGRADED] AutoUI examples pool not found. \
                     Verify auto-lang checkout or set AUTO_EXAMPLES_ROOT."
                        .to_string(),
                ));
            }
        };

        let demo_dir = match find_example_dir(&examples_dir, name_query) {
            Some(d) => d,
            None => {
                return Ok(ToolOutput::text(format!(
                    "Example demo matching '{name_query}' not found under {}.\n\
                     Tip: Run `app_examples_list` to see available demos.",
                    examples_dir.display()
                )));
            }
        };

        let demo_dirname = demo_dir.file_name().unwrap_or_default().to_string_lossy();

        if let Some(req_file) = file_opt {
            let req_clean = req_file.trim().replace('\\', "/");
            if req_clean.contains("..") || req_clean.starts_with('/') {
                return Err(ToolError::Args("path traversal '..' or leading '/' not allowed in file".into()));
            }

            let target_file = demo_dir.join(&req_clean);
            if !target_file.is_file() {
                return Ok(ToolOutput::text(format!(
                    "File '{}' does not exist in demo '{}'.",
                    req_clean, demo_dirname
                )));
            }

            let content = std::fs::read_to_string(&target_file).map_err(|e| {
                ToolError::Exec(format!("failed to read file '{}': {e}", target_file.display()))
            })?;

            let mut out = format!("### File: {}/{}\n```auto\n{}\n```\n", demo_dirname, req_clean, content);
            if out.len() > EXAMPLE_READ_MAX_BYTES {
                let tr: String = out.chars().take(EXAMPLE_READ_MAX_BYTES).collect();
                out = format!("{tr}\n\n[TRUNCATED: Exceeded 32KB limit]");
            }
            return Ok(ToolOutput::text(out));
        }

        // 默认返回 pac.at + src/front/app.at
        let pac_path = demo_dir.join("pac.at");
        let app_path = demo_dir.join("src/front/app.at");

        let mut out = format!("# Example Demo: {}\n\n", demo_dirname);

        if pac_path.is_file() {
            if let Ok(pac_content) = std::fs::read_to_string(&pac_path) {
                out.push_str(&format!("## pac.at\n```auto\n{}\n```\n\n", pac_content.trim()));
            }
        }

        if app_path.is_file() {
            if let Ok(app_content) = std::fs::read_to_string(&app_path) {
                out.push_str(&format!("## src/front/app.at\n```auto\n{}\n```\n", app_content.trim()));
            }
        } else {
            out.push_str("*Note: `src/front/app.at` not found in this demo directory.*\n");
        }

        if out.len() > EXAMPLE_READ_MAX_BYTES {
            let tr: String = out.chars().take(EXAMPLE_READ_MAX_BYTES).collect();
            out = format!("{tr}\n\n[TRUNCATED: Output exceeded 32KB limit]");
        }

        Ok(ToolOutput::text(out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_examples_ui_dir_exists() {
        if let Some(dir) = resolve_examples_ui_dir() {
            assert!(dir.is_dir(), "examples dir must be a directory");
            let entries = load_examples_catalog(&dir);
            assert!(entries.len() >= 30, "must catalog at least 30 demos, actual: {}", entries.len());
        }
    }

    #[test]
    fn test_find_example_dir() {
        if let Some(dir) = resolve_examples_ui_dir() {
            let found_exact = find_example_dir(&dir, "002-counter");
            assert!(found_exact.is_some());
            let found_num = find_example_dir(&dir, "002");
            assert!(found_num.is_some());
            let found_name = find_example_dir(&dir, "counter");
            assert!(found_name.is_some());
        }
    }
}
