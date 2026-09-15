---
plan_id: PLAN-066
status: executing
feature_name: vm-stability-semantics-closure
author: [zhaop]
created_at: 2026-09-07T11:10:00+08:00
updated_at: 2026-09-15T14:10:00+08:00
plan_revision: 2

# Provisional — /auto-plan:review finalizes after verified implementation:
supersedes_spec_components: []
new_spec_components:
  - "SD-02: VM 进程稳定性（退出审计消费口径/长跑 harness/MCP 子进程 census 常驻资产） → modules/vm-process-stability.md"
  - "SD-03: VM 数据读侧契约（__json_object 值本体/Regex 回归锁/computed 投影/SET_FIELD 可达/ThinkBlock 读侧独立） → modules/vm-data-semantics.md（暂定目标，复审定稿）"
touched_goals:
  - "goal-frontend-parity: VM 轨语义族根修消费（问卷卡直读/computed 投影/chevron/Sse 零抛）——双轨等价最后一里"
  - "goal-agent: VM 进程稳定性（KD-048a 静默退出根修 + MCP 子进程回收）——agent 运行面可靠性"

current_step: 2
total_steps: 12
---

# [PLAN-066] VM 轨稳定性与语义收尾——KD-048a 静默退出/MCP 子进程定罪 + auto-lang 二期语义族根修 + musk 消费

## 变更摘要

集中清偿 KNOWN-DEBT 中 **VM 轨进程稳定性**（组一）与 **auto-lang 上游语义族**（组二）两类敞口，双仓联动（auto-lang 根修为主、musk 消费回撤为辅）：

**组一：进程稳定性（消费上游定罪前缘）**
1. **KD-048a VM 实例静默退出**（~4-5 分钟 exit 1 无 panic）：**rev2 重定义**——不再新建 instrumentation，消费上游既有退出审计（PLAN-575 钩子常驻）与 625 T-07 调查结论（WER AppHangB1：UI 线程停泵 >5s 后被外部击杀），按 **P625-D1** 候选面（MCP 快照序列化阻塞 UI 线程/心跳日志洪水）定罪根修，并排除 musk 侧端口候选（055-4③）。
2. **MCP snapshot 拉起 ~66MB 子 auto 进程不退 + ~43MB 瞬态子进程**（062 残留观察）：spawn 链定位 + 子进程回收。

**组二：语义族根修（auto-lang，059-T9 明示"归 auto-lang 二期家族"）**
3. **`__json_object` 字符串字段读污染**（057①，上游未动）：GET_FIELD 臂返值本体。
4. **Regex.match 恒空**（057②）：**rev2 裁剪**——上游 583（`dbde35d1f`，起草基线后）已修 Regex 容器负哨兵 retain（症状同族），先复跑 wl_probe18 裁定：绿则仅回撤 musk 绕行+转正+回归锁，红则残余根修。
5. **computed 读侧投影恒空 + 跨模块 SET_FIELD 不可达**（KD-057④ + P536-D2）：**rev2 对表对象更换**——auto-lang 在途 PLAN-624（store-facade-cross-state，needs_fix 修复中）同引擎子域，须 sequencing 后开工。
6. **ThinkBlock chevron computed 读侧**（057③，随 5 同根）。
7. **Sse.* VM no-op 对 handler-as-value 实参容错**（KD-059-FU1/493②）：**rev2 基线重列**——musk 侧复盘清单 = 8b1ae23 四修 + 067 T-02（`cd4f62e`）+ 069 F-04（`c9742a4`）叠加层逐项裁定。
8. **059-T9 五余项**：五项内容不变；**rev2 锚点改符号锚**（原行号已被上游 625–631 重写冲掉），hover 原语已在上游落地（631）可供目验利用。

## 目标

- KD-048a：拿到定罪证据（审计记录/WER/挂起期线程转储在案）并根修；修复后 3×10 分钟长跑零静默退出。
- MCP 会话结束后 auto 子进程普查归零（无 ~66MB/~43MB 滞留）。
- wl_probe21/18 探针转正入回归测试族；musk 侧 indexOf 绕开回撤、问卷卡 VM 轨渲染打通。
- 发送消息后画布投影即时入列（computed 投影恢复）、ThinkBlock chevron 独立翻转——VM 轨聊天主循环无"最后一里"断点。
- StartStream 在 VM 零 Field not found 抛点；forge_store 四修+067/069 叠加层逐项裁定保留/回撤。
- 059-T9 五项逐项有实机证据（截图或行为记录）。

非目标：G1 阶段2 SSE 桥泛化（047 勘察推荐长杆，另立项）；auto-lang 624 的 facade 解析修复本身（仅 sequencing 消费其结论）。

## 架构方案

上游优先（auto-lang sibling worktree 根修 → 合 master → musk 消费复跑），布局按 AGENTS.md：`.wt/musk-066/auto-musk`（分支 `plan-066-dev`）+ 同组并排 `.wt/musk-066/auto-lang`（分支 `auto-musk-dev`）。auto-lang 改动消费验证通过后尽快合回 auto-lang master 并清理，不留悬挂 worktree。

**sequencing（rev2 新增）**：T-06/T-07（P536-D2 面）须等 auto-lang PLAN-624 收口（其 needs_fix 修复+复审完成）后开工，或开工当日 `git log` 实对确认无撞文件后再进。

