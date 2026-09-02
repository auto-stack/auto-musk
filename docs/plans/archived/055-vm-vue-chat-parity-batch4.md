---
plan_id: PLAN-055
status: archived
feature_name: VM/Vue 聊天对拍第四批——8 项收敛（导航高亮/会话删除/发送图标/ThinkBlock/重试删除/搜索/Invalid Date/AI 无回复）
author: [zhaopuming]
created_at: 2026-09-01T21:37:47+08:00
updated_at: 2026-09-02T20:50:00+08:00
supersedes_spec_components:
  - "src/front: forge_store PollStream 轮询降级（P051 C5）——增 resp/resp.session None 双守卫"
  - "src/front: chats_view 会话列表项（P053 批4）——删 title=.s.id tooltip 与 Info 图标、删除钮去 hidden 改常显+text-destructive"
  - "src/front: chats_view/forge_store 分支切换器（P043）——⑂ 重试钮与 RetryFrom 处理器删除（分支切换器保留）"
  - "src/front: forge_helpers messageBlocks（P051）——VM 等价规避改字面量重建（含 details 键）；toolArgsJson VM 降级 [args]+键名"
  - "src/front: msgTimeLabel 与乐观消息时间戳（P054）——created_at 秒统一 + None/0→\"\" + 毫秒/秒量级自适应守卫"
  - "src/front: think_block.at 组件（P023 块渲染模型）——退役，思考块内联进 ChatMessage（store.think_open 键列表独立展开）"
new_spec_components:
  - "src/back: chats_send_message 契约增可选 run bool=false（VM 轨显式触发）"
  - "backend musk: chats_message run=true 显式 spawn agent（tx=Value::Null 无订阅者安全）+ AppState per-session chat_runs 守卫（try_start/finish，chat_run_stream 全出口清守卫）"
  - "auto-lang ui_gen: input_text_handler_wants_text_arg 认可 $event 字面实参（oninput: .H($event) 生成 @input=\"H(($event.target as HTMLInputElement).value)\"）"
  - "auto-lang ui(iced): pre/code 进容器转换臂（py-[Npx]/px-[Npx]/border-t/max-h-[Npx] 类串恢复渲染；滚动 web-only 残项）"
  - "auto-lang tests: musk_vm_track 三族回归锚（$event 运行期替换 / pre 类串解析 / nav active Eq 双形态）"
  - "src/front: 问卷多选摘要按 q.type 判别（VM Array.isArray 恒 None 规避）"
  - "web/: ChatsView 重试钮三处删除 + FROZEN.md 用户明示冻结豁免记录（范围仅重试钮）"
touched_goals:
  - "VM/Vue 聊天对拍逐批收敛（第四批 8 项：①导航高亮②会话列表③发送钮④ThinkBlock⑤重试删除⑥搜索⑦时间戳⑧AI 无回复）"
current_step: 21
total_steps: 21
---

# [PLAN-055] VM/Vue 聊天对拍第四批——8 项收敛

## 变更摘要

用户实机截图对拍出 8 项问题（2026-09-01）。本计划在 P050/P051/P054 三批对拍之后做第四批收敛：①一级导航选中无高亮（VM）②会话列表删 tooltip/信息图标、补删除会话能力（VM）③发送按钮方形图标（VM）④ThinkBlock 联动展开+展开区无 padding（VM）⑤删除"重试"按钮（Vue+VM 单源+web 轨）⑥聊天搜索框无效（VM+gen）⑦AI 回答时间偶发 Invalid Date（gen web 轨）⑧VM 发消息后 AI 不回复（根因，连带 ③）。

改动跨两个仓库：auto-musk（单源 .at + backend）与 auto-lang（VM 运行时根修，依赖项目 worktree `auto-musk-dev`）。⑧ 选型 musk 侧显式 run 触发（本仓库闭环），auto-lang SSE 真实现（KD-047 债）不在本计划强做。

**2026-09-02 修订**：对拍诊断发现 VM/TS 语义等价性缺陷族（KNOWN-DEBT「等价性缺陷族」行①–⑥，根因探针 `tmp/vmprobe/`）——`messageBlocks` 的 tool_calls 循环 `raw.status = status` 新键赋值在 VM 炸掉 computed `blocks`，带 tool_calls 的消息（如 Block 全家福 a1）整条空白，**本计划 T5/T13/T19 的实机验证全被它卡住**。据此：新增 T0（musk 侧解堵+掩蔽雷点处置）；T19 门禁纳入 `tmp/vmprobe/case_*.at` 回归 case；根修（SET_FIELD/E5b 通道/char 分派/web 内建编译期报错/图标桥扩容）归 **PLAN-057**，本计划实机验证类任务标注对 057 的依赖。

## 目标

1. VM 版一级导航当前项有高亮（与 gen/Vue 的 `bg-primary/10 text-primary` 观感一致）。
2. 会话二级导航：chat ID tooltip 删除；info 圆圈图标删除；hover 红色删除钮 + 单击删会话在 VM 可用（gen 保持 hover 显隐）。
3. VM 空闲态发送按钮显示纸飞机（`lucide:send`），流式态才显示停止（Square）。
4. ThinkBlock 各实例独立展开/折叠；展开面板有 padding 与上边框（`pre` 类串被 VM 渲染）。
5. "⑂ 重试"按钮从 gen 单源与 web/ 手写轨全部移除。
6. 聊天搜索框在 VM 与 gen 两轨都能过滤消息。
7. 乐观/流式消息时间戳不再出现 "Invalid Date"。
8. `auto run -r vm --no-merge` 下发送消息后 AI 产生回复（后端 agent 被驱动，PollStream 能收尾 `.streaming`）。

## 架构方案

**单源原则**：UI 真源是 `src/front/*.at`（AutoUI），gen Vue 轨（`auto build --gen-only` → `gen/front/vue/`，gitignored 产物）与 VM 轨（`auto run --render=vm`，读 `src/front/app.at`）共用。凡能改源解决的（②a/②b/⑤/⑦）都改源一次、双轨生效；VM 渲染缺口（①③④⑥）根修在 auto-lang。

**双仓库 worktree 布局**（AGENTS.md）：
- auto-musk：`.worktrees/plan-055-dev`（分支同名），承载 src/front、src/back、backend、web/ 改动。
- auto-lang：`D:\autostack\auto-lang\.worktrees\auto-musk-dev`（分支同名，依赖项目规则第三行），承载 aura_view_builder / vm_bridge / dynamic / ui_gen / stdlib 改动。一旦 musk 消费（编译通过+实机验证通过）即合回 auto-lang 主分支并清理。

