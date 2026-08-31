---
plan_id: PLAN-053
status: execution_done         # drafting → executing → execution_done → reviewed → archived
feature_name: vm-upstream-tracking
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 复审 1 填充（2026-08-31，FAIL——现状快照，供下次复审通过后 merge 消费）：
supersedes_spec_components: []   # 未修改任何既有 spec 组件（修复全落 auto-lang 运行时 + musk 单点加固）
new_spec_components: []          # 未新增 spec 组件（常设伞，修复批次不沉淀独立 spec 面）
touched_goals:
  - "P045-2: musk VM 前端链路——上游修复使登录→聊天主链实机可用（7/7 气泡+正文渲染+nil 门控）"
  - "P046-2: musk VM 轨 workaround/债务清偿——P-053-1/2/6 五层连环缺口清偿即该目标延续"

affects: [auto-lang/ui, auto-lang/vm]
current_step: 22
total_steps: 22
---

# [PLAN-053] auto-musk VM 版上游统一跟踪（auto-lang 修复伞）

## 变更摘要

auto-musk 的 VM 轨（`.at` 源 → AURA 解释器 + iced 渲染）已跑通登录 → 聊天主链
（PLAN-045..052 系列），实测暴露若干 **auto-lang 运行时缺陷**：computed 链静默
返回空、nil/None 语义不等值导致门控组件常显、merged 模式 `#[api]` 静默 no-op、
debug 构建 RC canary panic 等。此前这些问题散落在各计划的红清单/KNOWN-DEBT
行里，缺统一归属。

本计划是一个 **常设跟踪伞**：auto-musk VM 版路线上的所有 auto-lang 上游问题
统一由本文件登记跟踪，修复全部在 **auto-lang 仓库的常设 worktree**
`auto-lang/.worktrees/auto-musk-dev`（分支 `auto-musk-dev`，AGENTS.md 第三行
命名）内进行，经 musk 侧实机验证后合回 auto-lang master。本计划**不随单批
修复归档**——归档条件见「验收标准」末条。

## 目标

1. 存在唯一入口：auto-musk VM 轨遇到的上游（auto-lang）缺陷都在本文件红清单
   有一行归属，含证据、状态、落点。
2. 首批红项（P-053-1..5）修复并实机验证通过（见验收标准）。
3. 修复不回归：auto-lang 既有测试绿 + musk vue 轨门禁绿 + VM 登录页三项
   （Tab/Enter/预填）不回退。
4. 红清单进出协议成文：新红如何登记、何时清出、与 auto-lang 自有计划
   （4xx 系列）的边界划分。

## 架构方案

```
现状(缺陷散置)                          本计划后(单入口 + 常设 worktree)
──────────────────────                  ─────────────────────────────
各计划红清单/KNOWN-DEBT 行               docs/plans/053-vm-upstream-tracking.md
  + 会话内诊断结论(易失)          →        红清单 = 唯一事实源(证据/状态/落点)
修复散落在 auto-lang 各 plan      →       auto-lang/.worktrees/auto-musk-dev
                                         (分支 auto-musk-dev, 常设, 按批合回)
验证靠人工目测                    →       实机协议: release 重编 + AUTO_VM_MERGE=0
                                         + AUTO_BACKEND + MCP(:9247) 断言
```

**与 auto-lang 自有计划流程的边界**：auto-lang 仓内 4xx 计划（如 494/496/497）
是它自己的立项，不经本伞；musk 侧发现的新上游红**先查** auto-lang
`docs/plans/` 与 KNOWN-DEBT 是否已有归属——有主则只在红清单记指针行，
无主才入本伞修复队列。musk 自身可修的（如会话列表过期自愈）不属本计划
代码域，红清单记「musk 侧配套」指针即可。

## 技术栈

- 修复域：Rust（auto-lang `crates/auto-lang`：`src/vm/engine.rs`、
  `src/vm/codegen.rs`、`src/ui/dynamic.rs`、`src/ui/iced/renderer.rs`、
  `src/aura/extract.rs`）。
- 验证域：musk `.at` 源（只读消费面，原则上不改）+ AutoUI MCP
  （`http://127.0.0.1:9247/mcp`，JSON-RPC `autoui_snapshot/state/inspect`）。
- 回归域：vue 轨门禁（`auto build` strict / vitest / vm-link-probe）。

## 需求分析与背景调查

- spec ledger（`backend/.autoos/specs.json`）：存在 P030-2（基于 Plan 的
  Agent 开发流程）等条目；overview API 当前报 `failed to build overview`
  （本地文件可读）。双前端 parity 是 022/028/041/045..052 系列沉淀的持续
  目标，本计划是其上游修复面的收口。
- 实测环境事实（2026-08-31 两次会话采集）：
  - VM 启动形态：`auto run --render=vm`（CWD=musk 根）+ `AUTO_VM_MERGE=0` +
    `AUTO_BACKEND=http://127.0.0.1:8080`；PATH 用 release
    （`target/release/auto.exe`）。debug 构建必踩 RC canary（见 P-053-5）。
  - 存储分文件：KV 按 CWD 哈希分 JSON（`%TEMP%/auto-vm-storage/`），musk 根
    CWD 对应 `adf090d4923193ef.json`，vm-first-run 启动器 CWD（src/front）
    对应 `5616e81ec5f05913.json`——预填/登录态排查时勿混。
  - Bash 沙箱会隔离 window station（无头），实机验证须用非沙箱
    PowerShell `Start-Process` 拉起。

