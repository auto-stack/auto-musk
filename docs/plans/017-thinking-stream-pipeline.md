# Plan 017 — Thinking(推理过程)流式链路打通

> 跨仓库任务:由 auto-musk 发起,实现落在 **auto-ai** 仓库。
> 创建日期:2026-08-04
> 关联:auto-musk 的对话工具卡改进(本轮已完成 Step 1-4)

## 1. 背景与目标

### 1.1 现状(已实证)

auto-musk 前端**已有**完整的 thinking 展示逻辑:
- `ForgeMessage.thinking?: string`(`web/src/types/forge.ts`)
- `useForge.ts:288-290` 监听 `thinking` SSE 事件并累加
- `ChatsView.vue:198-207` 渲染折叠式"思考中"卡片(带脉冲动画)

但**后端整条链路断了**——reasoning 内容在 auto-ai 的 daemon 层就被丢弃:

| 层 | 文件 | 现状 |
|---|---|---|
| Anthropic provider 解析 | `auto-ai-daemon/src/provider/anthropic.rs:207-261` | `content_block_start` 只认 `tool_use`;`content_block_delta` 只读 `delta.text`;thinking 块走 `_ => {}` 丢弃 |
| OpenAI provider 解析 | `auto-ai-daemon/src/provider/openai.rs:218-258` | 只读 `delta.content`,从不读 `delta.reasoning_content` |
| daemon→client SSE | `auto-ai-daemon/src/server.rs:281-284` | 只有 `{type:"delta", text}`,无 reasoning 事件 |
| client 累加 | `auto-ai-client/rust-ref/src/lib.rs:135` | 只累加 `text` |
| agent StreamEvent | `auto-ai-agent/rust-ref/src/agent.rs:89-118` | 无 Thinking 变体(曾经有,被删为"dead variant") |

> CLI 的 `Block::Thinking`(`auto-ai-cli/src/chat_model.rs:58`)是**本地启发式**——在工具调用到达时把之前的 Answer 降级标记成 Thinking,不是真正的 reasoning 数据。

### 1.2 目标

把 LLM(glm-5.2 / deepseek-v4-pro 等思考型模型)的推理过程,从 provider SSE 流一路透传到 auto-musk 前端,渲染为可折叠的"思考中"卡片。

### 1.3 关键约束

- **当前 provider 配置**:zhipu / deepseek 都标记 `kind: anthropic`,走 `anthropic.rs` 解析器;local 走 `openai.rs`。
- **向后兼容**:旧版 client 不认 reasoning 事件时应静默忽略,不能报错。
- **不破坏现有 189 个测试**。

## 2. 实现方案

### 设计决策(已定)

- **传输方式**:新增 `reasoning` SSE 事件(与 `delta` 平级),不改 `on_delta: Fn(String)` 签名。向后兼容性最好。
- **字段兼容**:Anthropic 解析同时识别标准 `thinking` 块和非标 `reasoning_content`;实现时实测确认实际字段名。

### Step 1:daemon provider 层 — 解析 reasoning(anthropic.rs + openai.rs)

**文件**:`auto-ai-daemon/src/provider/anthropic.rs`

在 `process_json` 闭包(anthropic.rs:199-263)里新增 reasoning 解析:

1. `content_block_delta` 分支:读取 `delta.thinking`(标准 Anthropic thinking delta)和 `delta.reasoning_content`(部分国产模型非标字段)。有内容时调用新的 `on_reasoning(String)` 回调。
2. `content_block_start` 分支:识别 `content_block.type == "thinking"`,记录当前 index 处于 thinking 模式(用于后续 delta 路由)。

**文件**:`auto-ai-daemon/src/provider/openai.rs`

在 `process_json`(openai.rs:218-258)里新增:
- 读取 `choices[0].delta.reasoning_content` / `delta.reasoning`,有内容时调用 `on_reasoning`。

**文件**:`auto-ai-daemon/src/provider/mod.rs`

provider trait 的流式方法签名需要支持 reasoning 回调。当前 `on_delta: Arc<dyn Fn(String)>`。**新增**一个 `on_reasoning: Option<Arc<dyn Fn(String)>>` 参数(可选,旧调用点传 None 不受影响)。

> 实现时确认 provider trait 的确切方法名和签名,按最小改动原则扩展。

### Step 2:daemon server 层 — 推送 reasoning SSE 事件

**文件**:`auto-ai-daemon/src/server.rs`

`streaming_response`(server.rs:256-342):
- 给 provider 传入 `on_reasoning` 回调
- 回调触发时发 `{"type":"reasoning","text":"..."}` SSE 事件(与 `delta` 平级,server.rs:281 附近)

### Step 3:client 层 — 透传 reasoning 事件

**文件**:`auto-ai-client/rust-ref/src/lib.rs`

`complete_stream`(lib.rs:96-202):
- 回调 `on_event(Value)` 已经传原始 JSON,无需改签名
- 但需确认:当前 client 是否在透传前过滤了字段。如果是,需让 `reasoning` 类型的事件也透传给 `on_event` 回调。