分派：
- **稳定性两件**在 auto-lang `ui/iced` runtime + `ui/mcp_server.rs`（autoui_snapshot 链）；musk 侧提供 repro harness（vm-first-run 长跑变体）与进程普查脚本。定罪消费上游既有审计（`AUTO_DESKTOP_EXIT_LOG`），不新建平行 instrumentation。
- **语义族**在 auto-lang `vm/ffi/stdlib.rs`（__json_object/Regex/Sse）、`ui/aura_view_builder.rs`（eval_computed/resolve_iterable 求值上下文）、`vm/dynamic.rs`（SET_FIELD 跨模块根态）、`ui/iced/renderer.rs`（fixed_both Image 居中，符号锚开工日重定位）。
- **musk 消费**：`src/front/forge_store.at`（8b1ae23 四修 + 067/069 叠加层复盘）、问卷卡 `questionnaireFor/stripQuestionnaire`（11b6c20 indexOf 绕开回撤）、`tmp/vmprobe/wl_probe21.at`/`wl_probe18.at`/`t3_filter.at` 转正。

## 技术栈

Rust（auto-lang vm/ui + musk backend）/ Auto（.at 前端源与探针）/ cargo-nextest（**门禁必须 nextest**——062 登记：裸 cargo test 进程内并行全局态互污 183 假红）/ vitest + vue-tsc / 实机 VM（iced release 二进制，051 注记：debug 构建踩 RC canary）/ 挂起期线程转储工具（procdump 或 wpr，P625-D1 指定手段；环境缺席时见待澄清①）。

## 需求分析与背景调查

**授权记录（rev2）**：2026-09-15 用户授权按复审 findings F-1~F-6 修订本计划出 rev2（status 保持 drafting），scope 沿袭原计划双仓授权（auto-lang 根修 + musk 消费）。rev1 = 2026-09-07 原起草版（彼时无 rev 字段，追溯编号）；rev2 变更面 = F-1~F-6 对应的 T1/T2 重定义、T5 前置复跑分支、T6 对表对象更换、T8 清单重列、T9 锚点改制、账本引用刷新；目标与验收阈值不变。

**现状锚点（2026-09-15 核对；rev1 所引 P0XX-N 账本项 ID 已随 065–070 账本重建失效，改挂现行结构）**：

musk 侧：
- `docs/plans/KNOWN-DEBT-AND-RISKS.md` 行 96（KD-493：048a 加剧样本 + 055-4③ 端口候选 8080→保留段 8068-8167 绑定失败超时 exit 1）、行 101（KD-055-4：⑥VM 过滤投影恒 0/④ThinkBlock 读侧，归 057）、行 102（KD-057：①__json_object 读污染 ②Regex 恒空 ③chevron ④投影+色差 ⑤hover 注记——⑤已过时，见下）、行 107（KD-062：残留观察=MCP 子进程两形态 + KD-048a 仍 ~4-5min 复现 + 门禁必须 nextest）。
- `docs/specs/modules/chat-streaming.md:40` KD-059-FU1 债务行（"VM 轨 Sse.open 抛点问题"）——SD-01 的 before 依据。
- `docs/specs/reports/048-sse-vm-survey.md`（Sse 桥勘察在案；G1 阶段2 泛化不在本计划）。
- 归档计划 `docs/plans/archived/`：055/057/059/062（各残项所属）。
- musk 绕行提交在祖先链：11b6c20（indexOf 绕开）、8b1ae23（PollStream 四修）；forge_store.at 叠加层：`cd4f62e`（067 T-02 叶链投影对齐+SSE 健康门）、`c9742a4`（069 F-04 流式期跳过回填，明注"VM 轨 Sse.open no-op 不受影响"）。

auto-lang 侧（master `a2458e13`，2026-09-15；rev1 基线 e64baccf5 已过时）：
- **P625-D1**（KNOWN-DEBT-AND-RISKS.md:2188）：VM UI 进程 AppHang 静默退出——WER AppHangB1 ×2 实证（UI 线程停泵 >5s 后被外部结束，exit 1/127 无 panic）；候选=MCP 快照序列化占 UI 线程/心跳日志洪水；根因定位指定挂起期线程转储（procdump/wpr）；缓解在案。625 T-07 已完结（调查任务，`218467f12` 归档）。
- **P536-D2**（同文件 :1447）：跨模块 store handler 帧内 SET_FIELD 重绑定不可达根态——T-06 的引擎级根因候选，仍敞口。
- **PLAN-624 在途**（store-facade-cross-state-resolution，execution_done→needs_fix 修复中，§9 2026-09-15 行）：合并单态跨状态字段解析（GET_FIELD 解析到错误 state 类型族），affects ui+vm——与 P536-D2 同子域，sequencing 见架构方案。
- **583 heap_rc 批**（`dbde35d1f`，2026-09-07）：Vec\<String\>/CSV/Regex 容器负哨兵缺子份额 retain 已修（元素 rc 从 0 起→首个消费者释放即 FREE→墓碑/静默空串）——与 wl_probe18"Regex.match 恒空"症状同族，T-05 前置复跑的依据。
- 631 F-5 mouse-area hover 样式对已落地（aura_view_builder hover 面 95 处）——KD-057⑤/055-4⑦"iced 无 hover"注记过时；T-09 scrim 目验与对拍可利用。
- 原对表对象 578/579 已完结（583 解 579 T9 阻断）；625/626/629/630/631 已落地（renderer.rs/aura_view_builder.rs 大量重写，rev1 行号锚全部失效）。

**T4/T8 上游前提核验**：stdlib.rs 自基线以来无 __json_object GET_FIELD 臂、无 Sse no-op 臂相关提交——两件根修仍归本计划。

