# auto-musk 项目概览

> 本 spec 基于 2026-08-14 全量代码扫描（后端 Rust + 双前端 + codegen），从代码提炼而非从文档推导。

## 定位

auto-musk 是 Forge 继任者——Rust 后端的 AI 编码 agent。既是 CLI（`musk run/chat`）也是 HTTP 服务（`musk serve` :8080），经 auto-ai-daemon（aaid）代理调 LLM，工具在进程内本地执行。

## 关键能力

1. **Agent 运行**：基于 `auto-ai-agent` 的 ReAct 循环（一次性 / 流式 SSE），9 基础工具 + 5 spec 工具 + 5 编排工具，path confinement 安全沙箱。
2. **Spec 双落点**：结构化 ledger（`.autoos/specs.json` 6 区 + 状态机）+ 文件树知识层（`docs/specs/`，本目录）。
3. **Plans 动态执行**：`docs/plans/NNN-*.md` 文件树，5 态状态机（drafting→executing→execution_done→review_done→merged），merge 沉淀到 Spec。
4. **Relay 编排**：PipelineEngine 流水线 + TaskPlan DAG + 子会话（spawn_relay/dispatch/bring_in）。
5. **双前端 parity**：原生 `web/`（Vue3 手写 SPA）+ Auto 轨 `.at` 源（`src/front/*.at` → `auto build` → `gen/front/vue/`）。Block 组件组已全量原生化（Plan 028）：纯函数/SSE/HTTP/样式以 .at 为单一真源，平台强依赖（markdown 渲染/SSE/HTTP）经平台协议声明（`platform:markdown`、`Sse.*`、`Http.*`），同源 .at 未来可直接复用于 VM/Rust 后端。

## 架构总览

```
auto-ai（LLM 层）               auto-lang（codegen 工具）
  aaid daemon :17654              auto build（.at → Vue SFC + Rust）
       ↑                               ↑
       |                          auto-musk（主项目）
  musk serve :8080 ←── backend/crates/musk（Rust axum）
       |                               ↑
       ├── web/ :3333（原生 Vue3 手写 SPA）
       └── gen/front/vue/ :3334（Auto 轨 .at 生成）
```

## workspace 数据隔离

每个工作区数据落 `{root}/.autoos/`：specs.json / chats.json / conversations/ / relay/ / wiki/ / raw/ / handoffs/ / task_plans/。Plans 例外落 `{root}/docs/plans/`。全局索引在 `~/.config/autoos/workspaces.json`。

## 关键依赖

| 依赖 | 路径 | 职责 |
|---|---|---|
| auto-ai-agent | `../auto-ai/crates/auto-ai-agent` | ReAct loop + ToolError + StreamEvent + Role/Skill |
| auto-ai-client | `../auto-ai/crates/auto-ai-client` | aaid daemon 连接（HTTP） |
| auto-atom / auto-val | `../auto-lang/crates/auto-atom` | .at 解析 + 值系统 |
| axum 0.8 | crates.io | HTTP + multipart + SSE |
| auto build | `auto-lang/crates/auto` | .at → Vue/Rust codegen |

## 配置

- `~/.config/autoos/apps/musk/config.at`：daemon_url / default_mode / serve_addr（运行时单源真相）
- `~/.config/autoos/ai-daemon.at`：aaid 监听 + provider/model 配置
- `~/.config/autoos/workspaces.json`：workspace 索引 + default
