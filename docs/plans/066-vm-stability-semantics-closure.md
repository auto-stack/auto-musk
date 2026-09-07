---
plan_id: PLAN-066
status: drafting
feature_name: vm-stability-semantics-closure
author: [zhaop]
created_at: 2026-09-07T11:10:00+08:00
updated_at: 2026-09-07T11:10:00+08:00

# Leave these EMPTY — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 11
---

# [PLAN-066] VM 轨稳定性与语义收尾——KD-048a 静默退出/MCP 子进程定罪 + auto-lang 二期语义族根修 + musk 消费

## 变更摘要

集中清偿 KNOWN-DEBT 中 **VM 轨进程稳定性**（组一）与 **auto-lang 上游语义族**（组二）两类敞口，双仓联动（auto-lang 根修为主、musk 消费回撤为辅）：

**组一：进程稳定性（实机定罪驱动）**
1. **KD-048a VM 实例静默退出**（~4-5 分钟 exit 1 无 panic；048 OPEN-a 首登、055-4④/062 多次加剧样本，062 修复版同形复现）：instrumentation 定罪 → 根修 → 长跑验证。
2. **MCP snapshot 拉起 ~66MB 子 auto 进程不退 + 多个 ~43MB 瞬态子进程**（062 残留观察）：spawn 链定位 + 子进程回收。

**组二：语义族根修（auto-lang，059-T9 明示"归 auto-lang 二期家族"）**
3. **`__json_object` 字符串字段读污染**（057①：JSON.parse 产物字符串字段读取返回类型名 `"str"`——问卷卡 VM 渲染最后堵点）。
4. **Regex.match 恒空**（057②：musk 已 indexOf 绕开，上游根修欠账 + musk 回撤绕行）。
5. **computed 读侧投影恒空 + 跨模块 SET_FIELD 不可达**（KD-057④ + auto-lang KD P536-D2 state-scope 专项——493 行 09-05 更新的"残余最后一里=画布投影"）。
6. **ThinkBlock chevron computed 读侧**（055-4④/057③：写侧 think_open 键全链通，读侧共享末值/双内容）。
7. **Sse.* VM no-op 对 handler-as-value 实参容错**（493②：`Sse.open(.OnStreamEvent)` 在 VM 抛 Field not found 杀 PollStream 门控；musk forge_store 8b1ae23 四修绕行复盘）。
8. **059-T9 五余项**：fixed_both 钮内 Image 不居中 / 浮层族 trigger 锚件在场 / 受控 open 无 ESC+外点 / scrim 暗幕目验 / 弹层宽度 prop 透传。

## 目标

- KD-048a：拿到退出点定罪证据并根修；修复后 3×10 分钟长跑零静默退出。
- MCP 会话结束后 auto 子进程普查归零（无 ~66MB/~43MB 滞留）。
- wl_probe21/18 探针转正入回归测试族；musk 侧 indexOf 绕开回撤、问卷卡 VM 轨渲染打通。
- 发送消息后画布投影即时入列（computed 投影恢复）、ThinkBlock chevron 独立翻转——VM 轨聊天主循环无"最后一里"断点。
- StartStream 在 VM 零 Field not found 抛点；forge_store 四修绕行逐项裁定保留/回撤。
- 059-T9 五项逐项有实机证据（截图或行为记录）。

## 架构方案

上游优先（auto-lang sibling worktree 根修 → 合 master → musk 消费复跑），布局按 AGENTS.md：`.wt/musk-066/auto-musk`（分支 `plan-066-dev`）+ 同组并排 `.wt/musk-066/auto-lang`（分支 `auto-musk-dev`）。auto-lang 改动消费验证通过后尽快合回 auto-lang master 并清理，不留悬挂 worktree。

分派：
- **稳定性两件**在 auto-lang `ui/iced` runtime + `ui/mcp_server.rs`（autoui_snapshot 链）；musk 侧提供 repro harness（vm-first-run 长跑变体）与进程普查脚本。
- **语义族**在 auto-lang `vm/ffi/stdlib.rs`（__json_object/Regex/Sse）、`ui/aura_view_builder.rs`（eval_computed/resolve_iterable 求值上下文）、`vm/dynamic.rs`（SET_FIELD 跨模块根态）、`ui/iced/renderer.rs:3339/3569`（fixed_both Image 居中）。
- **musk 消费**：`src/front/forge_store.at`（8b1ae23 四修复盘）、问卷卡 `questionnaireFor/stripQuestionnaire`（11b6c20 indexOf 绕开回撤）、`tmp/vmprobe/wl_probe21.at`/`wl_probe18.at` 转正。

