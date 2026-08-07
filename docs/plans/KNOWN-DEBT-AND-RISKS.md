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
| 018 | ~~`app_config` 的 `AAID_URL` env 覆盖在 a2r 产物中缺失~~ **✅ 已修复（Plan 021 B1，2026-08-07）**：`app_config.at` 的 `effective_daemon_url` 现读 `env::var("AAID_URL").ok()`（与 hw file < env < default 一致；391 D4 让方法链可解析）。`parity_app_config.rs::parity_effective_daemon_url` 断言两侧行为一致。 | `auto-src/app_config.at` |
| 018 | ~~`wiki.at` TreeNode file 节点 `modified = None`~~ **✅ 已修复（Plan 021 B2，2026-08-07）**：`wiki.at` 新增 `file_modified(entry)` 辅助 fn（`modified().ok() → duration_since(UNIX_EPOCH).ok() → let secs u64 = as_secs()` 抑制 cast）；build_tree file 节点 `modified` 取真实 mtime。`parity_wiki_http` 逐键比对（含 modified）。 | `auto-src/wiki.at` |
| 018 | a2r 输出验证器对 specs.at 报 `unbalanced parentheses (depth: 1)` 警告（编译通过、测试通过，疑为字符串字面量内括号被误判）。非阻断，记为待查。 | `auto-src/specs.a2r.rs` 转译输出 |
| 019 | ~~ag 流式 handler 即时错误走 200 SSE + error 帧~~ **✅ 已修复（§6.1，2026-08-06）**：`run_stream_handler`/`workflow_run_stream` 现在在 `mpsc_channel()` 前前置 `mode_exists`/`workflow_exists` 校验（与 hw fail-fast 一致），坏 mode / 坏 workflow → 400 JSON。契约测试 `ag_run_stream_bad_mode_returns_400` + `ag_workflow_run_stream_invalid_workflow_returns_400` 锚定。 | `src/auto_generated/server_stream.rs`（前置校验）/ `extern_impl.rs`（`mode_exists`/`workflow_exists`） |
| 019 | ~~conversation_stream 的 broadcast Receiver 存 side-table，客户端断开时条目不回收（累积泄漏）~~ **✅ 已修复（§6.2，2026-08-06）**：`conversations_subscribe` 返回 `BroadcastSub`（rx 包 `Arc<Mutex<Option<Receiver>>>`），被 `conv_event_stream` owned → stream drop（客户端断开）→ Arc 归零 → rx 析构。不再用 registry 存 receiver。测试 `broadcast_sub_drop_reclaims_receiver` + `conversation_stream_drop_reclaims_receiver` 断言 receiver_count 回落。 | `src/auto_generated/extern_impl.rs` `BroadcastSub`/`conversations_subscribe` |
| 019 | mpsc channel 缓冲 64 条：run/chat/workflow 流在流建立前爆发超过 64 事件时 `try_send` 静默丢帧。**与 hw 完全等价**（hw 三 handler 同样 `channel::<Value>(64)` + `try_send`，`server.rs:367/618/904`），非缺陷、无需修复，仅登记备忘。 | `src/auto_generated/extern_impl.rs` `mpsc_channel` / `server.rs:367,618,904` |
| 020 | **a2r spec trait 无 supertrait 语法（Send+Sync）**：`TaskPlanExecutor` 等 spec trait 无法声明 `Send + Sync` bound → 生成的 `execute` future 非 Send,`relay_task_plan_start` 无法用 `tokio::spawn`,改用独立线程 + current-thread runtime `block_on`(future 全程不跨线程)。行为与 hw `tokio::spawn` 等价(后台执行 + HTTP 立即返回),非功能阻塞。**根因在 auto-lang 侧(spec 语法 + codegen 决策,类比 391 D6),完全未在 auto-lang 登记** —— 待 auto-lang follow-up 立项解除;解除后可把 spawn 路径改回 `tokio::spawn`。 | `auto_generated/extern_impl.rs` `relay_task_plan_start`(独立线程 block_on)/ `auto_generated/task_plan_engine.rs` `TaskPlanExecutor` spec |
| 020 | **a2r 借用版本漂移(task_plan_engine 既有块)**：Plan 020 Phase H 期间发现,当前 auto-lang master 的 a2r 对 `self.field`(`String`)在 extern 调用点发射 `self.field.clone()`(owned),而之前提交的产物发射 `&self.field`(借用);`@T` 实参的 `&` 注入也有差异(`&&handoffs` vs `&handoffs`)。这导致 task_plan_engine.at **既有**块 re-transpile 与 `auto_generated/task_plan_engine.rs` 有 drift(编译/测试仍绿,仅 codegen 风格差异)。Phase H 新增的 `RelayTaskPlanExecutor` 块与既有无关、零 drift。**非阻断**;待 a2r 借用启发式稳定后,统一 re-transpile 全模块消除 drift。 | `auto-src/task_plan_engine.at` / `auto_generated/task_plan_engine.rs` |
| 022 | **Phase 7c RelayCard 为状态展示版**：`RelayCard.vue` 仅显示 relay run 的状态字段（run_id/flow/status/steps/summary/tokens），未接 `useRelay` composable 的完整交互（`subscribeToRun` 实时日志订阅 / `resolveGate` 审批 / `startRun`+`advanceRun` 命令启动）。原生 `RelayRunBox.vue` 强依赖这套 relay 接线，当前未接到生成工程。完整交互待 relay 后端接线计划补齐。命令解析（`/relay`/`/superpower`/`/spec1`）在 MentionInput 里透传原始文本（未路由到 startRun）。 | `src/front/components/RelayCard.vue` / `src/front/components/MentionInput.vue` |
| 022 | **Phase 7c professionOptions 用内置默认列表**：`forge_helpers.ts` 的 `DEFAULT_PROFESSIONS` 硬编码 9 个职业（assistant/advisor/.../gofer），因 `useAgentConfigs` composable 是空 stub（Harness agent-config 层未接线，configs 始终空数组）。@mention 下拉 + 高亮用此默认列表。待 Harness agent-config 接线后改为动态拉取。 | `src/front/forge_helpers.ts` / `web/src/composables/useAgentConfigs.ts`（stub） |
| 022 | **Phase 7c 流式 draft 走静态 markdown 渲染**：`useStreamingDocument` 增量 JSON 解析（plan §7.5 标记的高概率逃生舱）未做，流式 draft 用 `markdown { content: .store.current_draft }` 走 MarkdownRender 静态渲染（每个 delta 重渲染全文）。对短文本无碍，长流式输出可能有性能/闪烁问题。增量解析留作 §7.5 未来项。 | `src/front/chats_view.at`（流式 draft 块） |
| 022 | **Phase 7c user 消息 mention 高亮未做**：消息列表 user 分支用 `text .msg.content`（纯文本），未做 `@agent` 高亮（原生用 `v-html="renderMentions(msg.content)"`）。.at 无法表达 v-html（map_tag 无 html/raw tag），留作逃生舱 `UserMessage.vue` 未来增强。`renderMentions` helper 已就位（forge_helpers.ts）。 | `src/front/chats_view.at`（消息列表 user 分支） |
| 022 | **Phase 7c 逃生舱 ext 复制需显式声明**：codegen 的 ext 文件复制只看 widget 的 `use {}` 块，不递归逃生舱组件内部的相对 import。`MentionDropdown.vue`/`AgentAvatar.vue`（MentionInput 内部依赖）和 `forge_helpers.ts`（卡片组件内部依赖）必须在 `chats_view.at` 的 use 块显式声明（即使模板未直接使用），否则不被复制到 gen 工程导致 import 失败。属 codegen 设计限制，非缺陷，登记备忘。 | `src/front/chats_view.at`（use 块注释）/ `auto-lang ui_gen` ext 复制机制 |

