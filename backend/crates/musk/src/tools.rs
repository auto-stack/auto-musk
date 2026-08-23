//! auto-musk 基础工具:agent 在本地执行的能力(不经 daemon)。
//!
//! 这三个工具(读文件 / 写文件 / 执行命令)是 agent 操作文件系统和运行
//! 命令的最小集。它们实现 [`auto_ai_agent::Tool`],由 agent 的 ReAct 循环
//! 在本地直接调用 —— LLM 通信才走 daemon,工具执行永远在 musk 进程内。

use async_trait::async_trait;
use auto_ai_agent::{Tool, ToolError};
use serde_json::{json, Value};

/// PLAN-027 ①: path 越界错误 → 结构化 `SecurityDenied`（让 driver/前端识别
/// kind 并友好播报）。其他 path 错误（IO 等）仍走 `Exec`（纯字符串）。
fn map_path_error(e: String) -> ToolError {
    if e.contains("outside the project root") {
        ToolError::SecurityDenied {
            kind: "path_confined".into(),
            path: String::new(),
            root: crate::tool_safety::project_root().display().to_string(),
            hint: "AI 只能读写当前 workspace 内的文件；workspace 外的配置请让用户手动提供。".into(),
        }
    } else {
        ToolError::Exec(e)
    }
}

/// 读取文件内容(UTF-8 文本)。
pub struct ReadFile {
    /// 注入式 workspace root（PLAN-030 复审修复）。None = 沿用旧解析链
    /// （thread-local > startup CWD）；server/relay 注册路径一律注入，
    /// 规避 tokio 线程迁移下 thread-local 失效导致的越界写。
    root: Option<std::sync::Arc<std::path::PathBuf>>,
}

impl ReadFile {
    pub fn new() -> Self { Self { root: None } }
    pub fn with_root(root: std::sync::Arc<std::path::PathBuf>) -> Self { Self { root: Some(root) } }
    fn scope(&self) -> Option<&std::path::Path> { self.root.as_ref().map(|p| p.as_path()) }
}
#[async_trait]
impl Tool for ReadFile {
    fn name(&self) -> &str {
        "read_file"
    }
    fn description(&self) -> &str {
        "Read the UTF-8 text contents of a file. Output is truncated to \
         2000 lines or 50KB (whichever is hit first). Use offset/limit for \
         large files; when truncated, continue with the offset given in the \
         trailing note until you have the whole file."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "path to the file to read" },
                "offset": { "type": "number", "description": "line number to start reading from (1-indexed)" },
                "limit": { "type": "number", "description": "maximum number of lines to read" }
            },
            "required": ["path"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'path' argument".into()))?;
        let offset = args["offset"].as_u64().filter(|o| *o > 0);
        let limit = args["limit"].as_u64();
        // Path confinement (Design 004): reject paths outside project root.
        let resolved = crate::tool_safety::resolve_scoped(path, self.scope())
            .map_err(map_path_error)?;
        let content = std::fs::read_to_string(&resolved)
            .map_err(|e| ToolError::Exec(format!("read '{path}': {e}")))?;

        // PLAN-039 T3 分页(pi read.ts 语义):行切分含末尾换行产生的空行,
        // 总行数与 offset 越界判定都用这个口径。
        let all_lines: Vec<&str> = content.split('\n').collect();
        let total_file_lines = all_lines.len();
        let start_line = offset.map(|o| (o - 1) as usize).unwrap_or(0);
        let start_line_display = start_line + 1;
        if start_line >= all_lines.len() {
            return Err(ToolError::Exec(format!(
                "Offset {offset:?} is beyond end of file ({total_file_lines} lines total)"
            )));
        }
        // 用户显式给了 limit 优先;否则交给截断上限决定。
        let (selected, user_limited_lines): (String, Option<usize>) = match limit {
            Some(l) => {
                let end_line = (start_line + l as usize).min(all_lines.len());
                (
                    all_lines[start_line..end_line].join("\n"),
                    Some(end_line - start_line),
                )
            }
            None => (all_lines[start_line..].join("\n"), None),
        };

        use crate::tool_truncate::{
            format_size, truncate_head, TruncationResult, DEFAULT_MAX_BYTES, DEFAULT_MAX_LINES,
        };
        let t: TruncationResult = truncate_head(&selected, DEFAULT_MAX_LINES, DEFAULT_MAX_BYTES);
        // PLAN-027 挂接点:截断元数据(t.truncated_by / total_lines / …)应在
        // content/details 分离落地后放入 details;当前以尾注字符串承载。
        let output = if t.first_line_exceeds_limit {
            // 首行单独超字节上限:无法给出任何完整行,给 run_command 逃生通道。
            format!(
                "[Line {start_line_display} is {}, exceeds {} limit. Use run_command: \
                 sed -n '{start_line_display}p' {path} | head -c {DEFAULT_MAX_BYTES}]",
                format_size(all_lines[start_line].len()),
                format_size(DEFAULT_MAX_BYTES),
            )
        } else if t.truncated {
            let end_line_display = start_line_display + t.output_lines - 1;
            let next_offset = end_line_display + 1;
            match t.truncated_by {
                Some(crate::tool_truncate::TruncatedBy::Lines) => format!(
                    "{}\n\n[Showing lines {start_line_display}-{end_line_display} of \
                     {total_file_lines}. Use offset={next_offset} to continue.]",
                    t.content
                ),
                _ => format!(
                    "{}\n\n[Showing lines {start_line_display}-{end_line_display} of \
                     {total_file_lines} ({} limit). Use offset={next_offset} to continue.]",
                    t.content,
                    format_size(DEFAULT_MAX_BYTES),
                ),
            }
        } else if let Some(n) = user_limited_lines.filter(|n| start_line + n < all_lines.len()) {
            let remaining = all_lines.len() - (start_line + n);
            let next_offset = start_line + n + 1;
            format!(
                "{content}\n\n[{remaining} more lines in file. Use offset={next_offset} to continue.]",
                content = t.content
            )
        } else {
            t.content
        };
        Ok(output)
    }
}

