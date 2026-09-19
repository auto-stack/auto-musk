---
plan_id: PLAN-073
status: execution_done
feature_name: chat 运行挂死修复（看门狗命令盲区 / daemon 静默吞参 / 超时收束丢证据）
author: zhaop / zcode
created_at: 2026-09-18T22:00:00+08:00
updated_at: 2026-09-18T23:10:00+08:00
plan_revision: 2
current_step: 6
total_steps: 6
touched_repos: [auto-ai@630a98d, auto-musk@ae86fb0]
---

# PLAN-073 — chat 运行挂死修复

会话 `90ed3ae00e4d6a6321f7af76`（auto-edit，"@plan/001 实施这个计划"，21:28:59 →
21:34:19 空闲超时中止）实测暴露三缺陷。诊断已实证（诊断记录见 PLAN-072 §4 同期
会话），本计划实施修复。

## 0. 根因（已实证）

1. **看门狗命令盲区**：空闲看门狗只被 agent StreamEvent 喂（extern_impl.rs:2071）；
   run_command 执行期不产生 agent 事件（ToolStart 已发、Tool 结果等命令退出），
   100ms 输出进度走 tool_update 桥到 UI 但**不喂狗** → 长命令执行 ≥300s 必被误杀。
2. **双 300s 竞速**：命令默认超时 300s（tools.rs:369，AUTO_CMD_TIMEOUT_SECS）==
   看门狗窗口 300s（server.rs:61，AUTO_RUN_IDLE_TIMEOUT_SECS），同时到期看门狗先胜。
3. **daemon 静默吞参**：流式工具参数为空/JSON 解析失败时，openai.rs:348-370 与
   anthropic.rs:405-418 均静默替换为 `{}`（仅 stdout warn）→ UI 出现"无参数"工具卡，
   模型拿到 missing-args 错误盲目重试。
4. **超时收束丢证据**：空闲超时路径只持久化超时消息，已积累的
   thinking/tool_calls/blocks 全部丢弃（对比成功路径 extern_impl.rs:2327+）。

## 1. 实施

- [x] **T-01** musk：tool_update 桥喂看门狗——bridge 任务收到
  tool_update/tool_gate_waiting/relay_gate_waiting 事件时 `wd_tx.send(now)`。
- [x] **T-02** musk：命令默认超时 300→120s（AUTO_CMD_TIMEOUT_SECS 仍可覆盖），
  令命令自然上限远小于看门狗窗口，长命令超时走自身收束而非看门狗误杀。
- [x] **T-03** auto-ai：吞参显式化——`StreamDelta::Warning(String)` 变体；
  openai/anthropic provider 在空参/解析失败替换 `{}` 时 emit warning（含工具名、
  原因、原始长度/头部）；daemon server.rs 增 `{"type":"warning","text"}` 帧分支；
  agent `forward_sse_delta` 映射 warning 帧 → `StreamEvent::Warning`
  （warning 帧带 text，必须先于通用 text 路径分流，否则告警文本被拼进正文）。
  转换逻辑提取 `tool_calls_from_accum`/`tool_calls_from_blocks` 以可测。
- [x] **T-04** musk：超时收束持久化——空闲超时分支按成功路径同款组装已积累
  thinking/tool_calls/blocks，超时通知作尾块追加后落盘（单消息 + turns 镜像）；
  另：on_event 事件分派增加 `warning` 臂，⚠️ 文本块入时间线（用户可见）。
- [x] **T-05** 验证：auto-ai daemon 64 + agent 133 测试全绿（含新增双 provider
  降级行为单测）；musk 437 测试全绿（含新增 `default_cmd_timeout_resolves_to_
  120s`；既有 `chat_idle_watchdog_times_out_hung_run` 等看门狗用例不回归）；
  musk worktree 经路径依赖直接消费兄弟 auto-ai worktree 编译链接成功（组布局
  即验证）。live 复现指引（GLM key 在用户环境）见 §4——参数丢失属模型侧还是
  分片侧的最终定性留待该复现，provider 层行为已单测钉死。
- [x] **T-06** 收尾：auto-ai 合回 main（**630a98d**，worktree/分支/auto-lang
  兄弟快照清理）；musk 合回 main（**ae86fb0**，worktree/分支/组目录清理；
  wt-guard 三 worktree 全 clean）。

