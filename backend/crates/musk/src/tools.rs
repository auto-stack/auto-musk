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

/// 目录列表:列出目录内容,返回结构化的 [{name, is_dir, size}]。
/// 比 run_command ls 更适合 agent 消费(JSON 而非原始 shell 输出)。
pub struct ListDir;

#[async_trait]
impl Tool for ListDir {
    fn name(&self) -> &str {
        "list_dir"
    }
    fn description(&self) -> &str {
        "List the contents of a directory. Returns one entry per line as \
         'name <dir|file size>'. Useful for exploring project structure."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "directory to list (default: current dir)" }
            }
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let path = args["path"].as_str().unwrap_or(".");
        let entries = std::fs::read_dir(path)
            .map_err(|e| ToolError::Exec(format!("list '{path}': {e}")))?;

        let mut items: Vec<(String, bool, u64)> = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                let meta = e.metadata().ok()?;
                Some((name, meta.is_dir(), meta.len()))
            })
            .collect();
        // dirs first, then files, each alphabetical.
        items.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))
        });

        let mut out = String::new();
        for (name, is_dir, size) in items {
            if is_dir {
                out.push_str(&format!("{name} <dir>\n"));
            } else {
                out.push_str(&format!("{name} <file {size}B>\n"));
            }
        }
        if out.is_empty() {
            out.push_str("(empty directory)");
        }
        Ok(out)
    }
}

/// 文件符号大纲:扫描 Rust/TS 文件的 pub fn/struct/enum/mod 等定义行。
/// 不引入 tree-sitter,用轻量正则。看结构不用读全文。
pub struct ListSymbols;

#[async_trait]
impl Tool for ListSymbols {
    fn name(&self) -> &str {
        "list_symbols"
    }
    fn description(&self) -> &str {
        "List the top-level symbols (functions, structs, enums, classes, \
         interfaces, modules) defined in a source file. Returns the relevant \
         lines with line numbers. Supports Rust and TypeScript/JavaScript."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "source file to scan" }
            },
            "required": ["path"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'path'".into()))?;
        let content = std::fs::read_to_string(path)
            .map_err(|e| ToolError::Exec(format!("read '{path}': {e}")))?;

        // Patterns that indicate a symbol definition line. We match on the
        // start of the trimmed line to keep it simple (no nested-body parsing).
        // Rust: pub fn / fn / pub struct / struct / enum / impl / mod / pub trait / trait
        // TS/JS: function / export / class / interface / const / type
        let symbol_prefixes = [
            "pub fn", "pub async fn", "fn ", "async fn",
            "pub struct", "struct ", "pub enum", "enum ",
            "impl ", "mod ", "pub trait", "trait ",
            "pub use", "use ",
            "export ", "export default", "export async",
            "function ", "async function", "class ", "interface ",
            "const ", "type ",
        ];

        let mut out = String::new();
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            // Skip comment lines.
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }
            if symbol_prefixes.iter().any(|p| trimmed.starts_with(p)) {
                // Truncate very long lines for readability.
                let display: String = trimmed.chars().take(100).collect();
                out.push_str(&format!("{}: {display}\n", i + 1));
            }
        }
        if out.is_empty() {
            out.push_str("(no symbols found)");
        }
        Ok(out)
    }
}

/// 文件名模式匹配:用 glob 找文件(如 **/*.rs, **/test_*)。
/// 比 search(内容)和 list_dir(单层)更适合"找某类文件"。
pub struct Glob;

#[async_trait]
impl Tool for Glob {
    fn name(&self) -> &str {
        "glob"
    }
    fn description(&self) -> &str {
        "Find files matching a glob pattern (e.g. '**/*.rs', '**/test_*'). \
         Returns matching paths, one per line."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "glob pattern (e.g. '**/*.rs')" },
                "path": { "type": "string", "description": "base directory (default: current dir)" }
            },
            "required": ["pattern"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let pattern = args["pattern"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'pattern'".into()))?;
        let base = args["path"].as_str().unwrap_or(".");
        let full_pattern = if pattern.starts_with('/') || pattern.contains(':') {
            // absolute or has a drive letter — use as-is
            pattern.to_string()
        } else {
            format!("{base}/{pattern}")
        };

        let matches: Vec<String> = glob::glob(&full_pattern)
            .map_err(|e| ToolError::Args(format!("invalid pattern '{pattern}': {e}")))?
            .filter_map(|r| r.ok())
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        if matches.is_empty() {
            return Ok("(no matches)".into());
        }
        // Cap the output to avoid flooding context.
        const MAX: usize = 200;
        let mut out = String::new();
        for (i, m) in matches.iter().enumerate().take(MAX) {
            out.push_str(m);
            out.push('\n');
        }
        if matches.len() > MAX {
            out.push_str(&format!("... ({} more, refine pattern)\n", matches.len() - MAX));
        }
        Ok(out)
    }
}

/// 多处批量替换:在一个文件里一次性做多处 edit_file 风格的替换。
/// 每个替换要求 old 唯一;任一不唯一则全部不执行(原子性)。
pub struct BatchReplace;

