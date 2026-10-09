---
plan_id: PLAN-100
status: archived
feature_name: relay plan 相位失败根修（stage_incomplete）+ VM 轨运行框/批准卡渲染修复
author: [zcode agent]
created_at: 2026-10-10T00:00:00Z
updated_at: 2026-10-10T00:00:00Z
plan_revision: 1
current_step: 5
total_steps: 6
supersedes_spec_components: [docs/specs/modules/plan-flow.md, docs/specs/modules/chat-streaming.md]
new_spec_components: []
touched_goals: []
---

# [PLAN-100] relay plan 相位失败根修 + VM 轨 Run/Approve 块渲染修复

## 0. 变更摘要

用户实机报告（run `run-1791532877-7ff2c90`，focusboard 需求）暴露两个缺陷：

1. **plan 流相位失败（卡点根因）**：plan 相位结束时 advisor 未提交有效的
   `complete_plan_stage` 结构化结果，驱动器按 AC-04 直接判 run 失败
   （`stage_incomplete`），**从未到达 execute 相位的 Human 门**。聊天文案
   承诺"会在计划确认门暂停等审批"，实际 run 在门之前就死了——外层 chat 的
   spawn_relay 芯片又没有渲染运行框，用户无从得知真实状态。
2. **VM 轨 Run/Approve 块不渲染**：spawn_relay 工具块在 VM（iced）轨退化为
   通用工具芯片——`extractRunId(tc)` 取不到 run_id（tc.arguments/tc.result
   在 VM 链路为空），RelayRunBox（运行框）与 ToolGateCard（批准卡）分支
   不可达。违反 chat-streaming 契约⑧（PLAN-077 双端 Block 状态/回放验收）。

修复双管齐下：①在 plan 相位收束处增加**一次有界催交（nudge）**再失败
（保留 AC-04 终态语义）+ 相位模板反模式条款；②VM 轨工具块形状容错 + 渲染
链缺口修复 + 双端渲染验收面扩展。

## 1. 目标

- **G-1**：plan 流四相位因"忘交结构化结果"导致的 stage_incomplete 失败，
  驱动器先催交一次（同相位重入），仍无结果才响亮失败；失败文案带自助恢复
  指引（rerun 端点）。
- **G-2**：VM 轨聊天内 spawn_relay 块渲染 RelayRunBox（运行框：run 状态
  可见），run 抵达 human 门时渲染 ToolGateCard（approve/reject 可用且经
  UI 决议能推进 run）。
- **G-3**：双端渲染一致性入验收（web 与 VM 同一 spawn_relay/gate 场景
  行为等价），防回归。

**非目标**：不改 plan 流形状（四相位/门/gate 归属）；不改 `complete_plan_stage`
工具契约（参数/校验语义不变）；不处理 run 跨重启恢复（既有裁定：驱动不跨
重启 resume，会话双写仅为日志）；不动 forge（auto-forge）仓——本计划全部
改动落 auto-musk。

**受影响仓/模块**：仅 auto-musk（backend hw Rust：`relay/plan_control.rs`、
`relay/plan_flow.rs`；前端双轨 `.at`：`src/front/forge_helpers.at`、
`src/front/chat_message.at` 及 VM 渲染链定位到的文件）。

**约束**：worktree 纪律按本仓 AGENTS.md——执行期开 `.wt/musk-100/auto-musk`
（分支 `plan-100-dev`），计划文件在主检出维护；后端 hw/ag 双轨纪律（relay
行为改动尽量落 hw 单点，ag 经 extern 委托，parity 测试钉死）。

**成功样貌**：重放同形需求（requests/001.md focusboard）的 plan run，advisor
忘交结果时被催交一次后正常过 plan 相位；执行到 execute 相位 human 门时
run 停在 `waiting_approval`，用户在 **VM 轨聊天里**看到运行框与批准卡并能
批准推进。

## 2. 架构方案