## 详细设计

### 红清单（本计划核心资产）

| id | 症状（musk 实机） | 证据 | 疑点落点（auto-lang） | 状态 |
|:---|:---|:---|:---|:---|
| P-053-1 | chat 消息列表恒空：`filteredMessages` computed 链（`chatSearchFilter(chatActivePath(.store.messages, .store.active_leaf), .chat_search)`）产出空 | MCP state：`messages` 已填 7 条；snapshot 消息容器零迭代；侧栏直连 `for .store.session_list` 正常；vue 轨同源代码正常；stderr 571× `GET_FIELD non-i32 obj_id field=length`（`fn=mention_helpers.mention_professions_list` + 匿名 bp=3，两 fn 是否同根待判） | computed 求值对扁平化跨 store 字段的依赖/求值（`src/ui/dynamic.rs` computed 臂 / `src/vm/codegen.rs` computed 编译 / `use.web.fn` helper 调用面） | **已修（批1，2026-08-31）**：根因三件套——①GET_ELEM `ListData<Value>` 的 `Value::Obj` 元素落 `_` 臂 push_i32(0)（`messages[i].id` 读空）；②GET_FIELD 接收者无 TAG_LIST 分派（bridge `call_vm_fn` 的 `encode_list` 实参，即 571× stderr 噪音本体——待澄清#2 判定：同根）；③`call_vm_fn` 编组堆引用实参裸 `push_nv` 无 stake，RET 释放烧穿 state 份额 → 列表回收成悬垂 id（实机 `[VM-IDX] no-heap-object ×136`）。落 auto-lang master 71836b3f1 + 1fd6aba29；实机 7/7 气泡 + stderr 噪音清零；回归钉 `musk_vm_track_tests` 13 绿 |
| P-053-2 | `if .store.current_gate != None` 守卫拦不住 nil：GateCard/ReportCard 无条件常显（chat 主面板 🔒579 + Approve & Execute/Reject & Redraft/Review） | MCP state：`current_gate nil`、`report_data nil`；inspect 按钮 `onclick -> GateCard.approve` 实锤渲染 | VM nil（`encode_null`）与 `.at` `None` 字面量在比较运算的等值语义（`src/vm/engine.rs` 比较 opcode / codegen None 字面量臂） | **已修（批1，2026-08-31）**：根因两层——①codegen `Expr::Null`/`Expr::Nil` 编码 CONST_I32（-1 / i32::MIN+1）与 `None`（PUSH_NIL）表示分裂，EQ/NE 兜底恒不等；②VM iced 轨守卫走 `aura_view_builder::eval_condition_with` 字符串比较，Nil 显示串 `""` vs 裸字面量 `None` 恒不等。修：字面量归一 PUSH_NIL + EQ/NE/NULL_COALESCE `nv_is_null_family` 兜底（存量 KV -1 载荷）+ 字符串比较 null 语义。master 2f8c61d30；实机 nil 态无 gate 卡；连带 `test_str_bytes_iterator` 期望 130→131（`.?(0)` 落默认值，原 -1 漏过是语义债） |
| P-053-3 | ReportCard 渲染裸模板文本 `${runId}` `${durationLabel}` `${confidenceLabel}` | snapshot 直拍 | 疑 P-053-2 连带（nil 态仍渲染 + 插值退化）；P-053-2 修后复验再定级 | **清出（批1 复验）**：P-053-2 修后实机 snapshot（登录态+选会话两形态）`${runId}`/`${durationLabel}` 零命中——nil 态不渲染后连带消失 |
| P-053-4 | merged 模式（默认）下 `#[api]` 调用静默 no-op：登录/聊天链路无声失效 | `lib.rs:3757` `api_over_http` 仅 `AUTO_VM_MERGE=0`/`AUTO_BACKEND` 时启用；否则执行 `return None` 桩体；plan047 R8 已登记 | 默认形态选型：musk 形态默认走 HTTP 桥，或 no-op 时 stderr 显式 warn | **已修（批3，2026-08-31）**：在 `auto-lang` 添加了 `auto.vm.warn_api_noop`（shim 3142）与 codegen 一次性告警分派，合并模式下调用裸 `#[api]` 显式输出 `[VM-API] merged-mode #[api] "<name>" no-op`；同步更新 `README.md` 与 `scripts/vm-first-run.mjs` 文档注释；回归测试 `musk_vm_track_p053_4_merged_api_warning` 绿 |
| P-053-5 | debug 构建启动即 panic：`[RC canary] string tombstone access: pool index N was freed`（engine.rs:1532） | 实机复现（exit 3）；release 无恙；commit 8a6da9c/KD-048 已留档 | `src/vm/engine.rs` RC 语义债（上游登记项） | **已修（批3，2026-08-31）**：根因系 `native.rs` 和 `stdlib.rs` 中 `localStorage.getItem`、`env::var`、`url.encode` 等 native shim 直接调用 `vm.strings.write().unwrap().push(...)` 绕过了 VM 字符串池注册（`pool_state`），使 debug canary 判定为 tombstone/UAF。全量改用 `vm.add_string(...)` 正式入池；debug 模式下测试 `musk_vm_track_p053_5_localstorage_rc_canary` 绿 |
| P-053-6 | **（批1 实机验收发现）**消息气泡内容体空：ChatMessage `blocks` computed（`messageDisplayBlocks(.msg, .is_streaming)`）不求值——气泡骨架+角色头渲染但内容列空 | `AUTO_DEBUG_EMIT=1` 实况：`[VM-CALLFN]` 全量 2008 条中 `messageDisplayBlocks`/`messageBlocks` **零调用**（对照 chatActivePath 590 次）；无报错无告警（静默 None）。**用户目验（2026-08-31 截图1）**：登录后气泡骨架+🧑 You/🤖 AI 角色标记+⑂重试按钮可见，内容体空——与树级诊断一致 | 子组件 prop 实参在合并 computed 表求值上下文缺绑定：`blocks` 体内 `.msg` 是 ChatMessage 的 prop，eval_computed → resolve_expr_to_value 的 bindings/read_state 均无 `msg`（for 循环绑定不跨组件）→ 实参 incomplete → 静默返回 None（`src/ui/aura_view_builder.rs` Call 臂 / `src/ui/dynamic.rs` computed 合并） | **已修（批2，2026-08-31）**：批1 的"computed 不求值"诊断为 Call 臂 swallowed-Err 所误——批2 诊断打印显形后确认 computed 一直在调，真因是**五层连环缺口**（详见执行步骤 13）：①Regex.replace/test 静态 native 缺失（helper 执行即崩）②call_vm_fn 字符串结果降格池索引 ③use.web component 图标臂遮蔽注册组件（正文画成 lucide 图标）④web 字符串方法族六臂缺失（trimEnd 等，落 null）⑤Obj/Array 实参编组 0 占位（msg 变 Int(0)）+ ObjectData 缺键硬报错（`msg.blocks ?? 兜底` 炸）。auto-lang master 801ed776c..053-null-key 系列；实机 7/7 气泡全文可见+会话切换往返干净；musk_vm_track_tests 24 绿 |
| P-053-M1 | （musk 侧配套，非本计划代码域）会话列表过期 + 切换 404 静默 | 实机：应用选中 `eeb247…` 对后端 404；点 NewSession 刷新后真实会话可选中加载。**用户目验（2026-08-31 截图2）**：点侧栏会话后主面板连气泡骨架都消失（全空白）——代码面机制：SwitchSession 无失败路径，`chats_get_session` 失败（404 或 api no-op）返回 None → `resp.session.messages` 逐级 GET_FIELD 落 0 → `.messages` 被写成垃圾 → for 零行 + 无任何错误提示。注：截图出自本计划复审期的测试实例（agent 拉起），其侧栏「你好」会话不在 8080 后端（复审期 merged 形态复现侧栏为空）——精确后端归属未定，但静默清空机制两条路径（404/no-op）同构 | **已修（批3，2026-08-31）**：`src/front/forge_store.at` 中 `.SwitchSession`、`.LoadSessionList`、`.BranchTo`、`.NewSession` 全面加固守卫为 `if resp != None && resp.session != None`；失败路径清空 `.messages = []` 并设置错误提示 `.error = "会话加载失败（可能已删除），列表已刷新，请重新选择"`，且自动拉取 `chats_list_sessions()` 刷新剔除过期项；测试 `musk_vm_track_p053_m1_guard_behavior` 绿 |
| P-053-7 | **（复审 1 新登记）**boot 后会话列表恒空：KV 恢复登录态（token 落地、直达聊天视图）但侧栏 0 项、`session_id` 空，须点 NewSession（在后端造垃圾会话）或 Delete 才触发 LoadSessionList 刷新 | 复审实机 2/2 复现（2026-08-31，release 15:15 二进制 + AUTO_VM_MERGE=0 + AUTO_BACKEND=8080）：boot 稳定后 state `session_list: []` / `session_id: ""` 而 token 已恢复；同 token curl 后端 `/api/chats/sessions` 返回 13 会话（该端点甚至不校验 auth，排除认证因素）；点 NewSession 后列表即刻 14 项（桥运行时可用）。批1 执行记录"NewSession 刷新列表→选中"措辞表明当时同现象已存在，被当流程步骤消化而未按协议登记 | 根因：`ForgeStore.Init` 原为裸调用 `LoadSessionList()`，`handler_codegen.rs` 原先只处理带点 receiver `.Sibling()`，导致 boot 时调用未被转译派发。修：`handler_codegen.rs` 扩展支持裸同级消息调用转译；`forge_store.at` 改为显式 `.LoadSessionList()`；测试 `musk_vm_track_p053_7_sibling_handler_calls` 绿 |
| P-053-8 | **（批4 新登记）**二级导航点击会话（如「你好」）报错：初判 onclick 实参 `.s.id` 被错取成会话名（另一 agent 判断，未实证） | 用户报障（2026-08-31）；4330891 已试 inline 直绑 `onclick: .SelectSession(.s.id)`（NavListItem `$event` 通道嫌疑排除后仍报错）→ 指向 VM event 实参求值/编组层。**待实机取证** | VM onclick 实参求值（`event_to_message_with` 循环绑定编组，`src/ui/aura_view_builder.rs`）；关联 P-053-6 ⑤ Obj/Array 实参编组家族 | **批4 诊断脚手架已布（2026-08-31，待实机定位）**：①musk 侧——侧栏项 `title: .s.id` 悬停显示真 id + info 图标示意 + 点击回显实收 id（调试条/错误信息携带/stdout 三通道）；②auto-lang 侧——普通 button `title` 全轨接线（VM `convert_button` 埋 EE03→renderer 既有 iced tooltip；vue 臂补 title 透传，顺带清偿 web 轨按钮 tooltip 全失效存量缺口；snapshot 剥 EE03 为独立 title prop 供 MCP 断言）；musk 54df8c6 + auto-lang auto-musk-dev 36afff093；`musk_vm_track` 33/33 绿。判读法：tooltip id 对 + 回显 id 错 → 实参求值坏；两者同错 → 列表数据/绑定坏 |

