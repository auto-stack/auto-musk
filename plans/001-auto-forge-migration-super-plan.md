# 001 — auto-forge → auto-musk 移植总计划（Super Plan）

> **状态**：高层总纲。每个阶段的详细实施计划由对应的 `00x-*.md` 单独建立。
> **创建日期**：2026-06-16
> **维护**：阶段开始前用 `writing-plans` skill 生成详细计划；阶段完成后在此文件勾选进度。

---

## ⚠️ 架构演进通告（2026-06-17，重大修订）

**auto-musk（auto-forge 的 Auto 化）整体延期**，原因：发现 auto-forge 的 LLM 相关能力（ApiSource / Agent Profession / LLM 调用 Harness）应下沉为 **AutoOS 的全局资源管理能力**，而非做在单个 Agent 应用里。未来 AutoOS 会有多个 AI App（auto-musk 只是其一），它们共享底层 AI 能力，甚至通过 MCP 协议互相进行 AI 操作。

**新的三成分架构**（auto-forge 需先拆成这三部分，再考虑翻译成 Auto）：

| 成分 | 职责 | 当前 auto-forge 对应 |
|---|---|---|
| **auto-ai-daemon** | 系统级统一 AI 资源管理与调度服务（管 ApiSource、统一 LLM 调用、连接池/调度） | `provider/`（LLM 调用）+ `relay/config.rs` ApiSource + `relay/api.rs` api-sources 端点 + test-connection |
| **auto-ai-client** | AI 调用的客户端库（各 AI App 复用来调 daemon） | 新建——替换当前 5 处直接 `.chat_turn()` 调用为统一 client 接口 |
| **app**（auto-musk） | 应用业务（Forge 聊天、Spec、Relay、工具、Profession/Soul） | `forge/` + `relay/`（除 config 外）+ `mcp/` + `runtime/` |

**对原计划的影响**：
- 原阶段 5（Forge 聊天）、阶段 7（Relay）等"调用 LLM"的部分，**依赖 auto-ai-daemon + auto-ai-client 先就位**。
- 原阶段 8（MCP + 配置体系）中的 ApiSource/AgentConfig 部分**移出 auto-musk，归 daemon**。
- **前置工作变为：先在 auto-forge（Rust 原版）完成三成分拆分，再翻译成 Auto**。
- 详见 `designs/002-auto-forge-ai-capability-split.md`（拆分分析）。

**当前可继续推进的（不受架构演进影响）**：
- 前端骨架（已完成，纯 UI 不依赖 LLM 调用层）
- Spec 数据模型层（已完成 specs.at，待 AutoVM bug 修复后验证——Spec 是纯数据，不涉及 LLM）
- 等 AutoVM 基础缺陷（Plan 325）修复

**阻塞项（按优先级）**：
1. AutoVM 基础缺陷（Plan 325）——阻塞所有后端 Auto 代码
2. auto-forge 三成分拆分——阻塞 auto-musk 的 LLM 相关阶段
3. `#[api]` server panic（Plan 316）——阻塞前后端打通

---

## 1. 目标（原始，待架构演进后修订）

用 **Auto 语言**重写 [auto-forge](../auto-forge)（spec-driven、serial-agent 的 AI 编码助手），作为 auto-forge 的 Auto 实现版本。

## 1. 目标

用 **Auto 语言**重写 [auto-forge](../auto-forge)（spec-driven、serial-agent 的 AI 编码助手），作为 auto-forge 的 Auto 实现版本。

- **前端**：用 **a2vue** 转译成 Vue3 + TypeScript 工程（Vite + shadcn-vue）。
- **后端**：用 **AutoVM** 以脚本形式直接运行（不走 a2r→Rust 转译）。
- **策略**：**分层移植**——契约（API、数据模型、模块划分）照搬 auto-forge 以保证产品等价，实现用 Auto 范式重写。

**不追求一次性完成全量移植。** 本计划定义高层大步骤、依赖与顺序；每步再单独建立详细实施计划。

## 2. 已确认的关键决策

| # | 决策 | 选择 |
|---|------|------|
| 1 | 后端运行模式 | **AutoVM 脚本运行优先**（不走 a2r→Rust） |
| 2 | VM 能力补强归属 | 在 **auto-lang 仓库**实现（auto-musk 作下游消费者） |
| 3 | `#[api]` 在 VM 模式 | 补强后让解析器认识 `#[api]`，VM 运行时**自动注册为路由**——015-notes 式 `api.at` 无需改动即可 AutoVM 运行 |
| 4 | 前后端契约/调用模式 | 参照 **015-notes 最新模式**：前端 `use back.api: list_notes, ...`；后端 `#[api]` + `pub type` + `db.at` 业务层 |
| 5 | 移植策略 | **分层移植**（契约照搬，实现重写） |
| 6 | 计划文档 | 存 `auto-musk/plans/`，文件名 `00x-` 前缀；Spec 未来用 auto-forge 生成（本阶段不产出 Spec） |
| 7 | baseline 标记 | 本地打 tag `auto-musk-v0.1-baseline`，**不 push** |

