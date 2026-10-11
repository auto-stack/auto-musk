---
plan_id: PLAN-105
status: archived
feature_name: UAT 87af4d4b run 生命周期与工具执行层修复批次（run 不收束死等/在途消息丢弃/run_command 引号/specs 假成功/VM 白名单空桩/进程残留）
author: [agent]
created_at: 2026-10-11T11:16:00Z
updated_at: 2026-10-12T09:30:00Z
plan_revision: 1
current_step: 9
total_steps: 9
supersedes_spec_components:
  - docs/specs/modules/chat-run-policy.md
  - docs/specs/modules/workspace-sandbox.md
  - docs/specs/modules/specs-ledger.md
new_spec_components: []
touched_goals:
  - goal-agent
  - goal-frontend-parity
  - goal-spec-knowledge
  - goal-security
---

# PLAN-105 — UAT 87af4d4b run 生命周期与工具执行层修复批次

## 0. 变更摘要

2026-10-11 手动 UAT 会话 `87af4d4b7e7a9c25e2d44114`（demo2，/auto-plan 全流
程，74 分钟）事后时间线定谳（三源对齐：turns.jsonl 353 事件、aaid 73 次
LLM 请求、serve 生命周期日志），登记八项缺陷并修齐。PLAN-104 已认领
K9–K13 观察面（VM 内存/空渲染/FilesView/spec 工具通道/merge 收尾）且其
非目标明确「不修 K2」——本计划认领其余全部，两计划无任务重叠：

1. **P1 run 不收束 → 300s 看门狗死等（UAT-K2 实锤扩展）**：末轮 LLM 流已
   正常收完（aaid 17:46:37 `chat stream done` out=2647，末条 assistant 文本
   以冒号结尾=正要发工具调用），`agent.run_stream` future 未决，musk 零事
   件挂满 300s，17:51:37 被 idle-timeout 强杀（serve `idle=300s
   elapsed=824s`）。定罪层未定：musk 事件循环 / auto-ai-agent run_stream /
   daemon `candidates=2` 双流收齐语义。
2. **P2 idle 超时文案误导**：`⏱ 运行空闲超时…建议把任务拆小后重发`——
   在 P1 场景下归因错误（不是任务大，是收束/停滞），且未告知「发『继续』
   可续跑」这一实际可行操作。
3. **P3 在途消息丢弃**：guard 占用期间新用户消息只落 ⚠ 提示+`busy:true`
   （`b.queued=true` 自动重试仅 web 前端实现；VM 轨 Sse.open 为 no-op）。
   UAT 17:18:51「继续」被丢，用户白等 4 分钟。
4. **P4 run_command Windows 引号保真**：`Command::new("cmd").arg("/C")
   .arg(cmd)`——Rust Windows 参数引用+cmd /C 引号剥离，字面引号进 argv。
   UAT 四中：`git -C '"D:/…"' Invalid argument`、`git commit -m "…"` 引号
   丢失中止、findstr 多词、curl 带引号 URL exit 3。模型整段子流绕行（-F
   文件/无引号形式），白烧 LLM 往返。
5. **P5 安全层词法无引号感知**：`tool_safety` 以 `split_whitespace()` 切
   token，引号内空格路径误切（与 P4 应共用同一 tokenizer）。
6. **P6 specs transition 假成功簇**：①`from_str_lossy` 未知串兜底映射
   `Draft`（`_ =>` 臂）；②`can_transition` 同态恒过；③`TransitionOk` 回显
   请求串而非应用后状态——帕斯卡式 `Approved`/`Reviewed` 得 200「成功」但
   状态纹丝不动（UAT 18:14:32/18:15:25 两次假成功，version 1→3 空转）；
   ④400 错误体恒 `"transition failed"`，吞掉真实原因（首个小写 `approved`
   被 400 实为设计区状态机正确拒绝 `Draft→Approved` 不合法，但模型无从得
   知）；⑤GET 回帕斯卡、POST 只认小写 snake_case，读到的值写不回去；
   ⑥`specs_upsert_of` 完全绕过状态机可直写任意状态（UAT 最终靠它兜底）。
7. **P7 VM 白名单端口空桩假成功**：`src/front/ports/whitelist.vm.at` 的
   `addWorkspaceRoot` 不做任何事却返回 `{ok:true}`（注释自认 PLAN-070 VM
   轨缺口在案）。UAT 17:28:06 用户在 VM 视图添加 `.wt` 显示成功，实际零
   效果，白烧两轮人工往返；serve 全窗口唯一 `POST /api/workspace/roots/add`
   在 17:36:15（绕开 VM 视图后才生效）。
8. **P8 run 后台进程残留锁 worktree（低优先）**：run 内经 run_command 起
   的 `python -m http.server`（PID 8008）与无头 Edge 在 run 收束后存活，
   cwd 锁目录 → `git worktree remove` 半失败（元数据已注销、目录残留需手工
   清；UAT 18:17:54 Permission denied → 18:18:33 not a working tree）。

## 1. 目标

### 交付目标

- **run 收束即回**：末轮流以任何形态（含纯文本收尾/疑似截断）正常收完
  后，run 在秒级（≤10s）收束落盘，不再消耗 300s 看门狗窗（AC-01）。
- **有界自动续跑**：异常收束（看门狗/失败收束）后自动以「继续」语义续跑
  一次（env 可调/可关），无人值守流程不再依赖人工 nudge（AC-02）。
- **在途消息不丢**：run 占用 guard 期间到达的用户消息入服务端待发队列，
  run 收束后按序自动触发新一轮（AC-03）。
- **引号保真**：Windows 下带引号命令经 run_command 语义正确（commit -m、
  git -C 带空格路径、curl -o、findstr 多词）；安全层 confinement 判定与
  执行层共用引号感知词法（AC-04/AC-05）。
- **specs API 诚实化**：未知/大小写不符状态串显式 400 且错误体带原因；
  响应回显应用后状态；GET/POST 大小写对称（读到的可写回）；假成功形态
  不复现；upsert 与状态机的关系有明文裁决并落地（AC-06）。
