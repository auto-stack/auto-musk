---
plan_id: PLAN-051
status: executing
feature_name: VM 会话主界面闭环——气泡渲染/发送链路/流式降级（017-chat 实测定位）
author: [zhaopuming]
created_at: 2026-08-29
updated_at: 2026-08-30

supersedes_spec_components: []
new_spec_components: []
touched_goals: ["goal-frontend-parity: VM(iced) 轨会话主界面闭环——气泡渲染/Enter+按钮双路发送/流式轮询降级/timer 声明块 DSL；SSE 桥泛化(阶段2)留后续"]

current_step: 11
total_steps: 11
---

# [PLAN-051] VM 会话主界面闭环——气泡渲染/发送链路/流式降级

## 变更摘要

2026-08-29 以 auto-lang `examples/ui/017-chat` 为最小载体实机实测 VM 轨（release auto.exe
`--render=vm`，证据 `tmp/017-chat-vm-probe/`），**证伪了"iced 渲染器画不了聊天界面"**：

- **渲染结构两轨一致**：气泡（含 sender 名字/时间/`mine` 左右分向）、Thread 标题线、
  滚动区、输入框（placeholder/draft 双向绑定/InputChanged）、Send 按钮全部正确渲染；
  `for`/`if-else mine`/子 widget 内联均工作。musk 同级类串在 vue 轨渲染正确。
- **交互三断**（vue 轨 playwright 9/9 全绿，VM 轨全断）：
  ①Enter 不发送——键盘事件被路由到根 widget 不存在的 `App.key_enter`，输入框声明的
  `onenter: .DoSend` 无接线；
  ②按钮发送静默失败——DoSend 触发、draft 清空，但 `on_send(text)` 这类子→父 msg 参数
  回调被静默丢弃（messages 恒 5 条，零报错）；KD-048 UPSTREAM②（renderer.rs:7656
  非通用 emit）同族；
  ③SSE 整链毒死——`[CODEGEN] dropping poisoned export 'api.stream'`（根因 merged
  vm+vm 模式 `Undefined variable: bus`），实时推送结构性不可用。
- **musk 会话页"完全对不上"的定性**（与上项叠加）：气泡列表空 = `chats_view.at:185`
  `for msg in .filteredMessages` 以 computed 为 for 源，`aura_view_builder.rs
  resolve_iterable` 只读 state 不查 computed 表（KD-047 UPSTREAM①）；正文空白 =
  `use.web component Markdown` 无 .at 视图可内联（KD-047 ②，musk 侧
  `ports/renderer.vm.at` 纯文本降级组件已入库 c66dbe7 但未激活——装载器 fn-only）；
  发送不可达 = `onsend: .SendInput($event)` emit 无路由（KD-048 ②）+ Enter 形态
  （musk 用 `onkeydown.enter.exact.prevent`）无接线；流式 = `Sse.open` 无 VM 形态
  （KD-047 G1，勘察报告 `docs/specs/reports/048-sse-vm-survey.md` 在案）。

本计划据此做**会话闭环批**：上游 auto-lang 四能力（Enter 接线/回调路由/computed for
源/widget registry）+ musk 侧流式轮询降级 + 017-chat VM 冒烟门禁，使 musk VM 轨会话
主界面达到 vue 版效果（静态气泡 + 发送往返），流式以 G1 阶段1 轮询降级达标。

## 目标

1. musk VM 轨会话主界面达 vue 版效果：消息列表可见（用户气泡右侧主色/AI 左侧
   hairline 壳、🧑 You 与 🤖 AI 名徽章、时间戳、左右分向）；输入框可打字；Enter 与
   发送按钮**双路可用**；发送 → `chats_send_message` 后端往返 → 用户气泡入列。
2. 流式回复 VM 降级形态可用：`.streaming` 期间轮询增量回填（按 048 勘察阶段1 方案），
   `Sse.open` 在 VM 显式降级不炸；SSE 桥泛化（阶段2）明确出范围留档。
3. 017-chat 示例补 VM 轨冒烟门禁（发送闭环常驻回归，当前 pac.at `render: "vue"`、
   tests 只有 playwright——VM 轨从未有验收）。
4. vue 轨零回归（同一份 .at 源，既有门禁全绿）。

## 架构方案

| 能力 | 落点 | 方案 |
|---|---|---|
| C1 Enter 键盘接线 | auto-lang iced 渲染器键盘路由 | Enter 按焦点 input 的声明派发：`onenter`（017-chat 形态）与 `onkeydown.enter.exact.prevent`（musk 形态）两形态最小集，不再落根 widget `key_enter` |
| C2 msg 参数回调路由 | auto-lang iced renderer 事件派发 | `onsend: .SendInput($event)` / `on_send(text)` 通用 emit 路由（KD-048② 泛化，替换 renderer.rs:7656 特定硬编码思路） |
| C3 computed for 源 | auto-lang `aura_view_builder.rs` `resolve_iterable` | state 读 miss → 回退查 computed 求值表（KD-047① 清偿；EDGE-16 computed 臂 ~117 已有求值面可接） |
| C4 adapter widget 注册 + use.web 点分 kind | auto-lang `load_ext_imports_for_vm`（vm 装载器）+ parser | 增 widget 臂：ports adapter 同名 widget 注册进视图 registry，激活 musk `renderer.vm.at` Markdown 纯文本降级；同步落 `use.web.fn`/`use.web.component`/`use.web.composable` 点分 kind 文法（fn 别名显式化 + 装载校验，用户裁定 2026-08-29） |
| C5 流式轮询降级 | musk `forge_store.at` + `ports/platform.vm.at` | 消费 C7 timer block：`PollStream (every_ms: 500, when: .streaming)` 轮询 `chats_get_session` 增量回填；`Sse.open` VM 侧显式 no-op + warn |
| C6 示例 VM 门禁 | auto-lang `examples/ui/017-chat/tests/` | VM 冒烟脚本：起 `auto run --render=vm` + AutoUI MCP 断言（初始 ≥5 气泡、type+发送 → 第 6 条入列、draft 清空、Enter 路由） |
| C7 timer block DSL | auto-lang AST/parser + vue codegen + iced Subscription | widget/store 新增 `timer {…}` 声明块（语法见详细设计），周期到点派发 msg 进既有 on 流；vue=setInterval/onUnmounted，VM=iced Subscription（MCP action 通道轮询订阅先例 renderer.rs:5244） |