代码锚点（rev2 改符号锚，行号以 work 开工日 `git grep` 重定位为准）：
- `auto-lang/crates/auto-lang/src/vm/ffi/stdlib.rs`：JSON 物化 `__json_object` 注释与构造点、Regex 函数族、Sse VM no-op 显式降级臂。
- `auto-lang/crates/auto-lang/src/ui/aura_view_builder.rs`：resolve_iterable/eval_computed。
- `auto-lang/crates/auto-lang/src/ui/iced/renderer.rs`：fixed_both 判定臂 + fixed_both 按钮内容臂。
- `auto-lang/crates/auto-lang/src/ui/mcp_server.rs` + `mcp_types.rs`：autoui_snapshot。
- `auto-lang` 退出审计：stdlib.rs exit_audit（575 T1）+ `AUTO_DESKTOP_EXIT_LOG` + `docs/reports/p575-exit-audit-ledger.jsonl`。
- musk 探针（在位核验 2026-09-15）：`tmp/vmprobe/wl_probe21.at`/`wl_probe18.at`/`t3_filter.at`；`scripts/vm-first-run.{mjs,cmd}`、`scripts/vm-link-probe.cmd`。

## 详细设计

### 组一：稳定性

**T-01 KD-048a 定罪（rev2 重定义）**：不新建平行 instrumentation。消费既有面：①复核 575 退出审计钩子覆盖（shim_process_exit/panic hook/main_return）对复现样本的记录情况；②musk `scripts/vm-first-run.mjs` 加长跑变体（保活 ≥5 分钟、退出码+审计文件收集）复现并取样；③对复现窗实施挂起期线程转储（procdump/wpr，P625-D1 指定）——验证/排除"MCP 快照序列化占 UI 线程→AppHang→外部击杀"主候选；④A/B 排除端口候选（055-4③）：`-B <非保留段端口>`/AUTO_HTTP_PORT 规避对照。产出：定罪证据落账（审计记录/WER 条目/转储栈/对照结论）。若证据实锤外部击杀链（575 降档出口），转向 T-02 的挂起根因面继续。

**T-02 根修 + 长跑（rev2 对准 P625-D1）**：按定罪修复——主候选面：autoui_snapshot 序列化让出 UI 线程（后台化/分片）、心跳失败日志限频（洪水排除干扰）、端口绑定失败显式报错而非静默超时退出（若 ④ 实锤）；上游 P625-D1 候选与 musk 055-4③ 候选一并收口。验证 = 长跑 3×10 分钟全部存活（或正常退出码），vm-first-run alive reds=0。

**T-03 MCP 子进程回收**：定位 `autoui_snapshot` spawn 链（mcp_server.rs），补子进程生命周期（等待回收/超时 kill/快照进程复用单例）；musk 侧加进程普查脚本（census：MCP 会话前后 `auto` 子进程计数）。验证：会话结束后 census 归零（66MB/43MB 两形态均不滞留）。

### 组二：语义族（上游根修 → musk 回撤）

**T-04 __json_object 字符串读污染**：stdlib.rs GET_FIELD 对 __json_object 臂的字符串字段返回类型名（wl_probe21：`j.kind → "str"`，int 字段正常）——修为值本体；wl_probe21 转正入 `musk_vm_track` 测试族。musk 消费：问卷卡 `questionnaireFor` 的 `json.type` 判定恢复直读（057① 最后堵点）。验证：probe 绿 + 问卷卡 VM 实机渲染。

**T-05 Regex.match（rev2 裁剪：前置复跑分支）**：**第一步复跑 wl_probe18**（上游 583 已修 Regex 容器 retain，症状同族）——分支 a（绿）：跳过根修，直接回撤 musk indexOf 绕开（`questionnaireFor/stripQuestionnaire`，11b6c20）+ wl_probe18 转正 + 补一条 Regex.match 语义回归锁（锁定 583 修复面）；分支 b（红）：残余根修（stdlib.rs Regex 族）后同验收。两分支共同验收：probe 绿 + 回撤后 `pnpm vitest run` 全绿。分支判定属 work 现场证据裁定，不需用户决策。

**T-06 computed 读侧 + 跨模块 SET_FIELD（state-scope 专项；rev2 sequencing）**：**前置**：确认 auto-lang PLAN-624 已收口（needs_fix 修复+复审完成）且当日 `git log` 对表无撞；若 624 修复已覆盖 SET_FIELD 重绑定面，本任务裁剪为纯 computed 读侧。主体：aura_view_builder eval_computed 求值上下文接 store computed 表（055-4⑥ 函数层同形探针全绿、断点在求值上下文——同 047① resolve_iterable 先例修法）+ P536-D2 跨模块 SET_FIELD 不可达根态（`done` 臂自清依赖此，见 forge_store.at PLAN-062 注释）。musk 消费：画布投影恢复（过滤投影计数>0、发送消息气泡即时入列）；t3_filter 探针转正。

**T-07 ThinkBlock chevron 读侧**：与 T-06 同根（子件 if/computed 求值分歧——写侧 think_open 键实机验证 `["bsmsg-a1"]` 已通）。验证：实机展开/收起独立翻转，▼ 不再共享末值/双内容。