- **①相位收束催交（nudge）**：`PlanControl::on_stage_end_inner` 第 2 步
  （`latest_claim` 为 None）不再直接 `StageRouting::Fail`，改为：
  若该 (phase, attempt) 的催交预算未用（1 次）→ 消费预算、push
  `stage_nudge` facts、返回 `RewoundToExecute`（游标未动，同相位重入，
  驱动器以追加强调指令重入——复用输出截断续做（continuation）的既有机械
  形状，新增独立 `nudges: BTreeMap<String,u32>` 计数，不与截断续做混账）；
  预算耗尽仍无 claim → 维持现 `stage_incomplete` 失败，文案追加 rerun
  恢复指引。hw 单点（ag extern 委托 hw），parity 测试覆盖面不变。
- **①模板反模式条款**：`result_protocol("plan")` 增补两条机械禁令——
  禁止以计划全文文本输出作为相位结尾（必须 `create_plan` 落盘 +
  `complete_plan_stage` 提交）；提交参数中 plan_id/plan_revision 必须逐字
  取自 `create_plan` 工具返回值（不得从记忆/正文推断）。
- **②VM 渲染链**：先 bounded 取证（T-03）定位 `extractRunId` 入参在 VM
  链路的缺失环节（活流 `tc` 透传 vs 历史拍平两路），然后按点修复：
  `extractRunId` 形状容错（arguments/result 兼容 JSON 字符串形态）为
  无条件加固；缺口本体按 T-03 结论落点（预期在 VM 侧流式块归一化或
  result 回填应用层）。修复后把 spawn_relay 运行框 + gate_waiting 卡
  纳入双端渲染验收（扩展 chat-streaming 契约⑧的验收面）。

## 3. 技术栈

- backend：Rust（hw 轨 musk crate；ag 轨 `.at`→a2r 不涉本计划新增 extern）。
- frontend：AutoUI `.at` 双轨（web Vue 生成 + VM iced 直挂），现有
  `use.web` 分叉纪律。
- 测试：`cargo test -p musk`（plan_control/plan_tools 单测 +
  tests/parity_relay_api.rs 双轨 parity + tests/vm_serve_harness.rs VM
  服役面）；VM 轨人工/MCP 验收（截图留档 desktop-shots 同族）。

## 4. 需求分析与背景调查

**授权记录**：用户 2026-10-10 明确两项都要修（原话："这两个问题都要解决：
1. 修复卡点（为什么会失败）。2. 修复 Run 和 approve 的 block 渲染"）并
同意立计划。无额外预算/自动续跑授权约束。

**实机证据（run-1791532877-7ff2c90，musk 后端 :17892，已于取证后下线）**：

- run 终态：`status=failed`、`current_step=0`、`waiting_for_gate=null`、
  steps=`[plan(auto), execute(human), review(auto), document(auto)]`。
- 事件序（计数）：turn_delta×492 → turn_tool_call×2 → turn_tool_result×2
  → turn_complete → plan_stage_facts → run_failed×2。
- 失败文案：`stage_incomplete: agent ended phase 'plan' without a valid
  complete_plan_stage result`；末尾 delta 文本以计划正文+"```"围栏收尾
  （模型以文本输出计划而非工具提交的强信号）。
- 存活事实：runs 不落盘（`relay/store.rs:433`），会话双写是唯一持久日志
  （`{ws_root}/.autoos/conversations/{run_id}/turns.jsonl`）——T-01 取证
  优先复现，失败则按用户实例的 ws 根找双写日志。

**代码锚点（已核对）**：

- 判定点：`backend/crates/musk/src/relay/plan_control.rs:533-539`
  （`latest_claim` None → Fail）；催交可复用的机械先例：
  同文件 `:501-532` 输出截断续做（continuation 预算 1 次 + `RewoundToExecute`）。
- 工具注册：`backend/crates/musk/src/lib.rs:320-341`（relay step 的 mode
  `tools: Vec::new()`（`relay/driver.rs:93`）→ `is_empty()` 门放行全部
  orch 工具——`complete_plan_stage` 对 advisor **可用**，排除"工具缺席"候因）。
- 提交校验：`plan_control.rs:330-349`（plan_id/plan_revision/attempt 服务器
  盖章）；`latest_claim` 按 (stage, attempt) 过滤（`:353-362`）。