> agent.rs:378 的回调 `ev.get("text")` 只读 text 字段。reasoning 事件需要在这里被识别并转成 `StreamEvent::Thinking`。

### Step 4:agent 层 — StreamEvent 加 Thinking 变体

**文件**:`auto-ai-agent/rust-ref/src/agent.rs`

1. `StreamEvent` 枚举(agent.rs:89-118)新增:
   ```rust
   /// A chunk of the model's reasoning/thinking output (separate from the
   /// final answer). Emitted by reasoning-capable models (glm-5.2,
   /// deepseek-v4-pro) before or alongside Delta. Consumers render this as
   /// a collapsible "thinking" section.
   Thinking { text: String },
   ```

2. agent loop 的流式回调(agent.rs:375-383)扩展,识别 reasoning:
   ```rust
   let resp = self.client.complete_stream(&req, Arc::new(move |ev| {
       if let Some(t) = ev.get("text").and_then(|t| t.as_str()) {
           on_delta(StreamEvent::Delta { text: t.to_string() });
       }
       if let Some(t) = ev.get("reasoning").and_then(|t| t.as_str()) {
           on_delta(StreamEvent::Thinking { text: t.to_string() });
       }
   })).await?;
   ```
   > 注:client SSE 里事件是 `{type:"reasoning", text:"..."}`,这里按 type 判断或直接读 reasoning 字段(取决于 client 透传结构,实现时确认)。

### Step 5:auto-musk server 层 — 转 SSE 事件

**文件**:`auto-musk/backend/crates/musk/src/server.rs`

`stream_event_to_json`(已在本轮 Step 1 改过)新增 Thinking 分支:
```rust
StreamEvent::Thinking { text } => json!({"type": "thinking", "thinking": text}),
```

> 前端 `useForge.ts:288` 已监听 `data.type === 'thinking' && data.thinking` — 字段完全匹配,无需改前端。

### Step 6(可选):auto-ai-cli 接入

**文件**:`auto-ai-cli/src/chat_model.rs` / `main.rs`

让 CLI 也消费真正的 Thinking 事件,替代当前的本地启发式(`demote_answer_to_thinking`)。优先级低,可后续做。

## 3. Worktree 实现流程

```bash
# 在 auto-ai 仓库创建 worktree
cd /d/autostack/auto-ai
git worktree add ../auto-ai-thinking -b feat/thinking-stream

# 在 worktree 里实现 Step 1-4(provider/daemon/client/agent)
cd ../auto-ai-thinking
# ... 编码 + cargo build + cargo test ...

# 实测确认字段名(Step 1 完成后):
# 用 aaid 起服务,发一个带思考的请求,抓 daemon SSE 原始报文
# 确认 zhipu/deepseek 的 anthropic 端点返回的 reasoning 字段名

# Step 5 回 auto-musk main 分支改 server.rs(一个函数加一个分支)
```

## 4. 验证计划

| 验证项 | 方法 |
|---|---|
| daemon 解析 reasoning | 抓 SSE 原始报文,确认 reasoning 事件出现 |
| agent StreamEvent | `cargo test` — 新增 thinking 相关单测 |
| 全链路编译 | `cargo build`(auto-ai workspace)+ `cargo build`(auto-musk) |
| auto-ai 测试 | `cargo test --workspace` 全通过 |
| 端到端 | auto-musk serve + headless Chromium,发思考型问题,确认"思考中"卡片出现 |

## 5. 风险

| 风险 | 应对 |
|---|---|
| zhipu/deepseek 的 anthropic 端点不返回标准 thinking 块 | 实测确认;兼容多种字段名(thinking / reasoning_content / reasoning) |
| 改 provider trait 签名影响多个调用点 | 用 `Option` 参数保持向后兼容;逐步迁移 |
| reasoning 内容很长撑爆前端 | 前端已有折叠式渲染(ChatsView.vue),默认折叠 |
| local 模型(ornith)无思考能力 | openai.rs 解析时 reasoning 字段为空即不触发,无副作用 |

## 6. 改动文件清单

**auto-ai 仓库(worktree `feat/thinking-stream`)**:
- `crates/auto-ai-daemon/src/provider/anthropic.rs` — 解析 thinking 块
- `crates/auto-ai-daemon/src/provider/openai.rs` — 解析 reasoning_content
- `crates/auto-ai-daemon/src/provider/mod.rs` — provider trait 加 reasoning 回调
- `crates/auto-ai-daemon/src/server.rs` — 推 reasoning SSE 事件
- `crates/auto-ai-client/rust-ref/src/lib.rs` — 透传 reasoning
- `crates/auto-ai-agent/rust-ref/src/agent.rs` — StreamEvent 加 Thinking

**auto-musk 仓库(main)**:
- `backend/crates/musk/src/server.rs` — stream_event_to_json 加 Thinking 分支(1 处)

## 7. 不做的事

- 不改 auto-musk 前端(已就绪)
- 不动 auto-lang / .at 规范源(只改实际编译的 rust-ref 源)
- 本轮不改 CLI 的本地启发式(Step 6 可选,后续)