**T-08 Sse no-op 容错 + 绕行层复盘（rev2 清单重列）**：上游根修不变——stdlib.rs Sse.* 显式降级路径对 handler-as-value 实参（`Sse.open(.OnStreamEvent)`）不再抛 Field not found（实参解析容错或惰性求值）。musk 侧复盘清单对当前 HEAD 重列（不再是 8b1ae23 孤立四修）：①streaming 置位前移 ②头部直连 close ③回合增长守卫 ④when 门摘除+deadman 2 分钟窗（8b1ae23）+ ⑤叶链投影对齐与 SSE 健康门（067 `cd4f62e`）+ ⑥流式期跳过回填（069 `c9742a4`）——逐项裁定保留/回撤；**deadman 窗立场在新基线重估**（067/069 已改写流式门控语义，原"常驻兜底"建议须重新验证后维持或摘除）。验证：StartStream 在 VM 零抛点 + 发送链路 E2E 不回归（多轮 + 流式中断恢复）。

**T-09 059-T9 五余项（rev2 锚点改符号锚）**（auto-lang iced/minted 面，逐项上游修 + musk 实机验证；行号开工日重定位）：
1. fixed_both 钮内 Image 不居中（renderer.rs fixed_both 判定臂 + 按钮内容臂对 Image 未生效；实机对照 tmp/header-zoom.png）；
2. 浮层族（alert-dialog/dialog/dropdown-menu）trigger 锚件在场才绘浮层——空 trigger 退化流内（已实证），补"无 trigger 形态面板仍绘"；
3. 受控 open（显式绑定自管）形态补 ESC/外点自动关闭（web reka 原生行为的最小对齐面，不泛化全部 minted 形态）；
4. scrim 暗幕实机目验（暗色主题下歧义，专项亮/暗双主题截图；可利用 631 hover 原语辅助态验证）；
5. 弹层 chrome 固定宽度档 → 宽度 prop 透传。

**T-10 musk 门禁全量**：`scripts/vm-first-run.{mjs,cmd}`（alive reds=0，含 T-01 长跑变体）/ `scripts/vm-link-probe.cmd` / `cd gen/front/vue && pnpm build && pnpm vitest run` / `cargo nextest run -p musk`（**必须 nextest**）/ 对拍 30/30 / style-parity 基线差分恒等。

**T-11 双仓收尾**：auto-lang `auto-musk-dev` 合回 master（消费验证通过即合，不留悬挂 worktree）→ musk 主检出复跑门禁 → worktree/分支/组目录清理。

### 规范增量

| delta_id | 操作 | 目标（docs/specs/...） | before/after 规则 | 理由 | 验收 |
|---|---|---|---|---|---|
| SD-01 | modify | modules/chat-streaming.md | before：`:40` "历史债务：KD 059-FU1……VM 轨 Sse.open 抛点问题"（musk 绕行维持）；after：VM 轨 Sse.* no-op 面对 handler-as-value 实参容错为零抛契约，附 musk 绕行层（四修+067/069 叠加）逐项裁定结论与 deadman 立场 | T-08 根修落地后债务行升级为 enduring 契约行 | AC-05 |
| SD-02 | add | modules/vm-process-stability.md | 新增：退出审计消费口径（AUTO_DESKTOP_EXIT_LOG 钩子面 + 降档出口语义）、静默退出定罪/根修决策记录（P625-D1 候选裁定）、长跑 harness 与 MCP 子进程 census 作为常驻验证资产 | KD-048a/P625-D1 清偿后稳定性知识需常驻落点；脚本入库 scripts/ 与之配套 | AC-01, AC-02 |
| SD-03 | add | modules/vm-data-semantics.md（暂定目标，复审定稿） | 新增：VM 轨数据读侧契约——__json_object 字段读返值本体、Regex.match 语义回归锁、computed 读侧投影可达、store 跨模块 SET_FIELD 可达、ThinkBlock 读侧独立翻转 | T-04~T-07 根修落地后的双轨等价契约收口 | AC-03, AC-04 |

SD 目标为暂填：review 按已验证实现定稿（含 SD-03 最终挂载文件与 index.json 挂载关系）。

## 测试设计

- 上游每根修配最小复现单测（探针 .at → auto-lang `musk_vm_track_p0XX` 测试族命名惯例）；wl_probe21/18、t3_filter 转正；T-05 分支 a 须补 Regex.match 回归锁（锁定 583 修复面）。
- musk 回撤项以既有 vitest 基线全绿为准（36+1skip 口径，以复跑为准）。
- 稳定性：长跑 harness（T-01/T-02）+ census 脚本（T-03）作为可复跑资产入库 `scripts/`；定罪证据（审计 jsonl/WER/转储栈）归档 `docs/plans/attachments/` 或 `docs/reports/`。
- 实机验证项（问卷卡/画布投影/chevron/五余项）以截图或行为记录入 `docs/plans/attachments/`。
- 全程门禁用 cargo-nextest（062 登记），禁裸 cargo test 判红。

## 验收标准

- **AC-01** KD-048a 有定罪证据（审计记录/WER/线程转储至少其一在案，且明确裁定主候选与端口候选）且修复后 3×10 分钟长跑零静默退出。验证：证据文件在案 + 长跑 harness 三连跑记录。
- **AC-02** MCP 会话后 auto 子进程 census 归零。验证：census 脚本会话前后计数输出。
- **AC-03** wl_probe21/wl_probe18/t3_filter 转正且全绿；musk indexOf 绕开回撤后 vitest 全绿。验证：转正测试族绿 + `pnpm vitest run` 基线一致。
- **AC-04** 实机：发送消息气泡即时入列（computed 投影）、过滤计数正确、问卷卡渲染、ThinkBlock chevron 独立翻转。验证：截图/行为记录入 attachments。
- **AC-05** StartStream 在 VM 零 Field not found；四修+067/069 叠加层复盘结论（保留/回撤）逐项记录。验证：E2E 发送链记录 + 复盘裁定表入计划或 SD-01。
- **AC-06** 059-T9 五项逐项实机证据在案。验证：attachments 截图/记录。
- **AC-07** musk 门禁全绿（vm-first-run/probe/build/vitest/nextest/对拍/style-parity 基线）+ auto-lang lib 测试绿。验证：门禁命令输出留痕。