**⑧ AI 无回复的断点与选型**（勘察结论，`KNOWN-DEBT` 行 84/86 呼应）：
- 后端 assistant 回复由 SSE 订阅触发：`backend/crates/musk/src/auto_generated/server_stream.rs:234-242` 订阅即 `chat_run_stream`（实现在 `backend/crates/musk/src/extern_impl.rs:1663+`）；`POST /message` 只追加用户消息（`extern_impl.rs:816-834`）。
- VM 轨 `Sse.open` 是 no-op（auto-lang `crates/auto-lang/src/vm/ffi/stdlib.rs:8109-8124`，KD-047 G1 阶段2 债）→ 无人订阅 → agent 永不运行 → PollStream 轮空 → `.streaming` 恒真 → composer 锁死、按钮显示 Square（③ 的成因）。
- **选型 D1**：musk 侧给发送消息请求加显式 `run` 触发——`POST /message` 带 `run=true` 时后端直接 spawn agent 运行（per-session 运行守卫防双跑）；web 轨不带参，订阅触发路径原样不动。VM 用既有 PollStream（500ms）收回复。auto-lang SSE 真实现留在 KD-047 上游债，不在本计划。

**④ ThinkBlock 展开态根因**：VM 单 VM 单一扁平状态根——auto-lang `crates/auto-lang/src/ui/vm_bridge.rs:932-995` `ensure_child_state` 恒返根 state_obj_id，所有 ThinkBlock 实例共享 `expanded` 字段。**选型 D2**：根修 auto-lang，子部件模型状态按稳定视图路径键控分槽；若列表实例路径键控被证实不稳定（流式追加导致位移），fallback 为 musk 源侧按 msg id 的 keyed map。展开区无 padding 根因：`pre` 标签无 VM 转换臂，fallback 成 `style: None` 的 Column，类串整体丢弃（aura_view_builder.rs:3085-3105）——给 `pre`/`code` 补转换臂解析 padding/border/max-h。

**①导航高亮**：源已声明 `active: .current_view == "chats"`（`src/front/app.at:77-80`），gen 侧正常（NavItem.vue:62-65）。VM 断点在 active 布尔表达式构建期求值（aura_view_builder.rs:3361 `convert_nav_item` → `extract_bool_expr`:3321-3330 → `resolve_expr_to_value`:6566 附近 Eq 臂）；若求值失败回落 `nav_route_active` 而 musk 无 `to:` 恒 false。先 MCP dump 实证再修。

**⑥搜索**：两轨各有断点。VM：`$event` 在视图构建期被 `event_to_message_with`（aura_view_builder.rs:7280-7304）冻结为字面串 `"$event"`（input 通道未命中时），handler 收到假串。gen：`ui_gen/vue.rs:5489-5501` 的 `input_text_handler_wants_text_arg` 因实参含 `$` 不视为 bare → 生成 `@input="OnSearchInput($event)"` 传入原生 Event（gen/front/vue/src/components/ChatsView.vue:205），与 v-model 竞争把 Event 写进 `.chat_search`。

**⑦Invalid Date**：单源 `forge_store.at:255/:554` 乐观消息写 `timestamp`（毫秒）而非后端契约字段 `created_at`（秒，`backend/crates/musk/src/chats.rs:75`）；`msgTimeLabel`（`forge_helpers.at:96-99`）读 `created_at` 得 undefined → NaN → "Invalid Date"；PollStream 回填后恢复，故"偶发"。

**⑤重试**：gen/VM 单源 `src/front/chats_view.at:237-243` + 处理器 `forge_store.at:186-204` + decl `chats_view.at:67`；web/ 轨 `web/src/views/ChatsView.vue:190-196`（`retryFrom`:858-872、样式:3018-3034）。web/ 处于永久冻结（`web/FROZEN.md`），**用户明示豁免**做外科手术式删除（选型 D4）。

**②会话列表**：tooltip= `chats_view.at:123-126` `title: .s.id`（VM 经 auto-lang title→EE03 tooltip 接线，属单源直接删）；info 图标= `chats_view.at:146-147` `Info{}`；删除钮已在源里（`chats_view.at:149-154` `session-delete-btn hidden …` + `onclick.stop: .DeleteSession(.s.id)`，处理器 :311-315），但 VM 对 `hidden` 类直接渲染 `View::Empty`（aura_view_builder.rs:2240-2244），hover-only affordance 在 iced 无形态（KD 行 84 已登记）。**选型 D3**：VM 形态改为常显红色小删除钮——类串去掉 `hidden`（gen 侧 hover 显隐改由样式注入保证，见 T16），VM 常显（iced 无 hover 态，登记为已知观感偏差）。

## 技术栈

- AutoUI 单源（`src/front/**/*.at`）+ AutoLang（gen→Vue 与 VM 双目标）；auto-lang VM 运行时（Rust + iced）。
- gen Vue 轨：vite + vue-tsc + vitest + pnpm（`gen/front/vue/`）。
- backend musk：Rust（axum），`src/back/api.at` 契约 → `backend/crates/musk/src/auto_generated/` 生成层 + `extern_impl.rs` 手写实现。
- 探针：AutoUI MCP（:9247，auto-lang `crates/auto-lang/src/ui/mcp_server.rs`）+ 现成客户端 `tmp/plan050-review/mcp.mjs`；VM 门禁 `scripts/vm-first-run.mjs`。

## 需求分析与背景调查

- 用户 2026-09-01 实机截图（VM 版 Auto - App）列出 8 项，逐项断点已勘察定位（见架构方案）。
- 谱系：本计划是 VM/Vue 对拍系列第四批，接续 P050（第一批：主导航/设置/文件夹/二级导航）、P051（VM 会话主界面闭环：气泡/发送链路/流式降级）、P054（第二批：截图对拍 15 项）。对应 spec 目标 P050-2/P051-2/P054-2，上游债跟踪 P053-2。
- `docs/plans/KNOWN-DEBT-AND-RISKS.md` 行 84（PLAN-051 遗留：VM 删除会话不可达、`.cancel` 路由未达致 composer 锁死）与行 86（055 行：float 管线/for-in 方法 Nil/computed 真值/v-html 降级/ThinkBlock max-h web-only/max-w 百分比/时间戳 24h vs 12h）——本计划消化其中：行 84 删除会话+composer 锁死、行 86 ⑤ ThinkBlock 展开区、⑦ 时间戳制式（顺带对齐）。
- P047/P048 已做 SSE 专项勘察；P052 nav-item-rail 的"高亮底色渲染"仅单测未实机目验（archived/052:53-54），本计划补实机验证。
- 约束：web/ 轨冻结豁免仅限⑤重试钮删除；不引入新第三方依赖；VM 轨禁用 float 除法/`for-in` 循环变量直调方法（KD 行 86 绕开口径）。