/// 写入文件(覆盖已存在文件;自动创建父目录)。
pub struct WriteFile {
    /// 注入式 workspace root（PLAN-030 复审修复）。None = 沿用旧解析链
    /// （thread-local > startup CWD）；server/relay 注册路径一律注入，
    /// 规避 tokio 线程迁移下 thread-local 失效导致的越界写。
    root: Option<std::sync::Arc<std::path::PathBuf>>,
}

impl WriteFile {
    pub fn new() -> Self { Self { root: None } }
    pub fn with_root(root: std::sync::Arc<std::path::PathBuf>) -> Self { Self { root: Some(root) } }
    fn scope(&self) -> Option<&std::path::Path> { self.root.as_ref().map(|p| p.as_path()) }
}
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

        // Path confinement (Design 004).
        let resolved = crate::tool_safety::resolve_scoped(path, self.scope())
            .map_err(map_path_error)?;

        // Auto-create parent directories.
        if let Some(parent) = resolved.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ToolError::Exec(format!("create dirs for '{path}': {e}")))?;
        }
        std::fs::write(&resolved, content)
            .map_err(|e| ToolError::Exec(format!("write '{path}': {e}")))?;
        Ok(format!("wrote {} bytes to {}", content.len(), path))
    }
}

/// 执行一条 shell 命令,返回合并的 stdout+stderr。
///
/// 安全分级(Design 004):白名单命令直接执行;其他命令返回 PAUSED 状态
/// 提醒用户确认。设 `force: true` 可跳过白名单检查(用户 approve 后)。
/// 未来 run_command 后端将换为 Ash,由 Ash 的逐命令沙箱接管安全。
pub struct RunCommand {
    /// 注入式 workspace root（PLAN-030 复审修复）。None = 沿用旧解析链
    /// （thread-local > startup CWD）；server/relay 注册路径一律注入，
    /// 规避 tokio 线程迁移下 thread-local 失效导致的越界写。
    root: Option<std::sync::Arc<std::path::PathBuf>>,
}

