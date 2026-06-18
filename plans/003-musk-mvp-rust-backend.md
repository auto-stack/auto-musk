# 003 — auto-musk MVP:Rust 后端 + 单 agent + 基础工具

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 建立 auto-musk 的 Rust 后端骨架,实现一个可运行的 CLI agent —— 用 `auto-ai-agent` 的 Coder profession + 基础工具(读/写/执行),经 `auto-ai-daemon` 完成任务。这是 v2 阶段 0(骨架)+ 阶段 1(MVP)合并为首期交付。

**Architecture:** 新建 Rust workspace(`backend/`),一个 `musk` 二进制,依赖 `auto-ai-agent`(本地 path 依赖指向 `../auto-ai`)。`musk` CLI 接收一个任务描述,构建一个 Coder Agent,注册 3 个基础工具(`read_file`/`write_file`/`run_command`),调 `Agent::run`,打印结果。LLM 通信全部经 `auto-ai-daemon`(需先 `cargo install` 或在 PATH 里)。

**Tech Stack:** Rust,tokio,clap(CLI),`auto-ai-agent` + `auto-ai-client`(path dep)。

---

## 前置条件(执行前确认)

- [ ] `auto-ai` 仓库 main 分支已包含 auto-ai-agent(auto-ai-client/daemon/agent/ai-config),路径 `D:/autostack/auto-ai`。✅(已完成)
- [ ] `auto-ai-daemon`(`aaid`)可构建:`cd ../auto-ai && cargo build -p auto-ai-daemon`。执行本计划前先确认它能跑(或至少能编译)。
- [ ] 有一个可用的 LLM provider 配置(智谱/Anthropic/OpenAI 任一),否则 MVP 的 agent run 会因 daemon 无 provider 而失败。配置见 `~/.config/autoos/ai-daemon.at`(参考 `../auto-ai/crates/ai-config/examples/daemon.at`)。

---

## 文件结构

```
auto-musk/
├── backend/                      ← 新建 Rust workspace
│   ├── Cargo.toml                ← workspace 根
│   ├── crates/
│   │   └── musk/                 ← 主 crate(lib + bin)
│   │       ├── Cargo.toml        ← 依赖 auto-ai-agent(path), tokio, clap
│   │       └── src/
│   │           ├── main.rs       ← CLI 入口(clap): musk run "task"
│   │           ├── lib.rs        ← pub mod tools;(供集成测试)
│   │           └── tools.rs      ← 3 个基础 Tool 实现(read_file/write_file/run_command)
│   └── README.md                 ← 构建/运行说明
├── src/                          ← 现有 Auto 代码(见下方"处置")
├── plans/                        ← 本文件所在
└── pac.at                        ← 前端 workspace(保留)
```

## 现有 Auto 代码处置(2026-06-18 盘点)

转向 Rust 后,仓库里现有的 Auto(`.at`)代码去留如下:

| 文件 | 行数 | 内容 | 处置 |
|---|---|---|---|
| `src/back/specs.at` | 302 | Spec Ledger 数据模型(SectionType 7 类、23 种 Status、SpecItem),源自 auto-forge,注释详尽 | **保留作 Rust 迁移参考蓝本**(v2 阶段 5 Spec Ledger 时用);Auto 实现本身不进 Rust 构建 |
| `src/back/specs_test.at` | 69 | specs.at 的测试 | 随 specs.at 保留作参考 |
| `src/back/api.at` | 12 | 占位 `#[api] hello` | **废弃**(Rust 后端重新写) |
| `src/back/db.at` | 8 | 占位 | **废弃** |
| `src/front/*.at` | 16-57 | 前端页面占位骨架(chats/explorer/specs/relay/login),纯 UI,经 a2vue 生成 Vue | **保留**(前端层,本期不动;后续 v2 阶段 2 前后端打通时用) |
| `pac.at` | 11 | 前端 workspace 配置(`api: "rust"` 已标注) | **保留** |

**结论**:本期(Rust 后端 MVP)不删除任何现有文件 —— `backend/` 是新增的独立 Rust workspace,与 `src/` 的 Auto 代码物理隔离。`specs.at` 在 v2 阶段 5 移植 Spec Ledger 时作为数据模型参考(字段定义、Status 全集、SectionType 划分都照搬,只是实现改 Rust)。前端 `.at` 在 v2 阶段 2 前后端打通时复用。

**决策:不动前端。** 本期只交付后端 CLI agent。前端打通是 v2 阶段 2(HTTP API)的事。

---

## Task 1: Rust workspace 骨架 + 依赖 auto-ai-agent

**Files:**
- Create: `backend/Cargo.toml`
- Create: `backend/crates/musk/Cargo.toml`
- Create: `backend/crates/musk/src/main.rs`(最小 hello)