执行流（AGENTS 规则）：auto-lang 改动在其仓 worktree `.worktrees/auto-musk-dev`
（分支同名；PLAN-050 若已折叠则重建，未折叠则续用先合 050 增量）；musk 侧在
`.worktrees/plan-051-dev`。每能力项独立提交（TDD 先红后绿 + lib 全量回归）。

## 技术栈

auto-lang（iced renderer 键盘/事件路由、aura_view_builder.rs、vm 装载器、
AST/parser 与 vue codegen 的 timer block 新增面——C7 是**新增声明块**，不改
既有块契约）+ auto-musk（forge_store.at 轮询消费、platform.vm.at 显式
no-op、验收对拍；web/ 冻结不动、backend/ 不动）。环境：release auto.exe（debug 版
RC canary 阻断主界面渲染——KD-048-b 同族，不在本计划）。实测通道：`musk serve`
:8081 + `AUTO_BACKEND` 契约通路（PLAN-048 已定型活体）+ AutoUI MCP
（`AUTOUI_MCP_PORT`，本日 017-chat 探针即此通道）。

## 需求分析与背景调查

> 来源：2026-08-29 017-chat VM 轨实机调研（快照/check/日志存
> `tmp/017-chat-vm-probe/`：snapshot.json、check.json、vm-run.log）。spec ledger 脉络：
> P047（首跑/DEGRADED 四域）→ P048（数据桥/UPSTREAM①②④）→ P049（双轨样式收敛）
> → P050（四界面一致性批，消息气泡明确留后续批次——本计划即该批次）。

- 017-chat 实现完整（255 行：message_thread/composer/chat_store/api/db；vue 轨
  playwright T1-T9 全绿，plan399 Phase12 收官），但 VM 轨无任何门禁——本日为首次
  实测，三断点（Enter/回调/SSE）即此次新证。
- `autoui_check` 实拍 fallback 机制：.at 子 widget 落 "unknown tag → Column fallback"
  时子树视图仍被内联（017-chat 因此能渲染）；musk 的 Markdown 是 `use.web` 组件，
  无 .at 视图可内联 → fallback 即空白。降级组件 `ports/renderer.vm.at` 就绪未激活。
- musk 数据面（PLAN-048 定型）：`AUTO_BACKEND` 契约通路活体已证（登录 →
  session_list 10 会话入状态）；`chats_send_message` 代码通路已通（KD-048 MVP 7 fn
  清单）。断的是渲染与接线，不是数据。
- musk 消息链路：`chats_view.at:76` `filteredMessages => chatSearchFilter(chatActivePath(
  .store.messages, ...))`（computed，helpers 经 `use.web` 引入——fn 别名 VM 已通
  详见 C4 勘正，剩余贯通点见待澄清2）→ `:185` for 源；`:256` `onsend: .SendInput($event)`（C2 形态）；
  `mention_input.at:110` `onkeydown.enter.exact.prevent`（C1 形态）+ `:123` 按钮
  `onclick: .send(...)`；流式 `forge_store.at:211` `Sse.open(path, .OnStreamEvent)`。
- G1 勘察（KD-047，报告在册）：阶段1 轮询（0.5-1 日、零上游）→ 阶段2 SSE 桥泛化
  （2-3 日上游）。本计划落阶段1。
- **轮询触发的定时能力核实（2026-08-29，C7 立项依据）**：①task 体系
  （`ast/task.rs`，Plan 121/125 Actor 模型：state + start/stop 钩子 + on 模式匹配）
  **无 timer 概念**——无定时字段、无定时 native、无生命周期定时；②task 与
  AutoUI widget 体系**无桥**——TaskDef 消费方全在后端路径（ast/dep/indexer/
  parser/scope/infer/trans-rust/vm-codegen/vm-loader），UI 管线（ui_gen/
  aura_view_builder/iced renderer）完全不触碰，task 无法向 widget `on{}` 派发、
  widget 无法 spawn task；③`Time.sleep_ms(int)` native 存在但为
  `std::thread::sleep` 阻塞实现（stdlib.rs:646），UI 线程不可用；④iced renderer
  **已有 Subscription 基建**（MCP action 通道轮询订阅 renderer.rs:5244、shell
  事件订阅 renderer.rs:6078）——定时 Subscription 有在库先例可循。结论：定时触发
  须新建设计 → C7 timer block（用户裁定方向，语法见详细设计）。
- 已知不在范围：SSE VM 真形态（阶段2）、markdown 富文本 native（comrok 族，降级口径
  沿 047）、debug RC canary、VM 静默退出、双输入框双焦点（designs/011 已立项）。

## 详细设计

- **C1**：键盘事件处理臂（renderer 键盘派发，先 grep `key_enter` 定位——本日日志
  `[VM-HANDLER] App.key_enter failed` 即其路由落点）改为：Enter 先查当前焦点
  text_input 的 vnode 声明，命中 `onenter` 或 `onkeydown.enter.exact.prevent` 即
  派发该 handler（带 `.prevent` 语义=不再冒泡到根）。最小集只做 Enter；其他键
  （Tab/Esc 等沿既有行为）。
- **C2**：msg 参数回调（`onsend: .SendInput($event)` 消费侧在宿主 widget、触发侧在
  子 widget handler 内调用 `on_send(.draft)`）需通用路由：子 handler 求值遇
  "prop 名为 msg 类型" 的调用时，派发到宿主传入的闭包（msg 载荷作实参）。以
  017-chat `Composer.DoSend → App.SendMessage(text)` 与 musk
  `MentionInput.send → chats_view.SendInput` 双样点定契约；替换 renderer.rs:7656
  ash-gui 特定硬编码思路（该点自注"非通用 emit 修复"）。
- **C3**：`resolve_iterable`（aura_view_builder.rs ~262）state 读失败后回退：用
  EDGE-16 第五层 computed 求值面（~117，现只接 prop 面）求 computed 表；musk
  `filteredMessages`（链式纯 fn 调用形态）作回归样例。注意 fn 体在 `use.web`
  引入的 forge_helpers.at——T1 先勘察该别名在 VM 的解析（待澄清2）。