impl RunCommand {
    pub fn new() -> Self { Self { root: None } }
    pub fn with_root(root: std::sync::Arc<std::path::PathBuf>) -> Self { Self { root: Some(root) } }
    fn scope(&self) -> Option<&std::path::Path> { self.root.as_ref().map(|p| p.as_path()) }
}
#[async_trait]
impl Tool for RunCommand {
    fn name(&self) -> &str {
        "run_command"
    }
    fn description(&self) -> &str {
        "Run a shell command and return its combined stdout and stderr. \
         Whitelisted commands (cargo/npm/git status/echo/…) run directly; \
         all others are PAUSED for user approval. Pass \"force\": true to \
         run a paused command after the user approves it."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "cmd": { "type": "string", "description": "the shell command to run" },
                "force": { "type": "boolean", "description": "skip the whitelist check (set true only after user approval)" }
            },
            "required": ["cmd"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let cmd = args["cmd"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'cmd' argument".into()))?;
        let force = args["force"].as_bool().unwrap_or(false);

        // Safety classification (Design 004).
        if !force {
            match crate::tool_safety::classify_command(cmd) {
                crate::tool_safety::CommandTier::Allowed => { /* proceed */ }
                crate::tool_safety::CommandTier::NeedsApproval(reason) => {
                    // Return a PAUSED result — not an error. The agent should
                    // relay this to the user; if approved, re-call with force.
                    return Ok(format!(
                        "⏸ PAUSED: {reason}\n\n\
                         To run this command, the user must approve it. \
                         If approved, call run_command again with \"force\": true."
                    ));
                }
            }
        }

        // PLAN-027 ③: run_command 也受 workspace path confinement（堵 cat/type
        // 白名单放行 + 不设 cwd 导致能绕过读 workspace 外文件的安全漏洞）。
        crate::tool_safety::confine_command_paths(cmd)
            .map_err(ToolError::Exec)?;
        let root = self
            .scope()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(crate::tool_safety::project_root);

        let output = if cfg!(windows) {
            std::process::Command::new("cmd").args(["/C", cmd]).current_dir(&root).output()
        } else {
            std::process::Command::new("sh").args(["-c", cmd]).current_dir(&root).output()
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
        // PLAN-031 T36: 非零退出码追加标记——agent 与前端（cmd 结果块红色
        // Error: 前缀）共同的成功/失败判定信号。
        if !output.status.success() {
            result.push_str(&format!("
[exit: {}]", output.status.code().unwrap_or(-1)));
        }
        Ok(result)
    }
}

/// 精确字符串替换:把文件中 `old_string` 替换为 `new_string`。
/// 要求 `old_string` 在文件中唯一,否则报错(避免歧义替换)。
/// 比 WriteFile 安全(不覆盖整个文件),是 executing-plans 的核心工具。
pub struct EditFile {
    /// 注入式 workspace root（PLAN-030 复审修复）。None = 沿用旧解析链
    /// （thread-local > startup CWD）；server/relay 注册路径一律注入，
    /// 规避 tokio 线程迁移下 thread-local 失效导致的越界写。
    root: Option<std::sync::Arc<std::path::PathBuf>>,
}

impl EditFile {
    pub fn new() -> Self { Self { root: None } }
    pub fn with_root(root: std::sync::Arc<std::path::PathBuf>) -> Self { Self { root: Some(root) } }
    fn scope(&self) -> Option<&std::path::Path> { self.root.as_ref().map(|p| p.as_path()) }
}
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

        // Path confinement (Design 004).
        let resolved = crate::tool_safety::resolve_scoped(path, self.scope())
            .map_err(map_path_error)?;

        let content = std::fs::read_to_string(&resolved)
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
        std::fs::write(&resolved, &new_content)
            .map_err(|e| ToolError::Exec(format!("write '{path}': {e}")))?;
        Ok(format!("edited '{path}' (1 replacement)"))
    }
}

/// 内容搜索(grep/rg):在文件树里搜 pattern,返回匹配的行。
/// 用 rg(若可用)否则 fallback 到 grep -rn。
pub struct Search {
    /// 注入式 workspace root（PLAN-030 复审修复）。None = 沿用旧解析链
    /// （thread-local > startup CWD）；server/relay 注册路径一律注入，
    /// 规避 tokio 线程迁移下 thread-local 失效导致的越界写。
    root: Option<std::sync::Arc<std::path::PathBuf>>,
}

impl Search {
    pub fn new() -> Self { Self { root: None } }
    pub fn with_root(root: std::sync::Arc<std::path::PathBuf>) -> Self { Self { root: Some(root) } }
    fn scope(&self) -> Option<&std::path::Path> { self.root.as_ref().map(|p| p.as_path()) }
}
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
        let raw_path = args["path"].as_str().unwrap_or(".");

        // Path confinement (Design 004): constrain search to project root.
        let resolved = crate::tool_safety::resolve_scoped(raw_path, self.scope())
            .map_err(map_path_error)?;
        let path = resolved.to_string_lossy().to_string();

        // Prefer ripgrep if available (faster, respects .gitignore); else grep.
        let rg_available = std::process::Command::new("rg")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let output = if rg_available {
            std::process::Command::new("rg")
                .args(["-n", "--no-heading", "--max-filesize", "1M", pattern, &path])
                .output()
        } else if cfg!(windows) {
            std::process::Command::new("cmd")
                .args(["/C", &format!("findstr /S /N /R \"{pattern}\" {path}\\*")]
        )
                .output()
        } else {
            std::process::Command::new("grep")
                .args(["-rn", "--include=*", pattern, &path])
                .output()
        }
        .map_err(|e| ToolError::Exec(format!("spawn search: {e}")))?;

        let mut result = String::from_utf8_lossy(&output.stdout).to_string();
        if result.is_empty() {
            // No matches is a valid, non-error result.
            result.push_str("(no matches)");
        } else {
            // PLAN-039 T2: 单行超长先按字符截断(pi grep 行截断,500 字符)。
            result = result
                .lines()
                .map(|l| crate::tool_truncate::truncate_line(
                    l,
                    crate::tool_truncate::GREP_MAX_LINE_LENGTH,
                ))
                .collect::<Vec<_>>()
                .join("\n");
        }
        // Cap output length to avoid flooding the context. PLAN-039 T2: 走共享
        // 截断模块——字节切割点永远落在 UTF-8 字符边界(旧的裸
        // `String::truncate` 在中文等多字节内容下会 panic 掉整个进程),
        // 且改为尾部保留(与 pi 一致:最深路径的匹配排最后,最有信息量)。
        const MAX_BYTES: usize = 8 * 1024;
        let capped = crate::tool_truncate::truncate_tail(
            &result,
            crate::tool_truncate::DEFAULT_MAX_LINES,
            MAX_BYTES,
        );
        if capped.truncated {
            result = capped.content;
            result.push_str("\n... (output truncated, refine your pattern)");
        }
        Ok(result)
    }
}