## 3. 参考项目基线 commit

| 项目 | 角色 | commit | 日期 |
|------|------|--------|------|
| **auto-forge** | 移植目标（Vue+Rust 原版） | `03087fb` | 2026-06-16 |
| **auto-lang** | 语言/工具链/VM 补强目标 | `c05482a` | 2026-06-16 |
| **auto-coder** | 后端移植参考（早期 Auto→Rust 骨架） | `32dca46` | 2026-05-18 |

**baseline tag**：在 auto-forge 本地执行 `git tag auto-musk-v0.1-baseline 03087fb`（不 push）。
**未来同步**：`git diff auto-musk-v0.1-baseline..main`（在 auto-forge 内）取增量，对照更新 auto-musk。

## 4. 关键技术前提（已查证，非假设）

| 能力 | 状态 | 证据 |
|------|------|------|
| a2vue 前端转译 | ✅ 成熟 | 输出完整 Vite+Vue3+shadcn 工程；015-notes 实证 |
| AutoVM 直接跑脚本 | ✅ 成熟 | `auto x.at` |
| VM **客户端** SSE 解析（调 LLM 流） | ✅ 已实现 | auto-coder `coder/sse.at`（消费方向）+ auto-lang `http_stream.*`（`vm/ffi/stdlib.rs:2506-2633`） |
| `#[api]` 在 VM 自动路由 | ❌ 缺口 | 当前是代码生成专用注解；解析器报 `Unknown annotation 'api'`（`parser.rs:6588`），VM 运行时忽略 |
| VM 高级 HTTP server（路由/listen） | ❌ stub | `shim_http_server_*`（`stdlib.rs:1956-2065`）路由被丢弃、响应硬编码；但 tokio+spawn 骨架已存在 |
| VM TCP 流 flush | ❌ 缺口 | 无 flush 原语（`write_str` 用 `write_all`，`stdlib.rs:1813`） |
| VM **服务端** SSE 推送（给前端流式） | ❌ 缺口 | 无分帧/flush；auto-forge Forge 聊天 `/chats/{sid}/stream` 的核心特性 |

**SSE 方向澄清（重要）**：auto-coder 确实支持 SSE，但是**客户端解析方向**（agent 读 LLM 返回的 `text/event-stream`，`sse.at` 文件头注释明确：*"SSE parsing is inlined into runtime/agent.at"*）。
**服务端推送方向**（后端给前端推流，auto-forge 的 `/chats/{sid}/stream`）是**独立缺口**，列为阶段 2 的独立计划。

---

## 5. 大步骤总览

```
阶段 0  工程骨架与基线锚定（auto-musk）
   │
   ├─→ 阶段 1  补强 auto-lang：VM #[api] 自动路由 + HTTP server【auto-lang 仓库】
   │       │
   │       └─→ 阶段 2  补强 auto-lang：TCP flush + 服务端 SSE【auto-lang 仓库】
   │
   ↓（auto-lang 能力就绪）
阶段 3  auto-musk MVP：全栈骨架 + 最小 CRUD【auto-musk 仓库】
   │
   ├─→ 阶段 4  接入层移植：RBAC(最小) + App Shell + 只读视图
   │
   ├─→ 阶段 5  Forge 聊天移植：provider(LLM) + 会话 + 流式（依赖阶段 2 SSE）
   │
   ├─→ 阶段 6  Spec Ledger 移植：7类 spec 文件持久化 + 状态机
   │
   ├─→ 阶段 7  Relay 编排引擎移植
   │
   └─→ 阶段 8  MCP + 配置体系 + 收尾对齐
```

---

## 6. 各阶段详情

### 阶段 0 — 工程骨架与基线锚定

- **仓库**：auto-musk
- **目标**：建立可工作的目录骨架、文档约定，锚定三个参考基线。
- **范围**：
  - workspace 骨架（参照 015-notes / api-example 结构）：根 `pac.at`（workspace）+ `front/`（scene:ui, api 指向 back）+ `back/`（VM 后端）
  - README 说明定位、基线 commit、构建运行方式
  - 在 auto-forge 本地打 `auto-musk-v0.1-baseline` tag
  - 验证 `auto gen` / `auto run` 在骨架上可执行（即使只跑空壳）
- **产出**：`plans/001-*.md`（本文件）、骨架、tag、README
- **状态**：⬜ 待执行

### 阶段 1 — 补强 auto-lang：VM `#[api]` 自动路由 + HTTP server