- **C4**：`load_ext_imports_for_vm` 增 widget 臂：ports adapter（442 A3 链
  renderer.at → renderer.vm.at）中同名 widget 注册进视图 registry，`use.web
  component Markdown` 在 VM 解析到降级实现（pre-wrap 纯文本）。富文本 native 上游
  项维持 047 登记。
  **fn 臂现状勘正（2026-08-29 复核 lib.rs）**：`.at` 目标的 fn 别名在 VM **已通**
  ——`load_at_ext_imports`（lib.rs:2615）对 `.at` 路径走 adapter 链
  （`resolve_vm_at_adapter`，无 vm 变体则装原文件）装载模块并按文件 stem 限定别名
  （lib.rs:2629-2644），嵌套模块的顶层 use.web 亦有清扫（lib.rs:2599-2611，
  Plan 442 B-support）；TS/npm 目标落 arity 扫描 stub（不炸）。KD-047 ② 的缺口
  仅在 widget kind 不进视图 registry。C4 新增面=widget 臂 + 点分 kind 文法：

  ```auto
  use.web.fn msgTimeLabel, messageDisplayBlocks from "src/front/forge_helpers.at"
  use.web.component Markdown from "src/front/ports/renderer.at"
  use.web.composable useT from "src/front/ports/composables.at"
  ```

  - 文法：`use.web.<kind>` 点分限定符，与 use.rust/use.py 点分家族一致；parser
    `use_web_stmt`（parser.rs:6320）kind 匹配臂扩点分形态（`fn`/`component`/
    `composable`），空格旧形式（`use.web component X`）与裸形式（无 kind=Fn）
    保留兼容等价，不强制迁移。
  - 语义：`use.web.fn` 为**显式 fn 别名声明**（裸形式的"可能是函数/对象/常量"
    含糊性收敛为承诺纯 fn）；装载器对显式声明做校验——.at 目标未导出该符号时
    编译期报错（替代现状 typo 静默落 stub）；TS/npm 目标 vue 轨照常、VM 轨 stub
    照旧。
  - musk 侧迁移（T8 批）：chatActivePath 一族（chats_view.at:19-20）等 .at 纯 fn
    引用改 `use.web.fn`——显式化 + 校验收益；数据形态符号（agentAvatarData 类
    const）暂留裸形式（`use.web.const` 二期再议）。
- **C5**：按 `docs/specs/reports/048-sse-vm-survey.md` 阶段1，消费 C7 timer block：
  forge_store 声明 `PollStream (every_ms: 500, when: .streaming)`，handler 增量拉
  `chats_get_session` 回填 `store.messages`（msg id 去重）；`Sse.open` 在
  platform.vm.at 显式 no-op + warn（不再走断链路径）。
- **C7 timer block**（widget/store 第 12 个声明块，与 model/view/watch 并列）：

  ```auto
  widget Clock {
      msg { Tick, PollStream }
      timer {
          Tick (every_ms: 1000)
          PollStream (every_ms: 500, when: .streaming)
      }
      model { var streaming bool = false }
      on {
          .Tick -> { … }
          .PollStream -> { … }
      }
  }
  ```

  语法与语义：
  - 条目头 = **msg 变体名**（裸名，沿 actions 的 `menu (id: …)` 声明形），须在
    本 widget/store 的 `msg {}` 块声明——解析期校验（沿 Plan 451 actions
    handler 校验承诺）；到点派发该变体进**既有 msg 流**，`on {}` 零新增语法。
  - 属性括号列表沿 actions `action (id: "…", handler: .ActX)` 先例：
    `every_ms: <int>`（周期毫秒；运行期钳制下限 16ms 防忙轮询）；
    `when: <cond>`（可选门控，沿 actions `enabled_if` 的 `eval_condition_with`
    合并根 state 求值语义——条件假**不派发但不停止底层计时**，纯过滤）。
    `if` 不作属性名（关键字冲突，actions 用 `enabled_if` 同因）。
  - 生命周期：widget 计时器挂载即启动、卸载即停止；store 计时器随应用生命周期
    （StoreDecl 同步支持该块，musk 轮询落 forge_store）。
  - vue 轨映射：widget=onMounted `setInterval` / onUnmounted `clearInterval`；
    store=模块级 interval（app unmount 清）；`when` 假时跳过派发。
  - VM 轨映射：iced Subscription（勘察 `iced::time::every` 或沿 MCP action 通道
    轮询订阅先例开 ticker 通道）→ DesktopMessage → 既有 handler 泵。
  - 二期扩展（本批不做，留登记）：`after_ms:` 一次性延迟触发、`send:` 载荷、
    命令式 `Timer.start/stop` 动态启停（MVU 纯度优先，门控用 `when` 表达）。
- **C6**：017-chat VM 冒烟脚本入 `tests/`（node，起 release auto run --render=vm +
  MCP `autoui_type`/`autoui_action`/`autoui_state` 断言），断言集：初始 messages
  ≥5、输入 type 回写 draft、按钮与 Enter 双路发送后 messages 增 1 且 draft 清空。
  SSE 断链本计划不修（见待澄清4）。

## 测试设计

- auto-lang：每能力 `plan051_*` 前缀单测（沿 plan050 先例：class/解析断言、renderer
  断言、corpus 全链），`cargo test -p auto-lang --features ui-iced plan051_` 绿 +
  `--lib` 全量绿（以合入时基线为准）。
- 017-chat：VM 冒烟脚本 PASS（退出码 0）；vue 轨 playwright 9/9 不回归。
- musk：vue 三门禁（`auto build` strict 零 error、`npx vitest run`、scripts/lib-parity
  对拍）+ style-parity 58 用例 + `scripts/vm-link-probe.cmd` PASS +
  `node scripts/vm-first-run.mjs` alive reds=0。
- 会话闭环验收：`musk serve` :8081 + VM 实例，MCP `autoui_snapshot`/`autoui_state`
  双证（气泡入列/状态回写），材料存 `docs/plans/051-review/`。

## 验收标准

1. musk VM 会话页消息列表可见：用户气泡右侧主色圆角、AI 左侧 hairline 壳、
   🧑 You/🤖 AI 名徽章、时间戳；markdown 正文以纯文本降级渲染（内容可读）。
2. 输入框：打字回写 draft；Enter 与发送按钮双路发送；发送后用户气泡入列
   （`chats_send_message` 契约往返 + 列表刷新）。
3. 流式降级：AI 回复以轮询形态逐步出现；`Sse.open` VM 不炸（显式降级登记）。
4. 017-chat VM 冒烟脚本 PASS 且入库常驻门禁（vue 轨 9/9 不回归）。
5. vue 轨零回归（上述门禁全绿）；vm-link-probe PASS；first-run alive reds=0。
6. KNOWN-DEBT-AND-RISKS.md 增 051 行：四能力落点 + SSE 阶段2/markdown 富文本/
   轮询触发形态留档。

## 执行步骤