## 执行步骤

- [ ] T-01 KD-048a 定罪：575 审计钩子复核 ✓（三挂点在案：shim_process_exit/panic hook/main_return @ renderer.rs:16997）+ vm-first-run-soak.mjs 长跑取证 harness 入库（端到端小参实跑链路通）⏸ 实跑取证被 F-W1 VM 启动断裂阻塞（转储工具仍待澄清①）
- [ ] T-02 按 P625-D1 候选面根修（快照序列化让出 UI 线程/日志限频/端口显式报错按定罪取用）；验证 3×10min 长跑零静默退出 + vm-first-run alive reds=0（AC-01）⏸ 待 T-01
- [ ] T-03 MCP 子进程回收：auto-lang `ui/mcp_server.rs` autoui_snapshot spawn 链 + musk census 脚本；验证会话后子进程归零（AC-02）
- [ ] T-04 __json_object 字符串读根修：✅ 上游根修落地（见 rev2 工作记录 W-1：`.type` 属性抢占收窄，非 stdlib 臂缺陷）+ wl_probe21 全形态转正 musk_vm_track p066 测试族 4/4 绿 + tv 3711/3711；musk 侧零代码变更（questionnaireFor 的 json.type 直读本就在位）；⏸ 实机问卷卡渲染验证被 F-W1 阻塞（🔶 待用户窗口+VM 启动修复）
- [x] T-05 Regex：前置复跑 wl_probe18 裁定**分支 b（红，计数脸未修）**→ 残余根修落地 [✅ 2026-09-15：真根=shim_regex_match 为 is_match 1/0 语义且弹参错位（非 583 retain 脸）→ JS web 语义统一三参契约+编译期补参，auto-lang `c9e6e4737`；musk 回撤 11b6c20 两函数恢复 Regex 通道 `8536ff4`；p066_2 四测绿（wl_probe18 全形态双脸+组提取+元素存活 583 锁）+tv 3715/3715+auto build 绿+vitest 36+1skip 基线一致]（AC-03）
- [ ] T-06 state-scope 专项（前置：624 收口对表——✅ 624 已复审 pass 2026-09-15，待合并）：eval_computed 上下文 + P536-D2 SET_FIELD（624 已覆盖则裁剪）+ musk t3_filter 转正；验证画布投影实机即时入列（AC-04）
- [ ] T-07 ThinkBlock chevron 读侧（随 T-06 同根验收）；验证实机独立翻转（AC-04）
- [ ] T-08 Sse no-op 容错 + 绕行层复盘（8b1ae23 四修 + 067/069 叠加层逐项裁定，deadman 立场重估）；验证 StartStream VM 零抛 + 发送链 E2E 不回归（AC-05）⏸ E2E 面受 F-W1 影响
- [ ] T-09 059-T9 五余项逐项修 + 实机验证（fixed_both Image/trigger 锚件/受控 open ESC+外点/scrim 双主题/宽度 prop，符号锚重定位）；验证五项证据在案（AC-06）
- [ ] T-10 musk 全量门禁：vm-first-run（含长跑变体）/vm-link-probe/pnpm build+vitest/cargo nextest/对拍 30/30/style-parity 基线；验证全绿（AC-07）
- [ ] T-11 双仓收尾：auto-lang 合回 master + musk 主检出复跑 + worktree/分支/组目录清理；验证 wt-guard clean
- [x] T-12 VM 启动链修复（F-W1，2026-09-15 用户裁定修在 066 内）：上游 handler 合成对 composable（useI18n 声明/refs 绑定）与 web 全局（document）的 VM 降级 **[✅ 2026-09-15 收口]**：①document 降级 walker + mention_detect let→var（auto-lang `ad6dd76e2` + musk `bd0c384`，毒化 4→3，link failed 解除）→ ②i18n 路由收口（auto-lang `cb39769f7`）：receiver 编译位置钉死=native 发射前第二处 is_static_method 白名单（codegen.rs:9067）漏 `"i18n"`——静态分支路由 auto.i18n.t 与 id2460 惰性解析本已生效，交接所猜 func_name 降级形态不成立；补白名单后 p066_3 回归锁绿 + tv 3716/3716 + **vm-first-run alive=yes reds=0（codegen=0 link=0，observe 20s）**。非 ui 构建运行期 MissingNative(2460)=shim 随 ui 裁剪预期面。（AC-01 前置/T-04·T-08 实机面前置——实机目验仍 🔶 待用户）

依赖：T-02←T-01；T-06/T-07←624 收口；T-08 musk 复盘←上游根修；T-10←T-01..T-09；T-11←T-10。

## 复审记录

**2026-09-15 | stage: review | plan_id: PLAN-066 | plan_revision: 起草版（frontmatter 无 rev 字段，记 rev1-draft）| outcome: needs_replan**

