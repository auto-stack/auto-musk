---
plan_id: PLAN-092
status: drafting
feature_name: UAT 四缺陷收敛——会话id可见性 / VM流式刷新 / auto-plan技能自带 / 计划落盘
author: [agent]
created_at: 2026-09-27T17:30:00+08:00
updated_at: 2026-09-27T17:30:00+08:00
plan_revision: 1
current_step: 0
total_steps: 4
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-092 — UAT 四缺陷收敛（2026-09-27 VM 实机走查）

## 0. 变更摘要

2026-09-27 实机 UAT（demo-1 workspace，会话 f2602fffde05e55452984618）暴露四个
缺陷，本计划将其收敛为可执行修复合同：

1. **P1 会话 id 不可见**：右上角 i 符号 InfoIcon（SessionInfo）在 PLAN-084
   T-06 头部重做时退役，会话 id 从此无处显示/复制。
2. **P2 AI 回复不自动上屏（VM 轨)**：发送后 assistant 回复要点会话列表才刷新，
   直接违反 `docs/specs/modules/chat-streaming.md` 契约②「不得以'刷新后可见'
   替代」。VM 轮询降级机制（PLAN-051 T10 PollStream）实测全天零回填到达。
3. **P3 产品 agent 不认识 /auto-plan:new**：aaid 只扫全局 `~/.config/autoos/skills`
   （superpowers 九件），auto-plan 四技能仅 ZCode 可见，agent 降级用 writing-plans。
4. **P4 计划未落盘**：agent 加载 writing-plans 技能内容后轮次中断，write_file
   从未被调用；docs/plans/ 实际为空，计划/文件栏不可见是数据不存在，非视图缺陷。

影响仓库：auto-musk（P1/P2/P3 同步源）；auto-ai（P3 拾取时机、P4 轮次续跑，
跨仓改动按 AGENTS.md 第三行依赖 worktree 规则执行）。

## 1. 目标

- 会话 id 在 VM 会话详情页可见且可一键复制（恢复 084 退役的信息入口，紧凑形态）。
- VM 轨聊天满足 chat-streaming 契约②的实时性口径（无需手动刷新），
  并消除 PollStream 空转日志刷屏。
- 产品内 agent 自带 auto-plan 四技能（musk 为技能真源，vendored 分发）。
- 产品内「用 /auto-plan:new 建计划」端到端产出真实计划文件，计划/文件栏可见。

**非目标**：
- SSE 桥泛化（048 勘察阶段2，上游 iced shell-SSE 桥）——本轮仍以轮询降级为
  VM 直播机制，只修其失效环节。
- auto-lang「跨模块 SET_FIELD 不达根态」状态作用域专项根修（536 T12 已立案
  上游 KD）——本轮在 musk 侧绕开对该标量的依赖并登记，不追上游根因。
- auto-ai 日志观测体系整体建设——仅登记缺口，按 T-04 需要 最小加打点。

**成功判据**：AC-01..AC-06 全绿（见 §7）。

## 2. 架构方案

- **P1**：ContentHeader 右侧恢复信息入口（i icon → 展开行：会话 id + 复制按钮）。
  参考退役件 `src/front/session_info.at`（仍在树中，未挂载）的 id 展示与
  clipboard 复制；不恢复 token 求和/消息计数（保持紧凑，084 头部三件套格局不变）。
  复制走既有 clipboard native（session_data_helpers.at 先例）。
- **P2**：不新增通道，修复既有 PollStream 轮询链的三个失效环节：
  ①`poll_inflight` 死锁自愈（首拍 get_msg 响应丢失后永久沉默——改为带超时
  的单飞：发起时记时间戳，超过 N 秒未回收即放行下一拍）；
  ②`poll_window` 过期清理（expired 臂 pop 掉过期戳再 return，止血
  "[POLL] window expired" 每 500ms 刷两条）；③续窗信号去 `.streaming` 标量
  依赖（该标量 VM 读取为垃圾值 -2147483647，续窗分支恒真导致 wins 无界增长
  至 9999——改为「最近一次 PollBackfill 回填活动即续窗」的可靠信号）。
  叶链投影契约③（回填推进 active_leaf）核对现状，缺失则补。