- [x] **T1** musk 断点全清单实机取证：起 `musk serve` :8081 + VM 实例
  （`AUTO_BACKEND=http://127.0.0.1:8081 AUTOUI_MCP_PORT=9250 auto run --render=vm`），
  登录进会话页，逐断点验证 C1-C5 假设（filteredMessages WARN 刷屏、Markdown
  fallback、`.SendInput` 不派发、Enter 路由、`Sse.open` 形态、fn 别名链在
  computed 体调用场景的贯通）；产出 `tmp/plan051-survey/gaps.md`（每断点行：
  现状/证据/依赖能力号）。
  验证：gaps.md 存在且断点行与 C1-C5 对齐。
  [✅ 已完成] 六断点全坐实与假设对齐：①Enter 后 draft 不清且零派发日志；②按钮
  press 子 .send 派发+draft 清空但宿主 SendInput 静默丢（零报错零副作用）；③
  read_state_as_vec('filteredMessages') WARN 164 次刷屏、消息区仅 gate/run 卡；
  ④renderer.vm.at Markdown 降级组件在库未激活（装载器 fn-only）；⑤ports/ 全目录
  无 Sse 形态；⑥fn 别名链未及验证（断在 resolve_iterable 层，留 T4）。侧观察：
  i18n 参数模板未插值（{count} 条/${titleText}）记 gaps.md 备查。证据=tmp/
  plan051-survey/{gaps.md,01-main-ui.txt,02-chat-view.txt,vm-run.log}；后端沿
  用 :8081 既有实例（PID 30976）。
- [x] **T2** C1 Enter 键盘接线（auto-lang worktree）：grep `key_enter` 定位键盘路由
  落点，焦点 input 声明派发（onenter / onkeydown.enter.exact.prevent 两形态）；
  017-chat onenter 断链先红后绿。验证：`cargo test -p auto-lang --features ui-iced
  plan051_` 绿。
  [✅ 已完成] 三点落地（auto-musk-dev@8feccfca9）：①textarea keydown 收集修饰段
  整段过滤——`enter.exact.prevent`→键名 `enter`（此前 "enter.exact" 与 iced
  key_binding 归一化键名永不命中，真键盘 Enter 不派发）；②keydown 实参改
  event_to_message_with 烘焙 + parse_event_param_expr 裸单段名（this.text）Ident
  兜底（实测 parse_expr_fragment 不收裸名，.send(.text) 形态此前落字面量串）；
  ③MCP 键盘 Enter 先查焦点 input 声明（enter_handler_in_view：onenter→on_submit
  优先、textarea keydown["enter"] 次之）合成 Submit 派发，不再落根 key_enter
  （017-chat `App.key_enter failed` 即此路）。plan051_ 4 测先红后绿；模块回归
  70 测绿（aura_view_builder/mcp_server/keydown/textarea 过滤集）。
- [x] **T3** C2 msg 参数回调通用路由：`on_send(text)`/`onsend: .SendInput($event)`
  派发臂；017-chat DoSend→App.SendMessage 断链复现红→绿；musk SendInput 形状单测。
  验证：同上。
  [✅ 已完成] 新增 `ui/child_emit` 双表路由（auto-musk-dev@bc9446ff3）：ROUTES
  （Component 调用位 on* 回调绑定→父 widget/handler/params，render_child_widget
  双胎 props/events 双扫——实测绑定落 events、Plan 345 分流兜底 props）+
  STRIPPED（handler 合成期 strip_callback_calls 剥离的 on_*(arg) 记录，arg 文本
  存——Expr 非 Send）。on_with_input_for 派发后两形态回送：①声明式 lookup
  "on"+msg 名（musk `send(str)`+`onsend: .SendInput($event)`，载荷=首实参/输入
  值）；②体内式按**前置快照**实参（源序 on_send(.draft) 先于 .draft=""，快照
  必须先于子 handler 执行——测试坐实该语义）；统一根态 call_handler_for 父
  namespaced handler。stripped_arg_text 兼容 rewrite_state_refs 的 `__state.draft`
  重写形态。双样点集成测先红后绿（含根/子声明顺序勘误：from_decl 合成根=首
  声明）；plan051_ 6 测 + 模块回归 129 + plan370 18 绿。renderer 的 PromptBar
  特判保留（含 blocks 簿记副作用，摘除需 ash-gui 回归，通用路由已覆盖新形态）。
  踩坑记录：AuraWidget 合成路径（synthesize_handler_fn）无 strip_callback_calls
  ——体内 on_* 调用在该路径链接失败，生产 VM 走 from_decl 路径不受影响。
- [x] **T4** C3 computed for 源：`resolve_iterable` state miss → computed 表回退；
  filteredMessages 链式调用形状单测。验证：同上。
  [✅ 已完成] auto-musk-dev@24daaf95e：resolve_iterable 单段名与 tracked
  ForLoop 的 state 读 miss 均回退 eval_computed（value_to_iter_vec 统一
  Array/堆 ListData id(≥4M)/VmRef 三形态）；resolve_expr_to_value Call 臂增
  裸 fn 名调用（use.web 别名链经新增 VmBridge::call_vm_fn 在 VM 内执行，
  裸名→import_aliases 限定名双查，栈顶 nanbox 解码含 TAG_LIST）；**关键勘误**：
  push_value 对 VmRef/大 Int 实参落 push_i32(0) 占位（注释自述"not passed
  as scalar args"）——列表实参整体变 0，按 id 空间改 encode_list/
  encode_object；VmBridge 增存 import_aliases。集成测：子 widget computed
  链式双 fn（pass51c→tail51c）作 for 源 3 行渲染先红后绿；plan051_ 7 测 +
  模块回归 130 绿。
- [x] **T5** C4 adapter widget 注册 + use.web 点分 kind：`load_ext_imports_for_vm`
  widget 臂（musk renderer.vm.at 的 Markdown 形状做注册断言）+ parser
  `use_web_stmt` 扩 `use.web.fn`/`use.web.component`/`use.web.composable` 点分
  kind（旧形式兼容等价）+ 显式 fn 声明的 .at 目标导出校验（typo 报错）。
  验证：同上。
  [✅ 已完成] auto-musk-dev@a91b04423：①widget 臂——adapter 链解析到的 .at
  目标内同名 widget 声明经新 out-param 注册进视图 registry+child_decls
  （renderer.vm.at Markdown 降级激活）；②ExtImportKind 增 ExplicitFn 变体
  （use.web.fn 点分形态；`fn` 为关键字 token 需特判）——.at 目标未导出该
  符号编译期报错点名符号，裸/空格旧形式保持 Fn 不校验；③ExtImportRef 携带
  kind，plan370_test_support 同步。语料 test/ui/plan051_ext_widget（双 adapter）
  三测：Markdown 以 VM adapter 降级渲染 source（值=vm-plain-degraded 证 adapter
  链取向）/点分三形态与旧形式等价/typo 报错。plan051_ 10 测 + plan442/plan370/
  parser 192 + vue 249 绿。
