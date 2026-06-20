# 005 — 流式 `musk chat`(A,优先级第二)

> **状态**:设计 + 实施计划。
> **仓库**:auto-musk(`backend/crates/musk/src/main.rs`)。
> **优先级**:2️⃣ 高 —— 让 demo 可用(逐 token 打印 vs 按 turn 等十几秒)。
> **前置**:auto-ai 的 `Agent::run_stream` + `StreamEvent` 已实现并合并。

## 目标

把 `musk chat` 从"按 turn 整段打印"升级为**逐 token 流式打印**,像 Claude Code 那样边想边输出。当前每轮要等模型完整生成才显示,长回答体验差。

## 现状

- `musk chat`(`main.rs::chat_loop`)用 `agent.run(input).await` —— 同步,返回完整 `AgentResult` 后才打印。
- `Agent::run_stream(input, on_event)` 已存在(auto-ai),产出 `StreamEvent::{Delta, Tool, Done, Error}`。
- `run_stream` 的 Delta 是逐 token 的(经 daemon SSE → client → agent)。

## 设计

`chat_loop` 改为调用 `agent.run_stream`,把 `on_event` 回调接到 stdout:
- `Delta { text }` → **直接 `print!` + flush**(逐 token,不打换行)
- `Tool { tool, args, result }` → 打印一条简短的工具调用行(缩进)
- `Done { result }` → 打印换行 + 轮次/工具数摘要
- `Error { message }` → 打印错误

关键:Delta 用 `print!`(不 `println!`)+ `io::stdout().flush()`,实现"流式"视觉。工具调用行另起一行,不打断正文流。

## Tasks

### Task 1: chat_loop 改用 run_stream
- Files: `backend/crates/musk/src/main.rs`
- [ ] `chat_loop` 的 `agent.run(input)` → `agent.run_stream(input, on_event)`,`on_event` 是个闭包/函数,按事件类型打印
- [ ] Delta: `print!("{text}")` + `io::stdout().flush()`
- [ ] Tool: `println!("\n  [tool] {tool}: {preview}")`
- [ ] Done: `println!("\n──── {turn} turn(s), {n} tool(s) ────")`
- [ ] commit

### Task 2: 多轮 Memory 确认不破坏
- [ ] 验证 run_stream 也把对话加进 Memory(跨轮上下文累积不丢)
- [ ] 手动测:连问两轮,第二轮模型记得第一轮内容
- [ ] commit

### Task 3: 错误/中断处理
- [ ] run_stream 返回 Err(如 max_turns/loop_detected)时打印友好错误,不 panic,会话继续(下一轮还能问)
- [ ] Ctrl-C 处理(可选:MVP 先让默认 SIGINT 行为)
- [ ] commit

### Task 4: 真实 LLM 验证 + 提交
- [ ] `musk chat` 跑一个长回答任务,确认逐 token 输出
- [ ] push rust-impl

## 验收
- `musk chat` 输入后,模型回答**逐字出现**,不再卡十几秒一次性蹦出。
- 工具调用有简短提示行。
- 多轮上下文不丢。
- 错误不崩会话。

## 注意
- 这是**终端流式**(stdout),不是 HTTP SSE。HTTP 端的 `/api/run/stream` 已是 SSE,前端 chats 页已支持。本计划只优化 CLI REPL 体验。
