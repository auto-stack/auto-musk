//! auto-musk 基础工具:agent 在本地执行的能力(不经 daemon)。
//!
//! 这三个工具(读文件 / 写文件 / 执行命令)是 agent 操作文件系统和运行
//! 命令的最小集。它们实现 [`auto_ai_agent::Tool`],由 agent 的 ReAct 循环
//! 在本地直接调用 —— LLM 通信才走 daemon,工具执行永远在 musk 进程内。

use async_trait::async_trait;
use auto_ai_agent::{Tool, ToolError};
use serde_json::{json, Value};

/// 读取文件内容(UTF-8 文本)。
pub struct ReadFile;

#[async_trait]
impl Tool for ReadFile {
    fn name(&self) -> &str {
        "read_file"
    }
    fn description(&self) -> &str {
        "Read the full UTF-8 text contents of a file at the given path."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "path to the file to read" }
            },
            "required": ["path"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'path' argument".into()))?;
        std::fs::read_to_string(path)
            .map_err(|e| ToolError::Exec(format!("read '{path}': {e}")))
    }
}

/// 写入文件(覆盖已存在文件;自动创建父目录)。
pub struct WriteFile;

#[async_trait]
impl Tool for WriteFile {
    fn name(&self) -> &str {
        "write_file"
    }
    fn description(&self) -> &str {
        "Write text content to a file, overwriting if it exists. Parent \
         directories are created automatically."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "path to the file to write" },
                "content": { "type": "string", "description": "text content to write" }
            },
            "required": ["path", "content"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'path' argument".into()))?;
        let content = args["content"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'content' argument".into()))?;

        // Auto-create parent directories (a small convenience over std::fs::write).
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ToolError::Exec(format!("create dirs for '{path}': {e}")))?;
            }
        }
        std::fs::write(path, content)
            .map_err(|e| ToolError::Exec(format!("write '{path}': {e}")))?;
        Ok(format!("wrote {} bytes to {}", content.len(), path))
    }
}

/// 执行一条 shell 命令,返回合并的 stdout+stderr。
///
/// MVP 直接执行(无白名单/确认);后续阶段加安全护栏。
pub struct RunCommand;