/// 目录列表:列出目录内容,返回结构化的 [{name, is_dir, size}]。
/// 比 run_command ls 更适合 agent 消费(JSON 而非原始 shell 输出)。
pub struct ListDir {
    /// 注入式 workspace root（PLAN-030 复审修复）。None = 沿用旧解析链
    /// （thread-local > startup CWD）；server/relay 注册路径一律注入，
    /// 规避 tokio 线程迁移下 thread-local 失效导致的越界写。
    root: Option<std::sync::Arc<std::path::PathBuf>>,
}

impl ListDir {
    pub fn new() -> Self { Self { root: None } }
    pub fn with_root(root: std::sync::Arc<std::path::PathBuf>) -> Self { Self { root: Some(root) } }
    fn scope(&self) -> Option<&std::path::Path> { self.root.as_ref().map(|p| p.as_path()) }
}
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
        let raw_path = args["path"].as_str().unwrap_or(".");
        // Path confinement (Design 004).
        let path = crate::tool_safety::resolve_scoped(raw_path, self.scope())
            .map_err(map_path_error)?;
        let entries = std::fs::read_dir(&path)
            .map_err(|e| ToolError::Exec(format!("list '{raw_path}': {e}")))?;

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
pub struct ListSymbols {
    /// 注入式 workspace root（PLAN-030 复审修复）。None = 沿用旧解析链
    /// （thread-local > startup CWD）；server/relay 注册路径一律注入，
    /// 规避 tokio 线程迁移下 thread-local 失效导致的越界写。
    root: Option<std::sync::Arc<std::path::PathBuf>>,
}

impl ListSymbols {
    pub fn new() -> Self { Self { root: None } }
    pub fn with_root(root: std::sync::Arc<std::path::PathBuf>) -> Self { Self { root: Some(root) } }
    fn scope(&self) -> Option<&std::path::Path> { self.root.as_ref().map(|p| p.as_path()) }
}
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
        // Path confinement (Design 004).
        let resolved = crate::tool_safety::resolve_scoped(path, self.scope())
            .map_err(map_path_error)?;
        let content = std::fs::read_to_string(&resolved)
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
pub struct Glob {
    /// 注入式 workspace root（PLAN-030 复审修复）。None = 沿用旧解析链
    /// （thread-local > startup CWD）；server/relay 注册路径一律注入，
    /// 规避 tokio 线程迁移下 thread-local 失效导致的越界写。
    root: Option<std::sync::Arc<std::path::PathBuf>>,
}

