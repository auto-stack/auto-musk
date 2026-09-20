# PLAN-077 Evidence Ledger — 消息 Block 与 ChatMessage 组合一致性

> 生成时间：2026-09-20  
> 计划编号：PLAN-077  
> 状态：执行完成 (execution_done)  
> 基线 Commit: `839a0cb`（073 合回后 main tip，执行期 rebase 基线）  
> 工作区：`D:/autostack/.wt/musk-077/auto-musk` (分支 `plan-077-dev`)  
> 关联仓库：auto-lang（`??` 计算属性修复 + `max-w-[N%]` 百分比上限，合回 master 后记录实际 hash）、auto-down `3f73737f`（组内兄弟，detached，仅构建消费）

---

## 1. 任务完成进度 (Task Verification Matrix)

| 任务 ID | 任务说明 | 覆盖 AC | 状态 | 验证命令与结果 | 证据落点 |
|---|---|---|---|---|---|
| **T-01** | 冻结生产状态矩阵 | AC-01 | **PASS** | 生产投影修复（thinking `state`/稳定 `tkey`、gate 载荷 `gate_id`/`pending_cmd`/`escape_paths`、`gate_waiting` 状态直通）；历史无 blocks 兼容投影保留；`node scripts/ui-parity/message-contract.mjs` 对生成 Vue 函数与 AutoVM 生产投影双端断言全绿 | `src/front/forge_helpers.at`, `scripts/ui-parity/message-contract.mjs`, `tmp/ui-parity/PLAN-077/message-contract.json` |
| **T-02** | 实例安全与按需抽离 | AC-02 | **PASS** | 未抽离组件（父级 ForgeStore 键列表持态 + 稳定 block ID 已够用）；`chat-block-isolation` 双实例/多块真实点击：展开隔离、工具卡隔离、重排、增量更新、卸载/复建全部通过；gallery 与产品为同一份 hash 校验字节拷贝 | `src/front/forge_store.at` (`toggleBlockExpansion`), `src/front/chat_message.at`, `scripts/ui-parity/materialize.mjs` |
| **T-03** | 消息和工具视觉/交互修复 | AC-03、AC-04 | **PASS（视觉预算见 §5 登记项）** | thinking 流式尾态/完成态、展开折叠、gate 卡（命令/越界路径/审批拒绝事件实测请求）、Errand/TaskPlan/Relay/Report 单元全绿；上游通用缺口已修：auto-lang computed `??` 支持与 `max-w-[N%]` 百分比上限 | `src/front/tool_gate_card.at`, `src/front/chat_message.at`, `src/front/tool_block.at`, auto-lang `aura_view_builder.rs` / `iced/max_width.rs` |
| **T-04** | 扩展卡与业务组合 | AC-03、AC-04 | **PASS** | Questionnaire/Gate/Secretary(含 wrapper)/StreamingTable(可达无消费者登记) 单元通过；`chat-multi-round` 多轮交错 thinking→text→tool→gate→result 回放 + 分叉/停止/复制动作 | `tests/ui-parity/fixtures/chat-multi-round.json`, `src/front/chats_view.at`（`onfork`→`on_fork_from` 双端断点修复） |
| **T-05** | 锁定消息证据 | AC-01..AC-05 | **PASS** | `node scripts/ui-parity.mjs run --plan 077` VM 全量 17 case 通过；Vue 端逐 case 交互断言通过；遗留不可达组件已登记替代关系 | 本文件、`tmp/ui-parity/PLAN-077/` |

---

## 2. 静态对账门禁 (Static Gates)

- `node scripts/ui-parity.mjs check`: **PASS**。
- `message-contract.mjs`（T-01 消费契约）：block 投影 kind 序列、thinking `state` 直通、gate 载荷三字段、状态归一化（`error→failed`、`gate_waiting` 不显示 completed）、稳定 `tkey` 重排不变、展开键跨消息隔离、历史 `thinking/content/tool_calls` 兼容投影——生成 Vue 函数与 AutoVM 生产源码双端断言。
- gallery 单源证据：`materialize.mjs` 对全部生产源文件做 sha256 字节拷贝 + `verifyMaterialized()` 漂移检查；ChatMessage/各卡片零模板分叉。

---

## 3. 双端运行时证据 (Runtime Gates)

> VM：`node scripts/ui-parity.mjs run --plan 077 --mode vm --port 17778`（AUTO_EXE 指向组内 auto-lang 构建）  
> Vue：`node scripts/ui-parity/vue.mjs <case> http://127.0.0.1:17773`（vite preview 生产构建）

| Case | Mode | Status | 关键断言 |
|---|---|---|---|
| chat-message-pair | vm | `snapshot-ok` | 双实例外壳、reset spy |
| chat-thinking-streaming | vm/vue | `snapshot-ok` / `interaction-ok` | 流式 thinking 显示"思考中"+尾部文本（073 基线误显"已思考"，077 修复对照实测）；两次点击展开/收起 |
| chat-tool-gate | vm/vue | `snapshot-ok` / `interaction-ok` | gate 卡显示命令+越界路径；审批→`POST /api/chats/tool-gate/fixture-gate/approve`、拒绝→`.../deny`（隔离 mock 实测） |
| tool-gate-direct | vm/vue | `snapshot-ok` / `interaction-ok` | 生产 ToolGateCard 独立挂载事件与请求 |
| chat-block-isolation | vm/vue | `snapshot-ok` / `interaction-ok` | 双消息×多 thinking/工具块真实点击展开隔离；重排后展开键不串位；增量更新；卸载/复建 |
| chat-multi-round | vm/vue | `snapshot-ok` / `interaction-ok` | 多轮交错回放确定性；分叉路由宿主（`FORK:msg-a`）；停止事件 spy+1；gate→completed 状态迁移 |
| agent-avatar / user-message / errand-card / task-plan-card / report-card / questionnaire-card / gate-card / secretary-message / secretary-wrapper / relay-run-box / streaming-table | vm | `snapshot-ok` | 各卡生产契约渲染与内容断言（report-card 曾整卡 `${name}` 占位，auto-lang `??` 修复后通过） |