## 📋 未来增强

| Plan | 描述 | 参考 |
|---|---|---|
| 018 | **休眠镜像 full parity**：`tools.rs`/`spec_tools`/`orch_tools`/`server_serve` 等 ag 镜像为简化 dormant（description/schema 文本与 hw 有差异 + execute 依赖 extern stub）。计划 §10.6 Phase 4 评估 + §13 C 类已文档化为"设计内的等价镜像，非缺陷"。full parity 需 `.view` 手术 + 元数据对齐，留待后续接线计划。**注：Plan 019 已完成 `server_stream` 接线；Plan 020 Phase G/H 已完成 `relay_driver`(drive_run/loop/step 核心循环)+ `server_serve`(drive_* 已搬出,仅余 serve/settings_link 休眠骨架)的 full parity 接线——这两个模块不再属休眠镜像,现仅 `tools`/`spec_tools`/`orch_tools`/`server_serve` 休眠。Plan 021 §8 评估后明确这 4 镜像留待未来:激活成本极高(本地 trait Tool 不兼容 auto_ai_agent::Tool / orch_tools 缺 2 工具 / descriptions 缩水 / 需跨 auto-lang 改转译器)、收益为零(生产 agent 硬编码 hw tools 无切换路径)。** | `src/auto_generated/{tools,spec_tools,orch_tools,server_serve}.rs` |
| 018 | **HTTP 层测试缺口**（§13 E1）：`/api/run*`、`/api/chats/.../stream`、`/api/conversations/.../stream` 无 HTTP 层测试。**注：Plan 020 Phase D/E 已补 `/api/forge/*` + `/api/settings-link` 8 项；Plan 021 已补 `/api/files/*`(parity_files 5)+ workspace 端点(parity_workspace_endpoints 4:chats DELETE/drift/rebuild/related)+ config 端点(parity_config_endpoints 3:roles/app-config/harness,AUTOOS_HOME 隔离)。剩余 `/api/run*`/chats stream/conversations stream 仍无 HTTP 覆盖(plan 019 流式范围,逻辑层已覆盖)。** | — |
| 391 | **多段路径 codegen**：`std::env::var("X")`（多段 `::`）parser 现可解析，但 codegen 对小写模块段（`env`）仍发点（`std.env.var`）。单段 `env::var` + `use.rust std::env` 可用。多段需改 rust.rs:3097-3121 让小写段也认 `::`。 | `auto-lang trans/rust.rs:3097` |