- reviewed_commit: musk main `78c0af9`（0/11 未开工，无实现提交；本审为**契约漂移审查**——起草后 063/065/067–070 落地、auto-lang master 推进至 `a2458e13`，起草基线 `e64baccf5` 已过时）
- base_commit: 无（未建 worktree/分支）；dependency_revisions: auto-lang master `a2458e13`
- spec_inputs: musk `docs/specs/index.json` + modules 6 件（065–070 重建后）/ `docs/specs/reports/048-sse-vm-survey.md`（在位）/ KNOWN-DEBT 行 96·101·107；auto-lang KNOWN-DEBT P625-D1·P536-D2、`docs/plans/archive/625-ui-gallery-vm-usability.md` T-07、`docs/plans/624-store-facade-cross-state-resolution.md`
- acceptance_results: **N/A**——AC1–7 均未达可验证态，本次不产生 AC 判定；spec 组件三字段保持空（无实证实现，不预填规范增量）
- findings:
  - **F-1（high，T1/T2）静默退出前缘已被上游推进**：auto-lang 已有常驻退出审计（PLAN-575：shim_process_exit/panic hook/main_return + `AUTO_DESKTOP_EXIT_LOG` + p575-exit-audit-ledger.jsonl）；625 T-07 调查完结（WER AppHangB1 ×2 定性=UI 线程停泵 >5s 后被外部击杀，exit 1/127 无 panic）并以 **P625-D1** 立案（候选=MCP 快照序列化阻塞 UI 线程/心跳日志洪水；根修待挂起期线程转储）。musk 侧另有 055-4③ 端口候选（0.0.0.0:8080 落 Windows 保留段→绑定失败等待超时 exit 1 无 panic）。T1 拟建 AUTO_EXIT_TRACE 与既有审计重复；T2 应改消费 P625-D1 前缘 + 排除端口候选；验收（3×10min 长跑零静默）不变。
  - **F-2（medium，T5）前提疑被上游部分清偿**：583 heap_rc 批 `dbde35d1f`（2026-09-07，基线后）已修 Vec\<String\>/CSV/Regex 容器负哨兵缺子份额 retain（症状=元素墓碑/静默空串，与 wl_probe18"match 恒空"同族）。T5 须先复跑 wl_probe18：绿则缩为"11b6c20 回撤+转正+回归锁"；红则残余根修继续。
  - **F-3（high，T6）在途撞车**：auto-lang PLAN-624（execution_done→needs_fix 修复中，§9 2026-09-15 行，affects ui+vm）与 P536-D2 同引擎子域（跨状态字段解析）。待澄清#2 的对表对象 578/579 已完结失效，应改对表 624 并 sequencing（624 收口后再开 T6）。
  - **F-4（medium，T8）基线漂移**：forge_store.at 已叠 067 T-02（`cd4f62e` 叶链投影对齐+SSE 健康门）与 069 F-04（`c9742a4` 流式期跳过回填；明注"VM 轨 Sse.open no-op 不受影响"）。四修复盘须对当前 HEAD 重列清单；上游 Sse no-op 臂基线后无提交，根修前提成立。
  - **F-5（low，T9）锚点漂移**：renderer.rs/aura_view_builder.rs 被 625/626/629/630/631 重写，行号锚 3339/3569 失效（work 重锚）；五余项无被修证据仍归本计划。附带：KD-057⑤/055-4⑦"iced 无 hover"注记过时（631 F-5 hover 样式对落地）。
  - **F-6（low，需求分析）账本引用失效**：所引 P0XX-N 项 ID 在现行 docs/specs 不复存在；实质锚点（归档计划/KNOWN-DEBT 行/048 勘察报告）均在，修订时改挂现行结构。
  - 前提完好在案：T3（MCP 子进程）无上游动作证据仍敞口；T4（json GET_FIELD 臂）上游未动。探针与脚本资产核验在位（tmp/vmprobe 三探针 + scripts/vm-first-run.{mjs,cmd}/vm-link-probe.cmd）。
- evidence: auto-lang `dbde35d1f`/`88803e84e`/`218467f12`/`92b0fd984`；musk `c9742a4`/`cd4f62e`/`933d82d`；两仓 KNOWN-DEBT 行文如上（路径可解析，worktree 移除后仍可溯）
- next: ~~`/auto-plan:new` 修订 PLAN-066（rev2）~~ → **已执行，见下行 rev2 handoff**

**2026-09-15 | stage: new | plan_id: PLAN-066 | plan_revision: 2 | outcome: pass**

- 修订性质：rev1（2026-09-07 起草版，追溯编号）→ rev2，按复审 F-1~F-6 全数落实：T-01/T-02 重定义为消费 P625-D1 前缘（F-1）；T-05 前置复跑分支+回归锁（F-2）；T-06 对表对象换 624 + sequencing（F-3）；T-08 清单重列含 067/069 叠加层、deadman 立场重估（F-4）；T-09 符号锚+hover 注记刷新（F-5）；需求分析改挂现行账本结构（F-6）。目标与验收阈值不变，AC 编号化为 AC-01..07（原 1–7 语义保持）。规范增量表（SD-01..03）暂填，review 定稿。
- 授权：用户 2026-09-15 授权本次修订（status 保持 drafting）；scope 沿袭原双仓授权。
- changed IDs：T-01/T-02/T-05/T-06/T-08/T-09 语义变更，T-03/T-04/T-07/T-10/T-11 沿袭重编号；AC-01..07 编号化。
- next: **work**（624 收口为 T-06/T-07 前置，非整计划阻塞——T-01..T-05/T-08..T-11 可先行）。

**2026-09-15 | stage: work | plan_id: PLAN-066 | plan_revision: 2 | outcome: blocked（保持 executing）**