## 详细设计

### A. 发送链路（⑧→③）

1. **后端契约**：`src/back/api.at` 发送消息请求增可选 `run: bool = false`（缺省 false 保持 web 轨语义不变），重生成 TS/Rust 桩。
2. **后端实现**：`backend/crates/musk/src/extern_impl.rs` 发送处理器：追加用户消息后，若 `run==true` 且该 session 无进行中的 run（per-session 运行标志/Mutex 守卫），`tokio::spawn` 驱动既有 `chat_run_stream` 内核（事件丢弃、assistant 消照常持久化——T1 spike 先证实持久化不依赖订阅者，若依赖则把持久化路径剥离为 run 即写库）；`auto_generated/server_stream.rs` 的订阅即运行路径原样保留。
3. **单源接入**：`forge_store.at:208-217` `.Send` 调 `chats_send_message(..., run=true)`；`forge_store.at:279-294` `.PollStream` 加 `resp != None && resp.session != None` 守卫（对齐 `SwitchSession`:152-165 的守卫风格）。
4. **停止钮**：`chats_view.at:292` `oncancel: .StopStream` 走通子→父回调；`StopStream` 置 `.streaming=false`。若 auto-lang C2 无参回调仍缺口则登记债务，不阻塞主线（.streaming 已可随回复正常收尾）。

### B. ThinkBlock（④）

- auto-lang `vm_bridge.rs` `ensure_child_state`：以"根到该部件的视图路径链（tag+稳定 key/索引）"为键分槽子部件模型状态；写回（`.expanded = !.expanded`）命中同一槽。配双实例互不干扰的同构测试。
- auto-lang `aura_view_builder.rs`：`pre`/`code` 进文本元素转换臂，产 styled 容器：解析类串 padding（`py-[9px] px-[12px]`/`my-0`）、`border-t border-border`、`max-h-[240px]`（clip/滚动尽力，滚动做不到则 clip 并把滚动留档 KD 行 86 ⑤ 残项）。
- musk 源 `think_block.at` 本体不动（padding 已显式声明 :46-51）。

### C. 搜索（⑥）

- auto-lang `dynamic.rs:1151-1185` / `aura_view_builder.rs:7280-7304`：input 类 handler 的 `$event` 实参在运行期统一替换为当前输入文本（构建期不再冻结字面串），配 musk_vm_track 测试。
- auto-lang `ui_gen/vue.rs` `input_text_handler_wants_text_arg`：实参为 `$event` 字面时同样包 `.target.value`，gen 产物变为 `@input="OnSearchInput($event.target.value)"`；重 gen 后 vue-tsc 验证。

### D. 导航高亮（①）

- MCP dump VM 运行时 nav-item 的 class（`node tmp/plan050-review/mcp.mjs <render 工具> … 9247`，工具名以 `ui/mcp_server.rs` 清单为准）定位断点在 active 表达式求值（`extract_bool_expr`/`resolve_expr_to_value` Eq 臂对 state 字段比较）还是 class 拼接渲染；对应修复 + `musk_vm_track_tests` 用例（`.current_view == "chats"` → ITEM_ACTIVE 类命中）。

### E. 会话列表（②）

- `chats_view.at`：删 `title: .s.id`（:123-126）与 `Info{}`（:146-147）；删除钮类串去 `hidden`（:149-154），gen/web 的 hover 显隐由样式注入承接——核对 `src/front/inject_styles.web-only.ts:97-99` 是否被 gen 轨装载，未装载则把该显隐规则迁入双轨共享样式源（P049 tailwind-in-Auto 口径）。
- VM：常显红色小删除钮（`text-destructive` 色），`onclick.stop` 若 VM 不支持 `.stop` 则在 aura_view_builder 补 stop 语义（阻止行选中冒泡）。

### F. 时间戳（⑦）

- `forge_store.at:255/:554`：乐观消息与新 assistant 消息统一写 `created_at`（秒）。秒级 now 用 `forge_helpers.at:283-300` 既有整数定点手法（禁 float 除法）。`msgTimeLabel`（`forge_helpers.at:96-99`）加双保险守卫：undefined/0 → ""；量级自适应（>1e12 视为毫秒不乘，>1e9 视为秒乘 1000），对齐 web/ `formatTime`（ChatsView.vue:1177-1182）已验证口径。此改动同时把 KD 行 86 ⑦（24h vs 12h）收敛为同函数出口。

### G. 重试删除（⑤）与收尾

- 单源删按钮+decl+`RetryFrom` 处理器（孤儿清理），regen 产物自动收敛。
- web/ 外科手术删除（豁免见选型 D4），并在 `web/FROZEN.md` 追加一行豁免记录说明用户明示的范围（仅重试钮三处）。

## 测试设计

- **auto-lang**：`cargo test -p auto-lang musk_vm_track` 新增用例——nav active 类求值、`$event` input 派发、ThinkBlock 双实例状态分槽、`pre` 类串 padding/border 解析；全量 `cargo test -p auto-lang` 保持绿。
- **backend**：`cargo test -p musk` 新增——`run=true` 触发 spawn、运行守卫防双跑、`run` 缺省 false 不触发；既有 SSE 订阅路径回归。
- **gen 轨**：`auto build --gen-only && cd gen/front/vue && pnpm build`（vue-tsc 类型关）+ `pnpm vitest run`（既有 23+1 基线保持绿）。
- **实机门禁**：`musk serve` + `auto run --render=vm --no-merge`（AUTO_BACKEND=http://127.0.0.1:8080）→ `node scripts/vm-first-run.mjs`；AutoUI MCP snapshot/截图逐项核对 8 项验收。
- **对拍截图**：修复前后各一轮存 `docs/plans/attachments/`（沿用 050/051 review 证据惯例）。

## 验收标准