impl Glob {
    pub fn new() -> Self { Self { root: None } }
    pub fn with_root(root: std::sync::Arc<std::path::PathBuf>) -> Self { Self { root: Some(root) } }
    fn scope(&self) -> Option<&std::path::Path> { self.root.as_ref().map(|p| p.as_path()) }
}
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
        let raw_base = args["path"].as_str().unwrap_or(".");
        // Path confinement (Design 004): constrain glob base to project root.
        let base = crate::tool_safety::resolve_scoped(raw_base, self.scope())
            .map_err(map_path_error)?;
        let base_str = base.to_string_lossy().to_string();
        let full_pattern = if pattern.starts_with('/') || pattern.contains(':') {
            // absolute or has a drive letter — use as-is
            pattern.to_string()
        } else {
            format!("{base_str}/{pattern}")
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
        for (_i, m) in matches.iter().enumerate().take(MAX) {
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
pub struct BatchReplace {
    /// 注入式 workspace root（PLAN-030 复审修复）。None = 沿用旧解析链
    /// （thread-local > startup CWD）；server/relay 注册路径一律注入，
    /// 规避 tokio 线程迁移下 thread-local 失效导致的越界写。
    root: Option<std::sync::Arc<std::path::PathBuf>>,
}

impl BatchReplace {
    pub fn new() -> Self { Self { root: None } }
    pub fn with_root(root: std::sync::Arc<std::path::PathBuf>) -> Self { Self { root: Some(root) } }
    fn scope(&self) -> Option<&std::path::Path> { self.root.as_ref().map(|p| p.as_path()) }
}
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

        // Path confinement (Design 004).
        let resolved = crate::tool_safety::resolve_scoped(path, self.scope())
            .map_err(map_path_error)?;

        let mut content = std::fs::read_to_string(&resolved)
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
        std::fs::write(&resolved, &content)
            .map_err(|e| ToolError::Exec(format!("write '{path}': {e}")))?;
        Ok(format!("applied {} replacement(s) to '{path}'", replacements.len()))
    }
}

/// Display an image file inline in the chat. The agent calls this after
/// generating a visual artifact (e.g. a PNG chart via `run_command` + a
/// plotting script) so the image is shown to the user rather than only
/// existing on disk. Returns a markdown image link whose URL points at the
/// `/api/files/{workspace_id}/{rel_path}` endpoint.
pub struct DisplayImage {
    ctx: crate::tool_context::ToolContext,
}