- 相位模板：`relay/plan_flow.rs:109-128`（result_protocol 已含 AC-04 警告，
  未含反模式禁令与 plan_id 逐字来源约束——只有"取自创建的计划 frontmatter"）。
- ②前端：`src/front/forge_helpers.at:59`（extractRunId：arguments.run_id →
  result JSON.run_id，异常吞空）、`:394`（VM 侧 msg.blocks 数组原样透传活块）；
  `src/front/chat_message.at:324-358`（gate_waiting → ToolGateCard；
  spawn_relay → extractRunId 命中才渲染 RelayRunBox，否则通用芯片——用户
  截图即兜底芯片形态）。
- ②持久化契约：`backend/crates/musk/src/chat_page.rs:169-243`（历史拍平
  `tool_args_json` 恒合法 JSON、`tool_result` 截断保头——历史路径两字段
  恒在，缺口指向活流路径或 VM 消费层）。
- 规格锚点：`docs/specs/modules/plan-flow.md`（相位表/AC-04 语义）、
  `docs/specs/modules/chat-streaming.md`（契约⑧：PLAN-077 双端 Block
  状态/回放验收——本计划②直接违反该契约的 VM 半边）。

**规格现状评估**：两份相关 spec 均为新（PLAN-086/077 定型），无陈旧问题；
②的缺口是 spec 验收面覆盖不足（契约⑧未点名 spawn_relay/gate 卡场景），
以规范增量补齐。

## 5. 详细设计

### 5.1 ①相位收束催交（T-02）

`on_stage_end_inner` 重排（`plan_control.rs`）：

1. 取消旗标臂（不变）→ 截断续做臂（不变）。
2. `latest_claim` None 臂改写：
   - `pe.nudges.get(&pe.phase) == 0`（新字段，`PlanExecutionState` 增
     `nudges: BTreeMap<String,u32>`，serde default 兼容旧 JSON）→
     `mutate_plan_execution` 计数 +1、push `facts_of(&pe, phase, "stage_nudge")`、
     返回 `StageRouting::RewoundToExecute`（驱动器同相位重入；重入任务的
     追加强调由 phase_task 侧拼接一段 `nudge_directive()`：声明上一轮未交
     结构化结果、本轮唯一任务是立即调用 `complete_plan_stage`，不得重做
     相位工作）。
   - 已用尽（==1）→ 现状 Fail，文案追加：
     `; recover via POST /api/forge/relay/runs/{run_id}/rerun`。
3. `record_stage_claim` 的 attempt 盖章语义不变；nudge 后的新 attempt
   仍由 mutate 推进（沿用 continuation 臂的 attempt 作废形状——本计划不
   改 attempt 语义，重入不推 attempt，同 attempt 内补充提交即为合法最新 claim，
   `latest_claim` 的 rev 查找天然取新）。

边界：nudge 不适用于 `needs_replan/blocked` 臂（那是合法 claim 的路由，
先行短路不变）；取消/截断优先级高于 nudge（顺序即现排）。

### 5.2 ①模板反模式条款（T-02 同改）

`plan_flow.rs result_protocol` 追加：

- 「禁止以计划/总结的**纯文本输出**作为本相位结尾——收尾动作只有两个：
  `create_plan`（plan 相位）落盘 + `complete_plan_stage` 提交。」
- 「`complete_plan_stage` 的 plan_id/plan_revision **逐字复制**
  `create_plan` 工具返回值，不得凭记忆书写。」

### 5.3 ②VM 渲染链修复（T-03 取证 → T-04/T-05 修复）

- **T-03 取证（bounded，决策产物）**：VM 轨（`launch-vm.cmd` 同形环境）
  起后端 + VM 前端，构造 spawn_relay → plan run → human 门场景，逐跳
  dump 块 JSON：后端 SSE 帧载荷 → VM chats store 持有的 msg.blocks →
  `messageBlocks` 输出 → `extractRunId` 入参。记录 result/arguments 在
  哪一跳丢失（候选：活流 tc 透传时后端未带 result 字段且后续 ToolUpdate
  回填事件 VM 未应用；或 VM 拿到的 arguments 为 JSON 字符串形态）。
  产出决策记录（写入本计划 §9 进展注记）后选择修复落点。
