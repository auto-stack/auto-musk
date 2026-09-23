//! PLAN-090 T-02: bp 通道三工具 —— bp_list / bp_show / bp_check。
//!
//! 连接 auto-lang 既有 `auto bp` CLI（spec 查看 + 静态行为验收门）。
//! 当 auto.exe 或 blueprints/ 目录不可达时，提供明确的 [DEGRADED] 降级报文，不 panic。

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use auto_ai_agent::{Tool, ToolError, ToolOutput};
use serde_json::{json, Value};

use crate::tool_context::ToolContext;

pub const BP_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
pub const BP_SHOW_MAX_BYTES: usize = 16 * 1024; // 16KB 截断护栏

/// 解析 auto-lang 的 blueprints/ 目录位置：
/// 1. `AUTO_BLUEPRINTS_ROOT` 环境变量覆盖
/// 2. 编译期兄弟位 `../../../../auto-lang/blueprints`
/// 3. 主检出位置 `D:/autostack/auto-lang/blueprints`
pub fn resolve_blueprints_dir() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("AUTO_BLUEPRINTS_ROOT") {
        if !p.trim().is_empty() {
            let path = PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }
    }
    let sibling = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../auto-lang/blueprints");
    if sibling.exists() {
        return Some(sibling);
    }
    let main_checkout = PathBuf::from("D:/autostack/auto-lang/blueprints");
    if main_checkout.exists() {
        return Some(main_checkout);
    }
    None
}