- [x] **T6** 017-chat VM 冒烟脚本：`examples/ui/017-chat/tests/vm-smoke.mjs`（起
  release VM + MCP 断言集，含 Enter 与按钮双路）；SSE 断链不在此门（登记）。
  验证：脚本退出码 0；vue 轨 playwright 9/9 不回归。
  [✅ 已完成] auto-musk-dev@c4dae2f1c：vm-smoke.mjs 四断言（A seed≥5 上屏 /
  B type 回写 draft / C 按钮发送闭环 / D Enter 发送闭环），auto.exe 解析序
  env AUTO_BIN→仓根 release→PATH。实测（worktree 新构建 release exe——C1/C2
  落地后）四断言**一次全绿**；vue 轨 playwright 9/9 全绿（清 482/483 worktree
  残留 vite 腾 :3000 后跑通）。SSE poisoned export 不碍发送闭环（017-chat
  SendMessage 无 SSE 调用，毒导出仅影响 Init 期 stream 订阅路径，维持现状
  沿待澄清4）。
- [x] **T7** auto-lang 收口：`cargo test -p auto-lang --lib` 全量绿 + no-ff 合回
  master + 主检出 release 重装（`cargo build --release -p auto`）+
  `scripts/vm-link-probe.cmd` PASS + first-run alive reds=0。验证：命令全过。
  [✅ 已完成] ①--lib 默认 3257 绿 0 败；ui-iced 3946 绿 1 败=
  vm_code_editor_natives_end_to_end（master 先在——主检出 master 复核同败，
  050 提交已记录该基线）；②no-ff 合回 32dd2d4b1（master 未提交文件
  docs/.next-id、480/481、rust-workspace 零重叠原样保留；worktree 两处
  rust-workspace 生成物还原后折叠）；③主检出 release 重装 exit 0 +
  auto-musk-dev worktree 回同步 master；④vm-link-probe PASS（61118 bytes，
  WARN 线 90K 未触）；⑤first-run alive=yes reds=0（新 master release exe ×
  musk 主检出）。**侧记**：style-parity 现 12 条 border-t/b 基线红——
  b00280130（pre-merge）差分同败，坐实 master 先在（疑 050 后 480-483/
  052 合并引入），非本计划；T11 复核口径。
- [x] **T8** musk 发送链路+列表渲染实测：worktree plan-051-dev 起实例，Enter/按钮
  → `.SendInput` → `chats_send_message` → 列表刷新 → 气泡入列；气泡/名字/时间/
  分向/正文降级逐项核对；必要 .at 微调在 worktree。验证：MCP 快照+状态双证存
  `docs/plans/051-review/`。
  [✅ 已完成] 数据面全通（会话切换 NavListItem→SelectSession→messages 入态；
  按钮发送 MentionInput.send→C2①onsend→SendInput→store.Send+StartStream→
  streaming=true→chats_send_message 后端 count 2→3 末条即所发文本 curl 实证）。
  视觉核对（多模态 × vtree 双证，2026-08-30 补录）：左右分向 ✅（You 右/AI 左）、
  角色徽章 ✅、AI hairline 上下线 ✅（PLAN-050 T4 能力在会话页生效）、gate/run
  卡共存 ✅、输入区 ✅。残余三项均已定责登记：正文空白+用户气泡底色塌缩（VM
  for-in 栈失衡上游债，待澄清6）、时间戳空（Date.format 无 VM native）、
  i18n 模板未插值；**发送后用户气泡不可见**定责 T10（乐观 push 仅 web 轨
  forge_stream.ts，VM 无 SSE 无轮询，非 C1-C4 回归）。musk 侧 .at 修正 3 处 +
  上游随修 2 项（call_vm_fn retain 悬挂 id 根因+回归锁，auto-musk-dev@4e0d9ffff）。
  证据=docs/plans/051-review/{SEND-EVIDENCE.md（含视觉核对节）,01/02 快照,
  t8-01/02 截图,vm-run-t8.log}。
  **补录（2026-08-30 续测，日志铁证）**：SendInput 崩溃点定位——
  `handler_ChatsView_SendInput` 内 StartStream 的 `Sse.open(path, .OnStreamEvent)`
  第二实参（msg 变体引用 `.OnStreamEvent`）被当 state 字段 GET_FIELD →
  `Field 'OnStreamEvent' not found on App_State` RuntimeError（vm-run-t8.log
  唯一非心跳 VM-EMIT 行）。KD-047 G1 实机形态升级：非静默断链而是**崩溃中断
  + 疑似级联**（其后 messages=[] 写入者未定位，唯一精确写入点 NewSession/
  DeleteAllSessions 均未触发——复核时误触过期 vnode 实证了 NewSession 清空
  语义本身正确）。**T10 设计增补**：platform.vm.at Sse no-op 须连同解决
  裸 msg 变体引用在表达式位的求值（→ Nil/msg-ref 而非 GET_FIELD 崩溃），
  PollStream 替换 StartStream 后复验 messages=[]。
- [x] **T9** C7 timer block DSL 上游落地（auto-lang worktree）：AST（WidgetDecl/
  StoreDecl 增 `timer` 槽 + TimerBlock/TimerEntry 结构）→ parser 上下文关键字
  （parser.rs:12225 邻域 bind/watch/setup/actions 同列 + store 解析位）→ vue
  codegen（setInterval/clearInterval 映射）→ VM iced Subscription（勘察
  iced::time::every vs ticker 通道，沿 renderer.rs:5244 先例）→ `when` 门控接
  eval_condition_with；TDD 先红后绿（含 msg 变体校验、卸载清理断言）。
  验证：`cargo test -p auto-lang --features ui-iced plan051_` 绿 + 017-chat
  试点加 demo 计时器 vue 轨不回归。
  [✅ 已完成（auto-musk-dev@480514c42，27 文件 875 行）] Subscription 形态裁定
  =AppTickRecipe 泛化（新 WidgetEvent(String,String,u64) 变体，非 ticker 通道/
  非 iced::time——459 订阅基建直扩）；when 门控落 component 层平面根态求值
  （fire_timer：假丢弃本拍不停底层计时），update 侧按 is_timer_entry 分流。
  vue 轨两修：import 需求必须在语句 join 前声明（发射点晚于 join 的时序坑，
  实机 9 败定位）；store composable 用模块级 started 旗标防多次调用重复建
  interval（沿 stream_guard 先例）。TDD：plan051_timer 8 测先红后绿（parser
  widget/store/校验 + vue widget/store 发射 + VM 收集/派发/门控）；017-chat
  demo 计时器双轨全绿——VM vm-smoke 五断言（新增 E：clock_secs 3s 内 19→22
  精确递增）+ vue playwright 9/9；模块回归 parser205/ui_gen709/dynamic29/
  plan051_20 全绿。store 计时器经 lib.rs store→无视图 child WidgetDecl 转换
  （timer 字段随迁）走既有 handler 泉——PLAN-048 时代的"Init 代派"类桥接
  不再需要。