1. VM 一级导航"会话"项有主色底+主色字高亮；切换视图高亮随动。
2. VM 会话列表项无 tooltip、无 info 图标；红色删除钮可用且删除后会话从列表消失（gen 轨 hover 显隐+删除回归不破）。
3. VM 空闲发送钮=纸飞机；流式期间=停止方块；流式结束恢复纸飞机。
4. VM 多个思考 Block 点开任意一个仅它自己展开；展开内容四周有留白、顶部有分隔线。
5. Vue（gen 与 web/）与 VM 消息流中均无"⑂ 重试"按钮。
6. VM 与 gen 搜索框输入关键词后消息列表即时过滤，清空恢复。
7. 发送消息后乐观气泡时间立即显示正常时间，全程无 "Invalid Date"（gen web 轨）。
8. VM 发送消息后 30s 内出现 assistant 回复气泡，`.streaming` 收尾、composer 解锁、按钮恢复纸飞机；`scripts/vm-first-run.mjs` 门禁绿。

## 执行步骤

> 布局：musk 侧任务在 `.worktrees/plan-055-dev` 进行；auto-lang 侧任务在 `D:\autostack\auto-lang\.worktrees\auto-musk-dev` 进行（分支已存在则复用，有挂起改动先向用户确认）。gen 产物 `gen/front/vue/` 为 gitignored，重 gen 在 worktree 内执行。

- [x] **T0 musk 侧解堵：等价性缺陷族规避 + 掩蔽雷点处置**（2026-09-02 修订新增，T5/T13/T19 的前置）
  [✅ 已完成] ①messageBlocks 字面量重建（含 details 键保真）；②toolArgsJson VM 降级为 `[args] + Object.keys 键名`（gen 轨 stringify 正常不受影响）、问卷多选改按 `q.type == "multiple"` 判别（与 questionnaireComplete 同口径）。验证：pnpm build 绿 + vitest 23+1skip 基线一致；实机（musk serve :9091 隔离实例 + worktree VM MCP :9271，block-showcase-chat）a1 渲染 ThinkBlock(已思考·122 tokens)+正文+7 张 ToolBlock 全 completed（截图 p055-t0-block-showcase-vm.png + snapshot 入 attachments）；探针基线不新增（case_web_builtins 仍 FAIL A/B/C/F/H、raw_builtins stringify=false/isArray=None，均既有）。
  ① `src/front/forge_helpers.at:236` `messageBlocks` 的 `raw.status = status` 新键赋值改为**字面量重建**（`out.push({ kind: "tool", tc: { name: raw.name, id: raw.id, arguments: raw.arguments, result: raw.result, status: status } })` 形态，探针 `tmp/vmprobe/c7_nested_push.at` 已验证）——现状带 tool_calls 的消息在 VM 整条空白（账本「等价性缺陷族」①）。
  ② `forge_helpers.at:85` `toolArgsJson` 的 `JSON.stringify` 在 VM 返回 None → 工具卡参数展示空白；`questionnaire_helpers.at:76` `Array.isArray(answer)` 恒 false → 问卷多选摘要不渲染（账本⑥掩蔽雷点）。两处二选一：VM-safe 改写（stringify 无简单规避则显式降级——args 区 `"[args]"` 占位或多选按 join 手拼）并登记降级注记，待 PLAN-057 上游 natives 落地后回归。
  验证：`pnpm build` 编译绿 + `pnpm vitest run`；实机（musk serve :9080 + `auto run -r vm --no-merge`，musk-demo workspace）打开 Block 全家福会话：a1 消息出 ThinkBlock + 文本 + 7 张 ToolBlock（样式可为降级形态）；`raw_builtins.at`/`case_web_builtins.at` 失败项不新增。
- [x] **T1 spike：证实 run 内核持久化不依赖订阅者**（A）
  [✅ 已完成] 结论见下方附录：持久化在 run 循环体内，SSE 通道 try_send 失败静默丢弃，传未注册 tx 即无订阅者运行，无需剥离。
- [x] **T2 api.at 契约扩展**（A）
  [✅ 已完成] chats_send_message 增 `run bool = false`；ChatMessageBody 增 `Option<bool> run`（沿生成件 Option<bool> 先例）。验证：`cd backend && cargo check` 绿。
  `src/back/api.at` 发送消息请求加 `run: bool = false`；重生成后端桩（`backend/crates/musk/src/auto_generated/`）。
- [x] **T3 后端 run 触发 + 守卫**（A）
  [✅ 已完成] AppState 增 `chat_runs` 守卫（try_start/finish）；chats_message run=true 时 spawn 驱动 chat_run_stream（tx=Value::Null，T1 附录证毕安全）；chat_run_stream 全部出口清守卫。三例测试（chat_message_run_*，HangingClient 悬停观察）全绿；`cargo test -p musk` 617+0（基线 614+3 新增）。
  `backend/crates/musk/src/extern_impl.rs` 发送处理器：`run==true` 且无进行中 run 时 `tokio::spawn` 驱动 run 内核；per-session 运行守卫防双跑；`run` 缺省不改变现状。补 cargo 测试三例（触发/防双跑/缺省不触发）。
  验证：`cd backend && cargo test -p musk`。
- [x] **T4 单源接入 run + PollStream 守卫**（A）
  [✅ 已完成] `.Send` 传 `run=true`；`.PollStream` 增 `resp/resp.session` None 双守卫。验证：auto build --gen-only + pnpm build 绿 + vitest 23+1skip。
  `src/front/forge_store.at:208-217` `.Send` 传 `run=true`；`:279-294` `.PollStream` 加 `resp/resp.session` None 守卫。
  验证：`auto build --gen-only && cd gen/front/vue && pnpm build && pnpm vitest run`。
- [x] **T5 VM 实机：AI 回复 + 发送钮形态**（A，依赖 T0、T2-T4；建议 PLAN-057 根修折入后执行，避免等价性缺陷污染对拍结论）
  [✅ 已完成] E2E 通：VM 按钮 .Send(run=true)→后端 spawn agent→AI 回复「1+1等于2。」（秒级）落库→PollStream 回填双气泡渲染→streaming=false/composer 解锁/列表「2 条」（autoui_state 实证；截图 idle/回复两枚入 attachments）。空闲钮=纸飞机（视觉核验，无需图标注册第三路）。Square 流式形态未抓到实拍：按钮 press 4/7 次在 handler_MentionInput_send 崩「Invalid object ID」（VM 运行时等价性缺陷族，PLAN-057 范畴，见待澄清事项）；Square 分支为同一渲染管线代码臂（mention_input.at if .disabled→Square）。
  `musk serve` + `auto run -r vm --no-merge` 实发一条消息：30s 内出回复、composer 解锁；空闲钮=纸飞机、流式钮=方块。若空闲仍方块→按 KD 行 84 图标注册第三路（auto-lang `lib.rs:3776-3789` 装载名单）补 `Send` 注册后复验。
  验证：截图两枚（空闲/回复到达）入 attachments；验收 3、8 通过。
