---
plan_id: PLAN-103
status: archived
feature_name: 会话运行可见性修复——Windows 开关误判路径、审批门状态落快照、问卷卡可见性
author: [zcode]
created_at: 2026-10-10T16:50:00+08:00
updated_at: 2026-10-10T17:15:00+08:00
plan_revision: 1
current_step: 5
total_steps: 5
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-103 会话运行可见性修复

## 0. 变更摘要

实机事故（会话 `00c66383576f7f0b78818a9c`，2026-10-10 15:52–16:24，workspace=tmp）
定罪出四个叠加缺陷，本计划一次修复：

| id | 缺陷 | 根因位置 |
|:---|:---|:---|
| D1 | `dir /b /ad` 的 Windows 开关 `/b`、`/ad` 被 confinement 当作越界路径 → 误挂 30 分钟审批门（"卡住"直因） | `backend/crates/musk/src/tool_safety.rs` token 分类只跳过 `-` 前缀 |
| D2 | 审批门等待期间，轮询/重载路径的 tool 块不含 gate 载荷 → `ToolGateCard` 永不渲染，门等待显示为普通 "running" | `backend/crates/musk/src/auto_generated/extern_impl.rs` 快照装配不落 gate 字段 |
| D3 | 问卷卡只随最新一条 assistant 消息渲染 + 气泡文本无条件剥问卷围栏 → 非最新消息的问题在 UI 任何位置不可见 | `src/front/chats_view.at:751`（lastMsgId 条件）+ `forge_helpers.at:303`（stripQuestionnaire） |
| D4 | 门超时硬编码 1800s；期间看门狗/reaper 推窗 1800s——门一旦不可见即整会话纯死 30 分钟，无任何兜底文案 | `tools.rs:439`、`extern_impl.rs:2690-2718` |

## 1. 目标

- **G-01**：Windows 开关型 token（`/b`、`/ad`、`/s`、`/q` 等纯字母数字单段）不再误判为越界路径；真实越界路径（`C:\…`、`D:\…`、`\\?\…`、`..\`、带分隔符的 `/x/y`）仍全部拒收。
- **G-02**：审批门等待在轮询与重载路径可见、可决议（ToolGateCard 出现，approve/deny 推进 run）；决议后卡片状态随 tool_result 翻转。
- **G-03**：问卷问题对**任意**含问卷的 assistant 消息可见（与 web 版 `ChatsView.vue` 同口径），不再依赖"恰好是最新一条"。
- **G-04**：门超时改为可配置（`AUTO_GATE_TIMEOUT_SECS`，默认 300s），看门狗/reaper 推窗与门超时同源联动。

**非目标**：不重构 SSE/VM 事件通道本身；不切 Ash 沙箱（Design 004 既定路线）；不改 web 版（web 已是目标行为）；不做已答问卷的答案回填持久化。

**受影响仓库/模块**：仅 auto-musk——backend `musk` crate（tool_safety/tools/chats/auto_generated extern_impl/chat_page）+ VM 前端 `src/front`（chats_view.at 一处条件）。约束：旧档 chats.json 零 diff（新字段全部 serde default + skip）；VM 解释器规避项（r5 家族：块字面量禁嵌 JSON 产物对象）继续适用。

## 2. 架构方案

- **D1（tool_safety 开关豁免）**：token 分类在现"跳过 `-` 前缀"基础上增加 Windows 开关形态豁免：token 匹配 `/` + 纯字母数字（`^[A-Za-z][A-Za-z0-9]*$`）且不含 `\`、`.`、`:`、非 `//` 前缀、且按路径解析在磁盘上不存在 → 视为命令开关，不进入越界收集。落点：`confine_offending_paths_multi` / `confine_command_paths_multi` 与单根变体共用一个 `looks_like_path` 判定助手，避免多处漂移。**安全论证**：cmd 语义下单段 `/x` 不是合法路径引用（Windows 绝对路径必含 `\` 或盘符）；带分隔符的 `/x/y`、`\x`、盘符路径不受豁免照抓；磁盘存在性二次校验兜住 `/tmp` 这类与真实目录同形的极端情形。残余风险（Unix 风格单段根路径在 Windows 后端不可达）在规范增量中记明。
- **D2（门载荷落快照）**：运行循环 `on_event` 的 `tool_gate_waiting` 臂按 `tool_call id` 把 `{status:"gate_waiting", gate_id, pending_cmd, escape_paths}` 就地补进累积器 `tc2` 的对应 ToolCall；后续 1s 节流边界快照（`persist_chat_run_msg`/`assemble_chat_run_msg`）自动携带。`ChatSession::ToolCall` 结构加 `gate_id: String`、`pending_cmd: String`、`escape_paths: Vec<String>`（serde default + skip_serializing_if 空——旧档 JSON 零 diff，PLAN-084 archived 字段先例）。决议（approve/deny/超时）走既有 tool_result 事件臂翻转 status，卡收口。**前端零改动**：`normalizeToolBlocks`（forge_store.at:1731-1733）→ `messageBlocks`（forge_helpers.at:386-389）→ `ToolGateCard`（chat_message.at:353）全链已接。work 阶段需核实分页端点 `blocks_normalized` 直出形态对新增字段的透传（chat_page.rs）。
- **D3（问卷可见性）**：删除 `chats_view.at:751` 的 `.msg.id == .lastMsgId` 条件，凡 `hasQuestionnaire(.msg)` 为真即渲染卡片——与 web 版 `ChatsView.vue:217-220` 完全同口径（web 无 latest 限制）。`stripQuestionnaire` 维持现状（卡片常驻后剥围栏不再造成不可见）。
- **D4（门超时联动）**：`tools.rs` 门等待时长与 `extern_impl.rs` 看门狗/reaper 推窗消费同一个解析函数（env `AUTO_GATE_TIMEOUT_SECS`，非法/缺省回退 300s）；推窗量 = 门超时 + 300s 余量（维持"门超时先自然发生、看门狗只兜底管道级停顿"的 PLAN-073 r3 竞速结论）。