impl DisplayImage {
    pub fn new(ctx: crate::tool_context::ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Tool for DisplayImage {
    fn name(&self) -> &str {
        "display_image"
    }
    fn description(&self) -> &str {
        "Display an image file (generated by you) inline in the chat so the \
         user can see it. Pass the path to the image (relative to the project \
         root). Call this AFTER you have generated the image (e.g. after \
         run_command produced a PNG). Supported formats: png/jpg/jpeg/gif/svg/webp."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "path to the image file to display" }
            },
            "required": ["path"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::Args("missing 'path' argument".into()))?;

        // Confine + canonicalize (same guard as read_file).
        let resolved = crate::tool_safety::resolve_scoped(
                path,
                Some(&self.ctx.state.registry.get(&self.ctx.workspace_id).root),
            )
            .map_err(ToolError::Exec)?;

        // Accept only image extensions.
        let ext = resolved.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(
            ext.to_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp"
        ) {
            return Err(ToolError::Exec(format!(
                "display_image: '{ext}' is not a supported image format"
            )));
        }
        if !resolved.exists() {
            return Err(ToolError::Exec(format!(
                "display_image: file '{path}' does not exist"
            )));
        }

        // Build the URL: /api/files/{workspace_id}/{relative path}.
        let ws = self.ctx.state.registry.get(&self.ctx.workspace_id);
        let rel = resolved
            .strip_prefix(&ws.root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.replace('\\', "/"));
        let url = format!("/api/files/{}/{}", self.ctx.workspace_id, rel);
        // Return as a markdown image so the frontend's markdown renderer shows it.
        let alt = std::path::Path::new(&rel)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image");
        Ok(format!("![{}]({})", alt, url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// File-tool tests operate on the crate's own source (src/…) and on
    /// crate-local `.test-tmp/` fixtures. Tests don't run `main()`, so the
    /// process-wide project root (normally snapshotted at startup) is
    /// uninitialized and path confinement would fall back to literal `.`.
    /// Initialize it once to the crate dir (cwd) so resolution is
    /// deterministic regardless of test ordering (OnceLock: idempotent).
    /// Also pre-create the `.test-tmp/` fixture dir (tests that seed files
    /// with a direct `std::fs::write` expect it to already exist).
    fn init_root() {
        crate::tool_safety::init_project_root();
        let _ = std::fs::create_dir_all(".test-tmp");
    }

    #[tokio::test]
    async fn read_file_reads_existing() {
        init_root();
        let t = ReadFile::new();
        // The crate's own Cargo.toml lives inside the project root.
        let out = t
            .execute(&json!({"path": "Cargo.toml"}))
            .await
            .unwrap();
        assert!(out.contains("[package]"));
    }

    #[tokio::test]
    async fn read_file_missing_errors() {
        let t = ReadFile::new();
        let err = t.execute(&json!({"path": "definitely_nonexistent.xyz"})).await;
        assert!(err.is_err());
    }

    // ── ReadFile 分页/截断(PLAN-039 T3,对齐 pi read.ts)────────────

    fn write_fixture(rel: &str, content: &str) -> String {
        init_root();
        std::fs::create_dir_all(".test-tmp").unwrap();
        let path = std::path::PathBuf::from(rel);
        std::fs::write(&path, content).unwrap();
        path.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn read_file_offset_limit_window() {
        let content: String = (1..=10).map(|i| format!("line{i}\n")).collect();
        let p = write_fixture(".test-tmp/musk_read_paging.txt", &content);
        let out = ReadFile::new()
            .execute(&json!({"path": p, "offset": 3, "limit": 4}))
            .await
            .unwrap();
        // 窗口为第 3-6 行;文件还有余量(limit 提前停)→ 续读尾注。
        // (总行数含末尾换行产生的空行,与 pi split 口径一致 = 11。)
        assert_eq!(
            out,
            "line3\nline4\nline5\nline6\n\n[5 more lines in file. Use offset=7 to continue.]"
        );
    }

    #[tokio::test]
    async fn read_file_offset_out_of_bounds_errors_with_total() {
        let content: String = (1..=10).map(|i| format!("line{i}\n")).collect();
        let p = write_fixture(".test-tmp/musk_read_oob.txt", &content);
        let err = ReadFile::new()
            .execute(&json!({"path": p, "offset": 99}))
            .await
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("beyond end of file"), "msg: {msg}");
        assert!(msg.contains("11 lines total"), "msg: {msg}");
    }

    #[tokio::test]
    async fn read_file_line_truncation_note() {
        // 2500 行 → 默认 2000 行上限触发,尾注指导续读。
        let content = (1..=2500).map(|i| format!("l{i}")).collect::<Vec<_>>().join("\n");
        let p = write_fixture(".test-tmp/musk_read_lines.txt", &content);
        let out = ReadFile::new().execute(&json!({"path": p})).await.unwrap();
        assert!(out.contains("[Showing lines 1-2000 of 2500. Use offset=2001 to continue.]"), "out tail: {}", &out[out.len().saturating_sub(120)..]);
        assert!(out.starts_with("l1\nl2\n"));
    }

    #[tokio::test]
    async fn read_file_byte_truncation_note() {
        // 600 行 × 100B ≈ 60KB(行数在限内)→ 字节上限触发。
        let content = (0..600).map(|_| "b".repeat(99)).collect::<Vec<_>>().join("\n");
        let p = write_fixture(".test-tmp/musk_read_bytes.txt", &content);
        let out = ReadFile::new().execute(&json!({"path": p})).await.unwrap();
        assert!(out.contains("(50.0KB limit). Use offset="), "out tail: {}", &out[out.len().saturating_sub(160)..]);
        assert!(out.len() < 52_000, "capped at ~50KB, got {}", out.len());
    }

    #[tokio::test]
    async fn read_file_first_line_exceeds_gives_escape_hatch() {
        // 单行 60KB > 50KB 上限 → 给 run_command sed 逃生提示。
        let content = "x".repeat(60_000);
        let p = write_fixture(".test-tmp/musk_read_huge_line.txt", &content);
        let out = ReadFile::new().execute(&json!({"path": p})).await.unwrap();
        assert!(out.contains("exceeds 50.0KB limit"), "out: {}", &out[..out.len().min(200)]);
        assert!(out.contains("run_command"), "must point at run_command escape hatch");
        assert!(out.contains("sed -n '1p'"), "must give the exact sed command");
    }

    /// 验收标准:读 10MB 文件返回 ≤50KB 且尾注可指导续读。
    #[tokio::test]
    async fn read_file_10mb_returns_capped_output() {
        // 1M 行 × ~10B ≈ 10MB;远超 2000 行上限。
        let content = (0..1_000_000).map(|i| format!("data{i:06}")).collect::<Vec<_>>().join("\n");
        let p = write_fixture(".test-tmp/musk_read_10mb.txt", &content);
        let out = ReadFile::new().execute(&json!({"path": p})).await.unwrap();
        assert!(out.len() < 52_000, "10MB read must be capped ≤~50KB, got {}", out.len());
        assert!(out.contains("Use offset=2001 to continue."));
        let _ = std::fs::remove_file(&p);
    }

    #[tokio::test]
    async fn read_file_offset_continuation_reads_next_window() {
        // 续读:offset=2001 拿到剩余 500 行,不再有截断尾注。
        let content = (1..=2500).map(|i| format!("l{i}")).collect::<Vec<_>>().join("\n");
        let p = write_fixture(".test-tmp/musk_read_cont.txt", &content);
        let out = ReadFile::new()
            .execute(&json!({"path": p, "offset": 2001}))
            .await
            .unwrap();
        assert!(out.starts_with("l2001\n"));
        assert!(out.ends_with("l2500"));
        assert!(!out.contains("Use offset="), "no further truncation note expected");
    }

    /// PLAN-030 复审回归：注入式 root 下，相对路径必须落在注入的 workspace
    /// 根内——即使当前线程/CWD 完全不同（tokio 线程迁移场景的确定性等价）。
    #[tokio::test]
    async fn with_root_scopes_relative_paths_to_injected_root() {
        let dir = std::env::temp_dir().join(format!(
            "musk-scope-root-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let root = std::sync::Arc::new(dir.clone());
        let w = WriteFile::with_root(root);
        w.execute(&json!({"path": "notes/scoped.md", "content": "scoped"}))
            .await
            .unwrap();
        assert!(dir.join("notes/scoped.md").exists(), "written inside injected root");
        // 进程 CWD（crate 目录）下不应出现同名文件
        assert!(!std::path::Path::new("notes/scoped.md").exists(), "must NOT escape to CWD");
        let _ = std::fs::remove_dir_all(&dir);

        // 越界仍被拒绝：注入 root 外的绝对路径
        let outside = std::env::temp_dir().join("musk-scope-outside.md");
        let w2 = WriteFile::with_root(std::sync::Arc::new(dir.clone()));
        let err = w2
            .execute(&json!({"path": outside.to_string_lossy(), "content": "x"}))
            .await;
        assert!(err.is_err(), "absolute path outside injected root rejected");
    }

    #[tokio::test]
    async fn read_file_missing_path_arg_errors() {
        let t = ReadFile::new();
        let err = t.execute(&json!({})).await.unwrap_err();
        assert!(matches!(err, ToolError::Args(_)));
    }

    #[tokio::test]
    async fn write_file_then_read_back() {
        init_root();
        let t_write = WriteFile::new();
        let t_read = ReadFile::new();
        let path = std::path::PathBuf::from(".test-tmp/musk_tool_test_write.txt");
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
        init_root();
        let t = WriteFile::new();
        let dir = std::path::PathBuf::from(".test-tmp/musk_tool_test_subdir");
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
        let t = RunCommand::new();
        // `echo` works on both Windows (cmd /C echo) and Unix (sh -c echo).
        let out = t.execute(&json!({"cmd": "echo musk_test_token"})).await.unwrap();
        assert!(out.contains("musk_test_token"));
    }

    #[tokio::test]
    async fn run_command_missing_cmd_arg_errors() {
        let t = RunCommand::new();
        let err = t.execute(&json!({})).await.unwrap_err();
        assert!(matches!(err, ToolError::Args(_)));
    }

    // ── EditFile ───────────────────────────────────────────────

    #[tokio::test]
    async fn edit_file_replaces_unique_match() {
        init_root();
        let path = std::path::PathBuf::from(".test-tmp/musk_edit_test_unique.txt");
        std::fs::write(&path, "alpha\nbeta\ngamma\n").unwrap();
        let p = path.to_string_lossy().to_string();
        let out = EditFile::new()
            .execute(&json!({"path": p, "old_string": "beta", "new_string": "BETA"}))
            .await
            .unwrap();
        assert!(out.contains("1 replacement"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "alpha\nBETA\ngamma\n");
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn edit_file_errors_when_not_found() {
        // PLAN-030 复审修复：改用根内路径——temp 绝对路径会先触发 path
        // confinement 的 SecurityDenied，测不到 old_string 未命中分支。
        init_root();
        let path = std::path::PathBuf::from(".test-tmp/musk_edit_test_missing.txt");
        std::fs::create_dir_all(".test-tmp").unwrap();
        std::fs::write(&path, "alpha\n").unwrap();
        let p = path.to_string_lossy().to_string();
        let err = EditFile::new()
            .execute(&json!({"path": p, "old_string": "zzz", "new_string": "x"}))
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::Exec(_)));
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn edit_file_errors_on_ambiguous_match() {
        init_root();
        let path = std::path::PathBuf::from(".test-tmp/musk_edit_test_ambig.txt");
        std::fs::write(&path, "dup\ndup\n").unwrap();
        let p = path.to_string_lossy().to_string();
        let err = EditFile::new()
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
        init_root();
        // Search the crate's own lib.rs for a known string.
        let out = Search::new()
            .execute(&json!({"pattern": "pub mod", "path": "src/lib.rs"}))
            .await
            .unwrap();
        // rg or grep should find "pub mod" in lib.rs.
        assert!(!out.contains("(no matches)"));
    }

    #[tokio::test]
    async fn search_no_match_returns_empty_marker() {
        init_root();
        let out = Search::new()
            .execute(&json!({"pattern": "zzz_definitely_not_here_xyz", "path": "src/lib.rs"}))
            .await
            .unwrap();
        assert!(out.contains("(no matches)"));
    }

    /// PLAN-039 T2 回归:中文内容超过 8KB 截断上限时不得 panic。
    /// 旧实现 `result.truncate(8192)` 在多字节字符中间切割会直接崩掉
    /// 整个 musk 进程;新实现走共享截断模块(尾部保留 + 字符边界安全)。
    #[tokio::test]
    async fn search_multibyte_truncation_does_not_panic() {
        init_root();
        let path = std::path::PathBuf::from(".test-tmp/musk_search_multibyte.txt");
        std::fs::create_dir_all(".test-tmp").unwrap();
        // 60 行 × (100 个三字节汉字 + 前缀 + 换行) ≈ 20KB 匹配输出,
        // 必然触发 8KB 截断;8192 字节边界落在中文字符中间。
        let line = format!("{}match", "中".repeat(100));
        let content = std::iter::repeat_n(line + "\n", 60).collect::<String>();
        std::fs::write(&path, content).unwrap();
        let out = Search::new()
            .execute(&json!({"pattern": "match", "path": path.to_string_lossy()}))
            .await
            .expect("multibyte truncation must not panic");
        assert!(out.len() <= 9_000, "output should be capped, got {} bytes", out.len());
        assert!(out.contains("truncated"), "capped output should carry a truncation note");
        let _ = std::fs::remove_file(&path);
    }

    // ── ListDir ────────────────────────────────────────────────

    #[tokio::test]
    async fn list_dir_lists_files() {
        init_root();
        let out = ListDir::new().execute(&json!({"path": "src"})).await.unwrap();
        // src/ contains tools.rs, lib.rs, main.rs, etc.
        assert!(out.contains("tools.rs"));
        assert!(out.contains("lib.rs"));
    }

    #[tokio::test]
    async fn list_dir_missing_errors() {
        let err = ListDir::new()
            .execute(&json!({"path": "nonexistent_dir_xyz"}))
            .await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn list_dir_empty_shows_marker() {
        init_root();
        let dir = std::path::PathBuf::from(".test-tmp/musk_listdir_empty_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let out = ListDir::new()
            .execute(&json!({"path": dir.to_string_lossy().to_string()}))
            .await
            .unwrap();
        assert!(out.contains("(empty directory)"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── ListSymbols ────────────────────────────────────────────

    #[tokio::test]
    async fn list_symbols_finds_rust_structs() {
        init_root();
        let out = ListSymbols::new()
            .execute(&json!({"path": "src/tools.rs"}))
            .await
            .unwrap();
        // tools.rs defines these structs.
        assert!(out.contains("pub struct EditFile"));
        assert!(out.contains("pub struct Search"));
    }

    #[tokio::test]
    async fn list_symbols_missing_file_errors() {
        let err = ListSymbols::new()
            .execute(&json!({"path": "nonexistent.rs"}))
            .await;
        assert!(err.is_err());
    }

    // ── Glob ───────────────────────────────────────────────────

    #[tokio::test]
    async fn glob_finds_rust_files() {
        init_root();
        let out = Glob::new()
            .execute(&json!({"pattern": "**/*.rs", "path": "src"}))
            .await
            .unwrap();
        assert!(out.contains("tools.rs"));
        assert!(out.contains("lib.rs"));
    }

    #[tokio::test]
    async fn glob_no_match() {
        init_root();
        let out = Glob::new()
            .execute(&json!({"pattern": "**/*.nonexistent", "path": "src"}))
            .await
            .unwrap();
        assert!(out.contains("(no matches)"));
    }

    // ── BatchReplace ───────────────────────────────────────────

    #[tokio::test]
    async fn batch_replace_multiple_unique() {
        init_root();
        let path = std::path::PathBuf::from(".test-tmp/musk_batch_test.txt");
        std::fs::write(&path, "aaa\nbbb\nccc\n").unwrap();
        let p = path.to_string_lossy().to_string();
        let out = BatchReplace::new()
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
        init_root();
        let path = std::path::PathBuf::from(".test-tmp/musk_batch_atomic.txt");
        std::fs::write(&path, "dup\ndup\nunique\n").unwrap();
        let p = path.to_string_lossy().to_string();
        let err = BatchReplace::new()
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
        // PLAN-030 复审修复：同上，根内路径避开 SecurityDenied 分支。
        init_root();
        let path = std::path::PathBuf::from(".test-tmp/musk_batch_missing.txt");
        std::fs::create_dir_all(".test-tmp").unwrap();
        std::fs::write(&path, "keep\n").unwrap();
        let p = path.to_string_lossy().to_string();
        let err = BatchReplace::new()
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