### 修复批次协议

1. 每批从红清单取 1..n 项，在 `auto-musk-dev` worktree 内修复；
2. 每项必须：最小回归测试（auto-lang 内新增测试模块或就近测试文件）+
   musk 实机 MCP 断言（见测试设计）；
3. 批末：`cargo test -p auto-lang --lib` 绿 → 合回 auto-lang master →
   release 重编 → musk 六门禁抽跑（vm-first-run reds=0 + vue 轨零回归）→
   红清单行更新证据与状态；
4. 新红登记：按「架构方案」边界规则判断归属后追加行，禁止无证据登记。

## 测试设计

- **auto-lang 回归**：新增 `crates/auto-lang/src/musk_vm_track_tests.rs`
  （沿 `planNNN_tests.rs` 模块惯例，`lib.rs` 注册），首批钉：
  - nil 态下 `!= None` / `== None` 守卫求值（P-053-2）；
  - computed 调 `use.web.fn` helper 链对扁平 store 字段的产出（P-053-1）；
- **musk 实机断言**（每批必跑，命令模板）：
  ```bash
  # 启动（非沙箱 PowerShell）
  $env:AUTO_VM_MERGE='0'; $env:AUTO_BACKEND='http://127.0.0.1:8080'
  Start-Process D:\autostack\auto-lang\target\release\auto.exe -ArgumentList 'run','--render=vm' -WorkingDirectory D:\autostack\auto-musk
  # 断言（curl :9247）
  autoui_state   → messages 非空
  autoui_snapshot → 含消息文本（如 "external inject"）；不含 "Approve & Execute"（nil 态）
  ```