## 3. 技术栈

Rust（musk crate，tokio/axum/serde）+ auto-lang .at（VM 前端）。测试：`cargo test -p musk --lib`（基线 556 绿，f54011c）；VM 侧沿 p066_2 双脸锁/实机 UAT 口径（.at 视图层无单测框架）。

## 4. 需求分析与背景调查

- **授权记录**：用户 2026-10-10 会话内指令"OK，一起建立一个新计划，然后用 worktree 执行"——授权本计划的建立与 worktree 执行；无额外预算/自动续跑限制。范围限 auto-musk 单仓。
- **事故证据**：`D:\autostack\auto-musk\tmp\.autoos\chats.json` 会话 `00c66383576f7f0b78818a9c`：tc-4 `{"cmd":"dir /b /ad"}` 落盘 `status:"running"` 30 分钟无变化；16:22:53 门超时后错误文案 `run_command path argument '/b, /ad': denied by user (outside workspace root)`；后端进程（PID 19896）期间无 cmd 子进程、SSE 探针挂起零帧；守卫 16:24 随 run 收束释放。
- **代码事实**（均已实读核对）：
  - `tool_safety.rs:169-188`（confine_offending_paths token 循环，仅跳 `-` 前缀）、`:232+`（multi 变体）、`:313-318`（danger patterns）
  - `tools.rs:409-455`（human 门流程、`gate_id = tg-<nanos>`、`:439` 硬编码 1800s、`:452` 拒绝文案）
  - `extern_impl.rs:908`（run_key）、`:2395-2409`（touch_guard/on_event）、`:2664-2718`（bridge 订阅、gate_waiting 推窗 1800s）、`:2650`（tool_result 捕获 status="success"）
  - `chats.rs:95-120`（ChatMessage：content 恒序列化、blocks serde default）、ToolCall 结构（`tool/args/result/status/id`）
  - `forge_helpers.at:303-325`（stripQuestionnaire）、`:471-506`（messageDisplayBlocks）、`:332-391`（messageBlocks 块→tc 投影，gate 字段已接）
  - `forge_store.at:1686-1755`（normalizeToolBlocks：`tv.gate_id/pending_cmd/escape_paths` 已投影）
  - `chats_view.at:746-757`（问卷卡仅 latest）、`chat_message.at:350-356`（门卡条件 `tc.status=="gate_waiting"`）
  - web 对比：`web/src/views/ChatsView.vue:217-220`（问卷卡无 latest 限制——VM 应对齐的目标口径）