## 2. 验收标准

- [x] **AC-01** 长命令不再被看门狗误杀：命令执行期 tool_update 事件刷新空闲窗；
  静默命令 120s 由命令超时自身收束并返回结果事件（模型可见），看门狗仅对
  真停顿（LLM/管道级）兜底。
- [x] **AC-02** 吞参可见：畸形/空参时 UI 收到 warning 帧，消息时间线出现
  ⚠️ 块；daemon 不再无痕迹替换（双 provider 单测锚定 warn 触发与内容）。
- [x] **AC-03** 超时会话保留现场：空闲超时落盘的消息包含中止前的
  thinking/工具卡/时间线，超时通知在尾部。
- [x] **AC-04** 双仓既有测试全绿 + 新增行为测试（warning 帧/映射/默认超时值）。

## 3. 影响面与兼容

- warning 帧为新事件类型：老前端/CLI 未知类型忽略（既有行为），向后兼容。
- 命令默认超时变更只影响未显式传 timeout 的调用；模型显式传值不裁剪（文档化：
  显式值 > 看门狗窗口时仍可能被看门狗中止——进度流动则不受影响）。
- 已知残留（登记不阻塞）：human 审批门（gate）等待 1800s 上限 > 看门狗窗口，
  等待期无心跳仍会被中止——需门工具心跳或看门狗门状态感知，另立计划。

## 4. live 复现指引（参数丢失根因钉死，用户环境执行）

```bash
# 终端 A：debug 日志起独立 aaid（不同端口，避免动正在跑的 17654）
RUST_LOG=debug ZHIPU_API_KEY=<key> aaid --listen 127.0.0.1:17655 2>&1 | tee aaid.log
# 终端 B：独立 musk 指向它 + 缩短窗口
AAID_URL=http://127.0.0.1:17655 AUTO_RUN_IDLE_TIMEOUT_SECS=60 AUTO_CMD_TIMEOUT_SECS=30 \
  musk serve --addr 127.0.0.1:8095 2>&1 | tee musk.log
# 终端 C：对 8095 发同型消息（@plan/001 实施这个计划），观察
# aaid.log 的 `streaming tool_call: name=... args_len=` 与
# `streaming: tool_call ... empty/malformed arguments` warn 的先后关系
```

## 5. 记录

**实施完成（2026-09-18 23:10，r2）**：T-01..T-06 全勾，AC-01..04 全过。
auto-ai 分支提交 630a98d → main merge；musk 分支提交 ae86fb0 → main merge
（fast-forward）。收据：wt-guard 三 worktree clean；测试对 daemon 64 +
agent 133 + musk 437。**生效前提**：需重编译部署 musk.exe 与 aaid.exe
（本轮已在两仓 worktree 构建通过；用户运行态的 8090/17654 为旧二进制，
重启/替换后生效）。同会话遗留观察：human 审批门 1800s > 看门狗窗口，
gate 等待期无心跳仍会被中止（§3 已登记，另立计划）。

**r3（2026-09-19 00:20）——前端渲染层崩溃修复（用户 23:01 复测暴露）**。
重启用后用户实测：思考流正常、工具卡 ARGUMENTS/RESULT 全空、"卡住"。
控制台记录 + SSR/线上抓流（SSE 载荷 `arguments:{"path":"docs/plans/001-…"}`
完整）定位为**前端渲染层**三缺陷 + 一处键错位：

- **F-B1 崩溃主因**：gate_waiting（人工审批门）的工具调用永远等不到
  tool_result，`tc.result === undefined`，模板守卫 `block.tc.result != ""`
  对 undefined 恒真 → `<Markdown :source="undefined">` → StreamingRenderer
  `Gye/flatMap` 崩溃循环（每帧重渲必崩，整列表冻结——即"卡住"的观感）。
- **F-B2 审批 UI 未接入**：ChatMessage 内联工具模板无 gate_waiting 分支
  （ToolGateCard 只挂在已退役的 ToolBlock 链）→ 越界命令挂门后无
  approve/deny 入口，运行挂到门超时；且状态被 else 兜底渲染成绿色
  "completed"（假完成）。