## 技术栈

Rust（auto-lang vm/ui + musk backend）/ Auto（.at 前端源与探针）/ cargo-nextest（**门禁必须 nextest**——062 登记：裸 cargo test 进程内并行全局态互污 183 假红）/ vitest + vue-tsc / 实机 VM（iced release 二进制，051 注记：debug 构建踩 RC canary，日常用 release）。

## 需求分析与背景调查

specs 账本相关项（musk 侧）：
- **P062-2**（VM 内存泄漏修复）：残留观察项两条（MCP 子进程、KD-048a）即本计划组一。
- **P057-2**（VM/TS 语义等价性根修）：__json_object 读污染、Regex.match 为其 T11 残差。
- **P055-2 / P051-2**（聊天对拍/会话主界面闭环）：computed 投影族、ThinkBlock 读侧的所属目标。
- **P059-2**（VM 悬浮层基础设施与 overlay 组件族）：T9 五余项所属。
- **P048-2**（VM 数据桥 + SSE 专项勘察）：KD-048a 首登行；Sse 桥勘察报告 `docs/specs/reports/048-sse-vm-survey.md`（G1 阶段2 SSE 桥泛化**不在本计划**，长杆另立项）。
- **P053-2**（vm-upstream-tracking）：上游移交惯例。

auto-lang 侧现状（2026-09-07 核对）：master e64baccf5；在途 578/579 均为 app 层计划，与 vm/ui 语义面无撞文件预期；`.wt/` 现存组（auto-010/011、auto-down-053、auto-os-013、lang-541/566）无 musk-066 冲突。auto-lang 有自己的 KNOWN-DEBT 登记面，上游新债随批登记。

代码锚点（已核对存在）：
- `auto-lang/crates/auto-lang/src/vm/ffi/stdlib.rs:2523`（JSON 物化为 __json_object 注释）、`:2740/:2759/:2822`（__json_object 构造点）、`:6149+`（Regex 函数族）、`:8788+`（Sse VM no-op 显式降级）。
- `auto-lang/crates/auto-lang/src/ui/aura_view_builder.rs`（resolve_iterable/eval_computed——047① 同文件两处缺口先例）。
- `auto-lang/crates/auto-lang/src/ui/iced/renderer.rs:3339`（fixed_both 判定）/`:3569`（fixed_both 按钮内容臂）。
- `auto-lang/crates/auto-lang/src/ui/mcp_server.rs` + `mcp_types.rs`（autoui_snapshot）。
- musk 探针：`tmp/vmprobe/wl_probe21.at`（json 读污染）、`wl_probe18.at`（Regex）、`t3_filter.at`（computed 投影）。

## 详细设计

### 组一：稳定性

**T1 KD-048a 定罪**：auto-lang iced runtime 加退出追踪 instrumentation——panic hook 已有则补 `std::process::exit`/事件循环退出点（`ControlFlow::Break`/窗口 close 路径）的 tracing 落盘（`AUTO_EXIT_TRACE=<file>` env 开关）；musk 侧 `scripts/vm-first-run.mjs` 加长跑变体（保活 ≥5 分钟、退出码+trace 文件收集）。复现样本特征：~4-5min exit 1、无 panic 输出、非 CloseRequested、AUTOUI_MCP_DISABLE=1 无效。验证：harness 复现一次并拿到 trace/退出点证据。若 instrumentation 仍无法捕获（候选：Windows 事件日志/ETW、release+debug 符号重编），现场裁定手段并记录。

**T2 根修 + 长跑**：按定罪修复（退出点语义或资源路径），验证 = 长跑 3×10 分钟全部存活（或正常退出码），vm-first-run alive reds=0。

**T3 MCP 子进程回收**：定位 `autoui_snapshot` spawn 链（mcp_server.rs），补子进程生命周期（等待回收/超时 kill/快照进程复用单例）；musk 侧加进程普查脚本（census：MCP 会话前后 `auto` 子进程计数）。验证：会话结束后 census 归零（66MB/43MB 两形态均不滞留）。