- code_commit: auto-lang `auto-musk-dev` **`7a1f42dfc`**（T-04 上游根修）；musk `plan-066-dev` **`536156b`**（T-01 soak harness）
- task_ids: T-04（上游✅/实机⏸）、T-01（部分✅/实跑⏸）；其余未动
- W-1（T-04 根修，rev2 设计修正一处定性）：KD-057① 的"stdlib.rs GET_FIELD __json_object 臂返类型名"定性**不成立**——红相测试实证普通字符串字段读（j.kind→"x"）已在位，真病灶=**codegen Dot 臂对 field=="type" 无条件抢占为编译期型名 LOAD_STR**（`vm/codegen.rs:6312`，Plan 087 Phase 3 内建 typeof 属性）：JSON.parse 推断哨兵 StrFixed(0) → `j.type=="str"`，字面量 obj 同型。修=typeof 仅原始接收者生效（StrFixed(0) 哨兵一并排除，真实 str 字面量推断 StrFixed(len) 不受累），对象接收者落 GET_FIELD 通道与 web 轨 a2ts 同语义；musk ~90 处 `.type` 字段读（forge_store ev.type 分派/questionnaire q.type/seg.type 等）随之解锁。证据：p066 测试族红→绿 4/4；tv **3711/3711 绿**（491 skip 基线）；M3 语料 Rust 参考侧 t01/t03/t04/t05/t06 输出 int/float/str/bool/char 逐项不变；auto-os 侧普查仅 bar_chart 3 处字段读（受益方，无 typeof-on-object 依赖）。
- T-01 部分：575 退出审计三挂点复核在案（stdlib.rs:723-775 + renderer.rs:16997）；`scripts/vm-first-run-soak.mjs` 入库（审计 env 注入+575 判读矩阵+code-3 不误判），小参实跑（1×15s）端到端链路通。
- **F-W1（high，阻塞 VM 实机验证面）当前 main 的 VM App 启动即断裂**：`auto run --render=vm` 于 handler 合成期报 `Undefined variable: i18n`（wiki_nav.at:29 `useI18n()`、mention_input 计算域）与 `Undefined variable: document` → mention_detect/WikiNav_dropText/MentionInput 标签等 4 导出 poisoning → `link failed for 'App'` → 启动 exit 1。定性=**PLAN-063 i18n 全量化未消费 VM 轨**（composable+web 全局在 VM handler 合成不识别；KD-050 C7 期的 t()+i18n_lookup 通道是旧用法形态）。影响：T-01 实跑取证/T-04 实机渲染/T-08 E2E 全部前置受阻；且启动 exit-1 形态会污染 KD-048a 定罪观察（须先修复再做长跑）。证据：tmp/plan047-firstrun.log（worktree，reds=5 codegen=4 link=1）。
- evidence: 上文双 commit；p066 测试族 `crates/auto-lang/src/musk_vm_track_tests.rs`（musk_vm_track_p066_1_json_string_read）；soak summary `tmp/plan066-soak-summary.json`（worktree）
- blockers: ①F-W1 处置决策——在 066 内增补任务（上游 handler 合成补 composable/web-global 降级 stub，或 musk 侧 i18n VM 通道激活）vs 另立计划（倾向后者：独立缺口、修面在上游合成层，066 契约不含）；②待澄清① 转储工具仍待用户
- next: 用户裁定 F-W1 归属 → T-05 可先行（wl_probe18 进程内复跑不依赖 VM App 启动）

**2026-09-15（续）| stage: work | plan_id: PLAN-066 | plan_revision: 2 | outcome: pass（T-05 单元收口，整体仍 executing）**

- code_commit: auto-lang `auto-musk-dev` **`c9e6e4737`**（T-05 上游根修）；musk `plan-066-dev` **`8536ff4`**（11b6c20 回撤）
- task_ids: T-05 ✅（current_step 1/11）
- 分支裁定：wl_probe18 复跑=**红（分支 b）**——`Regex.match("a1b2c3","[0-9]","g")` 计数 0，583 修的是元素墓碑脸（matches[0] 静默空串），计数脸另根。
- W-2（T-05 根修定性）：wl_probe18"恒空"真根=`shim_regex_match` 为 **is_match 1/0 语义**且弹参错位（三参调用首弹实收 flags "g" 当 text、对 "g" 测匹配恒 0，残参死区回收掩盖失衡）。修=统一三参契约 `[flags, pattern, text]`（自顶向下）：无 'g' → `[全匹配,组1,..]` 堆列表（JS 非 global，musk colonMatch[1]/p0[1] 组提取解锁）；含 'g' → 全部匹配子串列表（围栏提取）；空匹配 → 空列表（JS null 对齐，`length>0` 守卫双轨同真值）；两参调用 codegen 编译期补 `flags=""`（057 T6 JSON.stringify 补参先例；静态 Regex.* 经 TYPE_CANONICAL 解析不进 P240 小写 map，补参置于通用调用发射点）；'i' modifier 支持；fresh 列表 rc_push(+1) 与 Plan 419 配平。
- 回撤面：questionnaireFor 路径 1 + stripQuestionnaire 恢复 Regex 通道（两文件 11b6c20 后另有演进，手工精准回撤非 checkout）。
- 验证：p066_2 四测绿（wl_probe18 全形态计数+元素内容双脸 / strip match→replace 通道 / 两参组提取锁 / 元素存活 583 锁）；tv **3715/3715**（含新增四测）；musk auto build 绿 + vitest **36+1skip 基线一致**（vitest 2.1.9 沿 050 锁版会话级补装）。
- evidence: p066_2 测试族 `musk_vm_track_tests.rs::musk_vm_track_p066_2_regex_fence`；回撤 diff=双 commit。
- blockers: 同前（F-W1 归属裁定 + 转储工具待澄清①）——T-05 不受影响已收口。
- next: work 续 T-03（MCP 子进程回收，不依赖 VM App 启动的部分先行）或待 F-W1 裁定后 T-01 实跑取证。