- **P3**：musk serve 启动时把仓内 vendored 四技能（`.agents/skills/auto-plan-*`）
  幂等同步到 `<config>/autoos/skills/`（musk 为真源，serve 为分发点）；aaid 侧
  SkillTool load-miss 时对技能目录重扫一次（auto-ai 小改），使已运行的共享
  daemon 无需重启即可拾取新同步的技能。
- **P4**：有界调查定罪「技能工具返回后轮次中断」环节（aaid agent loop 续跑 /
  取消语义 / 静默错误），修复使 write_file 得以执行；连带确认计划落盘后
  plans/files 视图切换即见（PlansLoaded 桥已存在，验证刷新语义即可）。

## 3. 技术栈

- auto-musk：`.at` 前端（VM 解释执行 + vue 双轨）+ Rust axum 后端；构建
  `cargo build`（backend/ 下）+ `auto build`；门禁 build strict / vitest /
  style-parity / vm-probe。
- auto-ai：Rust（auto-ai-cli/daemon）+ `.at` agent 核心（auto-ai-agent crate）。
- 无新依赖。

## 4. 需求分析与背景调查

### 授权记录

- 用户 2026-09-27 授权：将本会话 UAT 四问题综合成计划文件跟踪 musk 修改
  （本计划即交付物）；修复执行授权随 /auto-plan:work 流程另行开始。
- auto-musk 仓改动走 worktree 组 `musk-092`（分支 `plan-092-dev`）。
- auto-ai 仓改动（T-03 rescan、T-04 轮次修复）按 AGENTS.md 第三行：同组并排
  依赖 worktree（`.wt/musk-092/auto-ai`，分支 `auto-musk-dev`），消费后尽快折回。

### 实证记录（2026-09-27 走查，demo-1）

**P1**：
- `git show 9448b0c`（PLAN-084 T-03/T-04/T-06）：「聊天头部重做……SessionInfo
  退役登记 KNOWN-DEBT」——info icon 有意退役，需求回归=债转计划。
- `src/front/content_header.at` 现状无 session_id 展示位；`src/front/session_info.at`
  仍在树中含 id 展示 + clipboard 复制参照（`session_data_helpers.at` Plan 029 T7）。

**P2**（serve/vm 日志均在 `tmp/*-0924.log`，UTC 时间；本地=UTC+8）：
- UAT 发送 08:54:02Z、回复完成 08:54:14Z；回复已持久化（点击列表可见）→纯前端链断。
- `src/front/forge_store.at:796-860` PollStream handler：deadman 窗 120s +
  `.streaming==true` 续窗 + `poll_inflight` 单飞 + `Http.get_msg(...PollBackfill)`。
- vm log 全天 **PollBackfill 桥到达 0 次**（ForgeStore 其他事件 55 次，桥本身通）；
  `page?limit=50` 全天仅 6 次（boot 1 + UAT 窗口 5，均为点击/首载量级；机制健康
  应为 2 次/秒）→ 轮询请求链从未走通，首拍响应丢失 + inflight 卡死为最大嫌疑。
- vm log dbg 行实证 `streaming=-2147483647`（布尔读损坏）与 `wins=9999`
  （续窗分支在垃圾真值下持续 push，列表无界增长）；"[POLL] window expired"
  以 2 行/秒刷屏至今（vm-0924.log 已 33 万行，跨会话追加）。
- 既有上游立案：PLAN-536 T12「跨模块 SET_FIELD 不达根态」（auto-lang KD）——
  本计划读侧垃圾值同族登记。
- 契约锚点：`docs/specs/modules/chat-streaming.md` 契约②（实时性口径）③（叶链
  投影：「只写数组不推进叶 = 渲染不可见缺陷」）。