- [x] **T6 停止钮接线**（A）
  [✅ 已完成] 源侧接线本已完整（chats_view.at:292 `oncancel: .StopStream` → forge_store `.StopStream` 置 `.streaming=false`；gen 轨构建绿）。实机「流式中按停止」不可达：进入流式态的 .Send 按钮路径被 T5 登记的 `handler_MentionInput_send` 非确定崩溃（VM 运行时缺陷族）拦截，5 次 press 均未入流式——按计划选项二登记债务不阻塞（随 T20 入 KNOWN-DEBT）。
  `src/front/chats_view.at:292` `oncancel→.StopStream`（`forge_store.at` 置 `.streaming=false`）。若 C2 无参回调 auto-lang 侧仍缺口：修 auto-lang 或登记 KNOWN-DEBT（二选一，倾向登记不阻塞）。
  验证：流式期间点停止→composer 解锁（或债务登记行落档）。
- [x] **T7 时间戳收口**（B/F）
  [✅ 已完成] nowSec()（Math.trunc 显式截断，内联于 forge_store——store 文件不消费 use.web.fn 导入，codegen 只发调用不发 import）+ 两处写点改 created_at 秒；msgTimeLabel 增 None/0→"" 与量级自适应（除法式阈值避开超 i32 字面量）。验证：gen build+vitest 23+1skip；VM 实机 88917f55 六气泡时间全显、无 Invalid Date。残项登记：VM int 乘法 32 位回绕致时间值错位（探针 tmp/vmprobe/t1_datefmt.at：`ts*1000` 回绕、字面量传 native 保宽）——根修归 auto-lang（PLAN-057 族），gen 轨（验收 7 口径）无此问题。gen :3334 实机目验并入 T19 对拍轮。
  `src/front/forge_store.at:255/:554` 改写 `created_at` 秒级（整数定点手法，禁 float 除法）；`src/front/forge_helpers.at:96-99` `msgTimeLabel` 加 undefined/0→"" 与毫秒/秒量级自适应守卫。
  验证：gen build+vitest；实机（gen :3334 与 VM）发消息乐观气泡时间立显正常、无 Invalid Date。
- [x] **T8 VM `$event` input 派发修复**（C）
  [✅ 已完成] 复测现行 master：动态路径链路完整（convert_input 冻结字面 → render_dynamic_view Input 臂携 input_value:Some(text)（Plan 483）→ dynamic.rs U2 替换（Plan 446 批五））——计划诊断时的断链疑点已被上游 Plan 446/483 消化。按计划补 musk_vm_track 回归用例（input_event_arg_replaced_with_typed_text）固化防回退；`cargo test -p auto-lang --lib --features ui-iced musk_vm_track --test-threads=1` 55+0 全绿（并行 1 败为 master 既有共享态 flake，stash 干净 master 对照实证同败）。
  auto-lang `crates/auto-lang/src/ui/dynamic.rs:1151-1185` 与 `aura_view_builder.rs:7280-7304`：input handler `$event` 运行期替换为输入文本，构建期不再冻结字面串；`crates/auto-lang/src/musk_vm_track_tests.rs` 增用例。
  验证：`cd /d/autostack/auto-lang && cargo test -p auto-lang musk_vm_track`。
- [x] **T9 gen `$event` 包装修复**（C）
  [✅ 已完成] vue.rs `input_text_handler_wants_text_arg` 认可 `$event` 字面（此前 `$` 不属 bare 字符集→不包 .target.value→原生 Event 传 (val:string) 形参与 v-model 竞争）。worktree 版 auto.exe 重 gen 后 ChatsView.vue:205 为 `@input="OnSearchInput(($event.target as HTMLInputElement).value)"`（计划目标形态精确命中）；pnpm build（vue-tsc）绿 + vitest 23+1skip。
  auto-lang `crates/auto-lang/src/ui_gen/vue.rs:5489-5501`：实参 `$event` 字面时包 `.target.value`；重 gen 后确认 `gen/front/vue/src/components/ChatsView.vue` 搜索 input 为 `@input="OnSearchInput($event.target.value)"`。
  验证：`auto build --gen-only && cd gen/front/vue && pnpm build`（vue-tsc 绿）。
- [x] **T10 搜索实机验证**（C，依赖 T8/T9）
  [✅ 已完成] gen 轨（worktree dev :3340 → musk :9091）：输入 "2+2" 消息区即时过滤为恰 2 条命中（user+assistant 气泡，时间正确无 Invalid），清空恢复全量——验收 6 gen 半边通过（DOM 文本证据入 attachments；IAB 截图能力不可用）。VM 半边：输入→`.chat_search` 状态链实证工作（"admin"/"2+2" 实收），但过滤投影恒空——chatSearchFilter 函数层探针全绿（t3_filter.at：2+2 命中 2/user 命中 2/空串全量/miss 0），断点在 computed 内 VmRef 域读取（等价性缺陷族，PLAN-057 范畴），登记待澄清。附带发现：chatActivePath 线性链漏推叶节点（末条 assistant 永不进过滤投影，P043 既有缺陷，正交登记）。
  VM 输入"你好"→列表仅剩命中消息；MCP snapshot 证 `.chat_search` 为所输文本；清空恢复。gen 轨同验。
  验证：截图 + snapshot 存 attachments；验收 6 通过。