- **Spec 现状**：`docs/specs/modules/chat-streaming.md` 已明文要求"gate 卡片出现延迟 ≤2s，不得以刷新后可见替代"、"gate 载荷随块投影直达审批卡"——D2 属实现既存违规的收敛，不是新需求；`workspace-sandbox.md`"命令路径同口径"节需增补开关豁免规则；问卷 latest-only 仅存在于代码注释，无 Spec 条目。

## 5. 详细设计

### D1 tool_safety 开关豁免

新增判定助手（tool_safety.rs 内，单测直测）：

```
fn looks_like_windows_switch(token) bool:
  token 以 '/' 起始
  且去除前导 '/' 后仅 [A-Za-z0-9]+（无 \ / . : 空白）
  且 Path::new(token) 在磁盘上不存在
```

`confine_offending_paths` / `confine_offending_paths_multi`（及 `confine_command_paths*` 同口径）的 token 循环在现有 `-` 前缀跳过后追加：`if looks_like_windows_switch(token) { continue }`。

### D2 门载荷落快照

1. `chats.rs` ToolCall 增加字段（serde default + skip_serializing_if 空/None）：
   `gate_id: String`、`pending_cmd: String`、`escape_paths: Vec<String>`。
2. `extern_impl.rs` 运行循环 `on_event` 闭包内处理 `tool_gate_waiting`（bridge 臂已有该事件类型判定，`:2686-2688`）：从事件 payload 取 `gate_id/pending_cmd/escape_paths`，按 `tool_call id` 匹配 `tc2` 累积器中的 ToolCall 就地补字段并置 `status="gate_waiting"`，随后走既有 1s 节流快照通道落盘。
3. 决议路径零改动：approve → 命令执行 → tool_result 事件 → status 翻转；deny/超时 → ToolError → 错误结果事件 → status="failed"（错误文案即现 url）。
4. work 阶段核实：`chat_page.rs` 分页端点 `blocks_normalized` 直出形态是否原样透传 ToolCall 新字段（serde 视角应自动携带）。

### D3 问卷卡口径

`chats_view.at:749-757`：删除内层 `if .msg.id == .lastMsgId`。保留外层 `role=="assistant"` + `hasQuestionnaire(.msg)`。副作用（已答旧问卷重现表单）与 web 现行为一致，接受。

### D4 门超时常量化

`tools.rs`：`fn gate_timeout() -> Duration`（env `AUTO_GATE_TIMEOUT_SECS`，解析失败/非正回退 300s；单测锚点沿 `resolve_default_timeout_secs` T-02 形态）。`:439` 改用之。`extern_impl.rs` 两处推窗（看门狗 `wd_tx_bridge.send(now+1800s)` 与 `touch_guard_bridge(now+1800s)`）改为 `now + gate_timeout + 300s`——跨文件取值沿"store 文件不消费跨文件 fn 导入"先例，在 extern_impl 侧就地实现同解析（env 读取两处一致即可，不加跨模块依赖）。

### 规范增量