- **VM 白名单视图生效**：VM 轨增删白名单接真实后端（或接线前诚实报错，
  消灭 `ok:true` 空桩）；VM 内添加后同 workspace 会话工具即时可见（AC-07）。
- **进程收割**：run 收束/中止后 5s 内，该 run 经 run_command 启动的存活
  进程树被收割，不再锁 worktree（AC-08）。
- 全量回归零新增红 + 87af4d4b 场景五点（P1/P3/P4/P6/P7）复跑实证（AC-09）。

### 范围

- 主仓库 auto-musk：`backend/crates/musk/src/`（tools.rs、command_runner.rs、
  tool_safety.rs、specs.rs、auto_generated/extern_impl.rs、server.rs 相关臂）、
  `src/front/ports/whitelist.vm.at`、docs/specs 三个模块规范、测试与证据。
- 依赖 auto-ai（`backend/crates/musk/Cargo.toml` path 依赖
  `../../../../auto-ai/crates/auto-ai-agent|client`）：T-01 若定罪落点在
  agent run_stream / daemon candidates，按 AGENTS.md 第三行同组并排
  worktree（`.wt/musk-105/auto-ai`，分支 `auto-musk-dev-105`）修复，集成
  验证后尽快合回。
- 复现/取证资产：tmp/demo2 会话数据与三份日志（manual-uat-{aaid,serve,vm}
  .log）为只读证据现场；复跑一律新建演示 workspace，不回写现场。

### 非目标

- 不修 K9/K10/K11/K12/K13（PLAN-104 认领）；不重开其已登记结论。
- 不做 page 端点协议重构与 SSE/WS 化（104 非目标同款立场；run 期间可见性
  已有 PLAN-099 T-03 工具边界增量快照，观察面缺陷归 104）。
- 不动 daemon `candidates=2` 的双流机制本身（除非 T-01 定罪必需，且以最小
  修复为界）。
- 不做跨会话/全局任务队列系统——P3 只做「同会话单守卫下的待发队列」，
  PLAN-026 steering 语义的全面实现另立项。
- 不改 specs 状态机图本身（Designs 的 `Draft→UnderReview→Approved` 链是
  有意设计；修的是 API 的诚实性与对称性，不是放宽转移）。
- E-STREAM-HANG 已根修面（client 读循环 90s idle）不重开；P1 与其的交界
  由 T-01 取证划清。

### 成功边界

AC-01～AC-09 全部有可复验结果。若 T-01 定罪 P1 根因在 auto-ai 深层且修复
超出有界范围，按依赖政策登记并回用户裁定，不以削弱 AC 收口。

## 2. 架构方案

```text
T-01 定罪（有界调查，决策工件）：P1 run 不收束
      复现器：mock LLM client 按末轮形态收流（文本收尾/截断/双候选错峰）
      三层定位：musk 事件循环 / auto-ai-agent run_stream / daemon candidates
T-02 依定罪修收束（musk 就地；auto-ai 并排 worktree）        → AC-01
T-03 看门狗文案分类 + 有界自动续跑（env AUTO_RUN_AUTO_CONTINUE）→ AC-02
T-04 在途消息服务端待发队列（与 T-03 协同：有人工消息优先）    → AC-03
T-05 run_command 引号保真 + tool_safety 引号感知词法（共用）   → AC-04/05
T-06 specs transition 假成功簇（解析/错误体/回显/对称/upsert） → AC-06
T-07 VM 白名单端口接线（消灭空桩假成功）                       → AC-07
T-08 run 进程树收割（pid 登记 → 收束臂统一 taskkill /T）       → AC-08
T-09 全量回归 + 场景复跑收口                                   → AC-09
依赖：T-02←T-01；T-03/T-04 在 T-02 后合入（可先行开发）；T-05/06/07/08
相互独立可并行；T-09 最后。
```

机制取向（T-01 后冻结细节）：

- **收束判定**：run_stream 的正常返回臂即收束信号；若定罪在「双候选收齐
  语义」或「末轮无工具调用不触发返回」，以最小改动让流完成=循环退出。
- **自动续跑**：收束臂（含超时/失败收束）检查待发队列——有人工消息则以其
  为输入立即 spawn 下一轮；无消息且属异常收束时按 env 上限注入一次「继续」
  语义的自驱动轮（复用 chat_run_owner，不算新机制）。
- **引号保真**：Windows 侧 `cmd /S /C` + 命令串整体 raw 传参（`raw_arg`），
  `/S` 语义=剥最外层引号后原样执行；tokenizer 供 tool_safety 与执行层共用
  （引号感知切词）。
- **specs 诚实化**：API 边界把状态串解析从 `from_str_lossy` 换成显式
  `Result`（未知串→400 带合法值清单）；响应带应用后状态（服务端真值）；
  GET 序列化与 POST 解析共用同一大小写规约（snake_case，兼容层做一次
  帕斯卡归一仅限读侧）；错误体透传 `transition_item` 的 Err 原文。
- **进程收割**：run_command 执行期把 child pid 登记进 run 级注册表
  （ToolContext 既有面），chat_run_owner 全部收束臂（正常/超时/失败/drop
  守卫）统一收割——Windows `taskkill /F /T`（kill_process_tree 已有），
  Unix 进程组语义已备。

## 3. 技术栈