- [x] **T10** musk 流式轮询消费（G1 阶段1）：forge_store 声明 `PollStream
  (every_ms: 500, when: .streaming)`，handler 增量拉 `chats_get_session` 回填
  （msg id 去重）；`Sse.open` platform.vm.at 显式 no-op。验证：VM 实机流式回复
  逐步出现（降级登记）。
  [✅ 已完成（musk plan-051-dev@b9a0da4 + auto-lang auto-musk-dev@40744b2fa）]
  实施勘误两处：①回填采全量替换（.messages = resp.session.messages）而非
  id 去重增量——后端即真源、消息不可变场景等价且 .at 形状更简；②Sse no-op
  落点从 platform.vm.at 改为 VM stdlib native（Sse.open/close 返回 0 句柄
  令牌），并顺手补 Date.now（epoch ms）——实锤 T8"发送后用户气泡不可见"
  的另一半根因=StartStream 在 Date.now() 中止（乐观 push 之前）。
  实机验证（决定性证明）：登录(plan048user)→选会话→按钮发送→乐观 push
  即时入列（messages []→[vmref] state_changes 实证）+streaming=true→
  curl 经 API 注入第二条消息（后端 1→2）→**VM 状态 2s 内跟随 msgs 1→2**，
  轮询拍/门控/回填端到端工作；user 末条时 streaming 正确保持（完成启发式
  的 assistant 分支因 demo 后端无 LLM daemon 未及实测，逻辑单测覆盖）。
  环境限制登记：assistant 增量流不可现（无 aaid daemon），SSE 阶段2 桥
  泛化后本降级退役。VM 实例静默退出（~2-3min exit 1 无 panic）复现一次
  ——KD-048-a 已知债窗口内，非本次改动引入（GET_FIELD 每帧探针亦 T8 基线
  先在）。
- [x] **T11** 全量门禁 + 收尾：vue 三门禁 + style-parity + 探针 + first-run 全绿；
  KNOWN-DEBT 增 051 行（含 C7 timer block 能力登记与二期扩展留档）；worktree
  折叠（auto-musk-dev 与 plan-051-dev 合回各自 master/main 并清理）。
  验证：门禁输出贴计划复审记录。
  [✅ 已完成] ①auto-lang 侧：T9/T10 批 no-ff 合回 master（096786d70）+ 主检出
  release 重装 + 全量 --lib 4041 过 5 败（全部已知：plan050×2 基线/dock×2=
  473 在途债成对污染/code_editor 基线，与 051 零交集；首轮"挂起"系游离
  017-chat 后端 cargo 干扰，清理后 62.5s 完成）。②musk 门禁（plan-051-dev ×
  release PATH 前置）：auto build strict ✅（vendor/@autodown/engine dist 本地
  补齐后——gitignored 产物新 worktree 需复制，环境注记入 KNOWN-DEBT）；vitest
  46 过 2 跳 ✅；phase1-leaves 30/30 ✅；style-parity 12 条基线红**主检出差分
  恒等**（051 零新增，沿 T7 登记）；vm-link-probe PASS 60966B；first-run
  alive=yes reds=0。③KNOWN-DEBT 051 行已增（七能力交付+六项上游留档+基线红
  登记+环境注记）。④worktree 折叠：auto-musk-dev 增量已随 096786d70 落
  master（worktree 本体清理随 /auto-plan:merge 终态）；plan-051-dev 分支
  留待 /auto-plan:merge 合回 main（技能规则：终态折叠归 merge 技能）。

## Phase 2——会话壳视觉五缺陷（2026-08-30 用户实测回归）

> 复审归档后用户实测 VM 版 Chat 栏目仍乱（截图五点）。vtree+源码对拍诊断已
> 完成根因定罪，逐项修复（vue 轨零回归约束不变）：

- **P2-① icon 按钮空白**（二级导航头 Plus/Trash2、列表删除钮 Trash2、搜索框
  Search）：注册表收集（lib.rs:3762 邻域）只收"根 AST 顶层 use.web + widget
  内嵌 ext_imports"两路——**子模块文件顶层 use.web component（chats_view.at:26、
  nav_item.at:15）不在收集面**→is_imported_component 假→unknown fallback→Empty。
  rail 图标正常恰因其在根模块（对照组实证）。修：收集点补第三路（装载期
  visited 模块扫描，沿 ext_stubs 同款模式）。
- **P2-②a 列表项第三行垃圾**：delete span（absolute+hover）按 C6 降级为流内
  第三子。修（musk 侧）：span 类串加 `hidden`——web 侧 .session-delete-btn
  本就 display:none、hover 经 0,3,0 特异性覆盖 tailwind 单类，**零 web 回归**；
  VM 走既有 is_hidden 臂不渲染（hover-only 件不进 VM，登记）。
- **P2-②b `{count} 条` 未插值**：call_expr_t_key 只取首参，第二参（参数记录
  字面量）被丢。修：t() 臂增参数求值（bindings 感知）+ lookup 后 `{k}` 替换。
- **P2-③ 搜索框挤压**：`flex-[0_1_320px]` 任意值不支持→整类丢弃→塌 0。修
  （musk 侧）：类串补 `min-w-[200px] w-[320px]`（min-w 解析在册 class.rs:1138；
  web 仅极窄窗行为微改进）。
- **P2-④ 输入框不显示**：apply_container_style（renderer.rs:1840 normal 分支）
  **无 min_height/min_width 消费**→input-compose（div→Container）的 min-h-20
  丢弃→高度塌。修：补 min_h/min_w 消费（镜像 Column 臂 1524；语义近似=Fixed
  而非下限，textarea 自滚场景正确，登记）。