**说明:** 建立 Rust workspace,`musk` 二进制依赖 `auto-ai-agent`(本地 path 依赖 `../../auto-ai/crates/auto-ai-agent`)。先确认依赖能解析、能编译,再写业务。

- [ ] **Step 1: 创建 `backend/Cargo.toml`(workspace 根)**

```toml
[workspace]
resolver = "2"
members = ["crates/musk"]
```

- [ ] **Step 2: 创建 `backend/crates/musk/Cargo.toml`**

```toml
[package]
name = "musk"
version = "0.1.0"
edition = "2021"
description = "auto-musk — Forge-successor AI coding agent (Rust backend)"

[[bin]]
name = "musk"
path = "src/main.rs"

[dependencies]
# auto-ai-agent 提供 Profession/Agent/Workflow(经它间接依赖 auto-ai-client/ai-config)
auto-ai-agent = { path = "../../../auto-ai/crates/auto-ai-agent" }
# daemon 发现/调用(经 auto-ai-agent 重导出,但直接依赖更清晰)
auto-ai-client = { path = "../../../auto-ai/crates/auto-ai-client" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
clap = { version = "4", features = ["derive"] }
async-trait = "0.1"
serde_json = "1"
```

**路径核验**:`backend/crates/musk/` → `../`=`crates` → `../../`=`backend` → `../../../`=`auto-musk` → 需要到 `auto-ai/crates/auto-ai-agent`。从 `auto-musk` 到 `auto-ai` 是兄弟目录(`D:/autostack/auto-musk` → `D:/autostack/auto-ai`)。所以从 `backend/crates/musk` 是 `../../../auto-ai/crates/auto-ai-agent`?算一下:`backend/crates/musk` 上三级 = `auto-musk`,再 `../auto-ai` = `D:/autostack/auto-ai`。所以正确路径是 `../../../../auto-ai/crates/auto-ai-agent`(四级 `../`)。**Step 2 实现时务必用 `cargo` 报错来校准路径**(参考 auto-ai-agent Cargo.toml 里 auto-atom 的三级 `../` 模式),不要假设。

- [ ] **Step 3: 创建最小 `backend/crates/musk/src/main.rs`**

```rust
fn main() {
    println!("musk v0.1.0 — auto-musk agent (stub)");
}
```

- [ ] **Step 4: 验证编译 + 依赖解析**

Run: `cd backend && cargo build`
Expected: 编译通过,`auto-ai-agent` 及其依赖(auto-ai-client/ai-config/auto-atom)全部拉取成功。若路径错误,cargo 会报 "failed to read Cargo.toml",据报错校准 `../` 层数。

- [ ] **Step 5: 提交**

```bash
git add backend/
git commit -m "feat(backend): Rust workspace skeleton + auto-ai-agent dep"
```

---

## Task 2: 基础工具实现(read_file / write_file / run_command)

**Files:**
- Create: `backend/crates/musk/src/tools.rs`
- Modify: `backend/crates/musk/src/main.rs`(加 `mod tools;`)
- Create: `backend/crates/musk/tests/tools.rs`(集成测试)

**说明:** 实现 3 个 `auto_ai_agent::Tool`,参照 auto-ai-agent 的 Tool trait(`name`/`description`/`parameters`/`execute`)。这些是 agent 调 daemon 之外、在本地执行的工具。

- [ ] **Step 1: 写 `tools.rs` 的 read_file 工具(TDD:先写测试)**

`backend/crates/musk/tests/tools.rs`:
```rust
use musk::tools::{ReadFile, WriteFile};
use auto_ai_agent::{Tool, ToolError};
use serde_json::json;

#[tokio::test]
async fn read_file_reads_existing() {
    let t = ReadFile;
    let out = t.execute(&json!({"path": "Cargo.toml"})).await.unwrap();
    assert!(out.contains("[package]"));
}

#[tokio::test]
async fn read_file_missing_errors() {
    let t = ReadFile;
    let err = t.execute(&json!({"path": "nonexistent.xyz"})).await;
    assert!(err.is_err());
}
```

(测试引用 `musk::tools`,所以 tools 要在 lib.rs 或 main.rs 里 `pub mod tools;` 并让 crate 有 lib target。**决策:把 musk 做成 lib + bin 双 target**,lib 暴露 tools,bin 是 CLI。更新 Cargo.toml 加 `[lib]`。)

- [ ] **Step 2: 运行测试确认失败(ReadFile 未实现)**

Run: `cd backend && cargo test`
Expected: 编译失败(`musk::tools` 不存在)。

- [ ] **Step 3: 实现 `tools.rs`(ReadFile + WriteFile + RunCommand)**