**P3**：
- `auto-ai-cli/src/main.rs:286-289`：启动期一次 `SkillRegistry::scan(~/.config/autoos/skills)`
  并注册 SkillTool；目录现仅 superpowers 九件（ash-*/brainstorming/executing-plans/
  writing-plans 等），无 auto-plan-*。
- 会话 trace（/api/chats/session/f2602.../page）：agent thinking 自述「我的技能列表里
  没有 /auto-plan:new」+ glob `**/*auto-plan*` 零命中；auto-plan 四技能真源在
  musk 仓 `.agents/skills/auto-plan-{new,work,review,merge}/SKILL.md`。
- `auto-ai-agent/src/skill.at`：scan 走 `*/SKILL.md`；registry 启动期构建后固定。

**P4**：
- 会话 trace 最后一块 `{"kind":"tool","tool_name":"skill",...,"tool_status":"success"}`
  ——writing-plans 内容已返回，**其后无任何块**：无 write_file、无后续文本，
  轮次在技能加载成功后中断。
- `tmp/demo/docs/plans/` 实际为空（仅 archived/）；主检出 `git status` 干净
  （无写错路径的残留）；`write_file` 工具存在且已注册（main.rs:273）→ 非工具缺失。
- aaid 日志无 08:5x 错误行（aaid 不记请求行——观测缺口一并登记）。
- 视图侧：`/api/plans` 扫 workspace docs/plans（PlansLoaded 桥事件存在），
  数据落盘后切换视图即可见（T-04 验收确认）。

### 约束与依赖

- VM 轨调试遵循记忆先验：msg 桥大载荷丢弃（084-D1）、自足 MCP 冒烟脚本法、
  serve 共享注册表 MUSK_CONFIG_DIR 隔离（冒烟时）。
- 共享 aaid daemon（17654）为多消费方进程：rescan 改动须兼容已运行实例；
  杀前核连接、杀后主检出重启（089 尾清先验）。

## 5. 详细设计

### T-01 ContentHeader 会话 id 信息入口（P1）

- `content_header.at` 右侧槽（✎/🔍/🗑 同排）加 i info 按钮；点击展开/收起
  一行弱色信息条：`session: <id>` + 复制按钮（lucide Copy/CopyCheck，复制成功
  态常驻至下次复制——029 已登记 setTimeout 自动复位不可表达，沿用）。
- id 空态显示 `—`（session_info.at 先例）。不引入 tooltip（084-D3 债）。
- 退役件 session_info.at 保持不挂载（避免 token 求和等重逻辑回归），仅作参照；
  若实现复用其 fn，优先内联进 content_header 域（store 跨域读取最小化）。

### T-02 PollStream 刷新链修复（P2）

`forge_store.at` PollStream 域四处改动：

1. **inflight 超时自愈**：`poll_inflight` 布尔改为伴随 `poll_inflight_at`
   时间戳；tick 时 `poll_inflight && Date.now()-poll_inflight_at > 10000`
   视为丢失自动放行。阈值 10s 覆盖正常回填（页端点 <50ms）×10 余量。
2. **过期清理**：`Date.now()-started > 120000` 臂内先 `poll_window = []`
   （或 pop 尾元素）再 return——空闲态 tick 直接短路在 `wins.length==0`，
   "window expired" 不再刷屏。
3. **续窗信号置换**：删 `if .streaming == true { push }`（依赖损坏标量）；
   改为 PollBackfill 回填成功臂 push 续窗（回填活动本身即「轮询有产出」的
   可靠信号；窗口语义从「发送后 2 分钟」修正为「最后一次回填成功后 2 分钟」）。
   Send/StartStream 发送时 push 首戳不变（列表 push 经共享 vmref，536 T12
   已证实可靠）。
4. **叶链核对**：PollBackfill 回填写入后确认 active_leaf 同步服务端叶
   （chat-streaming 契约③）；若 PLAN-067 已实现则仅补断言，缺失则补推进。