- **T-04 形状容错（无条件加固）**：`forge_helpers.at extractRunId`
  兼容 `tc.arguments`/`tc.result` 为 JSON 字符串的形态（先 typeof 判定
  再 parse/取字段），异常路径维持返回 ""。
- **T-05 缺口修复**：按 T-03 结论落点实施（预期形态：VM 侧活块收到
  tool 完成事件时回填 result/arguments，或 messageBlocks 归一化臂对
  缺 result 的活块触发一次 relay run 回查）。不引入新的后端字段——
  数据面（chat_page 拍平契约）已齐备。

### 5.4 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/modules/plan-flow.md（AC-04/相位结束判定节） | before: 相位结束无有效 complete_plan_stage → 直接 stage_incomplete 失败；after: 先消耗 1 次同相位催交（nudge，重入仅强调提交），仍无结果才 stage_incomplete 失败（失败文案带 rerun 指引）；取消/截断优先级不变 | 实机 run-1791532877 实证：advisor 以文本收尾忘交结果的失败占 plan 流卡点，一次催交可机械挽活且不放松终态语义（预算 1 次封顶） | AC-01, AC-02 |
| SD-02 | modify | docs/specs/modules/chat-streaming.md（契约⑧双端 Block 验收） | before: 双端 Block 状态/回放验收未点名 spawn_relay 运行框与 gate_waiting 批准卡；after: 验收面显式扩展——web/VM 双轨 spawn_relay 块必须渲染 RelayRunBox（run 状态可见）、gate_waiting 必须渲染 ToolGateCard 且 UI 决议可推进 run；extractRunId 对 arguments/result 的 obj/str 双形态容错 | VM 实机截图实证运行框/批准卡不可达（extractRunId 空转走兜底芯片），违反契约⑧的 VM 半边 | AC-03, AC-04, AC-05 |

## 6. 测试设计

- **单测（hw）**：`plan_control` 新增——(a) 无 claim 首次收束 → nudge
  （facts 事件 + RewoundToExecute + 计数落账）；(b) nudge 后仍无 claim →
  stage_incomplete Fail 且文案含 rerun 指引；(c) nudge 与截断续做计数
  互不混账；(d) 取消旗标优先于 nudge。
- **单测（模板）**：`plan_flow` 模板断言扩展——plan 相位模板含反模式
  两禁令（沿 `:411` 既有 `tool named` 断言形状）。
- **单测（前端 helpers）**：`extractRunId` 形状矩阵——arguments obj/str ×
  result obj/str/空（VM .at 侧测试文件随 musk 前端测试形态落地；
  无 .at 测试面则落 backend parity 或手测脚本）。
- **parity**：`tests/parity_relay_api.rs` 既有面回归（nudge 为 hw 单点，
  ag extern 委托不受影响——跑通即证）。
- **端到端**：VM 服役面（`tests/vm_serve_harness.rs` 形状）+ 实机 MCP
  驱动：spawn_relay → execute 门 → VM 截图运行框 + 批准卡 → UI approve →
  run 推进（事件面断言）。

## 7. 验收标准

- **AC-01**：plan 相位结束无 claim 时，驱动器产生一次 nudge 重入（facts
  事件 `stage_nudge` 在案、run 不失败），重入后提交 claim 则正常路由进
  execute 门。验证：`cargo test -p musk plan_control` 新增用例全绿。
- **AC-02**：nudge 后仍无 claim → run 失败且失败文案含
  `POST /api/forge/relay/runs/{run_id}/rerun` 恢复指引；截断续做与 nudge
  预算互不占用。验证：同套件用例 (b)(c)。
- **AC-03**：VM 轨聊天里 spawn_relay 工具块渲染 RelayRunBox（运行框可见
  run 状态，不再退化为通用芯片），arguments/result 以 obj 或 JSON 字符串
  任一形态到达均可提取 run_id。验证：T-04 单测 + T-05 实机截图。