| delta_id | 操作 | 目标 | 前/后规则 | 理由 | 验收 |
|:---|:---|:---|:---|:---|:---|
| SD-01 | modify | docs/specs/modules/workspace-sandbox.md（命令路径同口径节） | 前：token 含 `/`/`\`/`..`/`~` 即入越界收集；后：追加 Windows 开关豁免——`/`+纯字母数字单段、磁盘不存在的 token 视为命令开关不收集；带分隔符/盘符/反斜杠路径与磁盘存在路径照收 | `dir /b /ad` 类命令误挂 30 分钟审批门（实机定罪） | AC-01/AC-02 |
| SD-02 | modify | docs/specs/modules/chat-run-policy.md（收束落盘与增量快照节） | 前：pending 快照 tool 块仅 name/id/args/result/status；后：门等待期间快照 tool 块必须携带 `status="gate_waiting"` + `gate_id`/`pending_cmd`/`escape_paths`，决议后随 tool_result 翻转 | chat-streaming 已要求 gate 卡 ≤2s 可见；轮询/重载路径现不可见（D2） | AC-03 |
| SD-03 | add | docs/specs/modules/ui-parity.md | 新条目：问卷卡渲染口径 VM=web——凡 assistant 消息解析出问卷即渲染，不限于最新一条 | 非最新消息问题在 VM 不可见（D3）；web 已是目标口径 | AC-05 |

## 6. 测试设计

- **T1（D1）** `tool_safety.rs` tests 增：`confine_offending_paths_multi("dir /b /ad", [root])` 为空；`dir /s /q`、`xcopy /y` 同；`type C:\Windows\win.ini`、`copy a \\?\C:\Windows\system32\config\sam`、`cat ../../../outside/x`、`sh -c /tmp/x`（磁盘存在分支）均非空；白名单根内绝对路径不越界（既有用例回归）。
- **T2（D4）** gate_timeout 解析单测：未设→300s；非法/零/负→300s；合法→env 值。
- **T3（D2）** server.rs::tests 沿 `chat_run_tool_boundary_snapshot_persists_mid_run` 形态：human 会话 + 越界命令挂门 → 轮询快照 JSON 断言 tc `status=="gate_waiting"` 且 gate_id/pending_cmd/escape_paths 在块上；resolve(deny) 后断言翻转。
- **T4（D3/D2 UI 面）**：实机 UAT 清单（work 收尾执行）：tmp workspace 发 `dir /b /ad` 类消息 → 门卡出现（轮询可见）→ deny 推进 run；含问卷的历史消息重进会话仍见问卷卡。
- **T5 回归**：`cargo test -p musk --lib` 全绿 ≥基线 556 + 新增。

## 7. 验收标准

- **AC-01**：`dir /b /ad` 在 human 会话不再挂门、直接执行返回目录列表。验证：T1 单测 + 实机。
- **AC-02**：真实越界路径（盘符/UNC/反斜杠/`..`/带分隔符正斜杠、磁盘存在路径）仍被拒/挂门。验证：T1 单测。
- **AC-03**：门等待期间轮询与重载均渲染 ToolGateCard（含待决议命令与越界路径），approve/deny 可推进 run 且卡状态收口。验证：T3 单测 + T4 实机。
- **AC-04**：门超时默认 300s，`AUTO_GATE_TIMEOUT_SECS` 可覆盖；看门狗/reaper 推窗随之联动。验证：T2 单测。
- **AC-05**：VM 问卷卡对任意含问卷的 assistant 消息渲染，与 web 同口径。验证：T4 实机 + 代码审。
- **AC-06**：`cargo test -p musk --lib` 全绿且用例数 ≥ 556 + 本计划新增数。验证：CI/本地 cargo。

## 8. 执行步骤

| id | 任务 | 依赖 | 文件/符号 | 产出 | 验收 | 验证命令 |
|:---|:---|:---|:---|:---|:---|:---|
| T-01 [x] | tool_safety 开关豁免 + 单测 | — | `backend/crates/musk/src/tool_safety.rs`（token 循环 + 新助手 fn + tests） | 豁免生效 | AC-01/02 | `cargo test -p musk --lib tool_safety` |
| T-02 [x] | 门超时常量化 + 推窗联动 | — | `tools.rs`（gate_timeout fn + :439）、`extern_impl.rs`（:2694/:2710 两处推窗） | 超时可配、推窗同源 | AC-04 | `cargo test -p musk --lib tools` |
| T-03 [x] | 门载荷落快照 | T-02 | `chats.rs`（ToolCall 字段）、`extern_impl.rs`（on_event gate 臂 patch tc2）、`chat_page.rs`（透传核实） | 门等待轮询可见 | AC-03 | `cargo test -p musk --lib chats server` |
| T-04 [x] | 问卷卡口径放宽 | — | `src/front/chats_view.at:749-757` | 与 web 同口径 | AC-05 | 实机 UAT（T4 清单） |
| T-05 [x] | 回归 + UAT 收尾 | T-01..04 | — | 全绿 + UAT 记录 | AC-06 | `cargo test -p musk --lib` |

**执行证据（2026-10-10，worktree `.wt/musk-103/auto-musk`，分支 `plan-103-dev`，基 `3e0354a`）**：

- T-01 [x]：提交 c5b39a9。`cargo test -p musk --lib tool_safety` → 13 passed（含新增 windows_switch_tokens_not_offending / real_out_of_root_paths_still_offending）。依赖 worktree：组内 `auto-ai`@f1c7794、`auto-lang`@3afea79（分支 auto-musk-dev-103；冷构建前补拷 gitignore 的 Cargo.lock——worktree 无锁文件会使 auto-lang build.rs panic）。
- T-02 [x]：提交 78ae839。gate_timeout_from_env 解析单测 default 300s / env 覆盖 / 非法回退全绿。
- T-03 [x]：提交 644f1de。新增端到端单测 `chat_run_gate_waiting_snapshot_carries_gate_payload`（挂门→快照载荷→deny 收口）通过；计划外发现并修复：chat_page.rs 分页 `blocks_normalized` 直出 gate 三字段硬编码空串（§10 预告的透传阻断点实锤），已随 T-03 接通。
- T-04 [x]：提交 aa2b5a0。chats_view.at 去 lastMsgId 条件；SD-03 规范增订（ui-parity §6）。
- T-05 [x]：`cargo test -p musk --lib` 全绿 **560 passed / 0 failed**（基线 556 + 新增 4：tool_safety ×2、gate_timeout ×1、门快照端到端 ×1）。
- 规范增量：SD-01（workspace-sandbox Windows 开关豁免）、SD-02（chat-run-policy 门载荷落快照 + 超时常量化）、SD-03（ui-parity 问卷口径）已随提交落 worktree 内 docs/specs。

## 9. 复审记录

- 2026-10-10 stage:new（plan_revision 1）——drafting 完成，任务覆盖全部 AC 与 SD；授权（建立+worktree 执行）已录于 §4。outcome: pass，next: work。
- 2026-10-10 stage: work | plan_id: PLAN-103 | plan_revision: 1 | outcome: **pass** | code_commit: aa2b5a0（plan-103-dev，基 3e0354a，4 提交 c5b39a9/78ae839/644f1de/aa2b5a0） | task_ids: T-01..T-05 全勾 | evidence: lib 560 绿（5.82s）；门快照端到端新测通过；证据明细见 §8 执行证据 | blockers: 无（AC-03/AC-05 的实机 UAT 臂需重建 musk.exe 并重启 VM 后端后按 §6-T4 清单走查——环境性前置，非代码缺口；现已停在 `dir /b /ad` 可直接执行、门卡/问卷卡轮询可见的预期上） | next: review。
- 2026-10-10 stage: review | plan_id: PLAN-103 | plan_revision: 1 | outcome: **pass** | reviewed_commit: aa2b5a00c8cc8c5e1a5fcef1f46df2387b2eeff5 | base_commit: 3e0354a | dependency_revisions: auto-ai f1c7794、auto-lang a424dc18b（组内 `auto-musk-dev-103`） | spec_inputs: docs/specs/modules/{workspace-sandbox,chat-run-policy,ui-parity}.md worktree 增量（冻结 diff hash `d0acbfb47db1445d`；三处均 modify/add 既有文件，无 retire、无 goal 影响） | acceptance_results: AC-01 pass（tool_safety 13 通过，含事故原样 `dir /b /ad` 与 4 类真实越界负例）｜AC-02 pass（同上负例组）｜AC-03 pass（`chat_run_gate_waiting_snapshot_carries_gate_payload` 挂门→载荷→deny 收口全链；UI 投影链审查：normalizeToolBlocks(forge_store.at:1731)→messageBlocks(forge_helpers.at:386，:372 gate_waiting 透传)→ToolGateCard(chat_message.at:350) 三层均为既有接线，本次仅接通数据面）｜AC-04 pass（gate_timeout_defaults_to_300s_env_overridable + 推窗 = 门超时+300s diff 审）｜AC-05 pass（diff 核对为纯条件删除；web ChatsView.vue:217 同口径比对；实机走查臂见 F-01）｜AC-06 pass（560 passed/0 failed，5.78s，≥556+4） | findings: F-01（low，非阻塞）实机 UAT 未执行——前置为重建 musk.exe + 重启 VM 后端，用户已在知情前提下指令 review+merge，走查清单移交部署后执行；F-02（info）chats_view.at `lastMsgId` computed 因本计划失去唯一消费方（死计算，VM 无 lint 门，不处理） | evidence: 本 HEAD 复跑 `cargo test -p musk --lib` → `test result: ok. 560 passed; 0 failed`（5.78s），四个新测逐条 ok；worktree 树 clean（0 脏文件） | next: merge。
- 2026-10-10 stage: merge | plan_id: PLAN-103:r1 | outcome: **pass** | delivery_commit: **a12b18e**（main tip，ff-only 无 merge commit） | 收据检查点：
  - `prepared`：reviewed aa2b5a0（worktree clean，0 脏）；canonical spec diff = SD-01/02/03 已随提交落 worktree docs/specs 三文件（冻结 hash d0acbfb47db1445d）；projection 目标 = ledger workspace `auto-musk` 三条目。
  - `landed`：rebase main（c913e52）后 ff-only；旧→新映射 c5b39a9→10eddb8、78ae839→760b11c、644f1de→3bc9b38、aa2b5a0→a12b18e；`git range-diff 3e0354a..aa2b5a0 c913e52..a12b18e` 四补丁逐条等价（=×4）；rebase 后复跑 lib 560 绿（5.65s）；main tip=a12b18e 与 delivery 一致；主检出冒烟 `cargo test -p musk --lib tool_safety` 13 绿。
  - `ledger_refreshed`：workspace `auto-musk`（store-mediated POST /api/specs/item @17201），version 21→24，读回校验三条目在案：designs/PLAN-103-chat-gate-visibility、reviews/PLAN-103-review、reports/PLAN-103-delivery（均 Draft，source 引 delivery a12b18e）。
  - `archived`：docs/plans/archived/103-chat-gate-visibility-and-flag-safety.md，status: archived，completion_kind: delivered。
  - 工件时点观察（landing ≠ deployment）：①backend `target/debug/musk.exe` 为 2026-10-10 00:04 旧构建——运行中实例**不含**本计划修复，重建（`cd backend && cargo build`）+ 重启 VM 后即生效，亦为实机 UAT（F-01 走查清单）前置；②release musk.exe 缺位（launch-vm.cmd 走 debug 回退）；③VM UI 轨源码即时编译，重启即生效；④web 轨未触及，gen/front bundle 不涉及。
  - 批量回归到期判定：PLAN-103 非洲期 id（103 % 5 ≠ 0）；见 `.last-batch-regression.json` 时点核记于 cleaned 后。

## 10. 待澄清事项

- T-03 的 `blocks_normalized` 分页形态：若服务端直出归一化块且字段白名单式重建，需同步补 gate 字段映射——work 阶段核实，属既授权范围内的路径/形态修正。
- D3 副作用（已答旧问卷重现空表单）按"与 web 同口径"处理，不做答案持久化；若 UAT 观感不可接受再立后续计划。