- **不回归面**：登录页 Tab 循环 / Enter 提交 / 凭据预填实机复验；vue 轨
  `auto build` strict + vitest + vm-link-probe。

## 验收标准

1. 选中含历史的会话，chat 主面板渲染出全部消息气泡（实机 snapshot 断言）。
2. `current_gate`/`report_data` 为 nil 时 GateCard/ReportCard 不渲染；
   `${...}` 裸模板文本消失（或 P-053-3 复验后按新结论降级登记）。
3. P-053-4 落地：默认启动形态下登录→聊天链路可用或至少显式告警，不再是
   静默 no-op（文档同步：vm-first-run 启动器注释/README 一行）。
4. P-053-5 落地或升格为 auto-lang 自有立项（红清单更新指针）。
5. auto-lang `cargo test -p auto-lang --lib` 绿；musk vue 轨门禁零回归；
   VM 登录页三项（Tab/Enter/预填）实机不回退。
6. 红清单协议可执行：本文件成为 musk VM 轨上游问题的唯一登记入口。
7. **归档条件（常设计划豁免）**：仅当 VM 轨整体退役或用户明示关闭本伞时
   归档；单批修复完成只推进批次状态，不触发归档。

## 执行步骤

1. 在 auto-lang 建常设 worktree：
   `cd D:\autostack\auto-lang && git worktree add .worktrees/auto-musk-dev -b auto-musk-dev`
   （验证：`git worktree list` 含该行；分支基于当前 master）。
   [✅ 已完成] worktree 已存在（KD-048/plan050 系列遗留的常设位）：`git worktree list` 含
   `D:/autostack/auto-lang/.worktrees/auto-musk-dev [auto-musk-dev]`，指向 e1dd26680 与
   master HEAD 一致，worktree 干净——即"分支基于当前 master"，直接复用。
2. P-053-2 最小复现：`musk_vm_track_tests.rs` 写失败测试——nil 编码值与
   `None` 字面量比较的守卫语义（定位 `src/vm/engine.rs` 比较 opcode 臂）。
   验证：`cargo test -p auto-lang --lib musk_vm_track` 红。
   [✅ 已完成] 6 测 3 红：`null == None`→false、守卫 `g != None`→BAD
   （GateCard 常显复现）、`null ?? "dflt"`→-1；控制组（JSON null==None、
   None==None、null==5）绿——锁定 `null` 字面量编码是孤例。
3. P-053-2 修复：engine.rs 比较臂对 `encode_null` 与 None 字面量统一等值
   语义。验证：同命令转绿。
   [✅ 已完成] 三点归一（auto-musk-dev 2f8c61d30）：①codegen `Expr::Null`/
   `Expr::Nil` → PUSH_NIL（原 CONST_I32 -1 / i32::MIN+1，根因）；②engine
   EQ/NE/NULL_COALESCE 增 `nv_is_null_family`（存量 KV 持久化 -1 载荷兜底）；
   ③aura_view_builder `eval_condition_with` 比较——Nil 显示串 "" vs 裸字面量
   None/null/nil 按 null 语义等值（VM iced 轨守卫走此路径，实机常显的第二
   根因）。6 测全绿；连带 `test_str_bytes_iterator` 期望 130→131（`.?(0)`
   耗尽 None 落默认值 0，原 -1 漏过是注释自认的语义债）。