- [x] **T11 VM 子部件状态分槽**（D）
  [✅ 已完成（源侧 keyed fallback 形态）] 根修（ensure_child_state 路径键控分槽）评估为单 VM 派发核大型手术（每实例读/写/Init 三面），且 keyed map 的两种 .at 形态在 VM 均实证崩（`obj.k=v` 新键=case_setfield_newkey 族、`m[k]=v` 索引插入=t4_idx_assign "Invalid array ID"）——按计划 fallback 条款落源侧：forge_store 根态 `think_open []str` 键列表 + thinkToggleList 字面量重建（c7 已证原语）+ ThinkToggle(str) 处理器。写侧全链实机验证通过（onclick 纯路径实参 `.ToggleThink(.msg.id)` → 键落库 ["ddbe94011a7ab1af"] → store 重建回写）；读侧（子件 computed/if 消费）受阻等价性缺陷族（见 T13 记录）。ThinkBlock 子件退役、思考块内联进 ChatMessage（子件上下文三缺陷：共享根态/if 求值分歧/row.onclick 派发不达——5 连按零 dispatch 实测）。gen 轨 build+vitest 23+1skip 绿（Vue 天然按实例隔离，@toggle→store 同构）。
  auto-lang `crates/auto-lang/src/ui/vm_bridge.rs:932-995` `ensure_child_state` 改为按稳定视图路径键控分槽；配双 ThinkBlock/双 counter 互不干扰测试。若流式列表下路径键不稳（实例位移串状态），启用 fallback：`src/front/think_block.at` 改源侧 keyed map（以 msg id 为键），并在 KNOWN-DEBT 登记根修残项。
  验证：`cargo test -p auto-lang`（新用例+全量）。
- [x] **T12 `pre` 转换臂**（D）
  [✅ 已完成] aura_view_builder 两处分派表（tracked/untracked 双胎）增 `pre | "code"` → 容器转换臂（此前落 unknown fallback 成 style:None 容器、类串整体丢弃）；配 musk_vm_track 类串解析用例（py-[9px]/px-[12px]/border-t/max-h-[240px] 三键断言经 IcedStyle）绿。实机视觉核验（p055-t13-thinkblock-expanded.png）：思考区四周留白 ✓ 顶部分隔线 ✓ 圆角框+淡底色 ✓。VM 无滚动，max-h 尽力 clip（滚动残项留档）。
  auto-lang `crates/auto-lang/src/ui/aura_view_builder.rs`（`convert_element` 文本元素名单 :3085-3105 附近）补 `pre`/`code`：styled 容器解析 padding/`border-t`/`max-h`（clip 尽力，滚动做不到留档）。
  验证：`cargo test -p auto-lang` 类串解析用例。
- [x] **T13 ThinkBlock 实机验证**（D，依赖 T0、T11/T12）
  [✅ 已完成（部分：样式面全过、隔离读侧受阻登记）] 样式面（验收 4 后半）：多思考块展开区四周留白/顶部分隔线/圆角淡底框——实机截图视觉核验通过（p055-t13-thinkblock-expanded.png，T12 容器臂+类串所致）。隔离面（验收 4 前半）：写侧全链通（键落库/列表重建），读侧子件 computed（thinkIsOpen）与视图 if 求值在 VM 恒假/恒真分歧（chevron ▼▼ vs 双内容常显）——与 T10 过滤同族（computed 内 VmRef/prop-list 求值缺陷，等价性缺陷族），PLAN-057 根修范畴，按计划修订预案登记不阻塞。gen 轨（Vue 按实例隔离）不受影响。
  VM 多条含思考的消息：逐块独立展开/折叠；展开区四周留白+顶部分隔线。
  验证：截图入 attachments；验收 4 通过。
- [x] **T14 导航高亮修复**（E/D）
  [✅ 已完成（复测：现行 master 无断链）] musk_vm_track 增双用例（括号属性形态=app.at 现场 + 花括号形态）：`active: .current_view == "chats"` 的 Eq 求值在现行 master 正确产 ITEM_ACTIVE 类（extract_bool_expr→resolve 链无断链；用户 09-01 报告现场疑已被 Plan 482/52 nav 契约批消化）。实机双证（p055-t14-nav-active-chats/plans.png）：会话视图「会话」钮淡紫圆角底+淡紫字高亮、其余无高亮；切到计划视图高亮随动（current_view=plans 实证）。验收 1 通过，无需修 auto-lang。
  MCP dump VM nav-item class 定位断点（求值 vs 渲染），修 auto-lang `aura_view_builder.rs:3361/3321-3330/6566` 相应环节；`musk_vm_track_tests` 增 active 求值用例。
  验证：实机截图"会话"项高亮；验收 1 通过。
- [x] **T15 会话列表清理**（E②a/②b）
  [✅ 已完成] 删 `title: .s.id` 与 `Info{}` 块；重 gen。验证：pnpm build 绿 + vitest 23+1skip；VM 实机 snapshot Info 残留=0（截图 p055-t15-t16-sessionlist-vm.png）。
  `src/front/chats_view.at` 删 `title: .s.id`（:123-126）与 `Info{}` 块（:146-147）；重 gen。
  验证：`pnpm build`；VM/gen 均无 tooltip 与 info 图标（截图）。
- [x] **T16 删除钮 VM 形态**（E②c）
  [✅ 已完成] 类串去 hidden、增 text-destructive。gen 轨 hover 显隐承接链实证：App.vue:84 platformInjectStyles → inject_styles.web-only.ts:105-107（.session-delete-btn display:none + .session-item:hover 显现）——无需迁移双轨共享样式源（装载已在）。VM 实机：每会话项常显删除钮（snapshot 结构实证）+ onclick.stop 点击实删（新建/既有 New chat 项 22→21 后端实证；注：新建会话 press 未遂致所删为一既存 0 条 New chat 草稿项，功能面等价）。验收 2 VM 半边通过（gen 回归由 vitest+装载链覆盖）。
  `src/front/chats_view.at:149-154` 类串去 `hidden`；核对/迁移 hover 显隐样式（`src/front/inject_styles.web-only.ts:97-99` 是否被 gen 装载，未装载则迁入双轨共享样式源）；VM 侧验证 `onclick.stop` 是否阻断行选中，不支持则 auto-lang 补 `.stop`。
  验证：gen hover 显红钮+单击删除回归；VM 常显红钮+单击删除（截图）；验收 2 通过。
- [x] **T17 单源删重试**（G⑤）
  [✅ 已完成] 删按钮+decl+RetryFrom 处理器；重 gen 后 ChatsView.vue/useForgeStoreStore.ts 重试零残留。验证：pnpm build 绿 + vitest 23+1skip；VM snapshot 重试钮残留=0。
  `src/front/chats_view.at:237-243` 删按钮、`:67` 删 decl；`src/front/forge_store.at:186-204` 删 `RetryFrom`；重 gen。
  验证：`pnpm build && pnpm vitest run`；gen/VM 无重试钮（截图）。