- **仓库**：auto-lang
- **目标**：让含 `#[api]` 的后端 `.at` 能用 AutoVM 直接起 HTTP server，自动路由到 `#[api]` 函数。
- **范围**：
  - **解析器**：让 `#[api(method,path)]` 成为合法注解（去掉 `Unknown annotation 'api'`，`parser.rs:6588`），保留 method/path 到 AST。
  - **VM 路由注册**：扫描模块级 `#[api]` 函数，注册到 VM http server 路由表（method + path pattern）。
  - **填充 `shim_http_server_get/post/...`**（`stdlib.rs:1965`）：真正存储 `(path, handler)` 而非丢弃。
    - **核心难点**：VM 函数句柄如何作为异步 HTTP handler 被回调——需打通 VM task 与 tokio 的桥（本阶段的设计核心）。
  - **填充 `shim_http_server_listen`**（`stdlib.rs:2019`）：解析请求行/头/body、按路由表分发到 handler、序列化返回值。复用已有的 tokio+spawn 骨架（`stdlib.rs:2040`）。
  - 路径参数（`:id`）提取并作为函数参数注入；JSON body 解析为函数参数；返回值自动 JSON 序列化。
- **验收**：015-notes 的 `src/back/api.at`（5 个 CRUD `#[api]`）**不改一行**，用 AutoVM 起 server，curl 打通 5 个端点。
- **产出**：`plans/002-vm-api-routing-and-http-server.md`
- **状态**：⬜ 待执行

### 阶段 2 — 补强 auto-lang：TCP flush + 服务端 SSE（独立计划）

- **仓库**：auto-lang
- **目标**：为 auto-forge 的 Forge 聊天流式推送提供 VM 服务端 SSE 能力。
- **范围**：
  - 新增 `Net.tcp_stream_flush` native（当前 `write_str` 用 `write_all` 无显式 flush）。
  - 服务端 SSE 分帧辅助：`text/event-stream` 头 + `data: ...\n\n` 分帧，长连接不 close。
  - 评估是否需要 `http.response_sse` / chunked 辅助，或由 Auto 层手写分帧（基于 flush 原语即可）。
- **验收**：写一个最小 AutoVM SSE server，浏览器 EventSource 连接，收到持续推送的分块数据。
- **说明**：客户端 SSE 解析（调 LLM 流）已实现，本阶段只补**服务端推送方向**。
- **产出**：`plans/003-vm-server-sse-and-flush.md`
- **状态**：⬜ 待执行

### 阶段 3 — auto-musk MVP：全栈骨架 + 最小 CRUD

- **仓库**：auto-musk（依赖阶段 1）
- **目标**：建立"前端按钮调后端 `#[api]`"的最小可运行全栈，形成产品雏形。
- **范围**：
  - 后端：移植 auto-forge 核心领域类型的最小子集（User/Project/Session 骨架），用 `#[api]` 暴露几个只读/简单端点；业务层 `db.at`（内存 List，仿 015-notes）。
  - 前端：根 widget `App`（左侧导航 shell + 一个列表视图），`use back.api: ...` 调后端。
  - 打通 dev 流程：`auto run` 起后端 VM server + 前端 Vite dev，前端 `/api` 代理到后端。
- **验收**：浏览器打开，左侧导航可见，一个列表从后端 `#[api]` 加载并显示。
- **产出**：`plans/004-mvp-fullstack-skeleton.md`
- **状态**：⬜ 待执行

### 阶段 4 — 接入层移植：RBAC(最小) + App Shell + 只读视图

- **仓库**：auto-musk
- **目标**：移植 auto-forge 认证外壳和布局，形成可见的产品框架。
- **范围**：
  - RBAC 最小版：登录端点（`#[api]`，内存用户表 + 简化 token，先打通流程）；前端 LoginView + authFetch 注入。
  - App Shell：左侧 9-tab 导航 rail（仿 `auto-forge/frontend/src/App.vue:134`），view 切换。
  - 移植一个只读视图作为样板（如 Project Explorer：项目树浏览）。
- **验收**：登录后进入主界面，导航切换视图，Explorer 能浏览项目文件。
- **产出**：`plans/005-rbac-and-app-shell.md`
- **状态**：⬜ 待执行

### 阶段 5 — Forge 聊天移植：provider + 会话 + 流式

- **仓库**：auto-musk（依赖阶段 2 + 阶段 3，**技术风险最高**）
- **目标**：移植 auto-forge 核心——Forge 聊天循环。
- **范围**：
  - provider 层：移植 LLM 提供方抽象（`#[api]` handler 内用 `http_stream` 调 LLM 流式 + 复用 auto-coder `sse.at` 解析）。
  - 会话管理：Session/Message 持久化（JSONL 文件，仿 auto-forge runtime）。
  - 流式端点：`#[api]` 返回 SSE 流，把 LLM delta 推给前端（验证阶段 2 服务端 SSE）。
  - 前端 ChatsView + 流式渲染（StreamingRenderer）。