### 组二：语义族（上游根修 → musk 回撤）

**T4 __json_object 字符串读污染**：stdlib.rs GET_FIELD 对 __json_object 臂的字符串字段返回类型名（wl_probe21：`j.kind → "str"`，int 字段正常）——修为值本体；wl_probe21 转正入 `musk_vm_track` 测试族。musk 消费：问卷卡 `questionnaireFor` 的 `json.type` 判定恢复直读（057① 最后堵点）。验证：probe 绿 + 问卷卡 VM 实机渲染。

**T5 Regex.match 根修**：stdlib.rs:6149+ Regex 族 `match` 恒空（wl_probe18 围栏提取 0 匹配）——根修后 musk 回撤 indexOf 绕开（`questionnaireFor/stripQuestionnaire`，commit 11b6c20）。验证：wl_probe18 绿 + 回撤后 `pnpm vitest run` 全绿。

**T6 computed 读侧 + 跨模块 SET_FIELD（state-scope 专项）**：aura_view_builder eval_computed 求值上下文接 store computed 表（055-4⑥ 函数层同形探针全绿、断点在求值上下文——同 047① resolve_iterable 先例修法）+ auto-lang KD P536-D2 跨模块 SET_FIELD 不可达根态（`done` 臂自清依赖此，见 forge_store.at PLAN-062 注释）。musk 消费：画布投影恢复（过滤投影计数>0、发送消息气泡即时入列）；t3_filter 探针转正。forge_store 8b1ae23 四修逐项复盘（详见 T8）。

**T7 ThinkBlock chevron 读侧**：与 T6 同根（子件 if/computed 求值分歧——写侧 think_open 键实机验证 `["bsmsg-a1"]` 已通）。验证：实机展开/收起独立翻转，▼ 不再共享末值/双内容。

**T8 Sse no-op 容错**：stdlib.rs:8788+ Sse.* 显式降级路径对 handler-as-value 实参（`Sse.open(.OnStreamEvent)`）不再抛 Field not found（实参解析容错或惰性求值）；musk 侧复盘 forge_store.at 8b1ae23 四修（streaming 置位前移/头部直连 close/启发式回合增长守卫/when 门摘除+deadman 2 分钟窗）：**建议 deadman 窗保留**（PollStream 轮询形态常驻兜底），其余按根修效果逐项回撤。验证：StartStream 在 VM 零抛点 + 发送链路 E2E 不回归。

**T9 059-T9 五余项**（auto-lang iced/minted 面，逐项上游修 + musk 实机验证）：
1. fixed_both 钮内 Image 不居中（renderer.rs:3339 判定臂 + :3569 内容臂对 Image 未生效；实机对照 tmp/header-zoom.png）；
2. 浮层族（alert-dialog/dialog/dropdown-menu）trigger 锚件在场才绘浮层——空 trigger 退化流内（已实证），补"无 trigger 形态面板仍绘"；
3. 受控 open（显式绑定自管）形态补 ESC/外点自动关闭（web reka 原生行为的最小对齐面，不泛化全部 minted 形态）；
4. scrim 暗幕实机目验（暗色主题下歧义，专项亮/暗双主题截图）；
5. 弹层 chrome 固定宽度档 → 宽度 prop 透传。

**T10 musk 门禁全量**：`scripts/vm-first-run.{mjs,cmd}`（alive reds=0）/ `scripts/vm-link-probe.cmd` / `cd gen/front/vue && pnpm build && pnpm vitest run` / `cargo nextest run -p musk`（**必须 nextest**）/ 对拍 30/30 / style-parity 基线差分恒等。

**T11 双仓收尾**：auto-lang `auto-musk-dev` 合回 master（消费验证通过即合，不留悬挂 worktree）→ musk 主检出复跑门禁 → worktree/分支/组目录清理。

## 测试设计