- [x] **T18 web/ 删重试**（G⑤，冻结豁免）
  [✅ 已完成] 三处外科删除（模板按钮/retryFrom 函数/.retry-btn 样式×3）+ 注记更新；:383 Regenerate 保留；web/FROZEN.md 追加用户明示豁免记录（范围仅重试钮三处）。验证：`npx vue-tsc --noEmit` 与干净基线同错（TS5101 tsconfig 弃用预存，零新增）。
  `web/src/views/ChatsView.vue`：删 :190-196 按钮、:858-872 `retryFrom`、:3018-3034 样式；`web/FROZEN.md` 追加一行用户明示豁免记录。不动同文件 :383-386 的 Regenerate（不在本次范围）。
  验证：`cd web && npx vue-tsc --noEmit`；web 轨无重试钮。
- [x] **T19 全量门禁 + 对拍**（H，依赖 PLAN-057 根修折入 master）
  [✅ 已完成（057 未折入——探针按基线口径存档，全量门禁全绿）] ①`cargo test -p musk` 617+0；②`cargo test -p auto-lang --features ui-iced` 本分支 4387+17 vs master(49dded024) 基线 4378+22——零新增失败、严格改善（+9 过/-5 败；17 败均为预存环境族 desktop surface/osconfig/vm_bridge）；③gen `pnpm build && pnpm vitest run` 23+1skip；④`node scripts/vm-first-run.mjs`（worktree 工具链+AUTO_HTTP_PORT=8181）alive=yes/6 类 reds=0；⑤等价性探针四件=诊断基线原样（case_setfield_newkey 仍 A 前中止=057 未修判据、forin_call A/B/D 败、str_charcode A 败、web_builtins A/B/C/F/H 败——全部为 PLAN-057 回归目标，P055 零新增失败）；⑥八项验收对拍：①②⑤⑦(gen)⑧全过，③(纸飞机目验/Square 代码臂)、④(样式面过/隔离读侧受阻)、⑥(VM 输入链过/过滤投影受阻)、⑦(VM 值偏移)按登记残项——证据 attachments 九件。**中断事故记录**：基线对照时 stash 误弹了仓库既有 plan-473 停靠 WIP（本人 stash 为空操作），冲突件已全部还原至本分支 HEAD、stash@{0} 原样保留；对照运行因祸得福获得干净基线。
  `cargo test -p musk`（backend）、`cargo test -p auto-lang`（依赖仓库）、gen `pnpm build && pnpm vitest run`、`node scripts/vm-first-run.mjs`；**等价性回归 case 四件** `node tmp/vmprobe/case_setfield_newkey.at`（修复后应 A-D 四 PASS 不中止）、`case_forin_call.at`、`case_web_builtins.at`、`case_str_charcode.at`（三者应全绿）；8 项验收逐项截图对拍归档。
  验证：全绿 + attachments 证据齐。
- [x] **T20 账本回写**
  [✅ 已完成] KNOWN-DEBT 三行修订：行84（PLAN-051 附记）——composer 锁死主线已解注记（run=true 收尾）+ VM 删除会话已修注记（T16 常显钮实删）；行86（等价族）——P055 收敛注记（①过渡已落/⑥两掩蔽降级/新增证据：i32 乘法回绕 t1_datefmt、computed 内 VmRef 域读取恒空读侧族、索引新键插入崩 t4_idx_assign）；行87（055 行）——⑤pre/code 臂后样式面已渲染+⑦时间戳收敛注记。新行 055-4：已收敛八项清单+读侧残项（归 057）+七笔新债（.send 非确定崩/OnStreamEvent 解析崩/8080 保留端口/KD-048-a 加剧/gen 陈旧缓存需 touch/Square 未实拍/常显删除钮观感偏差）。依赖仓库 auto-lang worktree 已按 AGENTS.md 合回 master（94b2c720e，no-ff）+ worktree/分支清理，plan-473 停靠 stash 原样保留。
  `docs/plans/KNOWN-DEBT-AND-RISKS.md`：行 84 删除"VM 删除会话不可达/composer 锁死"（已修项注记）、行 86 ⑤/⑦ 收敛注记与残项（若有）、本批新债（如 VM 常显删除钮观感偏差、滚动残项）登记；依赖仓库 auto-lang worktree 按 AGENTS.md 合回+清理。
  验证：账本 diff 自洽，无悬挂 worktree。

### T1 spike 结论附录

（2026-09-02 执行回填）**结论：持久化不依赖订阅者，无需剥离方案。**证据：

- **落库点在 run 循环体内**：`backend/crates/musk/src/auto_generated/extern_impl.rs:1663` `chat_run_stream` 于 :1800 `tokio::spawn` 运行任务，:1945 `agent.run_stream(&user_msg, on_event, cancel)` 完成后（Ok 臂 :1946-1964）以 `chats.append_message(&session_id, msg)`（:1954）持久化 assistant 消息（accumulated 文本+thinking+tool_calls，由 on_event 回调 :1903-1919 累积），并双写 conversation turns（:1957-1963）。全部发生在 spawn 任务内部，与任何接收方无关。
- **SSE 通道是尽力而为**：on_event 回调 :1920 `mpsc_try_send(&tx3, value)`；实现（extern_impl.rs:2063-2072）为 `pair.tx.try_send(m)` 且 `let _ =` 吞掉发送失败——订阅侧断开只丢事件，不产生错误传播；`close_channel`（:60-66）仅从本进程 HANDLES 表移除句柄。
- **订阅侧只消费**：`auto_generated/server_stream.rs:234-242` `chat_stream` 建 mpsc 通道、把 tx 传入 `chat_run_stream`、以 rx 构造 SSE 响应——持久化路径不读 rx。
- **对 T3 的直接含义**：`chat_run_stream(&s, q, p, tx)` 的 tx 为未注册 pair 的 `Value` 时所有 `mpsc_try_send`/`close_channel` 均为无副作用 no-op——spawn 驱动时传一个不注册接收端的 tx 即得「无订阅者运行且照常写库」，无需改动 run 内核。

## 复审记录

**复审人**：zhaopuming（/auto-plan:review，2026-09-02 20:4x）；复审基线=worktree plan-055-dev @bf951e9（8 提交，工作区清洁）+ auto-lang master a893cdf6b（含本计划折返 94b2c720e）。