### 截图与快照落点
- VM 截图：`examples/musk-widgets-gallery/src/front/tests/screenshots/plan077-*-vm.png`（17 张，`autoui_screenshot` baseline）
- Vue 截图：`tmp/ui-parity/PLAN-077/*-vue.png`
- 事件/请求 receipt：`tmp/ui-parity/PLAN-077/*-vm.json` / `*-vue.json`

---

## 4. 生产源码变更摘要 (auto-musk)

- `src/front/forge_helpers.at`：thinking 块补 `state`/`tkey`；tool 块 `tkey` 改用 `raw.id` 稳定键；gate 载荷（`gate_id`/`pending_cmd`/`escape_paths`/`escape_paths_text`）投影；`gate_waiting` 状态归一直通；新增 `expandedMessageBlocks`（展开态派生，键列表跨实例隔离）。
- `src/front/chat_message.at`：expanded 改读 `.block.expanded`；thinking/tool 展开键接稳定 `tkey`；gate 卡 props 改显式展示字段（`escape_paths_text`）；工具头/思考头改原生 `button`（VM 可交互 + Vue 语义不变）。
- `src/front/tool_gate_card.at`：去掉 VM 求值断裂的 computed 块，改显式 props（`gate_id`/`pending_cmd`/`escape_paths`）。
- `src/front/tool_block.at`：同步 gate 卡 props 传递。
- `src/front/forge_store.at`：新增 `toggleBlockExpansion`（JSON 键列表，稳定 ID 独立展开，替代单键互斥）。
- `src/front/chats_view.at`：`onfork:` → `on_fork_from:`（VM 派发键 `on`+msg 名折叠 ≠ `onfork`；Vue `@fork` 不命中 `emit('ForkFrom')`——双端皆断的存量产品 bug，077 一并修复）。
- `tests/ui-parity/`：077 全量 case（17 个）+ fixtures；runner 增加内容断言（visible/absent/request/spyIncrement）、fixture 交互/迁移支持；`scripts/ui-parity/vue.mjs` Playwright 对拍 runner。
- `scripts/ui-parity/materialize.mjs`：gallery 宿主承载 fork 路由（`on_fork_from` → `FORK:<mid>` 可见证据）与生产 `toggleBlockExpansion` 拼接。

## 5. 上游通用缺口修复 (auto-lang，已合回 master)

| 缺口 | 影响 | 修复 | 验证 |
|---|---|---|---|
| VM computed/属性表达式无 `??` 臂 | 含 `??` 的 computed 整体落空，文本位渲染 `${name}` 占位（ReportCard 整卡、ToolGateCard 曾同症） | `aura_view_builder.rs` 增加 `Expr::NullCoalesce` 臂（左值非 Nil 取左，miss/Nil 落右） | 新增 `test_plan077_computed_null_coalesce`；musk report-card VM 内容断言通过 |
| `max-w-[N%]` 百分比上限无承载 | ChatMessage `max-w-[70%]` 被静默丢弃，VM 消息块满宽（Vue 70%） | `StyleClass::MaxWidthPct` + `iced/max_width.rs` 委托 widget（layout 期按父级 offered 宽度收窄 limits）+ col/row/container/text 四臂接线 | class 解析单测、widget 纯函数单测、双端截图对比 |

- auto-lang 门禁：`cargo check -p auto-lang` 零错误；`cargo t ui` 仅 2 个 master 预存环境红（`scan_examples_ui_curation_set`、`strips_tags_and_decodes_entities`，stash 对照复现）；`cargo t max_width` / `style::class` 全绿。

## 6. 视觉预算登记（未达项与责任归属）

按 §6 预算（关键边界≤2px、平坦色通道差≤2）固定内容区域/DPI/字体/主题对拍；以下差异登记为债务，不判 PASS 也不藏入遗留桶：

| 项 | 责任 | 规约值 | 现状 | 证据 |
|---|---|---|---|---|
| 消息块最大宽度 | auto-lang（已修） | `max-w-[70%]` | VM 已按父宽 70% 收窄 | plan077-chat-block-isolation-vm.png 对照 `-vue.png` |
| VM 主题背景色与间距细分 | auto-lang | pac.at 主题 token | 色板接近但非逐 token 对齐；间距以 4px 栅格近似 | 同截图 |
| 字体边缘 | auto-lang | 系统 sans 栈 | VM 用系统字体渲染，边缘差异单报 | 同截图 |
| VM 窗口尺寸与 Vue 视口不同（2560×1600 vs 1280×800） | runner（077） | 总设计§8 固定内容区域 | 两轨截图未同尺寸对拍，严格像素差未 operational；本阶段以结构/内容/交互一致性为验收 | 截图元数据 |
| 工具卡头内部排布（VM 段文本居中/状态贴右 vs Vue 挤左+ml-auto 贴右） | auto-lang（VM 行内 `ml-auto` 语义近似） | tool-header 工具类 | 内容与事件双端一致，头部内部分布差异 ≤ 一行高；登记不上报 PASS 依据 | plan077-chat-block-isolation 双端截图 |