/// 执行 `auto bp <subcommand> [args...]` 子进程
pub async fn run_bp_command(
    args: &[&str],
    cwd: Option<&Path>,
) -> Result<(std::process::ExitStatus, String, String), String> {
    let exe = super::session::resolve_auto_exe().ok_or_else(|| {
        "auto executable not found: set AUTO_EXE or build auto-lang (target/release/auto.exe)"
            .to_string()
    })?;

    let mut cmd = tokio::process::Command::new(&exe);
    cmd.arg("bp").args(args);

    if let Some(c) = cwd {
        cmd.current_dir(c);
    }

    // 若未显式设置 AUTO_BLUEPRINTS_ROOT，注入解析出的 blueprints/ 路径
    if std::env::var_os("AUTO_BLUEPRINTS_ROOT").is_none() {
        if let Some(bp_dir) = resolve_blueprints_dir() {
            cmd.env("AUTO_BLUEPRINTS_ROOT", bp_dir);
        }
    }

    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let output = tokio::time::timeout(BP_COMMAND_TIMEOUT, cmd.output())
        .await
        .map_err(|_| format!("command timed out after {}s", BP_COMMAND_TIMEOUT.as_secs()))?
        .map_err(|e| format!("failed to execute {}: {e}", exe.display()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok((output.status, stdout, stderr))
}

// ── bp_list ─────────────────────────────────────────────────────────────────

pub struct BpList {
    #[allow(dead_code)]
    ctx: ToolContext,
}

impl BpList {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

pub fn parse_bp_list_text(raw: &str) -> Vec<(String, Vec<String>)> {
    let mut categories: Vec<(String, Vec<String>)> = Vec::new();
    let mut current_kind = String::new();
    let mut current_items = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(kind) = trimmed.strip_prefix('#') {
            if !current_kind.is_empty() {
                categories.push((current_kind, current_items));
                current_items = Vec::new();
            }
            current_kind = kind.trim().to_string();
        } else if trimmed.contains('/') {
            current_items.push(trimmed.to_string());
        }
    }
    if !current_kind.is_empty() {
        categories.push((current_kind, current_items));
    }
    categories
}

#[async_trait]
impl Tool for BpList {
    fn name(&self) -> &str {
        "bp_list"
    }

    fn description(&self) -> &str {
        "List available AutoUI blueprints from the catalog, grouped by category (kind). \
         Returns blueprint kind, name, and keys. When blueprints are unavailable, \
         returns a graceful [DEGRADED] report with instructions."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: &Value) -> Result<ToolOutput, ToolError> {
        let run_res = run_bp_command(&["list"], None).await;
        match run_res {
            Err(e) => Ok(ToolOutput::text(format!(
                "[DEGRADED] Auto CLI or blueprints unavailable: {e}.\n\
                 Ensure auto-lang is built (`target/release/auto.exe`) or set AUTO_EXE/AUTO_BLUEPRINTS_ROOT."
            ))),
            Ok((status, stdout, stderr)) => {
                if !status.success() || stdout.contains("(no blueprints found") {
                    return Ok(ToolOutput::text(format!(
                        "[DEGRADED] No blueprints found under blueprints/ directory.\n\
                         Details: {stdout} {stderr}\n\
                         Please verify auto-lang checkout or configure AUTO_BLUEPRINTS_ROOT."
                    )));
                }

                let categories = parse_bp_list_text(&stdout);
                let total_count: usize = categories.iter().map(|(_, items)| items.len()).sum();

                let mut out = format!(
                    "# AutoUI Blueprint Catalog ({} packages across {} categories)\n\n\
                     | Category | Blueprints | Available Keys |\n\
                     |---|---|---|\n",
                    total_count,
                    categories.len()
                );

                for (kind, items) in &categories {
                    let names = items
                        .iter()
                        .map(|it| it.split('/').last().unwrap_or(it))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let keys = items.join(", ");
                    out.push_str(&format!("| **{kind}** | {names} | `{keys}` |\n"));
                }

                out.push_str(
                    "\n> Use `bp_show` with `{ \"name\": \"<kind>/<name>\" }` to view \
                     the full specification, extension points, variants, and gotchas.\n"
                );

                Ok(ToolOutput::text(out))
            }
        }
    }
}

// ── bp_show ─────────────────────────────────────────────────────────────────

pub struct BpShow {
    #[allow(dead_code)]
    ctx: ToolContext,
}

impl BpShow {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

pub fn resolve_bp_key(kind: Option<&str>, name: &str) -> String {
    let n = name.trim();
    if n.contains('/') {
        return n.to_string();
    }
    if let Some(k) = kind {
        let kt = k.trim();
        if !kt.is_empty() {
            return format!("{kt}/{n}");
        }
    }
    n.to_string()
}

#[async_trait]
impl Tool for BpShow {
    fn name(&self) -> &str {
        "bp_show"
    }

    fn description(&self) -> &str {
        "Show the full specification, extension points, variants, and gotchas for a \
         specified blueprint (e.g. name: 'data-display/note-list' or kind: 'data-display', name: 'note-list'). \
         Provides design contracts, data source wiring, and pitfalls before generating or modifying code."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "kind": {
                    "type": "string",
                    "description": "optional blueprint category (e.g. 'data-display', 'form', 'navigation')"
                },
                "name": {
                    "type": "string",
                    "description": "blueprint name (e.g. 'note-list', 'login') or full key ('data-display/note-list')"
                }
            },
            "required": ["name"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let name = args["name"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'name' argument".into()))?;
        let kind = args["kind"].as_str();

        let key = resolve_bp_key(kind, name);

        let run_res = run_bp_command(&["show", &key], None).await;
        match run_res {
            Err(e) => Ok(ToolOutput::text(format!(
                "[DEGRADED] Auto CLI or blueprint '{key}' unavailable: {e}"
            ))),
            Ok((status, stdout, stderr)) => {
                if !status.success() {
                    return Ok(ToolOutput::text(format!(
                        "Blueprint '{key}' not found or auto bp show failed (status: {:?}):\n{}\n{}\n\
                         Tip: Run `bp_list` to see valid blueprint keys.",
                        status.code(),
                        stdout,
                        stderr
                    )));
                }

                let mut text = stdout;
                if text.len() > BP_SHOW_MAX_BYTES {
                    let truncated: String = text.chars().take(BP_SHOW_MAX_BYTES).collect();
                    text = format!(
                        "{truncated}\n\n\
                         [TRUNCATED: Blueprint output exceeded 16KB limit. Inspect blueprints/{key}/spec.md directly for more.]"
                    );
                }

                Ok(ToolOutput::text(text))
            }
        }
    }
}

// ── bp_check ────────────────────────────────────────────────────────────────

pub struct BpCheck {
    ctx: ToolContext,
}

impl BpCheck {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for BpCheck {
    fn name(&self) -> &str {
        "bp_check"
    }

    fn description(&self) -> &str {
        "Static acceptance check on an AutoUI .at file against blueprint behavior contract \
         (loading state, error state, and palette whitelist). Pass path to .at file relative \
         to workspace root, and optionally the blueprint spec key (e.g. 'data-display/note-list')."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "path to the .at file to check, relative to workspace root"
                },
                "spec": {
                    "type": "string",
                    "description": "optional blueprint specification key (e.g. 'data-display/note-list')"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<ToolOutput, ToolError> {
        let path_str = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'path' argument".into()))?;
        let spec_opt = args["spec"].as_str();

        let resolved = super::tools::resolve_within_sandbox(&self.ctx, path_str)?;
        if !resolved.exists() {
            return Err(ToolError::Args(format!(
                "target file does not exist: {path_str} (resolved: {})",
                resolved.display()
            )));
        }

        let resolved_path_str = resolved.to_string_lossy().to_string();
        let mut cmd_args = vec!["check", &resolved_path_str];
        if let Some(spec) = spec_opt {
            let s = spec.trim();
            if !s.is_empty() {
                cmd_args.push("--spec");
                cmd_args.push(s);
            }
        }

        let ws = self.ctx.state.registry.get(&self.ctx.workspace_id);
        let run_res = run_bp_command(&cmd_args, Some(&ws.root)).await;

        match run_res {
            Err(e) => Err(ToolError::Exec(format!("bp_check failed to run: {e}"))),
            Ok((status, stdout, stderr)) => {
                let combined = if stderr.trim().is_empty() {
                    stdout
                } else {
                    format!("{stdout}\n{stderr}")
                };

                if status.success() {
                    Ok(ToolOutput::text(format!(
                        "PASS: bp_check passed for '{}'\n\n{}",
                        path_str,
                        combined.trim()
                    )))
                } else {
                    Ok(ToolOutput::text(format!(
                        "FAIL: bp_check failed for '{}' (exit code: {:?})\n\n{}\n\
                         Please fix the behavior contract requirements (loading/error branches or palette).",
                        path_str,
                        status.code(),
                        combined.trim()
                    )))
                }
            }
        }
    }
}

// ── Blueprint Catalog Summary (for System Prompt Injection) ─────────────────

/// 紧凑的 Blueprint 目录摘要，用于 coding 模式系统提示注入（≤1.5KB）
pub fn get_blueprint_catalog_summary() -> &'static str {
    "### Blueprint Catalog (18 packages across 7 kinds)\n\
     - dashboard: overview (metrics summary, stat cards)\n\
     - data-display: data-table-crud, master-detail, note-list, row-list\n\
     - editor: note-editor (rich markdown/text editor)\n\
     - feedback: empty-state, result-page\n\
     - form: login, settings, signup, wizard\n\
     - layout: gallery-shell, sandwich, status-bar\n\
     - navigation: filetree, sidebar-nav, sidebar-shell\n\
     Tip: Call `bp_show` with kind/name to inspect the contract & gotchas before writing."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bp_list_text() {
        let sample = "\
# dashboard
  dashboard/overview

# data-display
  data-display/data-table-crud
  data-display/master-detail
  data-display/note-list
  data-display/row-list

# form
  form/login
  form/settings
";
        let parsed = parse_bp_list_text(sample);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].0, "dashboard");
        assert_eq!(parsed[0].1, vec!["dashboard/overview"]);
        assert_eq!(parsed[1].0, "data-display");
        assert_eq!(parsed[1].1.len(), 4);
        assert_eq!(parsed[2].0, "form");
        assert_eq!(parsed[2].1.len(), 2);
    }

    #[test]
    fn test_resolve_bp_key() {
        assert_eq!(
            resolve_bp_key(Some("data-display"), "note-list"),
            "data-display/note-list"
        );
        assert_eq!(
            resolve_bp_key(None, "data-display/note-list"),
            "data-display/note-list"
        );
        assert_eq!(
            resolve_bp_key(Some("form"), "data-display/note-list"),
            "data-display/note-list"
        );
        assert_eq!(resolve_bp_key(None, "login"), "login");
    }

    #[test]
    fn test_catalog_summary_budget() {
        let summary = get_blueprint_catalog_summary();
        assert!(summary.len() > 100);
        assert!(summary.len() < 1500, "summary size should be under 1.5KB");
        assert!(summary.contains("dashboard"));
        assert!(summary.contains("data-display"));
        assert!(summary.contains("form"));
    }
}