- **AC-04**：run 抵达 execute human 门时，VM 轨渲染 ToolGateCard，UI 上
  approve 后 run 状态推进（`waiting_approval` 解除、driver 继续）。验证：
  实机 MCP 驱动 + run 详情断言。
- **AC-05**：web 轨同场景回归无劣化（运行框/批准卡照常），双端 parity
  测试全绿。验证：`cargo test -p musk --test parity_relay_api`。

## 8. 执行步骤

- **T-01 取证定音（bounded 调查，决策产物）**：本地起 musk 后端（用户
  实例同形配置；端口以现场为准，此前观测 :17892），重放/新发同形 plan run
  （focusboard 或最小需求），捕获 advisor 的工具调用名与结果——在
  「未调用 complete_plan_stage」/「调用但 record_stage_claim 校验失败」
  两候因间定音；结论记入 §9。若 run 事件双写日志可得
  （`{ws_root}/.autoos/conversations/{run_id}/turns.jsonl`）优先离线取证。
  依赖：无。涉及：无代码改动。验证：§9 结论注记 + 命中候因。
- **T-02 nudge + 模板条款（对应 AC-01/02，SD-01）**：改
  `relay/plan_control.rs`（nudges 字段 + 收束臂改写 + 失败文案）、
  `relay/plan_flow.rs`（result_protocol 反模式条款 + nudge_directive）。
  依赖：T-01（候因定音影响 nudge 指令措辞；机械改动可并行预置）。
  验证：§6 单测 (a)-(d) + 模板断言。
- **T-03 VM 渲染链取证（bounded 调查，决策产物，对应 AC-03 前置）**：
  按 §5.3 逐跳 dump，产出落点结论。依赖：无（与 T-01/T-02 并行）。
- **T-04 extractRunId 形状容错（AC-03 部分前置）**：改
  `src/front/forge_helpers.at`。依赖：T-03（形态清单）。验证：形状矩阵。
- **T-05 VM 缺口修复（AC-03/04）**：按 T-03 落点实施；同步
  `src/front/chat_message.at` 相关分支核对（gate_waiting 臂依赖
  getRelayStatus——run_id 提取修复后自然可达，若 store 轮询面有缺口一并修）。
  依赖：T-03、T-04。验证：实机 VM 截图（运行框 + 批准卡）。
- **T-06 端到端验收 + 规格落账（AC-01..05 全量）**：VM 实机全流程
  （spawn → 门 → 批准 → 推进）+ web 回归 + parity；spec 增量 SD-01/02
  提交 review 裁定。依赖：T-02、T-05。验证：AC 清单逐条 + 截图留档。

## 9. 复审记录

- 2026-10-10（draft 交接）：`stage: new`，PLAN-100 revision 1。
  `outcome: pass`——两问题的代码/实机证据已锚定（§4），设计含一处规格
  语义变更（SD-01：AC-04 前置催交，预算 1 次）需 review 确认；其余为
  缺口修复。`next: work`（执行按本仓 AGENTS.md 开
  `.wt/musk-100/auto-musk`，分支 `plan-100-dev`；T-01/T-03 为先行调查步）。