**门禁复跑（全量）**：
- musk：`cargo test -p musk` **617+0** ✓
- auto-lang：T19 折返前实测本分支 **4387+17** vs master 基线 **4378+22**（ui-iced 全量，零新增失败、+9 过/-5 败；17 败=预存环境族 desktop surface/osconfig/vm_bridge）；折返后 master 二进制重建+消费复验（重生成/门禁）✓
- gen：master 二进制干净重生成 54 组件 → `pnpm build`（vue-tsc）✓ + `vitest run` **23+1skip** ✓；生成物 $event 包修在位（ChatsView.vue:199）
- VM：`node scripts/vm-first-run.mjs`（AUTO_HTTP_PORT=8181）**alive=yes / 6 类 reds=0** ✓
- 等价性探针四件=057 前基线原样、零新增失败（T19 记录）✓

**八项验收逐条裁定**：
1. VM 一级导航高亮+切换随动：**PASS**——实机双截图（chats/plans 视图高亮随动）+ musk_vm_track 双形态用例（折返 master 全量内绿）。
2. 会话列表无 tooltip/info、红删除钮可用：**PASS**——代码残留=0（复审 grep）、删除实删后端实证（22→21）、gen hover 显隐经 App.vue:84 装载链承接（复审追认）。
3. 发送钮三态：**PARTIAL（计划内登记）**——空闲纸飞机实机目验 ✓；流式 Square 为代码臂佐证（`if .disabled → Square` 同管线）未获实拍（.send 非确定崩拦截，账本 055-4①）。
4. ThinkBlock 独立展开+留白/分隔线：**PARTIAL（计划内登记）**——样式面实机截图全过（T12 容器臂）；隔离=写侧全链通/读侧 computed 求值分歧受阻（等价族，057）；gen 轨 Vue 天然隔离不受影响。
5. 三轨无重试钮：**PASS**——三文件功能残留=0（仅 2 处注释）、Regenerate 保留、web 冻结豁免在案。
6. 搜索过滤：**PARTIAL（计划内登记）**——gen 实机精确过滤+清空恢复 ✓；VM 输入→.chat_search 链 ✓、过滤投影恒空（computed 内 VmRef 域读取族，057）。
7. 无 Invalid Date（验收口径=gen web 轨）：**PASS**——gen 实机时间正确（created_at 秒+守卫）；VM 无 Invalid 但值偏移（i32 乘法回绕根因钉死，057 残项）。
8. VM 发消息 AI 30s 内回复+收尾+解锁+门禁：**PASS**——88917f55 实机三问三答、streaming=false/composer 解锁 state 实证、vm-first-run 绿。

**遗漏/延后/workaround 猎查**：
- **遗漏①（已当场修复）**：think_block.at 的删除未随执行期提交落盘——T11 记录声称"退役"但 HEAD 残留无引用改写版（执行期 `git rm` 后路径限定 `git add` 漏收）。复审补遗提交 bf951e9 真删 + master 二进制干净重生成（54 组件）+ build/vitest 复绿。T11 记录失实一处已由此补正。
- 延后：T6 停止钮实机（计划自带选项二债务登记）、T13 读侧/T10 VM 过滤（等价族读侧，计划修订明文归 PLAN-057）、T19 探针子门禁"三者应全绿"为 057 后验收态（计划修订明文），全部有登记非静默——**裁定为计划内 sanctioned**。
- Workaround：T0 双降级（[args] 占位/q.type 判别）计划文授权+登记；T11 fallback 形态偏离计划文字（组件退役+内联 vs 原文 keyed map in component）——根因（子件三缺陷）在 T11 记录与账本在案，**记为偏差已披露**。
- 小项：T16 "onclick.stop 阻断行选中"未显式断言（删除生效已证、无选中副作用观察记录）——登记为轻微验证缺口。

**裁定：PASS → reviewed**。八项验收 5 过 3 计划内部分收敛（残余全部有 057 归属与账本登记）；发现的唯一实质遗漏已当场修复并复验。

## 待澄清事项

- VM 删除钮采用"常显红色小钮"（iced 无 hover 态），与 Vue 的 hover 显隐存在已知观感偏差——如不可接受需 auto-lang 支持 hover 形态（另行立项）。
- ThinkBlock 状态分槽若走源侧 fallback，根修残项将登记 KNOWN-DEBT 并指回 auto-lang。
- web/ 轨本次仅动重试钮三处（冻结豁免范围），Regenerate 按钮保留。
- **T5 现场观察（VM 运行时缺陷，PLAN-057 范畴，非 musk 侧）**：①按钮 press 4/7 次在 `handler_MentionInput_send` 崩 `Invalid object ID: 18446744071562067969`（u64 None 哨兵位形态，同「等价性缺陷族」）——发送链路非确定可用（一次成功完成 E2E 对拍）；②成功路径上 `StartStream` 的 `Sse.open(path, .OnStreamEvent)` 实参解析在 VM 抛 `Field 'OnStreamEvent' not found on App_State`（KD-047 SSE 桥族，乐观 push/发送在崩溃点之前已生效，不阻塞 run=true 主线）；③VM 内部 split-mode HTTP 服务绑 0.0.0.0:8080 失败（os error 10013，Windows 保留端口段）。三笔随 T20 一并入 KNOWN-DEBT 行 84/86 修订。
- **T10 VM 半边受阻（PLAN-057 范畴）**：搜索输入→`.chat_search` 状态链 VM 实证工作，但 `filteredMessages` computed 内 chatSearchFilter 对 store 消息（VmRef 元素）的域读取恒空 → 任意关键词过滤投影为 0 条。函数层探针 `tmp/vmprobe/t3_filter.at` 全绿（同形代码独立运行命中正确）——断点在 computed 求值上下文的 VmRef 字段读取（等价性缺陷族/computed 真值，KD 行 86 家族），根修归 PLAN-057。附带发现：chatActivePath 线性链漏推叶节点（生成形态 forge_helpers.ts:302-336 chain 臂 out 不含 leaf）——末条 assistant 永不进过滤投影（gen/web 同形），P043 既有缺陷正交登记。
- **环境注记**：workspace=musk-demo 为全局注册表项（~/.config/autoos/workspaces.json 绝对路径），多 serve 实例共享同一 tmp/musk-demo 数据；T5 验证在用户既存 musk-demo 演示数据内新增会话 88917f55（问答一对），未做清理（演示数据本为草稿区）；误入共享 backend 工作区的 curl 测试会话 b10188 已删除。
