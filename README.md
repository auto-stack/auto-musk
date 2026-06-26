# 🦌 Auto Musk

Auto-forge 的继任者 —— 一个 spec-driven、serial-agent 的 AI 编码助手。Rust 后端 + Vue 前端，LLM 能力经 [auto-ai-daemon](../auto-ai) 统一调度。

> 这是 AutoStack 生态的应用层。AI 资源管理（ApiSource / LLM 调用）已下沉到 `auto-ai-daemon`；auto-musk 聚焦 Forge 聊天、Spec Ledger、（未来的）Relay 编排。

## 架构

```
auto-musk/
├── backend/          Rust workspace（musk 二进制：run/chat/serve）
│   └── crates/musk/  lib + bin：tools / specs / chats / server / auth / mode
├── web/              独立 Vue3 + TS SPA（Chats + Specs + Login）
├── skills/           agent 技能库（brainstorming/writing-plans/...）
├── plans/            实施计划（001-009）
└── designs/          设计参考（前端 widget / 三成分拆分 / ...）
```

## 前置依赖

1. **Rust**（编译后端）
2. **Node.js + npm**（构建前端）
3. **auto-ai-daemon（`aaid`）** 在跑 —— 提供 LLM 调用
   - 安装：`cd ../auto-ai && cargo install --path crates/auto-ai-daemon`
   - 启动：`aaid`（监听 127.0.0.1:17654）
   - 配置：`~/.config/autoos/ai-daemon.at`（provider + model，参考 `../auto-ai/crates/ai-config/examples/daemon.at`）

## 构建与运行

### 后端

```bash
cd backend && cargo build --release
# 二进制在 backend/target/release/musk
```

### 前端（web/dist）

```bash
cd web && npm install && npm run build
# 产物在 web/dist（被 serve 托管，gitignored）
```

### 使用

**CLI（最直接）：**
```bash
# 单次任务
musk run "用 list_dir 列出当前目录，读 backend/Cargo.toml 并总结"

# 多轮流式对话（REPL）
musk chat
```

**Web（开箱即用）：**
```bash
musk serve                 # 监听 127.0.0.1:8080
# 浏览器打开 http://127.0.0.1:8080
```
`musk serve` 同时托管 web app（根路径 `/`）+ 所有 API（`/api/*`）。
首次需先 `cd web && npm run build` 构建 web/dist（否则浏览器空白，API 仍可用）。

**开发模式（前端热重载）：**
```bash
# 终端 1：起后端 API
musk serve
# 终端 2：起 Vite dev server（代理 /api → :8080）
cd web && npm run dev      # → http://localhost:3000
```

## 子命令

| 命令 | 作用 |
|---|---|
| `musk run "<task>"` | 单次任务，打印结果 + 工具调用 |
| `musk chat` | 多轮流式 REPL（逐 token 打印）|
| `musk serve [--addr 127.0.0.1:8080]` | HTTP API + web app 服务 |
| `musk professions` | 列出内置 profession |
| `musk modes` | 列出 agent mode |

## 主要能力

- **Forge 聊天**：多轮会话（JSON 持久化）+ SSE 流式 + 工具调用展示
- **Spec Ledger**：7 类 spec section、per-section 状态机、关系图（rebuild_relations）、派生状态（derive_statuses）、overview/drift-check、LLM 经工具读写 + 审批队列
- **工具集**：read/write/edit/search/list_dir/list_symbols/glob/batch_replace/run_command + 5 个 spec 工具
- **技能库**：brainstorming / writing-plans / executing-plans / TDD / systematic-debugging / requesting-code-review / verification-before-completion
- **配置体系**：mode（basic/coding/review）、agent roles、app runtime config

## 状态与计划

当前进度见 [`plans/009-parity-roadmap-vs-auto-forge.md`](plans/009-parity-roadmap-vs-auto-forge.md)。
- ✅ P0 Spec 派生层 / P1a Spec 工具 / P1b spec 审批
- 🔶 P1b WorkMode+errand（延后）/ P2 Relay 引擎（最大缺口，待做）
- ⬜ P3 Wiki / MCP

## 与相关项目的关系

- [`auto-ai`](../auto-ai)：auto-ai-daemon（LLM 资源）+ auto-ai-agent（Profession/Agent/Workflow）+ auto-ai-client
- [`auto-forge`](../auto-forge)：成熟参考实现（移植目标）
- [`auto-lang`](../auto-lang)：Auto 语言 + 工具链（早期 .at 版本受 AutoVM 成熟度阻塞，已转 Rust 后端）