- **F-B3 键错位**：后端持久化 ChatBlock 工具载荷在 `tool` 键，模板读 `tc`
  → 历史视图工具卡无名/无参/无结果。
- **F-C1**：live push 的工具卡缺 `result`/`tkey` 初始化——前者即 F-B1，
  后者使 `tool_open == block.tkey` 双 undefined 恒真（全部卡片默认展开）。

修复（musk main 直提，.at 源 + 计划账面；gen/front/vue 为未跟踪生成物随
build 部署）：forge_store.at（tool_call push 补 `result:""`/`tkey:callId`；
新增 `normalizeToolBlocks` 于 4 处会话载入点归一 tool→tc + result 兜底）；
chat_message.at（gate_waiting 分支接 ToolGateCard approve/deny；状态/卡片
样式加 amber 门等待态；result 守卫回落普通比较）。约束记录：模板 v-if 表达
式不支持 `??`（UnexpectedToken），store 脚本域支持——守卫靠归一化保证非
undefined。重建产物部署（gen/front/vue/dist，ServeDir 直读，浏览器刷新即
生效，无需重启 musk）。

验证：codegen 通过；vite build 出 dist；线上抓流证明 SSE 载荷完整；
历史视图实测（同一会话）工具卡名称/ARGUMENTS/RESULT 全部渲染
（`{"path":"docs/plans/001-…md"}` + 计划正文）；后端 gate→approve→
执行→回灌 API 闭环实测通过（tg-…/approve 200 → tool_result success）。

**新登记债务**：①auto-lang vite 模板缺 `auto-select/overlay` 实现与
`vite-env.d.ts`（既有 vue-tsc 两个错误长期红着 build）——本次以 gen 目录
stub 绕开（生成物不入库，重装/迁移需重打），根修在 auto-lang 模板；
②eslint 缺装；③审批门等待期无心跳（§3）。

**r5（2026-09-19 11:40）——思考块单组件双状态（用户提议，顺带修"过去轮次
思考块恒挂思考中"）**。原设计思考块的 思考中/已思考 标签跟随**消息级**
is_streaming：运行进行中该消息所有思考块（含早已完成的过去轮次）都挂
"思考中"展开态，直到 done 才整批翻转。修正为**块级生命周期**：

- store：思考块创建即 `state:"streaming"`；`appendBlockText` 对已落地
  （done）的思考块不再追加文本（新轮次思考开新 streaming 块）；
  `landThinkingBlocks` 在 delta（同轮正文开始）/ tool_call（思考结束转入
  行动）/ turn_start / turn_end 四类事件落地全部 streaming 思考块；
  `normalizeToolBlocks` 补落盘思考块的 done 态（旧数据无 state 字段安全
  回退：无 state 一律按 done 渲染）。
- 模板：ChatMessage 思考块分支条件 `is_streaming` → `block.state ==
  "streaming"`（流式 = 尾部滚动条 + 思考中；否则 = 折叠头 + 已思考 +
  点击展开——同一形态的两个状态）；落地态标签固定"已思考"。
- 效果：过去轮次的思考块在轮次结束即落地折叠，只有当前块保持思考中；
  运行结束后全部落地。约束不变：.at 模板 v-if 不支持 `??`。

auto build 全绿（codegen + vue-tsc + vite，本次 pipeline 完整通过），
dist 重建部署（刷新即生效）。

**r4+（2026-09-19 01:55）——用户复测仍见空卡（01:30 截图）排查结论**。
校验线上 `/assets/index.js`：result 初始化/tkey/gate 分支/normalize 全部在
bundle 中 ✓；live 抓流（复现会话）`tool_call` 载荷完整（name=read_file +
arguments）✓；musko 日志（0919.log）无 daemon 空参/畸形告警 → **后端无
异常，判定用户浏览器加载的是 00:10 重建前的陈旧 JS**（产物无 hash +
no-cache 需显式刷新）。加固：侧边栏版本标记 `v0.1.0-p073r3`（app.at，
提交 71ceb39，dist 01:59 重建）——用户刷新后看侧边栏版本即可自检加载的
bundle 新旧。若新 bundle 下仍现空卡：抓 `musk-serve-0919.log` + console
定位（届时重点查 glm-5.3 对特定 prompt 的 tool_call 形态）。

