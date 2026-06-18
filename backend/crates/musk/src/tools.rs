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
}