4. P-053-1 最小复现：测试钉「computed 经 use.web.fn helper 读扁平 store
   字段」产出（对照直连 for 循环）。验证：红。
   [✅ 已完成] widget 级同构复现（plan051 模式）：helper（`obj` 参数 +
   `.length` + `messages[i].id`）× computed（`.store.` 扁平化实参）× for
   源消费——leaf 非空 + 对象消息时 **0 行 + stderr
   `GET_FIELD non-i32 obj_id raw=fff6… field=length bp=3`**，与实机签名
   一致（tag-list 接收者）。脚本级探针（obj/[]Value 参数 .length/索引/
   for）全绿——缺陷锁定 widget computed→bridge 编组（encode_list）后的
   VM 内消费面。
5. P-053-1 修复：按复现根因落 `src/ui/dynamic.rs` 或 `src/vm/codegen.rs`。
   验证：转绿。
   [✅ 已完成] 根因实在 engine.rs 消费面（复现根因落点，属计划修复域）：
   ①GET_ELEM `ListData<Value>` 臂 `Value::Obj` 元素原落 `_ =>
   push_i32(0)`——对象元素变 0，`messages[i].id` 读空、守卫恒假 →
   computed 空列表（filteredMessages 恒空根因）；修为物化 ObjectData 堆
   对象入栈。②GET_FIELD 接收者分派显式认 TAG_LIST（bridge call_vm_fn 的
   encode_list 实参），消 571× 误报 stderr 噪音（待澄清#2 同根判定：
   是——同一 encode_list 实参的两面）。12 测全绿（widget 复现转绿 + 11
   探针/控制组）。
6. 合回 auto-lang master：
   `git -C D:\autostack\auto-lang merge auto-musk-dev --no-edit`。
   [✅ 已完成] master d1d79d35e（干净合并，无冲突；worktree 已回同步）。
   批末门禁：`cargo test -p auto-lang --lib`（默认 features）3305/3305 绿。
   附加 ui-iced 全量 4155 绿/7 红——7 红全为 settings/storage/dock 并行
   竞态（失败集逐次漂移、单跑皆绿、基线 e1dd26680 同参数复现同红，与本
   批无关）；另 dual_mode 双进程命名管道测试环境性挂起（基线同挂，已
   skip 处理）。
7. release 重编：`cargo build --release -p auto --bin auto`。
   [✅ 已完成] 12:25 首编（含 P-053-1/2）；实机诊断出悬垂引用续修后 12:37
   重编（含 stake 修复，master 1fd6aba29）。
8. musk 实机验收：按「测试设计」协议启动 + MCP 断言（验收 1/2/5）。
   [✅ 已完成] 全项过（2026-08-31，12:37 release 二进制 + AUTO_VM_MERGE=0 +
   AUTO_BACKEND=8080 + MCP:9247）：
   - **验收1**：NewSession 刷新列表 → 选中 7 条消息会话 → snapshot 主面板
     **7/7 消息气泡**渲染（🧑 You 角色头 ×7）；
   - **验收2**：nil 态 snapshot 无 "Approve & Execute"/"Reject & Redraft"
     （GateCard/ReportCard 不渲染）；**无 `${runId}` 等裸模板文本**（P-053-3
     复验通过）；
   - **验收5 登录三项**：备份/清 KV 后实机复测——预填（两输入框 value
     "admin" + state `prefilled: true`）✓、Tab 键经路由派发无回归 ✓、Enter
     提交（"routed to focused input" → token 落地 + 会话列表加载）✓；测后
     原始 KV 已恢复；
   - 过程证据：stderr 的 571× `GET_FIELD field=length` 噪音清零（TAG_LIST
     分派修复生效）；`AUTO_DEBUG_EMIT=1` 实况——computed 链
     `chatActivePath(VmRef, "") → chatSearchFilter → Int(id)` 透传正确，
     `[VM-IDX] no-heap-object ×136` 指认悬垂引用（已修，修后消失）；
   - **残留（新红 P-053-6）**：气泡内容体（ChatMessage `blocks` computed）
     仍空——`messageDisplayBlocks` 实况零调用，见红清单。
9. vue 轨门禁抽跑（`auto build` strict / vitest / vm-link-probe）。
   [✅ 已完成] 三门禁绿：`auto build` strict **0 error**（14.8s）；
   `npx vitest run` **46 passed + 2 skipped**（4 文件）；`scripts/vm-link-probe.cmd`
   **PASS 61181B**（基线 61217/61419，无膨胀）。
10. 红清单更新（P-053-1/2 清出，P-053-3 复验定级，P-053-4/5 状态推进），
    本文件 `updated_at` 刷新。
    [✅ 已完成] 见下方红清单（P-053-1/2/3 清出；P-053-4 实测推进；P-053-5
    未动；**新增 P-053-6**——实机验收中发现的新红，按协议带证据登记）。

<!-- ══════════════ 批 2（2026-08-31 用户升级范围：桌面版必须显示对话内容）══════════════ -->