- 2026-10-10（work 执行轮）：`stage: work`，plan_revision 1，
  code_commit 160153c+探针清除提交（plan-100-dev @ musk-100 worktree，
  base cb7d8dc）。
  - **T-01 定音**：L1 探针（plan-flow-probe --keep）健康基线 = advisor
    工具序 `read_specs→list_plans→…→create_plan→read_plan→
    complete_plan_stage` 在位——工具注册（driver mode.tools 空 → lib.rs
    is_empty 门全放行）与模板强制均无缺；实机失败 = **模型随机不合规**
    （计划正文文本收尾、未调工具）。附带发现独立缺陷（execute 相位
    coder 改计划文件触发语义漂移 needs_replan——L1 因此 FAIL，登记
    KNOWN-DEBT 候选，不扩本计划 scope）。
  - **T-02 交付**：nudge 全套（nudges 预算/RewoundToExecute 重入/
    nudge_directive 注入/反模式双禁令/rerun 恢复指引文案）；relay 面
    91+6 测试全绿。
  - **T-03 定音**：VM 运行框退化 = chat_page 48KB 预算降桩清空长消息
    前段 spawn_relay 的 tool_result（run_id 唯一来源）+ VM 轮询回填
    快照为最终态。P100 探针另实锤：VM 对 JSON.parse 产物的深字段读
    退化（handler 上下文 blk 字段全 MISS）+ 视图模板内 fn 调用静默
    返空（坑①家族——extractRunId 恒空）。
  - **T-04/T-05 交付**：降桩豁免 relay 身份块（单测钉死）+ `tool_run_id`
    服务端直出（浅字段规避深读退化）+ extractRunId 浅字段优先/obj-str
    四臂容错 + chat_message 分支改判 tc.run_id 预计算字段。chat_page
    9 测试全绿；服务端实机 curl 验证（17 块超预算消息 spawn_relay
    result 保真 + tool_run_id 直出）。
  - **受阻（AC-03/04 终验）**：E2E 沙箱（auto run --render=vm +
    AUTO_BACKEND 隔离后端）会话载入链不稳定——SessionsLoaded/
    DetailLoaded 的 Http.get_msg 信封 JSON.parse 产物读间歇失效
    （r.ok 读出 -2147483648 位型垃圾），换 debug/release auto.exe
    构建、独立端口/工作区、大小种子均复现；用户实机桌面轨载入正常
    （上游 auto-lang VM 引擎债，同 KNOWN-DEBT"大会话载入"家族）。
    修复面在数据到达前端时逻辑闭环（web 轨同源 .at 已验证语法/分支）。
  - `outcome: pass`（①全交付+实机可验；②代码+服务端实机交付，VM UI
    终验挂上游受阻项）；`next: review`——review 裁定 SD-01/02 规格增量，
    并裁决受阻项路由（上游 auto-lang VM 读语义根修另立计划 vs 接受
    web 轨对照验收）。blockers 见 §10 Q-4。


- 2026-10-10（draft 交接）：`stage: new`，PLAN-100 revision 1。
  `outcome: pass`——两问题的代码/实机证据已锚定（§4），设计含一处规格
  语义变更（SD-01：AC-04 前置催交，预算 1 次）需 review 确认；其余为
  缺口修复。`next: work`（执行按本仓 AGENTS.md 开
  `.wt/musk-100/auto-musk`，分支 `plan-100-dev`；T-01/T-03 为先行调查步）。

### 9.1 复审记录（review 轮）

- `stage: review` | plan_id PLAN-100 | plan_revision 1 | **outcome: pass**
- reviewed_commit: 4f7de1bd02e20ab2aba8fd732ce0dba58443580b（plan-100-dev）
  base_commit: cb7d8dc8feb05cbc756f5207c770b567a05eea47（main）
  dependency_revisions: auto-ai@f1c7794（detached worktree）、
  auto-lang@78f65c103（detached worktree）——只读解析，无依赖改动。
- 评审测试门（worktree 独立复跑，4f7de1b）：`cargo test -p musk --lib relay`
  93 passed（含 nudge/模板 6 新用例）；`--lib chat_page` 9 passed（含降桩
  豁免 + tool_run_id 直出断言）；`--lib plan_tools` 6 passed；
  `--test parity_relay_api` 6 passed 1 ignored（hw/ag 双轨一致）。
- 验收判定：AC-01 pass（催交重入单测复跑）；AC-02 pass（二次失败文案含
  rerun 指引 + 预算独立单测）；AC-03/04 **pass（按用户 Q-4(b) 授权以等价
  证据关闭）**——服务端实机 curl（17 块超预算消息 spawn_relay result 保真
  + tool_run_id 直出）+ 单测 + 分支预计算逻辑（web 轨同源 .at）；VM UI
  直验未做（如实记录）：E2E 沙箱会话载入受阻于上游 auto-lang VM
  Http.get_msg 信封 JSON 读退化（引擎既有债，用户实机桌面轨不受影响）。
  用户以"review; then merge"指示裁决 Q-4 路由 = b（等价证据关闭 + 上游
  根修另立计划候选，登记 KNOWN-DEBT 候选）。AC-05 pass（parity 绿；
  web 轨同源代码面无独立改动）。