#[async_trait]
impl Tool for RunCommand {
    fn name(&self) -> &str {
        "run_command"
    }
    fn description(&self) -> &str {
        "Run a shell command and return its combined stdout and stderr."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "cmd": { "type": "string", "description": "the shell command to run" }
            },
            "required": ["cmd"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let cmd = args["cmd"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'cmd' argument".into()))?;

        let output = if cfg!(windows) {
            std::process::Command::new("cmd").args(["/C", cmd]).output()
        } else {
            std::process::Command::new("sh").args(["-c", cmd]).output()
        }
        .map_err(|e| ToolError::Exec(format!("spawn '{cmd}': {e}")))?;

        let mut result = String::new();
        if !output.stdout.is_empty() {
            result.push_str(&String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            if !result.is_empty() {
                result.push_str("\n[stderr]\n");
            }
            result.push_str(&String::from_utf8_lossy(&output.stderr));
        }
        if result.is_empty() {
            result.push_str("(no output)");
        }
        Ok(result)
    }
}

/// 精确字符串替换:把文件中 `old_string` 替换为 `new_string`。
/// 要求 `old_string` 在文件中唯一,否则报错(避免歧义替换)。
/// 比 WriteFile 安全(不覆盖整个文件),是 executing-plans 的核心工具。
pub struct EditFile;

#[async_trait]
impl Tool for EditFile {
    fn name(&self) -> &str {
        "edit_file"
    }
    fn description(&self) -> &str {
        "Replace a unique string in a file with a new string. The old_string \
         must appear exactly once in the file (ambiguous matches error). Use \
         this for targeted edits instead of rewriting the whole file."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "file to edit" },
                "old_string": { "type": "string", "description": "the exact text to find (must be unique)" },
                "new_string": { "type": "string", "description": "the replacement text" }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'path'".into()))?;
        let old_string = args["old_string"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'old_string'".into()))?;
        let new_string = args["new_string"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'new_string'".into()))?;

        let content = std::fs::read_to_string(path)
            .map_err(|e| ToolError::Exec(format!("read '{path}': {e}")))?;
        let count = content.matches(old_string).count();
        if count == 0 {
            return Err(ToolError::Exec(format!(
                "old_string not found in '{path}'"
            )));
        }
        if count > 1 {
            return Err(ToolError::Exec(format!(
                "old_string appears {count} times in '{path}'; it must be unique. \
                 Include more surrounding context to make it unique."
            )));
        }
        let new_content = content.replacen(old_string, new_string, 1);
        std::fs::write(path, &new_content)
            .map_err(|e| ToolError::Exec(format!("write '{path}': {e}")))?;
        Ok(format!("edited '{path}' (1 replacement)"))
    }
}

/// 内容搜索(grep/rg):在文件树里搜 pattern,返回匹配的行。
/// 用 rg(若可用)否则 fallback 到 grep -rn。
pub struct Search;

#[async_trait]
impl Tool for Search {
    fn name(&self) -> &str {
        "search"
    }
    fn description(&self) -> &str {
        "Search file contents for a pattern (regex). Returns matching lines \
         with file:line prefixes. Searches the current directory by default, \
         or a given path."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "regex pattern to search for" },
                "path": { "type": "string", "description": "directory or file to search (default: current dir)" }
            },
            "required": ["pattern"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let pattern = args["pattern"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'pattern'".into()))?;
        let path = args["path"].as_str().unwrap_or(".");

        // Prefer ripgrep if available (faster, respects .gitignore); else grep.
        let rg_available = std::process::Command::new("rg")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let output = if rg_available {
            std::process::Command::new("rg")
                .args(["-n", "--no-heading", "--max-filesize", "1M", pattern, path])
                .output()
        } else if cfg!(windows) {
            std::process::Command::new("cmd")
                .args(["/C", &format!("findstr /S /N /R \"{pattern}\" {path}\\*")]
        )
                .output()
        } else {
            std::process::Command::new("grep")
                .args(["-rn", "--include=*", pattern, path])
                .output()
        }
        .map_err(|e| ToolError::Exec(format!("spawn search: {e}")))?;

        let mut result = String::from_utf8_lossy(&output.stdout).to_string();
        if result.is_empty() {
            // No matches is a valid, non-error result.
            result.push_str("(no matches)");
        }
        // Cap output length to avoid flooding the context.
        const MAX_BYTES: usize = 8 * 1024;
        if result.len() > MAX_BYTES {
            result.truncate(MAX_BYTES);
            result.push_str("\n... (truncated, refine your pattern)");
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn read_file_reads_existing() {
        let t = ReadFile;
        // Cargo.toml lives at the workspace root, two levels up from the crate.
        let out = t
            .execute(&json!({"path": "../../Cargo.toml"}))
            .await
            .unwrap();
        assert!(out.contains("[workspace]"));
    }

    #[tokio::test]
    async fn read_file_missing_errors() {
        let t = ReadFile;
        let err = t.execute(&json!({"path": "definitely_nonexistent.xyz"})).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn read_file_missing_path_arg_errors() {
        let t = ReadFile;
        let err = t.execute(&json!({})).await.unwrap_err();
        assert!(matches!(err, ToolError::Args(_)));
    }

    #[tokio::test]
    async fn write_file_then_read_back() {
        let t_write = WriteFile;
        let t_read = ReadFile;
        let path = std::env::temp_dir().join("musk_tool_test_write.txt");
        let p = path.to_string_lossy().to_string();

        t_write
            .execute(&json!({"path": p, "content": "hello musk"}))
            .await
            .unwrap();
        let back = t_read.execute(&json!({"path": p})).await.unwrap();
        assert_eq!(back, "hello musk");
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn write_file_creates_parent_dirs() {
        let t = WriteFile;
        let dir = std::env::temp_dir().join("musk_tool_test_subdir");
        let path = dir.join("nested/deep/file.txt");
        let p = path.to_string_lossy().to_string();

        t.execute(&json!({"path": p, "content": "nested"}))
            .await
            .unwrap();
        assert!(path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn run_command_echo() {
        let t = RunCommand;
        // `echo` works on both Windows (cmd /C echo) and Unix (sh -c echo).
        let out = t.execute(&json!({"cmd": "echo musk_test_token"})).await.unwrap();
        assert!(out.contains("musk_test_token"));
    }

    #[tokio::test]
    async fn run_command_missing_cmd_arg_errors() {
        let t = RunCommand;
        let err = t.execute(&json!({})).await.unwrap_err();
        assert!(matches!(err, ToolError::Args(_)));
    }

    // ── EditFile ───────────────────────────────────────────────

    #[tokio::test]
    async fn edit_file_replaces_unique_match() {
        let path = std::env::temp_dir().join("musk_edit_test_unique.txt");
        std::fs::write(&path, "alpha\nbeta\ngamma\n").unwrap();
        let p = path.to_string_lossy().to_string();
        let out = EditFile
            .execute(&json!({"path": p, "old_string": "beta", "new_string": "BETA"}))
            .await
            .unwrap();
        assert!(out.contains("1 replacement"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "alpha\nBETA\ngamma\n");
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn edit_file_errors_when_not_found() {
        let path = std::env::temp_dir().join("musk_edit_test_missing.txt");
        std::fs::write(&path, "alpha\n").unwrap();
        let p = path.to_string_lossy().to_string();
        let err = EditFile
            .execute(&json!({"path": p, "old_string": "zzz", "new_string": "x"}))
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::Exec(_)));
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn edit_file_errors_on_ambiguous_match() {
        let path = std::env::temp_dir().join("musk_edit_test_ambig.txt");
        std::fs::write(&path, "dup\ndup\n").unwrap();
        let p = path.to_string_lossy().to_string();
        let err = EditFile
            .execute(&json!({"path": p, "old_string": "dup", "new_string": "x"}))
            .await
            .unwrap_err();
        match err {
            ToolError::Exec(msg) => assert!(msg.contains("2 times")),
            other => panic!("expected Exec, got {other:?}"),
        }
        let _ = std::fs::remove_file(&path);
    }

    // ── Search ─────────────────────────────────────────────────

    #[tokio::test]
    async fn search_finds_pattern() {
        // Search the crate's own lib.rs for a known string.
        let out = Search
            .execute(&json!({"pattern": "pub mod", "path": "src/lib.rs"}))
            .await
            .unwrap();
        // rg or grep should find "pub mod" in lib.rs.
        assert!(!out.contains("(no matches)"));
    }

    #[tokio::test]
    async fn search_no_match_returns_empty_marker() {
        let out = Search
            .execute(&json!({"pattern": "zzz_definitely_not_here_xyz", "path": "src/lib.rs"}))
            .await
            .unwrap();
        assert!(out.contains("(no matches)"));
    }
}