**r4（2026-09-19 00:30）——审批模式语义修正（用户裁定）**。用户指出：人工/
自动两种模式都不应"卡住然后失败"——人工应暂停+出审批界面，自动应自动放行。
实测发现的行为偏差与修正（musk main 直提）：

- **F-D1 auto 模式自动放行**：原行为 = 非白名单 PAUSED 文本（模型自诉
  force 重调）+ 越界硬拒——与"自动 = 不打断"预期不符。改为
  `RunCommand::with_roots_progress_policy(…, auto_approve)`（lib.rs 按
  `approval_mode == "auto"` 接线）：auto 模式非白名单/越界直接执行。
- **F-D2 human 模式非白名单也走门**：原仅越界路径挂门，非白名单返回
  PAUSED 文本（无 UI 承接）。改为统一走 live 门（tool_gate_waiting →
  UI approve/deny），deny/超时 → 拒绝回灌模型、运行继续。
- **F-D3 门等待看门狗窗口对齐**：桥上 gate 事件喂 `now+1800s`（门超时
  1800s + 300s 余量）——初版喂 now+1500s 与门超时同刻竞速再次误杀
  （实测），+1800 后门超时先自然发生。
- legacy（无会话 CLI/relay）行为不变（PAUSED/硬拒），测试口径不变。

验证（live，musk-demo/backend 双工作区）：auto 模式 `hostname`（非白名单）
直接执行成功；human 模式越界 `type` 门触发（载荷 cmd/paths 正确）→
**故意等待 582s（远超原 300s 窗口）→ approve 200 → 命令执行 → 结果回灌**
——看门狗不再误杀门等待。遗留：门卡 UI 渲染的最终目验（本轮 IAB 标签
不稳定未能截到图，代码路径与 relay 视图既有 ToolGateCard 同源）。

**r6（2026-09-19 下午）——Vue 版三前端修复（用户报障）**。worktree
musk-073（plan-073-dev@acd8e11，已合回 main 并清理）：

- **F-E1 Tab 补全焦点跳变**：`@001` 弹候选按 Tab，补全 `@plan/001` 后
  焦点跳到工具栏"思考"钮。根因 = mention_input.Keydown 的 Tab/Enter
  补全分支不阻断默认行为——Tab 默认移焦点到 tabindex 序下一可聚焦元素
  （正是工具栏首钮）。修 = 分支内 `e.preventDefault()`（Enter 原有
  `.prevent` 修饰符兜底，Tab 无修饰符臂必须显式阻断）。验证：IAB 真实
  键盘事件下 `defaultPreventedAfter=true` + 补全成功 + 焦点保持 textarea。
- **F-E2 审批模式"自动"不生效**：选"自动"后按钮仍"人工"且后端零调用
  （浏览器实测无网络请求、无 JS 错误）。根因 = codegen 事件名断裂——
  msg 名 `pick_approval` 原样 emit（`emit('pick_approval')`），父层
  ChatsView 模板却生成 `@pickapproval`；Vue3 事件名精确匹配
  （onPick_approval/onPickApproval 均≠onPickapproval），事件悬空。修 =
  msg 重命名 `pickapproval`（全仓排查仅此一处下划线 msg）。验证：点
  "自动" → 按钮文案变"自动" + `PATCH /api/chats/session/{id}/approval`
  200 + 会话详情 approval_mode=auto 落库；已恢复 human。
- **F-E3 @plan/@spec 引用 token 独立 badge 组件**：气泡内
  `@plan/001` 升级为独立 badge 小组件（胶囊圆角+边框+悬停加深+
  cursor-help），hover `title` 显示真实文件路径——plan 查
  `build_ref_path_map`（PlansStore.plans 的 path||filename 映射；
  UserMessage Init 懒加载，store.plans 非空即跳过），spec 的 relpath
  本身即路径 render 内直用；plans 未加载时降级纯 badge 无 title。
  `render_mentions` 签名加第 4 参 paths（render_mentions_default 传
  None 不变，新增 render_mentions_ref 供 UserMessage）。Agent @词
  渲染口径不变。验证：气泡内 badge DOM（title=001-bootstrap-auto-edit.md）
  + 截图目验。