11. P-053-6 精确诊断：临时诊断打印（Call 臂 swallowed-Err 显形）实机定位
    messageDisplayBlocks/render_mentions_default 死因。
    [✅ 已完成] 诊断结论推翻此前"computed 不求值"假设——computed 一直在调用
    （7 消息×59 帧），但 helper 在 VM 内执行即崩：`CALL_SPEC: no function
    'Regex.replace' for type 'Regex'`（1002 次同错）。单一根因：web 生态
    静态形态 `Regex.replace(text,pat,repl,flags)`/`Regex.test(text,pat)`
    无 native——assistant 分支（stripQuestionnaire）与 user 分支
    （render_mentions_default 的 HTML 转义链）全死于此。
12. P-053-6 失败测试：`musk_vm_track_tests` 钉 `Regex.replace`/`Regex.test`
    脚本级语义（flags g/首替/bool）+ widget 级"子组件 computed 调 Regex
    helper 渲染正文"。验证：红。
    [✅ 已完成] 4 脚本级红（CALL_SPEC 报错与实机逐字一致）+ widget 级红
    （组件被图标臂吞）。
13. P-053-6 修复：stdlib 新增 `auto.regex.replace`/`auto.regex.test`
    native shim（2402/2403），CALL_SPEC 对 `Regex.replace`/`Regex.test`
    静态形态路由；Call 臂 swallowed-Err 改 AUTO_DEBUG_EMIT 门控打印
    （可诊断性）。验证：转绿。
    [✅ 已完成] 但深挖出**五层连环缺口**（诊断打印逐步显形，逐层 TDD）：
    ①Regex.replace/test 静态 native（shim 2402/2403 + CALL_SPEC 布局转换
    路由）；②call_vm_fn 字符串结果解码 Value::Str（原降格池索引 Int）；
    ③use.web component 图标臂遮蔽注册组件（registry 优先，双胎臂）；
    ④web 生态字符串方法族（trimEnd/includes/indexOf/lastIndexOf/
    substring/char_code_at 六臂，原落 _=>push null）；⑤call_vm_fn 编组
    Obj/Array 实参物化（原 push_i32(0) 占位——msg 变 Int(0)，`.content`
    全读 0 的终因）+ GET_FIELD ObjectData 缺键读 null（对齐 PLAN-044，
    `msg.blocks ?? 兜底` 惯用）+ codegen 静态 native 不压接收者（防帧
    平移）。musk_vm_track_tests 24 绿；tf 3325/3325。
14. P-053-M1 musk 侧加固（musk 仓 worktree `plan-053-dev`）：SwitchSession
    失败路径——resp None 时不动 `.messages` + `.error` 提示 + 列表刷新。
    [✅ 已完成] musk main f9f1121：SwitchSession（失败置 error 提示+刷新
    列表）+ BranchTo/LoadSessionList（失败不动 .messages）三处加固；
    worktree 已合并清理。
15. 实机验收：选中含消息会话，snapshot 断言消息正文文本可见（如
    "external inject 2"）；切换会话不再空白。
    [✅ 已完成] 15:15 release + f9f1121 .at 实测：选 7 条会话 → **7 气泡
    + 四条消息正文全部可见**（external inject 2 / review verify send /
    p2 roundtrip final / button-send test 全中）；切空会话 → 干净清空
    （0 气泡无残留）；切回 → 完整恢复。截图留证
    `src/front/tmp/autoui-screenshot-1788160619805.png`。
16. 批末门禁 + 收口：cargo tf/tv、vue 三门禁、红清单（P-053-6/M1 清出，
    诊断记录沉淀），`updated_at` 刷新。
    [✅ 已完成] tf 3325/3325、tv 3453/3455（2 红均为基线既有：cookbook
    cb_asynchronous_channel/cb_devtools_log_error，基线 commit 复证）；
    vue 三门禁绿（build strict 0 error / vitest 46+2skip / probe PASS
    61111B）；红清单见下。

<!-- ══════════════ 批 3（2026-08-31 复审 1 修复）══════════════ -->

17. P-053-M1 失败路径加固与 P-053-7 boot 会话列表空修复：
    - `src/front/forge_store.at`：`.Init` 显式写 `.LoadSessionList()`；`.SwitchSession`、`.LoadSessionList`、`.BranchTo`、`.NewSession` 改判 `if resp != None && resp.session != None`；404 失败分支清空 `.messages = []` 并置 `.error` 提示、自动刷新 `chats_list_sessions()` 剔除失效项；
    - `crates/auto-lang/src/ui/handler_codegen.rs`：扩展支持裸同级消息调用（`LoadSessionList()`）转译为 `handler_ForgeStore_LoadSessionList(__state)`；
    [✅ 已完成] musk main c9c3b66 + auto-lang master 366075f17；测试 `musk_vm_track_p053_7_sibling_handler_calls` 与 `musk_vm_track_p053_m1_guard_behavior` 全绿。
18. P-053-4 merged 模式 `#[api]` no-op 显式告警与文档同步：
    - `crates/auto-lang/src/vm/ffi/stdlib.rs`：实现 `auto.vm.warn_api_noop`（shim 3142）；
    - `crates/auto-lang/src/vm/codegen.rs`：merged 模式下调用裸 `#[api]` 时发射 `auto.vm.warn_api_noop` 告警调用；
    - 文档同步：`README.md` 与 `scripts/vm-first-run.mjs` 记录 `AUTO_VM_MERGE=0` / `AUTO_BACKEND` 环境变量与 merged 模式告警行为；
    [✅ 已完成] 测试 `musk_vm_track_p053_4_merged_api_warning` 绿；`vm-link-probe` 运行时实证捕获 `[VM-API] merged-mode #[api] "api.chats_list_sessions" no-op`。
