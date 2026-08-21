# 🦌 Auto Musk

Auto-forge 的继任者 —— 一个 spec-driven、serial-agent 的 AI 编码助手。Rust 后端 + Vue 前端，LLM 能力经 [auto-ai-daemon](../auto-ai) 统一调度。

> 这是 AutoStack 生态的应用层。AI 资源管理（ApiSource / LLM 调用）已下沉到 `auto-ai-daemon`；auto-musk 聚焦 Forge 聊天、Spec Ledger、Relay 编排、Wiki。

## 架构

```
auto-musk/
├── backend/          Rust workspace（musk 二进制：run/chat/serve）
│   └── crates/musk/  lib + bin：tools / specs / chats / server / auth / mode
├── web/              独立 Vue3 + TS SPA（Chats + Specs + Login）
├── skills/           agent 技能库（brainstorming/writing-plans/...）
└── docs/
    ├── plans/        实施计划（NNN-*.md，状态机驱动）
    ├── designs/      设计参考（前端 widget / 三成分拆分 / ...）
    └── specs/        Spec 模块树知识库（008 §5；PLAN-025 文件树浏览器）
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

### 前端 Auto 化（`.at` 源 → 生成 vue 工程）

除原生 `web/` SPA 外，本项目还有一条 **AutoUI `.at` 源**路线（[Plan 022](docs/plans/022-frontend-auto-ization.md)）：5 个视图（Login/Chats/Plans/Specs/Wiki）的 `.at` 源经 `auto build` 生成独立的 vue 工程（`gen/front/vue/`，gitignored），与原生 `web/` 达成行为+视觉一致。

```bash
# 从 .at 源重新生成 vue 工程（需 auto-lang 的 auto.exe）
auto build --gen-only        # 生成到 gen/front/vue/
cd gen/front/vue && pnpm install && pnpm build   # 验证可构建
```

- **源**：`src/front/*.at`（component fn + store + fn 模块单一真源）+ `src/back/api.at`（API 契约）
- **产物**：`gen/front/vue/`（30 组件 SFC + 5 store + ext fn 模块 + platform/ 平台实现 + lib/api.ts）
- **生成器**：[auto-lang](../auto-lang)（F1–F9 语言特性 + .at fn 模块转译 + a2vue golden；见 [Plan 028](docs/plans/028-block-autolang-full-migration.md)）
- **平台协议**（Plan 028，替代逃生舱）：平台强依赖收敛为协议声明，Auto 侧只声明接口、各后端提供实现——
  - `Sse.open(url, .Handler[, ctx])` / `Sse.close(h)`：SSE 流（平台层 JSON 预解析、ctx 注入、onerror 合成事件分发）
  - `Http.get/post/patch/put/delete`：HTTP 客户端（fetch 薄封装）
  - `component: Markdown from "platform:markdown"`：流式 markdown 渲染（markstream-vue + prismjs 实现挂载 `gen/…/src/platform/`）
  - 块组件样式随 `.at` `style {}` 块走（inject_styles 仅留 design token 与非块组件组）
- **剩余 TS**：mention 域（回调式 regex replace，F4 子集外）+ 各视图组 helpers（附录 A 分组后续立项）
- **状态**：🟢 Block 组全量原生化（forge/relay/questionnaire 纯函数 + SSE/HTTP 消费 + 样式均以 .at 为单一真源；148 项新旧对拍全等 + vue-tsc/vite 全绿）

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

- **Forge 聊天**：多轮会话（ConversationStore / jsonl 持久化）+ SSE 流式 + 工具调用展示 + spec 变更审批队列（approve/reject/reject-all）
- **Spec Ledger**：6 类 spec section（goals/architecture/designs/tests/reviews/reports；PLAN-024 移除 plans，Plan 升级为独立一等公民）、per-section 状态机、关系图（rebuild_relations）、派生状态（derive_statuses）、overview/drift-check、LLM 经工具读写 + 审批队列
- **Relay 引擎**：消费 `auto-ai-agent::orchestration`；`spawn_relay` + `bring_in` 编排工具；relay run 后台驱动 + 事件流；运行日志归档进 conversation（Flow 会话），前端 ChatsView 内联渲染（RelayRunBox）；独立 RelayView 已移除（PLAN-030）
- **Wiki**：WikiStore（CRUD + 树形导航 + 全文检索）+ `/api/wiki` + `/api/raw` + WikiView 前端
- **Plan（PLAN-024）**：Plan 一等公民（`docs/plans/NNN-*.md` + YAML frontmatter 状态机 drafting→executing→execution_done→review_done→merged）+ PlansView 一级导航（聊天/计划/规范/知识库）+ Plan→Spec merge 沉淀（`/api/plans/*`，复审通过后拆解进 Spec 6 区并归档）
- **工具集**：read/write/edit/search/list_dir/list_symbols/glob/batch_replace/run_command + 5 个 spec 工具 + 编排工具
- **技能库**：plan-driven-development（PLAN-030 契约技能）/ brainstorming / writing-plans / executing-plans / TDD / systematic-debugging / requesting-code-review / verification-before-completion
- **配置体系**：mode（superpower/basic/coding/review）、agent roles（Nicole/Ash/plan-dev...）、app runtime config、三种工作模式（superpower + relay flows + bring_in）
- **Plan 驱动开发流程（PLAN-030，canonical）**：`plan` flow 四相位（plan → execute[Human gate=计划确认] → review → document）全部由单角色 `plan-dev` 执行，以 `docs/plans/NNN-*.md` 计划文件为全量交接载体（driver 提取 `PLAN_FILE:` 标记路由后续相位），document 相位经 `merge_plan` 沉淀 Spec 6 区并更新 docs/specs 模块树。assistant 初筛：简单任务 chat 直做，复杂任务 `spawn_relay(flow_id="plan")`。旧 7 角色 spec 流水线（default/relay flow）标记 deprecated 保留对拍

## 状态与计划

当前进度见 [`docs/plans/009-parity-roadmap-vs-auto-forge.md`](docs/plans/009-parity-roadmap-vs-auto-forge.md)（总览见 [`001`](docs/plans/001-auto-forge-migration-super-plan.md#v2-进度跟踪)）。
- ✅ P0 Spec 派生层（per-section 状态机 + rebuild_relations + derive_statuses）
- ✅ P1a Spec 工具集（read/list/write/update/write_goals + 审批）
- ✅ P1b spec 变更审批（approve/reject/reject-all 端点）
- ✅ P2 Relay 引擎（driver/store/api/flows + orch_tools；原「最大缺口」已实质落地）
- ✅ P3a Wiki 模块（WikiStore + 前端 WikiView）
- ✅ 对话工具卡流式可见 + display_image + thinking 推理链路（2026-08-04）
- 🔶 P1b WorkMode 三态 + errand（部分；延后）
- ⏸️ P2b.3 checkpoint 回滚 + P3b MCP 层（降级为按需 Backlog，见 `009` §11）

## 与相关项目的关系

- [`auto-ai`](../auto-ai)：auto-ai-daemon（LLM 资源）+ auto-ai-agent（Profession/Agent/Workflow）+ auto-ai-client
- [`auto-forge`](../auto-forge)：成熟参考实现（移植目标）
- [`auto-lang`](../auto-lang)：Auto 语言 + 工具链（早期 .at 版本受 AutoVM 成熟度阻塞，已转 Rust 后端）