`.streaming` 标量本身保留（web 轨 SSE 语义仍用），VM 轨不再依赖其真值做门控；
dbg 行增打 `inflight_at` 便于下次归因。

### T-03 auto-plan 四技能产品自带（P3）

- musk 仓：serve 启动序列（server.rs 装配处）增 `sync_builtin_skills()`——
  把 `.agents/skills/auto-plan-{new,work,review,merge}/`（含 SKILL.md 及附属文件）
  幂等拷贝到 `<config_dir>/skills/auto-plan-*/`（存在且内容相同则跳过；
  musk 版本新则覆盖）。config_dir 复用 serve 现有 MUSK_CONFIG_DIR 解析。
  vendored 源以仓内 `.agents/skills/` 为真源（与 ZCode 侧同文件，单一维护点）。
- auto-ai 仓：`skill.at` SkillTool load 路径 load-miss（名字不在 registry）时
  对原目录重扫一次再试；命中则回填 registry。已运行 daemon 无需重启即可见
  新同步技能。
- 启动顺序契约登记：首次部署（skills 目录尚无四技能）需 aaid 重启一次或
  依赖 rescan（rescan 落地后顺序无敏感）。

### T-04 技能加载后轮次中断定罪与闭环（P4）

- 有界调查（上限 0.5 日）：aaid 侧复现「skill 工具成功返回 → 下一模型调用」
  链路；检查 agent loop 续跑条件、客户端断连取消语义、静默错误吞噬点；
  在 aaid 日志补最小错误打点（观测缺口）。产出定罪结论入本节。
- 按定罪结果修复（嫌疑序：①run 取消语义把「客户端轮询断开」误当终止；
  ②loop 对 skill 工具结果后的继续调用缺路由；③模型调用静默失败无重试无日志）。
- 闭环验收：demo workspace 发「/auto-plan:new 建一个笔记 app 计划」→ agent
  走 auto-plan:new → write_file 落 `docs/plans/NNN-*.md` → 计划/文件栏切换可见。

### 规范增量

| delta_id | 变更 | 目标 | before/after | rationale | AC |
|:---|:---|:---|:---|:---|:---|
| SD-01 | modify | docs/specs/modules/chat-streaming.md | before：VM 轮询降级无自愈语义（inflight 布尔单飞/窗口依赖 streaming 标量/过期不清理）；after：单飞带 10s 超时自愈、续窗=最近回填成功后 2 分钟、过期即清理、空闲零日志 | P2 实证三失效环节；契约②「不得刷新后可见」的 VM 臂落地 | AC-02, AC-03 |
| SD-02 | modify | docs/specs/modules/workspace-ui.md | before：会话 id 无产品内入口（084 退役未替代）；after：ContentHeader 信息入口常驻，id 可见可复制 | 会话 id 是调试/汇报锚点，UAT 汇报需手工翻存储不可接受 | AC-01 |
| SD-03 | modify | docs/specs/modules/plan-flow.md | before：auto-plan 技能仅 ZCode 侧可见（双轨架构注记）；after：musk 为技能真源，serve 启动幂等分发四技能至用户 skills 目录，aaid load-miss 重扫拾取 | 产品 agent 须自带项目技能；消除「降级用 writing-plans」路径 | AC-04 |
| SD-04 | modify | docs/specs/modules/plan-flow.md | before：agent 技能驱动建计划无落盘保障（writing-plans 内容返回即中断无告警）；after：技能调用后轮次必须续跑至产出或显式报错；计划落盘 docs/plans/ 后计划/文件栏切换可见 | P4 数据不存在非视图缺陷；轮次静默中断为最隐蔽失败形态 | AC-05, AC-06 |

## 6. 测试设计

- 门禁全绿：`backend/` 下 `cargo build --release`；`auto build` strict；
  vitest（23+1）；style-parity diff=0；vm-probe PASS。