```rust
//! auto-musk 基础工具:agent 在本地执行的能力(不经 daemon)。

use async_trait::async_trait;
use auto_ai_agent::{Tool, ToolError};
use serde_json::{json, Value};

/// 读取文件内容。
pub struct ReadFile;
#[async_trait]
impl Tool for ReadFile {
    fn name(&self) -> &str { "read_file" }
    fn description(&self) -> &str { "Read the contents of a file at the given path." }
    fn parameters(&self) -> Value {
        json!({"type":"object","properties":{"path":{"type":"string","description":"file path"}},"required":["path"]})
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let path = args["path"].as_str().ok_or_else(|| ToolError::Args("missing 'path'".into()))?;
        std::fs::read_to_string(path).map_err(|e| ToolError::Exec(format!("read {path}: {e}")))
    }
}

/// 写入文件(覆盖)。
pub struct WriteFile;
#[async_trait]
impl Tool for WriteFile {
    fn name(&self) -> &str { "write_file" }
    fn description(&self) -> &str { "Write content to a file (overwrites if exists)." }
    fn parameters(&self) -> Value {
        json!({"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]})
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let path = args["path"].as_str().ok_or_else(|| ToolError::Args("missing 'path'".into()))?;
        let content = args["content"].as_str().ok_or_else(|| ToolError::Args("missing 'content'".into()))?;
        std::fs::write(path, content).map_err(|e| ToolError::Exec(format!("write {path}: {e}")))?;
        Ok(format!("wrote {} bytes to {path}", content.len()))
    }
}

/// 执行 shell 命令(带确认提示——本期 MVP 直接执行,后续加白名单/确认)。
pub struct RunCommand;
#[async_trait]
impl Tool for RunCommand {
    fn name(&self) -> &str { "run_command" }
    fn description(&self) -> &str { "Run a shell command and return stdout+stderr." }
    fn parameters(&self) -> Value {
        json!({"type":"object","properties":{"cmd":{"type":"string","description":"command to run"}},"required":["cmd"]})
    }
    async fn execute(&self, args: &Value) -> Result<String, ToolError> {
        let cmd = args["cmd"].as_str().ok_or_else(|| ToolError::Args("missing 'cmd'".into()))?;
        // Windows 用 cmd /C,Unix 用 sh -c。
        let out = if cfg!(windows) {
            std::process::Command::new("cmd").args(["/C", cmd]).output()
        } else {
            std::process::Command::new("sh").args(["-c", cmd]).output()
        }.map_err(|e| ToolError::Exec(format!("spawn: {e}")))?;
        let mut result = String::new();
        result.push_str(&String::from_utf8_lossy(&out.stdout));
        if !out.stderr.is_empty() {
            result.push_str("\n[stderr]\n");
            result.push_str(&String::from_utf8_lossy(&out.stderr));
        }
        Ok(result)
    }
}
```

- [ ] **Step 4: 让 musk 成为 lib + bin(更新 Cargo.toml 加 `[lib]`)**

`backend/crates/musk/Cargo.toml` 加:
```toml
[lib]
name = "musk"
path = "src/lib.rs"
```
创建 `src/lib.rs`:`pub mod tools;`

- [ ] **Step 5: 运行测试确认通过**

Run: `cd backend && cargo test`
Expected: read_file 测试通过。

- [ ] **Step 6: 加 WriteFile / RunCommand 测试 + 提交**

补 WriteFile 测试(写临时文件再读回),RunCommand 测试(`echo hi`)。
```bash
git add backend/
git commit -m "feat(musk): basic tools (read_file/write_file/run_command)"
```

---

## Task 3: CLI 入口 —— `musk run "<task>"`

**Files:**
- Modify: `backend/crates/musk/src/main.rs`

**说明:** 用 clap 定义 `musk run <task>` 子命令。构建 Coder Agent,注册 3 个工具,调 `Agent::run`,打印 output + 轮次 + 工具调用记录。

- [ ] **Step 1: 写 main.rs(clap + agent)**