- 附带确认 worktree 重建的两个环境坑（既有债务，未根修）：①auto-lang
  scaffold 写 `auto-select/overlay.ts`（import auto-sources）但不写
  `auto-sources.ts`/`vite-env.d.ts`（仅 `auto run`/增量同步点写）→
  vue-tsc TS2307/TS2339，本次手工补两文件（同 r4 gen stub 绕法）；
  ②`.gitignore` 的 `dist/` 规则把 `vendor/@autodown/engine/dist` 的
  code-split chunks 挡在 git 外 → 新 worktree vite build 报
  "Could not resolve render-node-*.js"，从主检出补拷 5 个 chunk。两项
  根修均在 auto-lang 模板/仓策略（登记，未动）。

验证：auto build 全 pipeline 绿（codegen+vue-tsc+vite）；dist 已部署
主检出 gen/front/vue/dist（serve 直读，刷新生效）；live 三修复逐一
浏览器实测通过。

**r7（2026-09-19 下午②）——streaming 工具卡空显示修复（用户报障）**。
现象：streaming 实时显示中工具卡无名/无目标/无结果，展开 ARGUMENTS/RESULT
空白、状态恒绿 "completed"（用户截图2），而日志证明工具实际调用成功。
诊断路径：IAB 真实键盘不可注入 → 改 dev server（vite 3001 代理 8090）+
页面内采样器（EventSource 帧记录 + 定时 DOM/store 双读）→ 抓到断点：
**store 数据完好（tool_calls/blocks 的 tc.name="list_dir" 等）而 DOM 渲染
空名** —— 纯渲染层断裂。

根因 = `forge_helpers.messageBlocks` 归一化分支只认持久化形态载荷键
`raw = b.tool ?? {}`；流式 live push 的块是 tc 形态（forge_store tool_call
臂产 `{kind,tc,tkey}`，无 tool 键）→ raw 洗成 `{}` → 输出
`tc:{name:undefined, status:"completed"(默认)}`：无名 + 假 completed +
ARGUMENTS 空 + `result:undefined != ""` 恒真 → RESULT 块渲染 undefined
（截图2 空白）。注释"流式侧自产块原样透传"与实现不符（透传未实现）。

修复（worktree musk-073，plan-073-dev@75b0708，已合回 main）：
- `raw = b.tool ?? b.tc ?? {}`——tc 形态字段名与 raw 同构，直取等价；
- 顺带：nstatus 对 `gate_waiting` 透传（原 fall-through 假 completed，
  chat_message 的 gate 卡分支不可达——r4 F-D2 的 UI 臂补全）。

验证：沙盒单测（live 三态块 completed/running/gate_waiting 的
name/status/summary 全正确）；浏览器端到端（修复后 dist 部署，真实 run
streaming 中 list_dir [specs] completed、glob [specs "**/*.md"] completed，
展开 ARGUMENTS `{"path":"specs"}` + RESULT 目录清单全渲染）。

**新登记缺陷（未修，独立立案候选）**：chat run 收束后 assistant 消息
落盘失败——9063dfd（@plan run，16:05）与 48b7e82a（musk-demo，16:15）
两个会话后端仅剩 user 消息，assistant（含工具卡/正文/思考块）前端有、
后端无 → 刷新即丢。疑似收束 append_message 路径回归（r4 T-04 补的是
超时收束，正常收束臂待查）。

**r8（2026-09-19 下午③）——标题栏收敛（用户报障）**。窄 rail 里
"Auto Musk"+版本号双 text 挤成两行（用户截图）。改为单一 deer icon 按钮
（与折叠态同款双主题图，点击仍可折叠），版本号转 hover title——
`Auto Musk v0.1.0-p073r8`（r3 的 bundle 新旧自检锚点保留，版本号随轮次
更新——本轮 3001 旧缓存混淆正是缺此锚点所致）。worktree
musk-073（plan-073-dev@f8d5c4b）已合回 main；dist 已部署，8090 刷新生效。

**部署口径备忘**：正式入口恒为 `http://127.0.0.1:8090`（musk serve 直读
`gen/front/vue/dist`）；worktree 临时 dev server（vite 3000/3001）仅诊断
用，收尾必清理——r7 诊断后 3001 残留进程向用户浏览器吐删除前旧模块，
造成"修复无效"误判（实为访问了失效入口）。