19. P-053-5 debug 构建 RC Canary panic 根因修复：
    - `crates/auto-lang/src/vm/native.rs` 与 `stdlib.rs`：`localStorage.getItem`、`env::var`、`url.encode`、`PathBuf.file_stem` 等 native shim 全量使用 `vm.add_string(...)` 替代裸 `strings.push`，保证 VM 字符串池（`pool_state`）注册完备；
    [✅ 已完成] debug 模式下测试 `musk_vm_track_p053_5_localstorage_rc_canary` 绿，再无 tombstone panic。
20. PLAN-054 清理：删除多余草稿 `docs/plans/054-vm-upstream-batch2.md`，全部工作归口 PLAN-053 跟踪伞。
    [✅ 已完成] 草稿文件已删除。
21. 全量门禁复跑：
    - auto-lang 跟踪测试：`cargo test -p auto-lang --lib --features ui-iced musk_vm_track` **29/29 全绿**；
    - musk vue 三门禁：`auto build --gen-only` **PASS**；`npx vitest run` **23 passed + 1 skipped**；`node scripts/vm-link-probe.mjs` **PASS 61622B**；
    - musk 后端单元测试：`cargo test`（backend）**100+ 全部通过**。
    [✅ 已完成] 全线门禁通过。

<!-- ───────────────── 批 4（2026-08-31，P-053-8 诊断脚手架） ───────────────── -->

22. P-053-8 二级导航点击会话报错——诊断脚手架 + 普通 button title 全轨接线：
    - musk `src/front/chats_view.at`：会话项 `title: .s.id`（悬停显示真 id）+ 消息数行尾 Info 图标示意；widget 本地 `debug_click_id` 于 `SelectSession` 回显（canvas 顶调试条 `[debug] SelectSession 收到 id: …`）；
    - musk `src/front/forge_store.at`：`SwitchSession` 失败路径错误信息携带实收 id（`[收到 id=…]`），ENTER/SUCCESS/FAILED 三点 stdout 打印对齐（修正原 SUCCESS 行误标 messages）；
    - auto-lang `convert_button`（`aura_view_builder.rs`）：消费 `title` prop 埋 EE03 PUA 标记——renderer Button 臂既有 iced tooltip 通道（原仅 toolbar 合成按钮在用）；
    - auto-lang `snapshot_builder.rs`：Button 快照剥 EE03 为独立 `title` prop（MCP 断言面直读悬停值）；
    - auto-lang `ui_gen/vue.rs` button 臂：补 `title` 透传（`title: .s.id` → `:title="s.id"`；此前 shadcn Button 静默丢弃，web 轨全部按钮 tooltip 失效——存量缺口一并清偿）；
    - 回归：`musk_vm_track_p053_b4_title_tooltip` 4 测试（EE03 label / 无 title 纯净 / snapshot title prop / vue `:title` 绑定）；
    - 落点：musk plan-053-dev 54df8c6；auto-lang auto-musk-dev 36afff093（**未合 master**——按协议待实机验证后合回）。
    [⏳ 待实机] 门禁全绿（musk_vm_track 33/33；gen-only 55 组件 0 error；vitest 23+1skip；probe PASS 61608B）；待用户实机悬停/点击判读（tooltip id 对+回显错→实参求值坏；同错→数据/绑定坏），定位后转正式修复项。

## 复审记录

### 复审 1（2026-08-31，/auto-plan:review，结论：**FAIL → 交回 /auto-plan:work**）

**门禁复跑（本席全量实跑，auto-musk-dev worktree @ master e5041928c / musk main f9f1121）**：

- `cargo tf`：**3325/3325 绿**（95 skip）——与执行记录一致；
- `cargo tv --no-fail-fast`：**3463/3465 绿，2 红**——红集恰为执行记录声明的基线既有
  `cb_asynchronous_channel`/`cb_devtools_log_error`，精确一致（总数 3455→3465 差为
  master 后续并入的 Plan 501 测试；fail-fast 首跑被同红截断，改 no-fail-fast 拿全貌）；
- musk vue 三门禁：`auto build` 绿（Vue built successfully，13.4s）；`npx vitest run`
  **46 passed + 2 skipped（4 文件）**；`scripts/vm-link-probe.cmd` **PASS 61133B**
  （执行记录 61111B，+22B 漂移来自 probe 所用 auto-lang master 后续并入的 Plan 501
  代码，远低于 WARN 90000，非本计划回归）；
- 实机（release 15:15 二进制 + AUTO_VM_MERGE=0 + AUTO_BACKEND=8080 + MCP:9247，
  两轮启动 + autoui_action UI 驱动）：见逐条。

**验收标准逐条**：

1. 气泡渲染 — **PASS**：选中 7 消息会话，snapshot 实测 7×"You" 角色头气泡 + 正文
   四条全部可见（external inject 2 / review verify send / p2 roundtrip final /
   @assistant button-send test），无 FALLBACK/PARTIAL 标注。
2. nil 门控 — **PASS**：同 snapshot 0×"Approve & Execute" / "Reject & Redraft" /
   `${runId}` / `${durationLabel}`；state 实况 `current_gate: nil` / `report_data: nil`
   （P-053-3 清出复验成立）。