- 上游每根修配最小复现单测（探针 .at → auto-lang `musk_vm_track_p0XX` 测试族命名惯例）；wl_probe21/18、t3_filter 转正。
- musk 回撤项以既有 vitest 基线全绿为准（36+1skip 口径，以复跑为准）。
- 稳定性：长跑 harness（T2）+ census 脚本（T3）作为可复跑资产入库 `scripts/`。
- 实机验证项（问卷卡/画布投影/chevron/五余项）以截图或行为记录入 `docs/plans/attachments/`。
- 全程门禁用 cargo-nextest（062 登记），禁裸 cargo test 判红。

## 验收标准

1. KD-048a 有定罪证据（trace/退出点记录在案）且修复后 3×10 分钟长跑零静默退出。
2. MCP 会话后 auto 子进程 census 归零。
3. wl_probe21/wl_probe18/t3_filter 转正且全绿；musk indexOf 绕开回撤后 vitest 全绿。
4. 实机：发送消息气泡即时入列（computed 投影）、过滤计数正确、问卷卡渲染、ThinkBlock chevron 独立翻转。
5. StartStream 在 VM 零 Field not found；四修复盘结论（保留/回撤）逐项记录。
6. 059-T9 五项逐项实机证据在案。
7. musk 门禁全绿（vm-first-run/probe/build/vitest/nextest/对拍/style-parity 基线）+ auto-lang lib 测试绿。

## 执行步骤

- [ ] T1 KD-048a instrumentation + 长跑 harness：auto-lang iced runtime 退出追踪（AUTO_EXIT_TRACE）+ musk `scripts/vm-first-run.mjs` 长跑变体；验证 harness 复现并拿到退出点证据
- [ ] T2 按定罪根修 + 3×10min 长跑验证；验证 vm-first-run alive reds=0 且零静默退出
- [ ] T3 MCP 子进程回收：auto-lang `ui/mcp_server.rs` autoui_snapshot spawn 链 + musk census 脚本；验证会话后子进程归零
- [ ] T4 __json_object 字符串读根修：auto-lang `vm/ffi/stdlib.rs` GET_FIELD 臂 + wl_probe21 转正 + musk 问卷卡 json.type 直读；验证 probe 绿 + 实机渲染
- [ ] T5 Regex.match 根修：auto-lang `vm/ffi/stdlib.rs:6149+` + musk 11b6c20 绕开回撤；验证 wl_probe18 绿 + vitest 全绿
- [ ] T6 state-scope 专项：auto-lang `ui/aura_view_builder.rs` eval_computed 上下文 + P536-D2 跨模块 SET_FIELD + musk t3_filter 转正；验证画布投影实机即时入列
- [ ] T7 ThinkBlock chevron 读侧（随 T6 同根验收）；验证实机独立翻转
- [ ] T8 Sse no-op 容错 + forge_store 8b1ae23 四修复盘（deadman 建议保留）；验证 StartStream VM 零抛 + 发送链 E2E 不回归
- [ ] T9 059-T9 五余项逐项修 + 实机验证（renderer.rs fixed_both Image/trigger 锚件/受控 open ESC+外点/scrim 双主题/宽度 prop）；验证五项证据在案
- [ ] T10 musk 全量门禁：vm-first-run/vm-link-probe/pnpm build+vitest/cargo nextest/对拍 30/30/style-parity 基线；验证全绿
- [ ] T11 双仓收尾：auto-lang 合回 master + musk 主检出复跑 + worktree/分支/组目录清理；验证 wt-guard clean

## 复审记录

（待 /auto-plan:review 填写）

## 待澄清事项

1. **T1 定罪手段现场裁定**：无 panic 输出的 exit 1 若 instrumentation 捕不到，候选=Windows 事件日志/ETW 或 release+debug 符号重编；不预设，执行时按证据选。
2. **T6 与 auto-lang 在途对表**：P536-D2 与 state-scope 若触及 536 系遗留改动集，开工前先 `git log` 对表避免撞文件（578/579 为 app 层，预期无冲突）。
3. **T8 四修逐项裁定**：deadman 窗建议保留（轮询形态常驻兜底）；其余三项在根修验证后逐项回撤或注记保留理由。
4. **G1 阶段2 SSE 桥泛化不在本计划**（047 勘察报告推荐路线的长杆项），T8 只修 no-op 抛点不实现真 SSE。
5. 实机验证依赖用户可用性（VM 目验/截图）；窗口期阻塞时该项记 🔶 待用户并继续后续任务。
