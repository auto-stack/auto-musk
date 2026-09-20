# auto-musk 项目概览

> 本 spec 基于 2026-08-14 全量代码扫描（后端 Rust + 双前端 + codegen），从代码提炼而非从文档推导。

## 定位

auto-musk 是 Forge 继任者——Rust 后端的 AI 编码 agent。既是 CLI（`musk run/chat`）也是 HTTP 服务（`musk serve` :8080），经 auto-ai-daemon（aaid）代理调 LLM，工具在进程内本地执行。

## 关键能力

1. **Agent 运行**：基于 `auto-ai-agent` 的 ReAct 循环（一次性 / 流式 SSE），8 基础工具 + 5 spec 工具 + 5 编排工具，path confinement 安全沙箱。文件工具 pi parity（PLAN-039）：edit_file 多编辑/模糊匹配/CRLF-BOM 往返、read_file 分页截断、输出字符边界安全截断。run_command pi parity（PLAN-040）：tokio 流式执行、可选超时杀进程树、超限全量落临时文件、非零退出码错误化、ToolUpdate SSE 实时进度。
2. **Spec 双落点**：结构化 ledger（`.autoos/specs.json` 6 区 + 状态机）+ 文件树知识层（`docs/specs/`，本目录）。
3. **Plans 动态执行**：`docs/plans/NNN-*.md` 文件树，5 态状态机（drafting→executing→execution_done→review_done→merged），merge 沉淀到 Spec。
4. **Relay 编排**：PipelineEngine 流水线 + TaskPlan DAG + 子会话（spawn_relay/dispatch/bring_in）。
5. **双前端 parity（PLAN-074–079）**：Auto 轨 `.at` 源（`src/front/*.at`）为全量单一真源，经 AutoUI 编译器平权交付双端目标：生成 Vue 3 SPA（`gen/front/vue/`）与原生 VM/Iced 桌面 UI（`auto run -r vm`）。Block/Message 组件、业务视图与全局外壳（App）在两端共享同一套数据契约与主题（`pac.at` 主题紫）。后端同时支持 AutoVM HTTP 服务、Rust merged 进程内服务与传统 `musk serve`（:8080）。

## 架构总览

```
auto-ai（LLM 层）               auto-lang / AutoUI（编译器与 VM）
  aaid daemon :17654              auto run / auto build（.at → Vue / VM 解释执行）
       ↑                               ↑
       |                          auto-musk（主项目 src/front/*.at）
  musk serve :8080 ←── backend/crates/musk（Rust axum / AutoVM HTTP / Merged）
       |                               ↑
       ├── 生成 Vue 3 目标（gen/front/vue/ :17200 / Vite）
       └── 原生 VM/Iced 目标（AutoUI MCP :17476 / 桌面图形终端）
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