- **P2-⑤ 消息区**：①-④ 通后端到端实测（发送→气泡→轮询）。

执行步骤：
- [x] **T12**（auto-lang/auto-musk-dev）P2-① 注册表第三路 + 回归锁（子模块
  顶层 use.web component 名进表；先红后绿）。
  [✅ 已完成 2c36322c0] visited 重解析第三路（沿 ext_stubs 模式）；语料
  plan051_p2_modules（子模块顶层声明双图标形态）红→绿。
- [x] **T13**（auto-lang）P2-④ apply_container_style 补 min_h/min_w + 单测。
  [✅ 已完成 2c36322c0] normal 分支 else-if 链补消费（9999 哨兵→Fill 镜像
  Column 臂）；语义近似=Fixed 下限而非弹性最小高（自滚场景正确）登记。
- [x] **T14**（auto-lang）P2-②b i18n 参数插值臂 + 单测。
  [✅ 已完成 2c36322c0] t_call_params（第二实参 Expr::Object 逐字段 bindings
  求值）+ substitute_params（{k} 替换、未提供原样保留）；文本/prop 双位接线。
- [x] **T15**（musk）P2-②a hidden 类 + P2-③ 搜索框宽度类。
  [✅ 已完成 1eb2e38→main 05e29c9] hidden 类（web hover CSS 0,3,0 特异性
  覆盖零回归/VM is_hidden 不渲染）+ min-w-[200px] w-[320px]（web 仅极窄窗
  微改进）。执行波折登记：worktree 目录被残留 node.exe 文件锁拖入嵌套
  重建循环，最终 T15 直落 main（05e29c9）。
- [x] **T16** release 重装 + VM 实机五项逐一对拍（截图存档）+ vue 门禁抽查。
  [✅ 已完成（结构级四项实证+像素级交用户目验）] ①vtree [Image] 16 枚
  （原~5，会话栏头双钮/搜索框/发送钮/工作区钮全出图标）②列表项恰两行
  （title+"N 条"，无第三行垃圾钮）②b"3 条/0 条"插值实机生效 ③搜索框
  w-[320px]/min-w 类进 Width/MinWidth 消费 ④textarea+80px 容器链在树
  ⑤选会话→发送→backend 第 4 条即所发（往返闭环）。截图=tmp 下
  autoui-screenshot-1788059593387.png（judge 子代理与本会话 Read 均只能
  转 CDN 无法目检——环境限制登记，像素级终验交用户）。vue 门禁：build
  strict ✅ + vitest 60 过 2 跳 ✅；plan051_ 全前缀 23 测绿。
- [x] **T17** 计划标记/KNOWN-DEBT 补行 + 收尾。
  [✅ 已完成] 本标记 + KNOWN-DEBT 051 行 Phase 2 附记（含新债：MentionInput
  .cancel 声明式路由未达——.MentionInput.cancel 派发而宿主 .StopStream 未
  接上，streaming 锁死 composer；C2 无参 oncancel 对缺口，下游批次）。
  auto-musk-dev@2c36322c0 待随 Phase 2 收口合回 master。

## Phase 3——用户二轮实测三修（2026-08-30 P2 合并后）

- **P3-① 输入框有框无法输入**：textarea（absolute inset-0）在 VM 轨降级流内
  后仅自然高度悬于 80px 容器顶部，点击容器中下部不命中输入件。修：类串补
  `h-full` 填满容器（web 侧 absolute 几何已定尺寸，冗余无副作用）。
- **P3-② 搜索框打字显示"0"**：`.OnSearchInput(val)` 单参 oninput 双轨契约下
  val 已是纯文本（vue codegen 自动包 $event.target.value），源里再取
  `val.target.value` 在 VM 得 0（GET_FIELD on Str）。修：`.chat_search = val`
  （顺带修复 web 侧 undefined）。审计：.at 视图层仅此一处；specs_helpers 的
  stepValueOf 同族形态留 specs 域批次。
- **P3-③④ "切换不生效"+面板杂物**：SwitchSession 不清会话域状态——旧会话的
  current_gate/report_data/errands/relays/task_plans 跨会话残留，切到空会话
  时 gate/run 卡仍杵着（观感=切换无效）。修：切换时全清 + streaming/draft/
  thinking 复位。gate/run 卡本体是会话数据（assistant 工具使用史），行为正确。
- 实机验证（9db3a0d 后重启）：chat_search="hello"（经真实 handler 路径）✓；
  切 New chat→current_gate/report nil+messages 空+面板无卡 ✓；textarea
  "p3 check" 写入 ✓（真键盘命中区为 h-full 确定性修复，交用户目验）。

## 复审记录

**复审人**：zhaopuming（/auto-plan:review，2026-08-30 02:10）
**方法**：计划 × 真实差异对拍（musk plan-051-dev 3 提交 4 文件 +53/-10；auto-lang
master 096786d70 合并链 4e0d9ffff/480514c42/40744b2fa）+ 全量门禁复跑 + 实机
全链复验（当前 master release，含 8302f0e54 .length 白名单修复后构建）。

**逐准则裁定**：

1. 消息列表可见 —— **部分达成（降级口径，债登记在案）**：结构/左右分向/
   🧑 You/🤖 AI 徽章/AI hairline 壳实机复验 ✅（本次快照双 user 徽章 + T8
   多模态×vtree 双证）；**正文空白**（上游 VM for-in 直接调用源栈失衡，
   待澄清 6——musk 已 let 绑定绕开+回归锁，.length 修复后复验仍未解，
   符合单元级定性）与**时间戳空白**（Date.format 无 VM native）为上游
   债降级达成。Markdown 降级组件本体已激活（plan051_ext_widget 语料
   vm-plain-degraded 断言）。
2. 输入双路发送 + 往返入列 —— **通过**：本次实机 type→press→draft 清空
   →streaming=true→乐观 push（messages 2→3）→后端 count 3 末条即所发
   文本（curl）✅。侧记：type→press 零间隔连击有 draft 回写竞态（MCP
   人工时序，非产品缺陷，人手速不可达）。
3. 流式轮询降级 —— **通过**：T10 决定性证明（后端 API 注入 1→2，VM 状态
   2s 跟随）+ 本次 streaming 门控复验；Sse.open no-op native 不炸 ✅。
   demo 后端无 aaid daemon，assistant 增量不可现=环境限制（登记）。