**2026-09-15（续二）| stage: work | plan_id: PLAN-066 | plan_revision: 2 | outcome: blocked（T-12 留点，保持 executing）**

- 授权落账：F-W1 修在 066 内（T-12 增补，total_steps 12）；procdump/wpr 批准引入（已落 `D:/autostack/tools/procdump64.exe` v12.01，仓外不入库）。
- code_commit: auto-lang `ad6dd76e2`（T-12 1/2：document 降级 walker + i18n.t 原生 id2460 + 路由尝试与诊断插桩）；musk `bd0c384`（mention_detect let→var）。
- W-3（T-12 进展与留点）：F-W1 从「App 启动即退」降为「3 个 i18n computed 导出缺失（功能面损伤，进程存活）」——link failed 解除（link=0，vm-first-run exit 1→0 观察期存活）。已验证生效：①document.<x>→Expr::None 合成改写 walker（import fn/computed/handler 三路径挂接）；②mention_detect el/val let→var（VM let 不可变纪律，两连错 document→el→val 逐层剥出）。**留点**：i18n.t 路由未通——诊断插桩（vm_debug static-check）实证静态分支已进入（is_local_var=false+is_stdlib_module=true），但内层 `let func_name = Some(format!("i18n.t"))` 按名解析失败后仍跌回实例编译；auto.i18n.t（id2460+shim 接 i18n_lookup）与 P240/static-exit 两处路由尝试已在位未生效。**下一手**：在 codegen Ident 臂 Undefined variable 抛点（codegen.rs:6246）加 expr 归属诊断，或读 static 分支后 inner func_name 的完整消费流（静态 TYPE.method 发射通道）钉死 receiver 编译位置。
- blockers: T-12 i18n 路由收口（单一留点）；待澄清①已解除。
- next: work 续 T-12 收口（上下文预算耗尽，交接下一会话；全部证据与插桩在码）。

**2026-09-15（续三）| stage: work | plan_id: PLAN-066 | plan_revision: 2 | outcome: pass（T-12 收口，整体仍 executing）**

- code_commit: auto-lang `auto-musk-dev` **`cb39769f7`**（T-12 2/2 收口）；musk 侧本单元零代码变更（`bd0c384` 为 1/2 已提交件）
- task_ids: T-12 ✅（current_step 2/12）；T-01/T-02/T-03/T-06..T-11 未动
- W-4（T-12 收口定性，修正交接猜想）：receiver 编译实际位置=**native 命中后的第二处 `is_static_method` 判定**（codegen.rs:9067 白名单）——`i18n` 补入后与第一处 static-check（:7887 已含 i18n）对齐。链路实证：静态分支路由产出 `auto.i18n.t` → `resolve_qualified` 经 NATIVE_ID_MAP 惰性注册命中 id2460（ui/非 ui 皆然）→ 白名单漏项使 `compile_expr(Ident("i18n"))` 仍执行→Undefined variable。交接记录所猜"内层 func_name=`format!("i18n.t")` 按名解析失败"形态不成立（那是实例兜底臂的产物，非实际路径）。非 ui 构建新行为=编译过、运行期 `MissingNative(2460)`（shim_i18n_t 随 ui-iced 裁剪，设计内降级面；p066_3 实测）。
- 验证：musk_vm_track p066_3 回归锁绿（锚 Undefined variable 毒化消除，Err 分支断言）；tv **3716/3716 绿**（3715+新锁，491 skip 基线）；musk vm-first-run **alive=yes reds=0**（codegen=0 link=0 io=0，observe 20s，`release/auto.exe` 2m56s 重建后实跑）。
- evidence: worktree 内插桩复核（vm_debug static-check + MissingNative 探针 `--nocapture` 输出）；firstrun summary 行 `observe_ms=20013 alive=yes … reds=0`。
- blockers: 无（待澄清①已解除；实机目验类仍待用户窗口，属 T-04/T-08 验收面）。
- next: T-01 实跑取证（3×10min soak + procdump 挂起期转储；VM 启动链已通，F-W1 污染源解除）→ T-02 根修。

## 待澄清事项

1. **T-01 转储工具引入**：~~请用户裁定~~ **✅ 2026-09-15 用户批准引入 procdump/wpr**（P625-D1 指定手段；工具落 `D:/autostack/tools/` 仓外，不入库）。若证据实锤外部击杀链，按 575 降档出口处理（审计常驻，一轮真实复现重启归因）。
2. **F-W1 归属（已裁定 2026-09-15）**：~~在 066 内增补 vs 另立计划~~ → **修在 066 内（T-12）**。
3. **T-06 与 PLAN-624 对表（rev2 更换对象）**：624 已复审 pass（2026-09-15，待合并）；开工前确认其折叠状态并 `git log` 对表。若 624 修复已覆盖 P536-D2 的 SET_FIELD 重绑定面，T-06 裁剪为纯 computed 读侧——裁定记录入计划。
4. **T-08 绕行层逐项裁定（rev2 清单扩展）**：deadman 窗立场在新基线（067/069 后）重估；四修+叠加层共六项逐项保留/回撤并注记理由。
5. **G1 阶段2 SSE 桥泛化不在本计划**（047 勘察报告推荐路线的长杆项），T-08 只修 no-op 抛点不实现真 SSE。
6. 实机验证依赖用户可用性（VM 目验/截图）；窗口期阻塞时该项记 🔶 待用户并继续后续任务。