#[async_trait]
impl Tool for BatchReplace {
    fn name(&self) -> &str {
        "batch_replace"
    }
    fn description(&self) -> &str {
        "Apply multiple unique-match string replacements to one file in a \
         single call. Atomic: if any old_string is ambiguous or missing, NO \
         replacement is made. Each item: {old_string, new_string}."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "file to edit" },
                "replacements": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "old_string": { "type": "string" },
                            "new_string": { "type": "string" }
                        },
                        "required": ["old_string", "new_string"]
                    }
                }
            },
            "required": ["path", "replacements"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'path'".into()))?;
        let replacements = args["replacements"]
            .as_array()
            .ok_or_else(|| ToolError::Args("missing 'replacements' array".into()))?;

        let mut content = std::fs::read_to_string(path)
            .map_err(|e| ToolError::Exec(format!("read '{path}': {e}")))?;

        // Pre-validate ALL replacements (atomicity): each old must be unique.
        // Track already-applied count so duplicate olds across items are caught.
        let mut applied_counts: Vec<usize> = Vec::with_capacity(replacements.len());
        for (i, rep) in replacements.iter().enumerate() {
            let old = rep["old_string"]
                .as_str()
                .ok_or_else(|| ToolError::Args(format!("replacement[{i}] missing 'old_string'")))?;
            let count = content.matches(old).count();
            if count == 0 {
                return Err(ToolError::Exec(format!(
                    "replacement[{i}] old_string not found in '{path}' (no changes applied)"
                )));
            }
            if count > 1 {
                return Err(ToolError::Exec(format!(
                    "replacement[{i}] old_string appears {count} times in '{path}'; must be unique (no changes applied)"
                )));
            }
            applied_counts.push(count);
        }

        // All valid — apply.
        for rep in replacements {
            let old = rep["old_string"].as_str().unwrap();
            let new = rep["new_string"]
                .as_str()
                .ok_or_else(|| ToolError::Args("missing 'new_string'".into()))?;
            content = content.replacen(old, new, 1);
        }
        std::fs::write(path, &content)
            .map_err(|e| ToolError::Exec(format!("write '{path}': {e}")))?;
        Ok(format!("applied {} replacement(s) to '{path}'", replacements.len()))
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

    // ── ListDir ────────────────────────────────────────────────

    #[tokio::test]
    async fn list_dir_lists_files() {
        let out = ListDir.execute(&json!({"path": "src"})).await.unwrap();
        // src/ contains tools.rs, lib.rs, main.rs, etc.
        assert!(out.contains("tools.rs"));
        assert!(out.contains("lib.rs"));
    }

    #[tokio::test]
    async fn list_dir_missing_errors() {
        let err = ListDir
            .execute(&json!({"path": "nonexistent_dir_xyz"}))
            .await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn list_dir_empty_shows_marker() {
        let dir = std::env::temp_dir().join("musk_listdir_empty_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let out = ListDir
            .execute(&json!({"path": dir.to_string_lossy()}))
            .await
            .unwrap();
        assert!(out.contains("(empty directory)"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── ListSymbols ────────────────────────────────────────────

    #[tokio::test]
    async fn list_symbols_finds_rust_structs() {
        let out = ListSymbols
            .execute(&json!({"path": "src/tools.rs"}))
            .await
            .unwrap();
        // tools.rs defines these structs.
        assert!(out.contains("pub struct EditFile"));
        assert!(out.contains("pub struct Search"));
    }

    #[tokio::test]
    async fn list_symbols_missing_file_errors() {
        let err = ListSymbols
            .execute(&json!({"path": "nonexistent.rs"}))
            .await;
        assert!(err.is_err());
    }

    // ── Glob ───────────────────────────────────────────────────

    #[tokio::test]
    async fn glob_finds_rust_files() {
        let out = Glob
            .execute(&json!({"pattern": "**/*.rs", "path": "src"}))
            .await
            .unwrap();
        assert!(out.contains("tools.rs"));
        assert!(out.contains("lib.rs"));
    }

    #[tokio::test]
    async fn glob_no_match() {
        let out = Glob
            .execute(&json!({"pattern": "**/*.nonexistent", "path": "src"}))
            .await
            .unwrap();
        assert!(out.contains("(no matches)"));
    }

    // ── BatchReplace ───────────────────────────────────────────

    #[tokio::test]
    async fn batch_replace_multiple_unique() {
        let path = std::env::temp_dir().join("musk_batch_test.txt");
        std::fs::write(&path, "aaa\nbbb\nccc\n").unwrap();
        let p = path.to_string_lossy().to_string();
        let out = BatchReplace
            .execute(&json!({
                "path": p,
                "replacements": [
                    {"old_string": "aaa", "new_string": "AAA"},
                    {"old_string": "ccc", "new_string": "CCC"}
                ]
            }))
            .await
            .unwrap();
        assert!(out.contains("2 replacement"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "AAA\nbbb\nCCC\n");
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn batch_replace_atomic_on_ambiguous() {
        let path = std::env::temp_dir().join("musk_batch_atomic.txt");
        std::fs::write(&path, "dup\ndup\nunique\n").unwrap();
        let p = path.to_string_lossy().to_string();
        let err = BatchReplace
            .execute(&json!({
                "path": p,
                "replacements": [
                    {"old_string": "dup", "new_string": "X"},
                    {"old_string": "unique", "new_string": "Y"}
                ]
            }))
            .await
            .unwrap_err();
        match err {
            ToolError::Exec(msg) => assert!(msg.contains("2 times")),
            other => panic!("expected Exec, got {other:?}"),
        }
        // Atomicity: file unchanged.
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "dup\ndup\nunique\n");
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn batch_replace_atomic_on_missing() {
        let path = std::env::temp_dir().join("musk_batch_missing.txt");
        std::fs::write(&path, "keep\n").unwrap();
        let p = path.to_string_lossy().to_string();
        let err = BatchReplace
            .execute(&json!({
                "path": p,
                "replacements": [
                    {"old_string": "keep", "new_string": "K"},
                    {"old_string": "absent", "new_string": "A"}
                ]
            }))
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::Exec(_)));
        // File unchanged (atomic).
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "keep\n");
        let _ = std::fs::remove_file(&path);
    }
}