| 层 | 沿用方案 | 本计划约束 |
|---|---|---|
| 后端 | musk axum + 既有工具/安全层骨架 | 修复落 `src/`（手写面）；`auto_generated/extern_impl.rs` 为既有生成骨架的手写实现臂，按现状就地改 |
| 依赖 | auto-ai path 依赖（agent/client/daemon） | 跨仓改动走并排 worktree，合回遵 AGENTS.md 收尾序 |
| VM 前端 | src/front/ports/*.at（vm/web 双轨） | 只动 vm 轨白名单端口；web 轨已真接线不动 |
| 验证 | `cargo test -p musk`（逐目标串行）+ 复现器脚本 + 手动 UAT 复跑 | 新增单测/复现器随仓；UAT 复跑用新演示 workspace |
| 规范 | docs/specs 三模块 + KNOWN-DEBT K 表 | K2 行由本计划收口时改写（merge 阶段） |

## 4. 需求分析与背景调查

### 授权与流程边界

- 用户授权（2026-10-11 本会话）：「OK，总结发现的所有问题，然后形成一个
  修复计划」——授权范围=问题定谳汇总+本计划撰写；**执行、预算、自动续跑
  默认值需后续指令**（与 PLAN-104 对 K2 的授权口径一致）。
- 计划编号 105：双目录扫描（active 097/102/104，archived 最大 103）取
  max+1；单写者（本会话），创建前无同名冲突。
- 前置依赖：无硬前置。PLAN-104 executing 中，其 T-05 与本计划 T-06 都触碰
  `auto_generated/extern_impl.rs`/`server.rs` 的 spec 面——提交序协调
  （104 先行则 105 rebase 适配），无任务级重叠。

### 调查版本与关键事实（main 工作区，2026-10-11；证据现场 tmp/ 只读）

| 锚点 | 事实 | 对应缺陷 |
|---|---|---|
| tmp/manual-uat-aaid.log | 17:45:44 req→17:46:37 `stream done`（out=2647）后至 17:51:56 零请求；全程 73 请求/2991s 流式/156,751 out tok/52 tok/s | P1 时间主轴 |
| tmp/manual-uat-serve.log | 17:51:37 `chat run idle-timeout: idle=300s elapsed=824s`；17:18:51 `chat run spawn skipped (guard held)`；17:36:15 全窗口唯一 `POST /api/workspace/roots/add` | P1/P3/P7 |
| turns.jsonl（353 行） | 末条 assistant 以冒号结尾（#132「对策：…」后无工具调用）；#30「⚠ 已有运行在途，本条消息未执行」；#139 超时文案；#288-313 transition 四连试错 | P1/P2/P3/P6 |
| backend/crates/musk/src/auto_generated/extern_impl.rs:2781-2851 | 看门狗 select 臂 + 超时文案 + `persist_chat_run_msg("idle_timeout")` | P1/P2 |
| 同上 :907-952 | guard 占用分支：落 ⚠ 提示、`b.queued` 仅前端重试语义 | P3 |
| 同上 :275-319 | `specs_upsert_of` 直写状态（无 can_transition）；`specs_transition_of` Err→`Value::Null`（400 常串） | P6 |
| backend/crates/musk/src/command_runner.rs:121-127 | `Command::new("cmd").arg("/C").arg(cmd)`（引号经 Rust argv 规则+cmd /C 剥离） | P4 |
| backend/crates/musk/src/tool_safety.rs:190/255/279-296 | `cmd.split_whitespace()` 词法（引号盲） | P5 |
| backend/crates/musk/src/specs.rs:146-172 | `from_str_lossy` `_ => SpecStatus::Draft` 兜底 | P6① |
| 同上 :706-712 | `can_transition` 同态恒过（`from == to` 即 true） | P6② |
| 同上 :660-677 | Designs/Architecture 合法链 `Empty→Draft→UnderReview→{Approved|Rejected}` | P6④ 背景 |
| backend/crates/musk/src/auto_generated/server.rs:480-481 | `to_response(…, "transition failed", 400)` 错误体常串 | P6④ |
| src/front/ports/whitelist.vm.at:8-14 | `addWorkspaceRoot` 空桩返回 `{ok:true, error:""}`（PLAN-070 注释自认缺口） | P7 |
| backend/crates/musk/src/server.rs:59-66/1251 | `run_idle_timeout` env `AUTO_RUN_IDLE_TIMEOUT_SECS` 默认 300s | P1/P2 参数面 |
| KNOWN-DEBT-AND-RISKS.md:120-121 | UAT-K2（无自动续跑，登记为设计现状）/K3（循环检测击杀） | 本计划收口 K2 评估口 |

### 已排除项（不建任务，防复审重查）

- 18:03:07 `search` 工具 path_confined：turns 实参 `path=D:/autostack/auto-musk`
  （musk 主检出，本就不在白名单），安全层拒绝正确。
- K9/K10 造成「长流期间看不到进度」的感知问题：服务端已有工具边界增量
  快照（PLAN-099 T-03），缺陷在客户端渲染面，归 PLAN-104。
- 18:15:51 新会话 3e4f5d7c 来源不明（疑似 agent curl 试错误建）：现象单次、
  无稳定性影响，观察不改。

### 与既有计划的边界

- PLAN-104（executing）：其 T-05（spec 工具注册/HTTP DTO/技能文本）与本
  计划 T-06（specs.rs 状态机 API 面）同文件不同函数族；合并时序后到者
  rebase 适配。其非目标「不修 K2」= 本计划 P1/P2/P3 的所有权划分依据。
- PLAN-097/102：无文件交集。
- PLAN-070（archived）：whitelist.vm.at 空桩注释援引其 VM 轨缺口——本计划
  T-07 即该缺口的白名单域收口（其原登记范围更广，不整体重开）。
- E-STREAM-HANG（099 根修）：client 读循环 90s idle 超时只覆盖「流中停
  滞」；P1 是「流已完成而 run 不返回」，交界由 T-01 划清（若复现出停滞面
  099 环境变量失效，转登记不吞并）。

## 5. 详细设计

### 5.1 P1 run 不收束定罪与修复（T-01/T-02）

- 复现器（新 `backend/tests/` 或既有测试模块内）：mock Client 按 UAT 末轮
  形态收流——A 文本收尾无工具调用；B 文本+半截工具调用后流正常关；C 双
  候选错峰完成（先 done 者与后 done 者间隔>0）。断言 `chat_run_owner`
  在流完成后限时返回（当前复现形态应红）。
- 三层定位取证：musk 侧 on_event 最后事件时间线 vs auto-ai-agent 循环退出
  条件 vs daemon `candidates=2` 聚齐返回语义（auto-ai-daemon `server.rs`
  chat 流聚合处）。决策工件落 evidence（§6）。
- 修复按落点：musk 面就地；auto-ai 面并排 worktree（分支 `auto-musk-dev-105`），
  上游带回归锚，集成验证（musk 全量测试）后按依赖政策尽快合回。

### 5.2 P2/P3 收束语义与消息不丢（T-03/T-04）

- 看门狗超时文案分型：事件时间线判定「停滞」（流中有间隔）vs「收束」
  （流已收完后挂起）——收束型文案指引用户发「继续」并说明已保留部分产
  物；停滞型保留现有拆小建议但附停滞时长与最后事件时刻。
- 待发队列：`chats_message` guard 占用分支改为把消息追加进 per-session
  pending 队列（内存态即可，重启丢失可接受——与现 ⚠ 提示同级），run 收束
  臂（全部出口）检查队列并以其首条为新输入 spawn 下一轮；`b.queued` 前端
  重试语义保留（web 轨不破坏）。
- 有界自动续跑：收束臂无人工消息且属异常收束时，按 `AUTO_RUN_AUTO_CONTINUE`
  （缺省 1，0=关）注入一次「继续：从中断处推进，不重做已完成部分」自驱
  轮；注入轮自身的异常收束不再续（防链式）。遥测事件标注 auto_continue
  来源，前端可区分。
- 红线：自动续跑不绕过 cancel 旗标与守卫；注入轮仍走同一 chat_run_owner
  入口（单守卫语义不变）。

### 5.3 P4/P5 引号保真与引号感知词法（T-05）

- 执行层：Windows `cmd` 调用改 `/S /C` + 命令串整体 `raw_arg`（外层自行
  包引号），保证 cmd 语义=剥最外层引号原样执行；Unix `sh -c` 不变。既有
  `run_command_echo` 等测试全绿为兼容底线，新增四类引号用例（commit -m、
  git -C 带空格路径、curl -o 带空格文件名、findstr 多词）。
- 词法层：新引号感知 tokenizer（双引号段内不切分、反斜杠按 Windows 习惯
  最小转义），`classify_command`/`confine_*_multi` 与执行层共用同一份切词
  结果，杜绝「安全层看到的 token ≠ shell 看到的 argv」。
- 风险面：confinement 判定变化可能影响既有白名单/越界用例——逐条核对
  `tool_safety` 测试语义（判定应变严不变松：引号内路径越界此前漏判的场景
  补红）。

### 5.4 P6 specs transition 假成功簇（T-06）

- 解析显式化：API 边界（`specs_transition_of`/`specs_upsert_of`）状态串
  解析改 `Result`——未知串返回结构化错误（HTTP 400，body 含合法值清单与
  收到的原串）；`from_str_lossy` 保留给内部宽容面（不删）。
- 错误体透传：`to_response` 对 `transition_item` 的 Err 原文（如
  `invalid status transition for designs: draft -> approved（合法链 …）`）
  透传，替换常串 `"transition failed"`。
- 回显真值：`TransitionOk` 增/改 `applied_status`（服务端应用后枚举的
  snake_case 真值），`new_status` 保留请求原串仅作对照。
- 大小写对称：GET 侧序列化统一 snake_case（与 POST 解析同规约）；读侧对
  既有磁盘上的帕斯卡值做一次归一兼容（`"Draft"`→`"draft"`），写侧只收
  snake_case。
- upsert 裁决（默认方案，复审可改）：upsert 保留直写能力但响应中标注
  `bypassed_transition: true` 当写入状态不同于现状且不属合法转移——供
  merge 技能与审计识别；是否彻底禁由待澄清 §10 跟踪。

### 5.5 P7 VM 白名单端口接线（T-07）

- `whitelist.vm.at` 三 fn 接真实后端桥：优先走 VM 后端桥既有 HTTP face
  （等价 `/api/workspace/roots` 三口，与 web 轨 `whitelist_web.ts` 同语义：
  400 body 原样透传）；桥 face 缺失则按 PLAN-066 同族手法补最小 face。
- 兜底红线：接线完成前，空桩必须诚实——`add/remove` 返回
  `{ok:false, error:"VM 轨白名单暂未接线，请在 web 端或 API 操作"}`，
  消灭 `ok:true` 假成功（这一步即使 T-07 后半受阻也必须先落）。
- 验收：VM 实例内添加目录 → 同 workspace 新 run 的文件工具即可访问该目录
  （实机或 vm-first-run 场景断言）。

### 5.6 P8 进程收割（T-08）

- `ExecOutcome`/执行路径把 child pid 登记进 run 级注册表（ToolContext
  ProgressSink 同族的 run 域状态）；显式退出的即时注销。
- `chat_run_owner` 全部收束出口（正常/超时/失败/drop 守卫）统一收割存活
  pid：Windows `kill_process_tree`（taskkill /F /T 已有），Unix 进程组杀
  （spawn 已 process_group(0)）。
- 边界：仅收割本 run 经 run_command 起的进程；用户自启进程不碰。CDP 无头
  Edge 由命令自身脚本负责（不扩大到进程名扫描）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/modules/chat-run-policy.md | run 收束仅隐含 run_stream 返回；guard 占用消息丢弃（前端 queued 重试）；超时文案固定；无自动续跑 → ①流完成=run 收束（秒级）；②guard 占用消息入服务端待发队列、收束后按序触发；③异常收束有界自动续跑（env 上限、注入轮不再链式、遥测标注）；④看门狗超时文案分型（停滞/收束）并给可操作指引 | P1/P2/P3 实锤；UAT-K2 评估口收口 | AC-01/02/03 |
| SD-02 | modify | docs/specs/modules/workspace-sandbox.md | run_command 引号行为未约定（实际被 argv 规则损毁）；安全层词法引号盲；run 结束无进程收割 → ①Windows cmd /S /C 整串 raw 传参=引号保真合同（Unix sh -c 不变）；②confinement/classify 与执行层共用引号感知 tokenizer；③run 收束统一收割本 run 启动的存活进程树（≤5s，仅 run_command 面） | P4/P5/P8 实锤（四类命令损坏+目录锁残留） | AC-04/05/08 |
| SD-03 | modify | docs/specs/modules/specs-ledger.md | transition 未知串静默 Draft、错误体常串、回显请求值、GET/POST 大小写不对称、upsert 无界直写 → ①状态串解析显式 Result（未知串 400 带合法值清单）；②错误体透传真实原因；③响应回显应用后状态；④GET/POST 共用 snake_case 规约（读侧归一兼容）；⑤upsert 直写标注 bypassed_transition（裁决见 §10） | P6 假成功簇实锤（两次 200 空转+模型四连试错） | AC-06 |

（注：specs-ledger.md 同为 PLAN-104 SD-02 目标——两 delta 合并时序依计划
合入先后，后到者适配，规范终态以两计划 AC 并集为准。）

## 6. 测试设计

### 命令与证据约定

| 编号 | 命令／工作目录 | 预期结果 |
|---|---|---|
| V01 | worktree/backend：`cargo test -p musk --lib`（含 T-01 复现器三形态） | 复现器在修复前红（run 挂起不返回形态被限时断言捕获），修复后绿 |
| V02 | worktree/backend：`cargo test -p musk`（逐目标串行） | 全套绿，零新增红 |
| V03 | 引号矩阵单测（tools/command_runner 新用例） | commit -m/git -C 空格路径/curl -o/findstr 多词四类全过；confinement 引号用例判定正确 |
| V04 | specs API 单测（transition/upsert/GET 对称） | 未知串 400+原因；合法转移回读一致；帕斯卡输入显式 400（不再假成功）；upsert 标注 bypassed |
| V05 | 收束/队列/续跑单测 + 复现器复跑 | 流完成 ≤10s 收束；guard 期消息队列重放；异常收束自动续跑 1 次（env=0 关闭路径同测） |
| V06 | 进程收割钉测试（Windows 优先，Unix 进程组路径条件编译） | run 收束 ≤5s 端口释放/进程不在 |
| V07 | VM 白名单：vm-first-run 或实机最小场景 | VM 内添加目录后新 run 可访问；接线前空桩诚实报错断言 |
| V08 | 87af4d4b 场景复跑（新演示 workspace，同款四阶段消息流） | P1 不再触发看门狗；「继续」不丢；引号命令一遍过；transition 首次即带因 400/合法路径直过；白名单 VM 内添加即生效 |

### 场景矩阵

| 场景组 | 输入/故障 | 证据 |
|---|---|---|
| 收束 | 末轮 A/B/C 三形态（文本收尾/半截工具调用/双候选错峰） | V01/V05 |
| 队列/续跑 | guard 期发消息；异常收束无人值守 | V05/V08 |
| 引号 | 四类命令 × 白名单/越界路径 | V03 |
| specs | 未知/帕斯卡/合法/同态/绕行 upsert | V04 |
| 白名单 | VM 添加→新 run 访问 | V07 |
| 收割 | http.server 残留/正常退出注销 | V06 |
| 回归 | V02 全套 | 零新增红 |

### 证据落点（均为新）

- `docs/reports/105-uat-run-lifecycle-fixes-evidence.md`：T-01 定罪纪要
  （三形态复现+落点结论）、命令表、AC 映射、失败与修复记录。
- `docs/reports/105-uat-run-lifecycle-fixes-spec-delta.md`：SD-01/02/03
  对照（含与 104 SD-02 的合并时序说明）。
- 复现器/引号矩阵/specs 用例随仓（tests 或模块内）。

## 7. 验收标准

- **AC-01**（P1，V01/V05）：末轮流任意收尾形态（纯文本/半截工具调用/双候
  选错峰）下，LLM 流完成后 run ≤10s 正常收束并落盘部分产物；300s 看门狗
  不再作为收束手段（仅保留为兜底且触发时属停滞型）。
- **AC-02**（P2，V05）：看门狗/收束事件文案按「停滞/收束」分型，收束型
  含「发『继续』可从中断处续跑」指引；单测断言两种文案内容与触发条件。
- **AC-03**（P3，V05/V08）：run 占用期间发送的用户消息在 run 收束后自动
  作为下一轮输入执行（不再出现「本条消息未执行」丢失形态）；异常收束且
  队列空时自动续跑一次（`AUTO_RUN_AUTO_CONTINUE` 缺省 1，0=关闭路径同测）。
- **AC-04**（P4，V03）：Windows 下 `git commit -m "a b"`、
  `git -C "path with space" …`、`curl -o "f name"` 、findstr 多词模式经
  run_command 语义正确（对照直接 cmd 执行结果一致）。
- **AC-05**（P5，V03）：confinement/classify 对引号包裹的含空格路径按整
  体判定；「引号内越界路径此前漏判」补红用例至少 2 条。
- **AC-06**（P6，V04）：未知或大小写不符状态串 → 400 且 body 含原因与合
  法值清单；合法转移响应含应用后状态且回读一致；GET 返回值可直接作为
  POST 入参（大小写对称）；`"Approved"`/`"Reviewed"` 输入不再出现 200
  假成功；upsert 直写响应标注 `bypassed_transition`。
- **AC-07**（P7，V07）：VM 白名单视图添加目录后，同 workspace 下一轮 run
  的文件工具立即可访问该目录；接线前空桩返回可见错误（无 `ok:true` 假成
  功）。
- **AC-08**（P8，V06）：run 收束/中止后 ≤5s，该 run 经 run_command 启动且
  仍存活的进程树被收割（钉测试：http.server 端口释放）；正常退出命令不
  误杀无关进程。
- **AC-09**（总，V02/V08）：`cargo test -p musk` 全套绿零新增红；87af4d4b
  同款场景复跑五点（P1/P3/P4/P6/P7）全绿，全程无人工 nudge 完成四阶段。

## 8. 执行步骤

- [x] **T-01** 复现器与三层定罪（P1）
  新增：复现器测试（mock client 三形态收流）+ 定罪纪要。涉及
  `backend/crates/musk/src/auto_generated/extern_impl.rs`（事件循环读点）、
  auto-ai-agent `run_stream`（D:/autostack/auto-ai/crates/auto-ai-agent）、
  auto-ai-daemon chat 流聚合。→ AC-01 前置（决策工件：落点结论+修复面清单）。
  - [✅ 已完成] 定罪：**auto-ai-client rust-ref `AiClient` send 阶段无超时**
    （reqwest 裸构；099 的 90s 只覆盖体读循环）。末轮流收完+工具批全执行后，
    下一轮请求 send().await 永悬（aaid 零日志×turns 工具结果在块×idle=300s
    秒级吻合，三源对齐；E-STREAM-HANG 交界划清）。复现器：musk 形态 A/B
    绿钉（0.07s/0.06s）+ 定罪形态 D 红（10.15s 守卫不释放）；client 层钉
    测试红（30.01s 悬挂）。worktree：musk 321d003 前一提交 / auto-ai 321d003。
    纪要：`docs/reports/105-uat-run-lifecycle-fixes-evidence.md`。计划预设
    的「双候选错峰」形态 UAT 未出现（单流 start/done 成对），不单列复现器
    ——A/B/D 三形已覆盖实质矩阵。
- [x] **T-02** 收束修复（P1，依 T-01）
  落点= musk 就地或 `.wt/musk-105/auto-ai`（分支 `auto-musk-dev-105`）。
  验证 V01 红→绿。→ AC-01
  - [✅ 已完成] 落点 auto-ai（`5ffc55b`）：client send 阶段有界
    （env `AUTO_AI_CLIENT_SEND_TIMEOUT_SECS` 缺省 60s、超时单次重试、再败
    Http 错误；`complete` 体读同窗包界）+ daemon done/error 尾帧有界 await
    （灭 try_send 静默丢帧假收束）。V01：musk 形态 D 红 10.15s→绿 2.11s；
    client 6/6、daemon 73/73、A/B 钉恒绿。musk 侧零改动（Err 臂既有）。
- [x] **T-03** 看门狗文案分型 + 有界自动续跑（P2，含 env 面与遥测标注）
  `extern_impl.rs` 看门狗/收束臂 + server.rs env 解析。V05。→ AC-02
  - [✅ 已完成] 文案分型（停滞型=在途工具点名+拆小建议+继续指引；挂起型
    =部分产物保留+继续指引）；`AUTO_RUN_AUTO_CONTINUE` 缺省开/0 关（AppState
    `run_auto_continue`）；注入轮 transient `auto_continue` 事件、不落中间
    消息、防链式。配套 auto-ai agent 入口孤儿 tool_use 修补（注入轮请求线
    合法化）。V05 分型两态+续跑三态绿。musk a18913b / auto-ai 2a4d606。
- [x] **T-04** 在途消息服务端待发队列（P3）
  `extern_impl.rs` guard 分支 + 全部收束出口的队列重放。V05/V08。→ AC-03
  - [✅ 已完成] `chat_pending`（AppState 内存态）：guard 占用入队+⏳ 提示；
    收束出口按序重放（人工边界=新 run_msg_id+累积器复位+注入额度回复）；
    web 前端 b.queued 重试语义保留。V05 队列重放测绿（两轮按序+提示先于
    重放应答）。musk a18913b（与 T-03 同提交）。
- [x] **T-05** run_command 引号保真 + 共用引号感知 tokenizer（P4/P5）
  `command_runner.rs`（raw_arg//S /C）+ `tool_safety.rs`（词法替换）+ 用例。
  V03。→ AC-04/05
  - [✅ 已完成] `/S /C` + 整串 raw_arg（tokio From<std Command> 接线）；
    `tokenize_command` 引号感知切词替换四处 `split_whitespace`（confine×3 +
    classify 面不变）。V03：引号矩阵四测绿（commit -m「fix: a b c」回读
    一致 / git -C "space dir" exit 0 / curl -o "my out.txt" 落盘 /
    findstr /C:"two words" 命中）+ 漏判补红两例（type/git -C 根内首残片
    假放行形态）+ 根内带空格不误伤对照。runner/safety 全套 27 绿。
    musk 71dacb7。
- [x] **T-06** specs transition 假成功簇（P6）
  `specs.rs`（显式解析/序列化规约）+ `extern_impl.rs`（transition/upsert/
  错误体）+ 用例。V04。→ AC-06
  - [✅ 已完成] `parse_exact`（未知/帕斯卡→400 带该区合法值清单+原串）；
    transition Err 原文透传（`from -> to` 详情）；`TransitionOk.applied_status`
    应用后真值；线格式统一 snake_case（手动 Deserialize 读侧帕斯卡归一，
    GET/POST 对称——前端本就 snake 约定，对齐双赢）；upsert 保留直写+
    `bypassed_transition` 标注（Q-2 默认）。V04 四测绿 + specs lib 39 绿 +
    全 lib 575 绿。musk 71dacb7 后一提交。
- [x] **T-07** VM 白名单端口接线 + 空桩诚实化（P7）
  `src/front/ports/whitelist.vm.at`（+必要时后端桥 face）。V07。→ AC-07
  - [✅ 已完成] 走 PLAN-066 同族 host! 后端桥（vm_backend 注册
    workspace_roots_list/add/remove，in-process registry 直调）——VM http
    face 非 2xx 丢 body（{"error":"HTTP 400"}），故不走 HTTP 是有意决策，
    错误文案经桥原样透传=web 轨 whitelist_web.ts 同语义；宿主未注册 →
    诚实 {ok:false,error}。V07：host 桥回环钉测绿（增/查/错误透传/未知 ws
    拒/删五点）+ VM serve 冒烟门 2/2（.at 全量编译含新端口）。musk f6182b8。
- [x] **T-08** run 进程树收割（P8）
  `command_runner.rs`/`tool_context.rs`（pid 登记面）+ `extern_impl.rs`
  收束臂收割。V06。→ AC-08
  - [✅ 已完成] `ExecOptions.on_proc` spawn 即时上报凭证（Windows
    kill-on-close Job Object——唯一能收 `start` 分离孙的形态，UAT PID 8008
    实锤；Unix pid+进程组）→ `AppState.chat_procs`（键同守卫，经
    build_agent_with_context 闭包注入 RunCommand）→ chat_run_owner 收束
    出口 + GuardOnDrop panic 路径统一收割。顺手修真缺陷：超时臂读任务
    裸 await 悬死（管道被存活孙持有→EOF 永不到，15 分钟复现）→2s 有界。
    V06：runner 级（start /b 分离 server→job close→端口 ≤5s 释放）+ run
    级端到端（看门狗中止→收割→端口释放，兼验停滞型文案）双绿；全 lib
    578 绿。musk 2c5444b。
- [x] **T-09** 回归 + 场景复跑收口
  V02/V08；证据两报告落盘；KNOWN-DEBT K2 行改写（merge 阶段收口）。→ AC-09
  - [✅ 已完成] V02 逐目标串行：--lib **578 绿** + integration 14 目标绿
    （含 plan105 新目标 4/4 与修复的 parity_workspace_endpoints 4/4）；
    22 目标编译红为**基线 11c5db5 同款旧伤**（HashSet/ToolCall/authorization
    三类，基线干净 worktree 22 目标 23 errors 实证）——零新增红；顺手修
    21 文件 AppState 字面量。V08 实机（v08-demo 新工作区+真栈 aaid）：
    **P6 三形态全中**（帕斯卡带 designs 区专属清单 400 / applied_status
    回显 / from→to 带因 400）；**P4 真 LLM 带引号 commit 成功**
    （b8d5808 消息原样「v08 quote test: a b c」回读一致）；P1/P3/P7 双层
    确定性证据（复现器红→绿/集成测/host 桥+VM 起服门）。证据：
    `docs/reports/105-uat-run-lifecycle-fixes{,-spec-delta}.md`。KNOWN-DEBT
    K2 行改写留 merge。musk 2c5444b 后一提交。

依赖：T-02←T-01；T-03/T-04 建议在 T-02 后合入（开发可并行）；T-05/T-06/
T-07/T-08 独立；T-09 收口。每个 T 均独立可验证、单独可交付。

## 9. 复审记录

- **2026-10-12T10:40Z，stage: merge，PLAN-105:r1 收据（reviewed→archived，
  outcome: pass / delivered）**
  - prepared: reviewed 基线 `62e6a2c` + 规范增量落 worktree（3 模块文档
    +index 时间戳，`628311f`）——与复审冻结件 blob `77dd2ca2` 一致，
    实现/依赖零改动。
  - landed: rebase 到 main（`1f0f888`）后 tip=`e35b84a`，range-diff 补丁
    逐条等价（旧→新：9c7671a→8aa862a…62e6a2c→9c48605/628311f→e35b84a）；
    rebase 后 --lib 578 绿；主检出 `--ff-only` 无 merge commit，main
    tip=交付提交 ✓。
  - ledger_refreshed: workspace `auto-musk`（store-mediated
    POST /api/specs/item @17822，serve 用合并后二进制），version 27→30，
    读回三目：designs/PLAN-105-run-lifecycle-and-tool-layer、
    reviews/PLAN-105-review、reports/PLAN-105-delivery（draft）。
    **限制（K12 既有）**：HTTP DTO 四字段不带 file/tags——canonical 指针
    已写入 content；字段补全归 PLAN-104 T-05（spec DTO 域）。
  - archived: `docs/plans/archived/105-uat-run-lifecycle-and-tool-layer-fixes.md`
    （status: archived，completion_kind: delivered）。
  - cleaned: musk-105 组——auto-musk worktree+`plan-105-dev` 分支删
    （guard clean 后）；auto-ai `auto-musk-dev-105@2a4d606` ff 折回
    auto-ai main（三仓测试：client 6/6、daemon 73/73、agent 119/120 既有
    环境红）+ worktree/分支删；auto-lang 只读镜像删；v08 瞬态文件删，
    **保留 `D:/autostack/.wt/musk-105/v08-demo`**（P4 实机证据仓
    b8d5808，组目录因此留存）。
  - KNOWN-DEBT：K2 行改写为已收口（Q-1 缺省=1 生效、可 env 关，实跑
    观察后再裁定）；K14 注记引号诱因已修 + 宿主保护语义（taskkill /IM
    自毁拦截）残余开放留后续计划。
  - 产物时效观察项：①musk release 二进制不存在（无陈旧面；debug 二进制
    已随合并重建 e35b84a）；②**aaid（PID 15124，10-11 16:56 起）早于
    auto-ai 折回**——在跑 daemon 不含 done 尾帧有界投递修复，下次 daemon
    重启生效；③web bundle（10-03）与本计划无涉（纯后端+VM 轨改动）。
    另：手动 UAT 残留 serve（:17201，旧二进制）已于合并中停止。
  - 批量回归：**due**（receipt 覆盖至 98；102+105 已落地，105 为 5 倍数）
    → 交接 `/auto-plan:regress`（主检出单实例）。
  - next: regress（批量全量门）。

- **2026-10-12T09:30Z，stage: review，PLAN-105 rev 1（execution_done→reviewed）**
  - outcome: **pass**。独立性声明：与实施同会话复审——裁决从工件重建
    （全部验证重新运行，未采信执行摘要）；局限在案。
  - reviewed_commit: musk `62e6a2c`（base `11c5db5`，worktree clean）；
    dependency_revisions: auto-ai `auto-musk-dev-105@2a4d606`（base
    `f1c7794`，clean）+ auto-lang 只读镜像 `83e1ab6`（detached）。
    spec_inputs: docs/specs/modules/{chat-run-policy,workspace-sandbox,
    specs-ledger}.md（存在性核对 ✓；四 touched_goals 均为 index.json
    实有 goal）。delta 冻结件：spec-delta 报告 blob `77dd2ca2…`、
    evidence 报告 blob `bc3eb7b1…`（worktree 62e6a2c 内提交）。
  - acceptance_results（全部本复审重新运行）：
    - AC-01 **pass**——形态 A 0.07s/B 0.06s/D（UAT 定罪形态）2.17s 收束
      复跑绿；send 60s（lib.rs:91）< 看门狗 300s 代码锚定=仅兜底。
    - AC-02 **pass**——停滞型（点名 run_command）/挂起型两测复跑绿；
      触发条件=块状态巡检（extern_impl 分型代码锚定）。
    - AC-03 **pass**——续跑恢复/防链式/关闭路径/队列重放四测隔离复跑
      4/4 绿（批跑受 F-2 偶发，见 findings；隔离恒绿=确定性成立）。
    - AC-04 **pass**——引号矩阵四测复跑绿 + 实机证据存续核验
      （v08-demo 仓 b8d5808「v08 quote test: a b c」仍在）。
    - AC-05 **pass**——tokenizer 单测+漏判补红两例+不误伤对照（19 绿）。
    - AC-06 **pass**——V04 集成 4/4 复跑绿；实机三形态（区专属清单/
      applied_status/from→to 带）为工作阶段证据（存续可查）。
    - AC-07 **pass**——host 桥回环 + VM serve 冒烟门 2/2 复跑绿。
    - AC-08 **pass**——runner 级 job close + run 级端到端收割复跑绿。
    - AC-09 **pass**——全 lib **578 绿**复跑；integration 14 目标绿；
      22 编译红=基线 11c5db5 干净 worktree 同款（22 目标 23 errors，
      HashSet/ToolCall/authorization 三类）；auto-ai client 6/6、daemon
      73/73、agent 119/120（roles::registry_loads_builtins 主检出
      f1c7794 同款环境红——本复审重跑确证）。
  - findings（均非阻塞，已定级）：
    - **F-1**（已裁决偏差）：「双候选错峰」复现器未单列——UAT 无该形态
      （每请求恰一对 start/done），A/B/D 覆盖实质；接受。
    - **F-2**（测试基建债，既有族扩大）：chat_ 测试族并行批跑偶发（第二
      post 404/收束后回读 miss，~1/6–1/3 对跑率；隔离恒绿；基线 stash
      对照证既有 fs 竞态族）——本计划新增 7 测入族，聚合暴露面升。建议
      后续立项：store 读路径竞态修复或该族串行化。
    - **F-3**（低，建议）：「半截工具调用」无直接钉测（畸形 tool input
      不构成新悬挂面；done-带-tool_calls 路径已由形态 D 钉）——后续补
      一条畸形 input 用例。
    - **F-4**（登记）：22 integration 编译红=基线旧伤，建议独立清偿计划
      （三类明细见 evidence 报告）。
  - evidence: `docs/reports/105-uat-run-lifecycle-fixes-evidence.md` +
    `-spec-delta.md`（worktree 内，blob 已记）；auto-ai 钉测试随仓
    （client/agent 各 1）。
  - next: **merge**（`/auto-plan:merge`；K2 行改写 + Q-1/Q-2 随 merge
    收口；组内 auto-ai/auto-lang 兄弟随本组清理）。

- **2026-10-11T15:10Z，stage: work，PLAN-105 rev 1（executing→execution_done）**
  - outcome: **pass**（九任务全勾，AC-01～AC-09 均有可复验证据）。
  - code_commit: musk `plan-105-dev`（9c7671a…至 T-09 收口提交，wt-guard
    clean）；auto-ai `auto-musk-dev-105`（321d003/5ffc55b/2a4d606，119/120
    ——roles::registry_loads_builtins 为主检出同款既有环境红）。
  - task_ids: T-01…T-09 全部完成；依计划预设偏差一处（有据）：复现器
    「双候选错峰」形态不单列（UAT 未出现该形态，A/B/D 三形覆盖实质，
    纪要在案）。
  - evidence: `docs/reports/105-uat-run-lifecycle-fixes-evidence.md`
    （定罪纪要+V01–V08 全量+基线对照）+ `-spec-delta.md`（SD-01/02/03
    对照）+ 随仓测试（musk 14 新钉测/集成目标；auto-ai 2 钉测）。
  - blockers: 无阻塞项。待复审关注：①integration 22 目标编译红为基线
    同款旧伤（HashSet/ToolCall/authorization 三类，基线干净 worktree
    22 目标 23 errors 实证，建议另行立项清偿）；②tests 偶发 fs 竞态
    （基线同款）；③Q-1/Q-2 按 §10 默认方案执行待 merge 阶段确认。
  - next: **review**（`/auto-plan:review`；worktree
    `D:/autostack/.wt/musk-105/auto-musk` + auto-ai/auto-lang 兄弟保留至
    merge——path 解析耦合，musk-102 组同款先例）。

- **2026-10-11T12:00Z，stage: work 入口，PLAN-105 rev 1（drafting→executing）**
  - 用户指令「计划105: auto-plan-work 实施它」= 执行授权；Q-1/Q-2 按默认
    方案执行（自动续跑缺省 1 可 env 关；upsert 保留+标注 bypassed）。
  - worktree：`D:/autostack/.wt/musk-105/auto-musk`（分支 `plan-105-dev`，
    base commit `11c5db5`）；主检出预检仅 docs/plans 记账改动，无代码 WIP。
  - next: T-01 定罪起手。

- **2026-10-11T11:16Z，stage: new，PLAN-105 rev 1（drafting）**
  - 背景：用户要求汇总 87af4d4b 会话时间线分析发现的所有问题并成计划。
  - 定谳：八项缺陷（P1–P8）全部三源证据闭环；两项疑点排除（search 拒绝
    属正确 confinement、新会话 3e4f5d7c 现象单次观察不改）。
  - 边界：PLAN-104（executing）认领 K9–K13，本计划无任务重叠；spec 文件
    specs-ledger.md 两计划共改，合并时序后到者适配。
  - outcome: **pass**（可进入 work 的合同完备性——任务/AC/验证/路径均
    落地）；授权边界：撰写已授权，**执行与预算待用户指令**（含自动续跑
    缺省值裁定）。
  - next: **work**（用户指令后以 `/auto-plan:work` 执行；起手 T-01 定罪）。

## 10. 待澄清事项

| # | 事项 | 影响 | 默认方案（无裁决时按此执行） | 归属 |
|---|---|---|---|---|
| Q-1 | 有界自动续跑缺省次数（本计划取 1，env `AUTO_RUN_AUTO_CONTINUE`）与是否默认开启 | 无人值守体验 vs 失控轮次成本 | 缺省 1、可 env 关；merge 阶段随 K2 行改写一并回用户确认 | T-03 |
| Q-2 | `specs_upsert` 直写权限：保留+标注 `bypassed_transition`，还是彻底禁（HTTP 面也走状态机） | merge 技能回退路径与审计口径 | 保留+标注（merge 技能文本在 104 T-05 同批更新时提及） | T-06 |
| Q-3 | P1 若定罪 auto-ai 深层且修复超有界范围（如 agent 循环重构） | 本计划 AC-01 收口方式 | 登记依赖+回用户裁定，musk 侧先落看门狗短窗兜底（env 已有，缺省不动） | T-01/T-02 |