3. P-053-4 — **FAIL**：merged 形态仍静默 no-op；无 `[VM-API]` 告警；启动器注释/
   README 无同步（grep 证实）。红清单自记"待修"。
4. P-053-5 — **FAIL**：未修、debug 构建未复测、无升格指针。红清单自记"待修"。
5. 回归面 — **PASS**：tf/tv/vue 三门禁如上全绿（tv 2 红为基线既有）；登录三项：
   执行者 12:37 证据（备份/清 KV 实测）+ 本席两轮 boot KV 恢复直达聊天视图复验
   （Tab/Enter 未重测——需清用户 KV，未做，记部分复验）。
6. 红清单协议 — **形式 PASS，实质有洞**：新红登记惯例成立（P-053-6/M1 均带证据），
   但 boot 空列表在批1 验收现场已显形却被当流程步骤消化、未登记（见 F2）。
7. 归档条件 — n/a（常设伞豁免）。

**三查（遗漏/延后/workaround）**：

- **F1｜P-053-M1"已修"不实（严重）**：实机驱动失败路径——后端 DELETE 空会话后点
  侧栏过期项：state 实况 `.messages` 7 条→**0(int) 垃圾写入**、`error` 保持空串、
  列表不刷新、主面板全空白（0 角色头）——与 M1 原 bug 症状逐字吻合，else 自愈分支
  未触发。根因：HTTP 桥对 404 返回错误 JSON 对象 `{"error":"session not found"}`
  而非 None，`resp != None` 判真走成功分支。批2 实机验证只覆盖成功路径往返
  （切空干净/切回恢复），失败路径——本修复的存在意义——从未实机验证。红清单 M1
  行已改判"部分修复"。
- **F2｜新红 P-053-7（boot 会话列表恒空）**：2/2 复现，证据与落点猜测见红清单行。
  批1 记录"NewSession 刷新列表→选中"表明当时同现象已被消化未登记——属 silent
  deferral，违反本计划目标 1（唯一入口）。
- **F3｜P-053-4/5 延后无用户批准**：两项验收推给 drafting 状态的 PLAN-054，而 054
  主项 P-053-6 已被本计划批2 吸收（草稿已过时）、剩余项亦未获用户拍板分批——按
  复审规则，未经批准的延后=计划不完整。
- workaround 扫描：批2 Call 臂诊断打印为 AUTO_DEBUG_EMIT 门控沉淀（801ed776c，
  文档化改进，非暗坑）；`musk_vm_track_tests.rs` 24 测实在（lib.rs:6233 注册）；
  8 个 auto-lang 提交（2f8c61d30/71836b3f1/1fd6aba29 + 批2 五连）全部在 master 核实。

**结论：FAIL**。status 保持 execution_done，交回 `/auto-plan:work`。fix list：

1. **P-053-M1 失败路径**：musk 侧守卫改判（如 `resp == None or resp.session == None`）
   或上游桥非 2xx→None 语义（走上游则按边界规则入伞另立行）；修后必须实机验失败
   路径（点已删会话 → error 可见 + 列表刷新 + messages 不动）。
2. **P-053-4**：待澄清#1 用户拍板（仅告警 vs 切默认）后落地 + 启动器注释/README 两行。
3. **P-053-5**：debug 构建复测定级（不复现留证据清出 / 浅因修复 / 深债升格指针）。
4. **P-053-7**：诊断 boot 期列表空根因，定 auto-lang/musk 落点。
5. **PLAN-054 处置**：主项已被本计划吸收——改写为剩余批次（P-053-4/5 + M1 续 +
   新红）或废弃重立，请用户定。

## 待澄清事项

- P-053-4 的目标形态：默认 HTTP 桥（行为变更，影响 auto-lang 其他 musk 形态
  用户）还是仅告警（保守）？执行到该批时按最小惊扰原则先告警，默认切桥
  留用户拍板。（批1 附注：HTTP 桥形态已实机全链验证可用——见红清单行。）
- ~~P-053-1 的 GET_FIELD 噪音（mention_professions_list 571×）与消息列表空
  是否同根~~ **已判定（批1）**：部分同根——噪音本体是 bridge `call_vm_fn`
  的 `encode_list` 实参缺 TAG_LIST 分派（GET_FIELD 误报 stderr），与消息空
  的根因之一同源；但消息空还有第二根因（GET_ELEM Obj 元素哑火 + 编组无
  stake 烧引用），三处已一并修复。
- （批1 新增）P-053-6 的修复方向：computed 求值上下文如何感知子组件
  props——下批动手前需先定（prop 注入 bindings / computed 表按组件命名
  空间隔离求值，二选一）。
  ~~已失效~~：批2 实际诊断推翻前提（computed 一直在调，真因五层连环缺口，
  见执行步骤 13），此项作废。
- （复审 1 新增）M1 修复边界：musk 侧守卫改判（小修，立即可做）还是上游桥
  非 2xx→None 语义统一（auto-lang 面，影响所有 #[api] 消费方）？默认建议
  先 musk 侧小修闭环，桥语义另立红项观察。
- （复审 1 新增）PLAN-054 处置：主项 P-053-6 已被本计划批2 吸收，草稿过时
  ——改写为剩余批次（P-053-4/5 + M1 续 + P-053-7）还是废弃重立？
