# Known Debt & Risks

> 复审（plan-archiver Step 2.5）登记的 workaround / 一致性遗漏 / 已知限制 / 未来增强。
> 每行一项，按严重度分级。新增条目请在对应分级表追加。
>
> **a2r 转译器限制统一登记在 auto-lang `docs/plans/391-a2r-parity-debt-from-musk.md`**；
> Plan 391 的 D1-D6 已全部闭环并合入 auto-lang master（2026-08-06），
> auto-musk 侧对应的 .at 变通已逐项去除（D1/D2/D3/D5 + 此前的 wiki 两项）。

严重度图例：
- 🔴 **高风险** — 特定（非平凡）条件下可能引发 UB / 数据损坏
- 🟡 **一致性遗漏** — 功能正确，但代码未达计划自身的一致性目标
- 🟢 **已知限制** — 设计决策（非 bug），值得记录
- 📋 **未来增强** — 留待后续计划的优化 / 清理

---

## 🟡 一致性遗漏

_无。_

## 🟢 已知限制

| Plan | 描述 | 参考 |
|---|---|---|
| 018 | `task_plan.at` C1：`impl TryFrom<Node>` trait impl → `static fn from_node`（Auto 无 trait impl 语法）。**a2r 计划391 D6 已让 `impl Trait for Type` 报清晰错误**（不再静默反转），但 Auto 语言层面的 trait impl 支持仍是未来设计决策。parity 分别调 hw `try_from` / ag `from_node` 比行为。 | `auto-src/task_plan.at:272` / [391 D6](../../auto-lang/docs/plans/391-a2r-parity-debt-from-musk.md) |
| 018 | `app_config` 的 `AAID_URL` env 覆盖在 a2r 产物中仍缺失：a2r **计划391 D4 已让 `env::var(...).ok()` 方法链可解析**，但 app_config.at 的 extern 委托路径尚未改用 env 覆盖（需接线侧改动，非 a2r 阻塞）。`parity_app_config.rs::parity_effective_daemon_url` 固定当前分歧行为。 | `src/app_config.rs:153` / [391 D4](../../auto-lang/docs/plans/391-a2r-parity-debt-from-musk.md) |
| 018 | `wiki.at` TreeNode file 节点 `modified = None`：D1 闭环后 `size` 已取真实 metadata.len()，但 `modified` 涉及 `duration_since(UNIX_EPOCH)` 方法链（闭包内 `.len()` 仍 cast），暂留 None。tree 结构/排序/size 是功能性面，parity 测试已断言。 | `auto-src/wiki.at` |
| 018 | a2r 输出验证器对 specs.at 报 `unbalanced parentheses (depth: 1)` 警告（编译通过、测试通过，疑为字符串字面量内括号被误判）。非阻断，记为待查。 | `auto-src/specs.a2r.rs` 转译输出 |
| 019 | ~~ag 流式 handler 即时错误走 200 SSE + error 帧~~ **✅ 已修复（§6.1，2026-08-06）**：`run_stream_handler`/`workflow_run_stream` 现在在 `mpsc_channel()` 前前置 `mode_exists`/`workflow_exists` 校验（与 hw fail-fast 一致），坏 mode / 坏 workflow → 400 JSON。契约测试 `ag_run_stream_bad_mode_returns_400` + `ag_workflow_run_stream_invalid_workflow_returns_400` 锚定。 | `src/auto_generated/server_stream.rs`（前置校验）/ `extern_impl.rs`（`mode_exists`/`workflow_exists`） |
| 019 | `conversation_stream` 的 broadcast `Receiver` 存 side-table：`broadcast_recv` 收事件时 remove + re-insert、Closed 时不再 re-insert（条目回收）。但客户端断开时 `rx` 卡在 `recv().await`，SSE 流 future 被 axum drop 后 owned `rx` 随之析构——残留仅"被 remove 但卡在 future 里"的 receiver（随 future drop 回收）。实际泄漏面小，conversation 事件量低，影响有限。干净回收需 Drop guard 挂在 .at 流上（转译器不可表达）。低优先级。 | `src/auto_generated/extern_impl.rs` `conversations_subscribe`/`broadcast_recv` |
| 019 | mpsc channel 缓冲 64 条：run/chat/workflow 流在流建立前爆发超过 64 事件时 `try_send` 静默丢帧。**与 hw 完全等价**（hw 三 handler 同样 `channel::<Value>(64)` + `try_send`，`server.rs:367/618/904`），非缺陷、无需修复，仅登记备忘。 | `src/auto_generated/extern_impl.rs` `mpsc_channel` / `server.rs:367,618,904` |

## 📋 未来增强

| Plan | 描述 | 参考 |
|---|---|---|
| 018 | **休眠镜像 full parity**：`tools.rs`/`spec_tools`/`orch_tools`/`server_serve`/`relay_driver` 等 ag 镜像为简化 dormant（description/schema 文本与 hw 有差异 + execute 依赖 extern stub）。计划 §10.6 Phase 4 评估 + §13 C 类已文档化为"设计内的等价镜像，非缺陷"。full parity 需 `.view` 手术 + 元数据对齐，留待后续接线计划。**注：Plan 019 已完成 `server_stream` 的 `.view` 手术 + 全部 6 个 handler（非流式 run/workflow_run + 流式 run_stream/workflow_run_stream/chat_stream/conversation_stream）接线，该模块不再属休眠镜像。** | `src/auto_generated/{tools,spec_tools,orch_tools,server_serve,relay_driver}.rs` |
| 018 | **HTTP 层测试缺口**（§13 E1）：`/api/run*`、`/api/settings-link`、`/api/chats/.../stream`、`/api/conversations/.../stream`、`/api/files/*`、全部 `/api/forge/*` 无 HTTP 层测试（🔴 手写 handler + forge 独立）。 | — |
| 391 | **多段路径 codegen**：`std::env::var("X")`（多段 `::`）parser 现可解析，但 codegen 对小写模块段（`env`）仍发点（`std.env.var`）。单段 `env::var` + `use.rust std::env` 可用。多段需改 rust.rs:3097-3121 让小写段也认 `::`。 | `auto-lang trans/rust.rs:3097` |