4. 017-chat VM 冒烟常驻门禁 —— **通过**：当前 master release 复跑五断言
   全绿（A seed/B type/C 按钮/D Enter/E timer clock_secs 5→8）；vue
   playwright 9/9（T9 时点）。
5. vue 轨零回归 + 探针 + first-run —— **通过**：cargo tf **3280/3280 全绿**
   （本次复审全量档，含 schema 三件套+1M churn）；musk build strict ✅ /
   vitest 46 过 2 跳（本次复跑）/ phase1-leaves 30/30 / style-parity 12 条
   基线红主检出差分恒等（零新增）/ vm-link-probe PASS 60966B / first-run
   alive reds=0（T11 时点，同构建链）。
6. KNOWN-DEBT 051 行 —— **通过**：七能力交付+六项上游留档+基线红登记+
   环境注记（vendor engine dist 本地产物）。

**遗漏/延后/workaround 扫描**：musk diff 零 TODO/FIXME；八项延后全部显式
登记（待澄清 4/5/6 + KNOWN-DEBT 051 行），无静默缩水。债候选（下游批次）：
①VM for-in 直接调用源栈失衡（RET 帧校验/临时局部编译期引入——上游）；
②Date.format native；③i18n 参数模板插值；④call_vm_fn retain 配对释放
（长会话内存）；⑤SSE 阶段 2 桥泛化（PollStream 退役条件）；⑥VM 静默退出
KD-048-a（本次复现第二次，~4min 无 panic）。

**路由**：reviewed（通过）——准则 1 两子项为上游独立缺陷所致降级达成，
根因二分+回归锁+musk 侧绕开+显式登记齐备，修复体量超出本计划边界且已经
用户多轮汇报知情；其余准则全部实证通过。

## 待澄清事项

1. **轮询触发机制（已裁定 2026-08-29）**：task 体系无 timer 且与 widget 体系无桥、
   `Time.sleep_ms` 阻塞不可用于 UI（核实记录见需求分析节）→ 走 C7 timer block
   DSL（语法见详细设计）。VM 侧 Subscription 形态二选一（iced::time::every vs
   ticker 通道）由 T9 勘察裁定；二期扩展（after_ms 一次性/send 载荷/命令式启停）
   不在本批。
2. **`use.web` fn 别名（已裁定 2026-08-29）**：复核 lib.rs 证 `.at` 目标 fn 别名
   VM 已通（adapter 装载链，见详细设计 C4 勘正），KD-047 ② 缺口仅 widget kind；
   语法裁定=**`use.web.fn` 点分形式**（用户裁定）替代引入空格 kind 关键字——
   `use.web.<kind>` 与 use.rust/use.py 家族一致，显式声明换取装载校验（typo
   不再静默 stub）。旧空格/裸形式兼容保留。剩余实测点（T1）：fn 别名链在
   **computed 体调用**场景的贯通（C3 的 resolve_iterable→computed 求值→限定名
   reloc 是否解析）——这是集成验证而非"是否断"。
3. **MCP `autoui_type` id→action 对位错位（已修 2026-08-29）**：根因双证——
   ①`find_view_by_path` 手写子枚举缺 Button{content}/Table/Tabs 臂（与
   extract_children 同构不变量破坏）；②styled_vtree 在 `__bounds_collected`
   被加工树（convert_view_messages，Tabs 族折 Empty）覆盖而 shared.view 为
   裸树，双生产者 path 空间错位。已由 auto-lang PLAN-483 T6 承修并落码
   （plan-483-dev@6bf36aca5：find_view_by_path 委托 extract_children_ref +
   bounds_collected 改取同源 mcp_sync_vtree 缓存；tests_plan483_d4 4 测
   先红后绿）。实测补充：master release exe 上最小双 input/条件子 widget/
   真实 musk 登录页三场景派发均已正确（Plan 446 J1 每帧同源推送已关稳态
   窗口），T8 的 MCP 驱动验收不再依赖此项——合并随 PLAN-483 收口节奏。
4. **017-chat `api.stream` poisoned export**（merged 模式 `Undefined variable:
   bus`）：本计划不修 SSE in VM（阶段2 出范围）——但若"毒死导出"影响同文件其他
   端点装载（list_messages/send_message 实测未受影响），维持现状；受影响则最小
   降级（导出空 Stream + warn）随 T6 处理。
5. **worktree 依赖**：上游改动续用 `.worktrees/auto-musk-dev`（PLAN-050 T10-T12
   折叠进度决定其存续——未折叠则先合 050 增量再续作；auto-lang 主检出另有未提交
   docs/Cargo.toml 改动，折叠前需用户处置，沿 050 待澄清6）。
   执行注（T7 时点）：C1-C4 批已 no-ff 合回 master（32dd2d4b1）——主检出未提交
   文件（.next-id/480/481/rust-workspace/484）零重叠原样保留；PLAN-050 的
   T10-T12 收口与本计划 T8+ 批（4e0d9ffff 起）共用该 worktree 续作。
6. **VM for-in 栈失衡（T8 勘察新证，2026-08-30）**：直接以**函数调用结果作
   for-in 迭代源**（`for b in fn(...)`）且循环体较富（if/else-if/else + 嵌套
   let + ?? + 调用）时，函数 return 的栈顶值错位（实机 messageDisplayBlocks
   返回非列表对象 → blocks for 回退 rows=0 → 气泡正文空白）。单位级二分：
   `let src = fn(...)` + `for b in src` 形状**通过**（已加回归锁
   plan051_list_iteration_let_bound_call_result）；直接调用源形态仍红——
   属 VM 字节码/RET 栈平衡上游债，musk 侧已 let 绑定绕开，**通用修复留上游
   计划**（候选：迭代源编译期为调用结果引入临时局部；或 RET 帧校验）。
   连带登记：call_vm_fn 返回堆引用 retain 已修但**未配对释放**（computed 每
   帧新建列表被 pin，长会话内存增长——per-frame 生命周期需上游语义）；
   `Date.format` 无 VM native（msgTimeLabel 时间戳空白）；i18n 参数模板
   （`{count} 条`/`${runId}`）VM 未插值。 项 6 续（T10 前置）：`Sse.open(path, .OnStreamEvent)` 的 msg 变体引用实参
   GET_FIELD 崩溃（见 T8 补录）——T10 落 Sse no-op 时同点修：裸 `.MsgVariant`
   在表达式位求值为 msg-ref/Nil；StartStream 乐观 push 依赖的 `Date.now()`
   native 缺位同批核实。