- **验收**：聊天框输入 → 后端转发 LLM → token 流式逐字显示在前端。
- **产出**：`plans/006-forge-chat-and-streaming.md`
- **状态**：⬜ 待执行

### 阶段 6 — Spec Ledger 移植

- **目标**：移植 7 类 spec 的文件持久化 + 状态机 + 关系图（auto-forge `forge/mod.rs` 后半，最复杂模块）。
- **范围**：`.ad` / `manifest.at` 解析、SpecItem 三层结构、22 种 Status、drift check、双向追溯。逐 section 类型移植。
- **说明**：auto-forge 该模块 ~1500 行 + 三种格式（legacy JSON / flat TOML / module）迁移，是最重的纯移植工作。
- **产出**：`plans/007-spec-ledger.md`
- **状态**：⬜ 待执行

### 阶段 7 — Relay 编排引擎移植

- **目标**：移植 serial-agent 流水线（handoff/gate/budget/checkpoint/flow/task_plan，auto-forge `relay/`）。
- **范围**：pipeline 状态机、handoff 文档传递、human gate、token 追踪、YAML flow + task_plan 多 flow 编排。逐子模块移植。
- **产出**：`plans/008-relay-engine.md`
- **状态**：⬜ 待执行

### 阶段 8 — MCP + 配置体系 + 收尾对齐

- **目标**：补齐 MCP server 暴露、配置体系，并做与 auto-forge 的最终对齐回归。
- **范围**：MCP（评估复用 auto-lang 已有的 `auto mcp`）、9 个 Profession、配置 CRUD（ApiSource/Agent/Skill）、端到端回归。
- **产出**：`plans/009-mcp-config-and-finalization.md`
- **状态**：⬜ 待执行

---

## 7. 阶段间依赖与并行性

- **阶段 1、2 是前置瓶颈**（在 auto-lang），决定后续所有后端工作能否展开。建议 1→2 串行或高度协同（2 的 SSE 分帧依赖 1 的 server 骨架）。
- **阶段 0 可与阶段 1/2 并行**（骨架不依赖补强）。
- **阶段 3 依赖阶段 1**；**阶段 5 依赖阶段 2 + 阶段 3**。
- 阶段 4/6/7/8 依赖阶段 3，彼此相对独立（建议 4→6→7→8 顺序以匹配数据依赖）。

## 8. 风险点

1. **VM 函数句柄作异步 handler 回调**（阶段 1 核心难点）——把 VM task 嵌入 tokio 异步 server 的桥梁，设计不当会卡住整个后端。阶段 1 详细计划需重点论证。
2. **服务端 SSE flush 时机**（阶段 2）——`write_all` 的刷出行为需实测，必要时给 `TcpStream` 包 `BufWriter` + 显式 flush。
3. **Spec Ledger 复杂度**（阶段 6）——auto-forge 最重的纯移植工作（~1500 行 + 三格式迁移）。
4. **auto-lang 上游变动**——补强期间 auto-lang 自身演进，需定期 rebase 补强分支。

## 9. 工作流约定

- 每个阶段开始前，用 **`writing-plans` skill** 生成对应的 `plans/00x-*.md` 详细实施计划，经确认后再动手。
- 阶段完成后，回到本文件把对应阶段的 `状态` 从 ⬜ 改为 ✅ 并补一行"完成摘要 + 实际 commit"。
- 与 auto-forge 同步：定期 `git diff auto-musk-v0.1-baseline..main`（在 auto-forge）查看上游变化，必要时更新基线 tag（递增 `auto-musk-v0.2-baseline` 等）。

## 10. 进度跟踪

| 阶段 | 状态 | 计划文件 | 完成摘要 |
|------|------|----------|----------|
| 0 — 骨架锚定 | ⬜ | `001-*.md`（本文件） | — |
| 1 — VM #[api]+http server | ⬜ | `002-*.md` | — |
| 2 — VM flush+服务端 SSE | ⬜ | `003-*.md` | — |
| 3 — MVP 全栈骨架 | ⬜ | `004-*.md` | — |
| 4 — RBAC + App Shell | ⬜ | `005-*.md` | — |
| 5 — Forge 聊天流式 | ⬜ | `006-*.md` | — |
| 6 — Spec Ledger | ⬜ | `007-*.md` | — |
| 7 — Relay 引擎 | ⬜ | `008-*.md` | — |
| 8 — MCP + 配置 + 收尾 | ⬜ | `009-*.md` | — |