- findings：无阻塞项。非阻塞记档：①execute 相位 coder 改计划文件触发
  语义漂移 needs_replan（L1 探针实录，KNOWN-DEBT 候选）；②VM 深读退化
  与视图 fn 调用返空（坑①家族）上游根修候选。
- spec_inputs：docs/specs/modules/plan-flow.md（SD-01 落 AC-04/相位结束
  判定节）、docs/specs/modules/chat-streaming.md（SD-02 落契约⑧验收面）
  ——两文件现版与 SD-01/02 before/after 规则核对一致，实现即描述。
  supersedes_spec_components 已定稿（两路径）。
- evidence（耐久面）：单测（上列命令可复跑）；服务端实机证据 =
  `GET /api/chats/session/{id}/page` 的 blocks[0].tool_run_id/tool_result
  字段（17 块超预算夹具，chat_page 单测同形固化）；E2E 沙箱受阻证据 =
  auto-musk tmp/p100-e2e/vm4-vm13.log（临时，结论已录本节）。
- next: merge。

### 9.2 merge 收据（PLAN-100:r1）

| 检查点 | 证据 |
|---|---|
| prepared | reviewed 基线 4f7de1b；SD-01/SD-02 文本落 worktree docs/specs/modules/{plan-flow,chat-streaming}.md（e7911c6，documentation-only descendant——实现/依赖零改动，逐 delta 核对） |
| landed | rebase main 后 range-diff 等价（160153c→6461c2a、4f7de1b→d1a887b，第三提交为新增 spec）；ff-only 落 main，tip=e7911c66062a7040bbf22da89ad2effaa22bce26，无 merge commit |
| ledger_refreshed | workspace=auto-musk（store-mediated POST /api/specs/item ×4，200）：designs PLAN-100-plan-flow-nudge / PLAN-100-relay-runbox、reviews PLAN-100-review、reports PLAN-100-delivery；读回验证 specs v14→v18 四项在案 |
| archived | docs/plans/archived/100-relay-plan-gate-and-vm-runbox.md（status: archived，本节即收据） |
| cleaned | wt-guard clean → musk-100 组三 worktree（auto-musk/auto-ai/auto-lang detached）+ 分支 plan-100-dev 移除 |

产物新鲜度观察项（landing ≠ deployment）：主检出 backend/target/{debug,release}/musk.exe 与 gen/front/vue/dist 均先于本计划——debug musk 已于 cleaned 后重建（含修复）；release 与 web bundle 留待部署轮（沿 PLAN-099 收据先例：release 仅真部署轮）。

## 10. 待澄清事项

- **Q-1**（owner: 用户，T-01 前确认）：SD-01 把"无结果即失败"改为"催交
  一次后仍无结果才失败"——预算 1 次是否符合预期（现按截断续做同款裁定
  预置为 1，可调）。
- **Q-2**（owner: T-01 现场）：用户 dev musk 后端的启动方式/工作区根
  （观测端口 :17892 与 manifest 默认 17201 不一致；会话双写日志在实例的
  ws 根下，取证需要该根）。
- **Q-3**（已消解，T-03 执行中定音）：历史拍平路径服务端恒填（chat_page
  契约），缺口=预算降桩+VM 深读退化；活流tc 形态容错已由 T-04 覆盖。
- **Q-4**（owner: 用户/review，AC-03/04 终验路由）：E2E 沙箱会话载入
  受阻于上游 auto-lang VM 的 Http.get_msg 信封 JSON 读退化（详见 §9
  work 轮受阻记录；用户实机桌面轨不受影响）。裁决项：a) 上游根修另立
  auto-lang 计划后再补 VM UI 终验；b) 接受 web 轨对照验收 + 单测/服务端
  实机证据关闭 AC-03/04。当前全部修复面已提交（plan-100-dev）。