```rust
mod tools;

use std::sync::Arc;
use clap::{Parser, Subcommand};
use auto_ai_agent::{Agent, Client, Coder};
use auto_ai_client::AiClient;
use musk::tools::{ReadFile, RunCommand, WriteFile};

#[derive(Parser)]
#[command(name = "musk", version, about = "auto-musk — Forge-successor AI agent")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run an agent on a task.
    Run { task: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Run { task } => run_task(&task).await,
    }
}

async fn run_task(task: &str) -> anyhow::Result<()> {
    // 1. 连 daemon(自动发现/自启动)。
    let client: Arc<dyn Client> = match AiClient::new() {
        Ok(c) => Arc::new(c),
        Err(e) => anyhow::bail!("cannot reach auto-ai-daemon: {e}\n  Is `aaid` running? Start it or install it."),
    };

    // 2. 构建 Coder agent + 注册工具。
    let mut agent = Agent::new(Coder, client);
    agent.register_tool(ReadFile);
    agent.register_tool(WriteFile);
    agent.register_tool(RunCommand);

    // 3. 跑 ReAct 循环。
    println!("musk: running Coder on task: {task}\n");
    let result = agent.run(task).await?;

    // 4. 打印结果。
    println!("─── result ({} turns, {} tool calls) ───", result.turns, result.tool_calls.len());
    println!("{}", result.output);
    if !result.tool_calls.is_empty() {
        println!("\n─── tool calls ───");
        for tc in &result.tool_calls {
            println!("  {}: {} → {}", tc.tool, tc.args, tc.result.chars().take(100).collect::<String>());
        }
    }
    Ok(())
}
```

- [ ] **Step 2: 加 anyhow 依赖**

`backend/crates/musk/Cargo.toml` 的 `[dependencies]` 加 `anyhow = "1"`。

- [ ] **Step 3: 验证编译**

Run: `cd backend && cargo build`
Expected: 编译通过。

- [ ] **Step 4: 提交**

```bash
git add backend/
git commit -m "feat(musk): CLI 'musk run <task>' with Coder agent + tools"
```

---

## Task 4: 端到端验证 + README

**Files:**
- Create: `backend/README.md`

**说明:** 端到端跑一次(需 daemon + provider),写 README 说明构建/运行。这一步的"验证"依赖真实 LLM,若环境没有 provider,至少验证 CLI 能启动并给出清晰的 daemon-unavailable 错误。

- [ ] **Step 1: 确认 daemon 可跑**

Run(在 auto-ai 仓库): `cd ../auto-ai && cargo build -p auto-ai-daemon`
然后确保 `~/.config/autoos/ai-daemon.at` 存在(参考 `crates/ai-config/examples/daemon.at`),或设了 `ZHIPU_API_KEY` 等环境变量。

- [ ] **Step 2: 跑 musk(有 provider 时)**

Run: `cd backend && cargo run -- run "List the files in the current directory using the run_command tool, then read Cargo.toml."`
Expected: agent 调 run_command 列目录、读 Cargo.toml,返回汇总。轮次 ≥ 2,有 tool_calls 记录。

(若无 provider,跑 `cargo run -- run "anything"`,确认它给出清晰的 "cannot reach auto-ai-daemon" 错误,不 panic。)

- [ ] **Step 3: 写 `backend/README.md`**

包含:定位、依赖 auto-ai-agent、构建(`cargo build`)、运行(`musk run "task"`)、前置条件(daemon + provider 配置)、与 auto-ai 的关系。

- [ ] **Step 4: 提交**

```bash
git add backend/README.md
git commit -m "docs(musk): backend README + e2e verification"
```

---

## 验收标准(MVP 完成 = 本计划完成)

- [ ] `cd backend && cargo build` 成功,依赖 auto-ai-agent 正确解析。
- [ ] `cargo test`(在 backend)通过:read_file/write_file/run_command 工具测试全绿。
- [ ] `musk run "<task>"` 在有 daemon + provider 时,能跑完一个 ReAct 循环,调用工具,返回结果。
- [ ] 在无 daemon 时,给出清晰错误而非 panic。
- [ ] `backend/README.md` 说明清楚。

## 不在本期范围(明确排除)

- **前端打通**(v2 阶段 2):本期不碰 Vue 前端,只交付 CLI。
- **HTTP API / SSE 流式**(v2 阶段 2-3):本期 agent.run 是一次性同步返回,不做流式。
- **多角色 Workflow**(v2 阶段 4):本期只用单个 Coder profession。
- **RBAC / Spec Ledger / MCP**(v2 阶段 5-6):远期。
- **工具安全护栏**(run_command 白名单/确认):MVP 直接执行,后续加。

## 自检

- **Spec 覆盖**:Rust 骨架(Task 1)→ workspace + 依赖;工具(Task 2)→ 3 个 Tool 实现 + 测试;CLI(Task 3)→ musk run;端到端(Task 4)→ 验证 + 文档。覆盖 MVP 目标。
- **类型一致**:`Tool` trait(Task 2)来自 auto_ai_agent,签名与 auto-ai-agent 的 `tool.rs` 一致;`Agent::new(profession, client)`(Task 3)与 auto-ai-agent 的 `agent.rs` 一致;`Client` trait 经 `auto_ai_agent::Client` 重导出。
- **路径核验**:Task 1 Step 2 明确标注"用 cargo 报错校准 `../` 层数",不假设 —— 这是跨仓库 path 依赖最易错处。
- **无占位符**:每个步骤有完整代码或明确的核验指令。