- T-01：VM 冒烟截图（i 入口 + id 行 + 复制态）；剪贴板实贴对拍 session id。
- T-02：VM 冒烟——发送后回复 ≤3s 自动上屏（无点击）；空闲 5 分钟 vm log
  零 "[POLL]" 行；发一轮长回复验证续窗（回填活动续期）。
- T-03：MUSK_CONFIG_DIR 隔离冒烟——serve 启动后技能目录出现四技能；
  已运行 aaid 不重启，新会话技能列表可见 auto-plan:new（rescan 生效）。
- T-04：端到端 UAT 重放——真实建计划请求产出文件；serve 日志/目录双验。
- 契约对拍：chat-streaming 契约②③逐条核（乐观上屏/首增量/叶推进）。

## 7. 验收标准

- **AC-01**：VM 会话详情页可见当前会话 id，点复制后剪贴板内容 == session id。
  验证：VM 实机操作 + 粘贴对拍。
- **AC-02**：VM 轨发送消息后，assistant 回复与工具 Block 无任何手动刷新
  ≤3s 自动上屏。验证：VM 实机冒烟（chat-streaming 契约② VM 臂）。
- **AC-03**：空闲态 vm log "[POLL]" 行 0 条/分钟；连续运行 1h poll_window
  长度有界（≤10）。验证：日志 grep + dbg 行。
- **AC-04**：产品内新会话 agent 技能列表含 auto-plan:new/work/review/merge，
  skill 工具能加载其 SKILL.md 内容。验证：会话 trace + 技能目录。
- **AC-05**：产品内请求「用 /auto-plan:new 建计划」端到端产出
  `docs/plans/NNN-*.md`（workspace 内真实文件）。验证：目录 + git/serve 双验。
- **AC-06**：AC-05 产物在计划栏目与文件栏目切换后即可见。验证：VM 实机截图。

## 8. 执行步骤

- **T-01** ContentHeader 会话 id 信息入口（P1）。依赖：无。
  文件：`src/front/content_header.at`（参照 `src/front/session_info.at`）。
  验证：§6 T-01 项。AC：AC-01。
- **T-02** PollStream 刷新链修复（P2）。依赖：无（可与 T-01 并行）。
  文件：`src/front/forge_store.at`（PollStream/PollBackfill/StartStream 域）。
  验证：§6 T-02 项 + 门禁。AC：AC-02, AC-03。
- **T-03** auto-plan 四技能产品自带（P3）。依赖：无（与 T-01/T-02 并行；
  auto-ai 子项走依赖 worktree）。
  文件：`backend/crates/musk/src/server.rs`（启动同步）、
  auto-ai `crates/auto-ai-agent/src/skill.at`（load-miss 重扫）。
  验证：§6 T-03 项。AC：AC-04。
- **T-04** 技能加载后轮次中断定罪与闭环（P4）。依赖：T-03（验收需正确技能
  存在；调查本身可先行）。文件：auto-ai agent loop（定罪后定）+
  aaid 日志打点。验证：§6 T-04 项。AC：AC-05, AC-06。

## 9. 复审记录

- 2026-09-27 draft r1（new handoff）：四缺陷证据链闭合（§4 实证记录），
  stage: new，PLAN-092 rev1。outcome: pass（T-01/T-02 可直接 work；
  T-03/T-04 含 auto-ai 跨仓改动，按 §4 授权规则走依赖 worktree）。
  next: work。

## 10. 待澄清事项

1. **auto-ai 跨仓改动的执行确认**（T-03 rescan、T-04 轮次修复）：是否随本
   计划同批执行（同组依赖 worktree、消费即折回）？还是拆分独立推进？
   默认按同批执行。
2. **P4 中断根因未定罪**：T-04 为有界调查任务（0.5 日上限），若三嫌疑均排除，
   以定罪结论回报并按新证据修订本计划（revision 语义变更走用户确认）。
3. **技能同步的覆盖策略**：用户 skills 目录已有同名技能时以 musk vendored
   为准覆盖（本计划默认），还是保留用户改动？默认覆盖（真源单一）。
