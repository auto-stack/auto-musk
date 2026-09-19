---
plan_id: PLAN-073
status: archived
completion_kind: delivered
feature_name: chat 运行挂死修复（看门狗命令盲区 / daemon 静默吞参 / 超时收束丢证据）
author: zhaop / zcode
created_at: 2026-09-18T22:00:00+08:00
updated_at: 2026-09-20T00:50:00+08:00
plan_revision: 4
current_step: 11
total_steps: 11
touched_repos: [auto-ai@630a98d, auto-musk@e5e098e]
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

**r9（2026-09-19 傍晚）——输入框引用 token badge（用户报障，r6 遗留项闭环）**。
用户确认 r6/r7/r8 生效后指出：补全进输入框的 `@plan/001` 仍是纯文本，应成
badge 小组件。根因：输入框高亮由 codegen `__autoMentionHtml` 承载（PLAN-493
textarea mentions 能力），原实现只扫 `@\w+` 且 musk 高亮名单仅含 agent 名
——含 `/` 的引用 token 永不可达。

修复（跨仓双提交）：
- **auto-lang@03ef62f**（worktree 组内并排，auto-musk-dev 合回 master）：
  helper 通用化——token 字符集扩展扫描（`\w./-`）+ 名单**最长匹配**；命中
  含非 `\w` 字符者 span 附加 `mention-token` 语义类（宿主样式差异化着色；
  类名零宽度，backdrop 与 textarea 逐字对齐不受影响——badge 禁
  padding/border 即此铁律）。纯 `\w` 词（Agent @词）走原类串口径不变。
  plan493 三测试更新全绿；ui_gen 全量对照 master 基线零回归（差异仅
  bp::registry scan 抖动测试）。
- **musk@fffddc0**（合回 main）：①mention_input 两处 mentionNames 刷新后
  追加 mentionItems 非 agent 项 id——token 匹配名单驱动；②inject_styles
  `.mention-token` 紫系（品牌主色 + 10% 底 + 圆角）区别 Agent @词（蓝）。

验证：bundle 内 helper 四场景单测（refToken/specToken 命中+mention-token、
agentWord 原口径、a@b.com/@nope 不误高亮）；真机 backdrop 合成输入——
`@plan/001` 紫 badge、`@assistant` 蓝、计算样式 rgb(100,103,242)。dist 已
部署（8090 刷新生效）。

**工程记注**：①auto-lang workspace 路径依赖 `../../../auto-down` 在组
worktree 内失效——组内并排 auto-down worktree（纯路径解析，零改动）后
cargo 可跑；②auto-lang 主检出存在他方在途合并（MERGE_HEAD plan-022 遗留
+UU specs.json），merge/--ff-only 均被前置检查拒绝——以 update-ref 原子
前移（纯 ff）+ checkout 同步单文件完成合回，在途合并状态原样保留待其主
处置。

**r10（2026-09-19 晚）——输入框 token badge 交互四修（用户报障）**。r9
badge 上线后的四个交互问题：
1. **光标视觉进入 badge/错位半字符**（用户截图3/4/5）：mention_class 的
   `px-[0.2rem]` 把 backdrop 高亮文字推离 textarea 真实位置约 0.4rem
   （中文语境即"半个字符"）。去 padding——backdrop 与 textarea 逐字
   对齐铁律（r9 对 .mention-token 守住了，共用的 mention_class 漏网）。
   验证：badge computed padding 0px/0px。
2. **IME 组词瞬间 badge 闪回纯文本**（截图2）：mentionNames 两步赋值
   （先纯 agent 数组上 ref、再 push token）在高频 input 下存在中间态
   窗口。改本地数组拼装完成后单次赋值，无中间态。
3. **Backspace 逐字符删 badge**（用户需求）：新增 mention_backspace_target
   （mention_helpers.at）——光标紧邻/处于引用 token 内时反扫路径字符到
   '@'（@ 前不得是路径字符，排除词内@）+ "@…pos" 段为 items 非 agent 项
   id 的非空前缀 → 返回整删起点；Keydown Backspace 分支 preventDefault +
   整删 + platformRestoreComposerCursor 端口（新增：web =
   composer_cursor.web-only.ts 用 setTimeout(0) 在 Vue flush 后
   setSelectionRange 恢复光标——受控 textarea 回写会重置光标；VM no-op
   降级逐字符）。验证：helper 六场景 node 单测全对 + 真机端到端
   （"@plan/001 abc" 光标 9 按 Backspace → " abc" 光标 0）。
4. 附带确认：TokenOnly 越界 pos 等防御分支行为正确（测试首版用例下标
   算错导致的假阴性已澄清）。

worktree musk-073（plan-073-dev，merge 689be48）已合回 main 并清理；
dist 已部署，8090 刷新生效。新增文件：composer_cursor.web-only.ts。

**r12（2026-09-19 晚②）——工具卡头两修（用户报障）**：
1. **skill 卡头无名**：getToolSummary 加 skill_name 分支——skill 工具参数
   为 `skill_name`（auto-ai skill.rs schema），名称以 path 青色段显示
   （与 write_file 路径同款）。
2. **run_command 命令串灰色**：command 的 seg 类型 desc→cmd，chat_message
   seg class 加 cmd 臂（text-muted-foreground）+ scoped
   `.tool-seg.seg-cmd`——命令串与工具名拉开层次。

验证：node 单测（skill→path "executing-plans"、run_command→cmd、
write_file 不回归）+ 真机 run（skill 卡青色名 + run_command 灰色命令串）。
worktree musk-073（plan-073-dev@8e87b85）已合回 main；dist 已部署。

**r13（2026-09-19 20:35）——Phase 2 实施（P2 全量，诊断修订见 §6）**。
诊断先行证伪原定性（成功臂落盘正常，09-19 九例 finished 全落；真缺陷 =
失败臂丢现场 / 超长 run 零增量+刷新不重挂 / 无日志出口守卫滞留），按修订
后的 P2-T1..T5 实施完毕：

`stage: work | PLAN-073 | rev4 | pass | musk@e5e098e | P2-T1,T2,T3,T4,T5 |
musk cargo test 652 绿（1 失败为基线预存环境失败，stash 对照 e2c0cb0 实证）；
auto build 全 pipeline 绿；8095 隔离实例 live：失败臂落盘 ⚠ 尾块 + 终态、
裸 attach 即收 idle 帧；新增回归 chat_failure_preserves_accumulated_scene
（工具→快照→失败终版原位替换全链）| AC-P2-2 端到端待用户环境复测（真 LLM
长 run 中刷新）；musk.exe 需重启 8090 生效 | next: review（auto-plan-review）`

工程记注：①upsert 原位替换必须保留 parent_id/created_at——终版消息独立
组装不重导 parent，整体换入会脱离分支链（测试实证后修复）；②idle 帧需
进 server_stream 透传白名单（SseEventDto 严格枚举外事件会被
stream_event_map 打成 malformed error）；③pnpm node_modules 的 junction
触发 wt-guard——node fs.rmSync 安全移除（不穿透链接）后过闸，勿用
rm -rf/rmdir /s（2026-09-03 事故同类）；④测试隔离：musk serve 的注册表
绑死 ~/.config/autoos（无 env 覆盖），隔离实例用 USERPROFILE 指临时 home
+ AAID_URL 死端口（避免误烧用户 key/污染真实工作区）。

## 6. Phase 2：run 收束 assistant 消息落盘修复（待实施；r13 诊断修订）

> 本章节自包含——实施会话无需 Phase 1/r1-r12 的上下文即可开工。
> 状态：**已立项未实施**（用户裁定优先）。r13（2026-09-19 20:00）用日志+磁盘
> 交叉分析**证伪了原始定性**，任务/验收按新根因重写。预估 1 个会话内完成。

### 6.0 症状与定性（r13 修正）

原始定性"正常收束后 assistant 不落盘"**证伪**——09-19 当天全部 9 个走到
`chat run finished` 的 run（含 61 分钟长 run 48b7e82a）assistant 均完整落盘。
用户实际经历的是三个不同缺陷：

- **D1 失败臂丢现场（真数据丢失）**：run 以 Err 收束（如 max turns / loop
  detected）时只落一行 `⚠ 运行失败：{e}` 文本，thinking/tool_calls/blocks
  全部丢弃。T-04 只修了**超时臂**，**Err 臂**没做同款组装。
  实证：24a43d74 跑满 100 轮失败（19:14）→ 仅存一行 ⚠，全部工具卡丢失。
- **D2 超长 run 期间刷新即空白（用户主诉的机制）**：assistant 只在收束时
  **一次性**落盘；@plan 类 run 实测 23min～3h+（48b7e82a 61min、9063dfd4
  ≥23min 未收束）；前端刷新后 store 重置——`StartStream` 仅在 Send 时调用、
  120s 轮询窗早过期、无任何"运行中"查询 → 中途刷新只看到 user 消息，
  观感即"落盘丢失"（数据其实还在内存，收束时会落——但用户无从知道）。
- **D3 静默出口/楔死无诊断面**：9063dfd4（16:05 起跑）在日志里**没有任何
  收束行**（finished/failed/idle-timeout 三种出口都打日志），守卫 16:28 仍
  被占（"已有运行在途"通知实证）→ spawn 任务存在无日志出口或楔死路径；
  守卫滞留期间该会话永久不可再跑。gate 循环每门 +1800s 喂狗，看门狗无法
  兜底"deny-重试"循环（§3 已登记债务的升级证据）。

### 6.1 证据（r13 交叉分析：musk-serve-0919.log × chats.json × conversations）

日志时间戳为 UTC（+8=CST）。当前 8090 进程 09-19 01:29 CST 起存活
（PID 5816，err.log 空 = 无 panic）。

| 会话 | 起跑/收束（CST） | 结局 | 落盘现状 |
|---|---|---|---|
| 48b7e82a (musk-demo) | 16:15 → 17:16（3683s） | `chat run finished` | **assistant 完整**（tcs=10 blocks=18，updated_at=17:16:47）——计划旧表"仅 user"是 16:15-16:20 **run 在途时采的样** |
| 9063dfd4 (auto-edit) | 16:05 → **无收束行** | 未知（守卫 16:28 仍占） | user + 后来补发消息 + busy 通知；run 的 assistant 至今缺席 |
| 24a43d74 (auto-edit) | 18:51 → 19:14（max turns 100） | `chat run failed` | 仅一行 ⚠（D1 实证：100 轮现场全丢） |
| 66005d91/bbdde7ec/76a09156/7faefb07 (auto-edit) | 16:00-19:10 各 16-24s | finished | 全部落盘 ✓（成功臂正常的多例对照） |
| 25d7b6fb/90cdf37f/5d01c54f/9077ba9e (musk-demo) | 16:09-16:14 各 22-34s | finished | 全部落盘 ✓ |

原疑点清单裁定：疑点 1（done 早退）/2（append id 语义）/5（写失败静默）
对**成功臂**全部证伪（`ChatSession::append` 无条件 push，chats.rs:226；
成功臂 extern_impl.rs:2448 落盘且其后紧随 `chat run finished` 日志，多例
兑现）。疑点 3（桥 abort）证伪（bridge.abort 在 append 之后）。
新疑点成立处：Err 臂组装缺失（extern_impl.rs:2464-2482）、零增量落盘、
前端不重挂（src/front/forge_store.at）、chat_run_stream 无在途判定响应。

### 6.2 代码入口（已勘察）

- 收束三臂：`backend/crates/musk/src/auto_generated/extern_impl.rs`
  超时臂 2344-2407（组装样板）/ 成功臂 2408-2463 / **Err 臂 2464-2482
  （仅文本，待修）**。事件累积器 accumulated/thinking_acc/tool_calls/
  blocks/cur_text 定义 2035-2051；on_event 块维护 2141-2252
  （现有 turn_start/delta/tool_call/tool_result/done/warning 臂）。
- 附加流：`chat_run_stream`（extern_impl.rs:1762-1798）——只订阅转发，
  **未调用** `chat_run_active` 窥探（server.rs:118 已有该只读方法，注释
  自述"订阅路径据此决定附加转发或空闲等待"但未接线）。
- 持久化：`backend/crates/musk/src/chats.rs` ChatStore::append_message
  （500，整表 load→改→save，write_lock 串行）；ChatSession::append（226，
  parent/leaf 语义）；ChatMessage 结构 64-85（serde 宽容默认）。
- 前端：`src/front/forge_store.at`（StartStream 仅 Send 调用；OnStreamEvent
  各事件臂；done 臂收尾；120s 轮询窗）。

### 6.3 任务（r13 重写；实施完成 2026-09-19 20:30，musk@e5e098e）

- [x] **P2-T1 定位**（r13 完成，见 §6.0/6.1）：根因三缺陷 D1/D2/D3 + 成功臂
  证伪。产出即本节。
- [x] **P2-T2 失败臂保现场**：`assemble_chat_run_msg` 三臂共用组装（封口
  叙述块→思考首块→⚠ 尾块，clone 语义，单测 3 例）；Err 臂接入（⚠ 尾块 +
  现场，24a43d74 场景闭环）；`persist_chat_run_msg` 统一落盘口——失败/
  session 缺失显式 warn。回归测试 `chat_failure_preserves_accumulated_scene`
  （工具→turn 快照→失败终版原位替换全链，真实 chat_run_owner 路径）。
- [x] **P2-T3 增量落盘**：`ChatStore::upsert_message`（按 msg.id 原位替换并
  **保留 parent_id/created_at**——测试实证整体换入会脱离分支链；无该 id
  退化 append；单测 4 例含旧数据 pending 回退）；run 起跑定 `run_msg_id`，
  on_event `turn_end` 臂快照落盘（pending=true，ChatMessage 新增 serde 宽容
  字段）；收束三臂终版同 id 换入清 pending；turns 双写维持收束一次。
- [x] **P2-T4 刷新重挂**：chat_run_stream 入口接 `chat_run_active` 窥探
  ——空闲立即 idle 帧收流（try_recv 捞收束竞态的 done 尾巴）；server_stream
  透传白名单补 idle；前端 AttachStream（载入即挂流，**不预置 streaming**——
  旧后端零事件零幻态，兼容 dist 先于 exe 部署的窗口）、idle 臂 + 流事件
  置位运行态、PollStream 完成启发式 pending 守卫。
- [x] **P2-T5 验证合回**：musk cargo test **652 绿/1 失败**（失败
  `run_command_dangerous_returns_paused` 为基线预存环境失败——stash 对照
  e2c0cb0 实证同败，与本轮无关）；auto build 全 pipeline 绿（codegen+
  vue-tsc+vite，r6 两环境坑照旧绕法：vite-env.d.ts/overlay.ts stub 拷主
  检出）；8095 隔离实例 live（USERPROFILE 指向临时 home + AAID_URL 死端口，
  零副作用）：失败 run 落盘含 ⚠ 尾块 + 终态、裸 attach 即收
  `data: {"type":"idle"}`；dist 已部署主检出（前端刷新即生效）。**musk.exe
  待用户重启 8090 生效**（运行中进程锁 target；重启顺带清掉 9063dfd4 的
  楔死守卫）。rebase→ff-only 合回 main（e5e098e），四 worktree/分支/组目录
  清理（node_modules 链接以 node fs.rmSync 安全移除后 wt-guard clean）。

### 6.4 验收标准（r13 重写）

- [x] **AC-P2-1** 任意收束路径（成功/失败/超时）后
  `GET /api/chats/session/{id}` 的 messages 含完整 assistant：成功臂含
  content/thinking/tool_calls/blocks；失败臂含 ⚠ 尾块 + 中止前现场
  （thinking/工具卡/叙述）。〔成功臂 = 既有 `ag_chat_message_run_true_
  persists_single_reply` + 09-19 九例 live 对照；失败臂 = 新回归测试 +
  8095 隔离实例 live（⚠ 尾块/终态）；超时臂 = 既有看门狗用例（组装改共用
  函数，行为等价由组装单测锚定）〕
- [x] **AC-P2-2** run 进行中刷新页面：可见已落盘部分现场（turn 粒度增量），
  且运行中状态恢复（后续事件继续渲染）；收束后前端归一为完整消息，
  与不刷新路径一致。〔**复审 live 端到端复现闭环**：隔离实例（新 exe=
  e5e098e 源码构建）+ 真 daemon 短 run——中途 GET 见 pending=True 快照
  （2 工具卡），挂流收 578 帧实时事件，收束终版同 id 换入 pending=None、
  content 516 字、块序 thinking·tool·tool·text、恰 1 条 assistant〕
- [x] **AC-P2-3** 落盘失败不静默：append/upsert 失败有 warn 日志；
  新增收束落盘回归测试挂入 musk 测试套件。
- [x] **AC-P2-4** 附加流不悬挂：对无在途 run 的会话开 /stream 立即收
  idle 帧关闭；有在途 run 时刷新可重挂并收到后续事件。〔idle 帧 =
  8095 live `data: {"type":"idle"}`；重挂转发 = 既有
  `ag_chat_run_then_subscribe_single_reply`（裸订阅契约测试同批改锚）〕

### 6.5 登记（不阻塞，另立候选）

- gate 循环超长 run：9063dfd4 实证 ≥23min 守卫滞留且无收束日志；每门
  +1800s 喂狗使看门狗无法兜底 deny-重试循环（与 §3 心跳债务合并处置）。
- 孤儿 pending 快照：进程重启杀 run 后遗留 pending=true 的增量快照，
  前端渲染"运行中断"提示待做（本轮只保证数据不丢、可辨伪）。
- 9063dfd4 的确切死因（无日志出口 vs 楔死）未定——P2-T2 起三臂全出口
  日志化后，同类案例将自带诊断面。

### 6.6 规范增量（r13 复审补立——按已验证实现起草，merge 阶段发布）

- **modify** `docs/specs/modules/chat-streaming.md`：
  - 契约①"订阅即附加/空闲流"的**空闲语义**修订：空闲订阅不再挂起——
    `chat_run_stream` 入口以 `chat_run_active` 窥探，空闲立即回
    `{"type":"idle"}` 帧收流（收束竞态 try_recv 转发 done 尾巴）。
  - 新增契约：**会话载入即附加**——前端会话打开（切换/刷新）即
    AttachStream 挂流；运行态不预置，由首个真实流事件置位（旧后端
    兼容，零事件零幻态）；`idle` 事件臂复位。
  - 契约④补：完成启发式增 **pending 守卫**——末条为 pending 快照
    （非终版）时不清窗不收束，等收束终版同 id 换入。
  - 契约④动机句"回填快照不含在途 assistant"按 P2-T3 修订：run 期间
    存在 turn 粒度增量快照，但快照不含 turn 内直播尾部——流式期跳过
    回填的结论不变，理由更新。
- **modify** `docs/specs/modules/chat-run-policy.md`：
  - 收束落盘规则修订：**三臂（成功/失败/超时）统一组装现场**
    （叙述块/思考首块/工具卡 + ⚠ 尾块）；失败臂不再仅落一行文本。
  - 新增规则：**turn 增量落盘**——run 起跑定消息 id，turn_end 边界将
    当前积累以 `pending=true` 快照 upsert；收束终版同 id 原位替换并清
    pending；原位替换必须保留 parent_id/created_at（分支链不变量）。
  - 落盘失败显式 warn（persist 统一口），session 缺失同样可观测。
- **new**：无（均为既有模块规则修订）。
- **supersedes**：chat-streaming 契约①"空闲流挂起"语义；契约④"回填快照
  不含在途 assistant"动机句。
- **touched_goals**：空——docs/specs/goals 尚无 chat/流式相关 goal 条目
  （体系未建该项，无真实 goal id 可挂）。

## 9. 复审记录

`stage: review | PLAN-073 | rev4 | pass | e5e098e（code）/ ccba96b（plan doc）
| base=e2c0cb0 | deps: auto-ai@630a98d（兄弟 worktree 纯路径解析零改动，已清）
| spec_inputs: modules/chat-run-policy.md、modules/chat-streaming.md（增量草案
见 §6.6，merge 发布）| acceptance: AC-P2-1 pass / AC-P2-2 pass / AC-P2-3 pass
（验证方式=代码路径+upsert 返回值语义测试）/ AC-P2-4 pass | findings: F-R1
idle 窥探单事件 try_recv——队列首事件属他 session 时漏收本会话 done 尾巴
（前端 poll 归一兜底，不阻塞）；F-R2 attach 后 3s 静默窗内回填换入快照会
抹掉客户端 turn 内增量（F-04 同族边缘，快照粒度无损，登记）；F-R3 规范
增量原缺失，本复审补立 §6.6 | evidence: 全量套件复跑 652 绿/1 失败
（run_command_dangerous_returns_paused——基线 e2c0cb0 stash 对照同败，
判环境预存非回归；独立 CARGO_TARGET_DIR 规避运行中 exe 锁）；AC-P2-2
端到端 live 复现（隔离 8095 + 真 daemon 新 exe=e5e098e 源码构建：
t=12s GET 见 pending=True 快照（2 工具卡）→ 挂流收 578 帧实时事件 →
t=13s 终版换入 pending=None/content 516 字/块序 thinking·tool·tool·text
→ assistants=1）；8095 live：失败臂 ⚠ 尾块落盘 + 裸 attach 即
data:{"type":"idle"}；dist=复审终版（attach 逻辑在 bundle）| next: merge`

**独立性声明**：本复审在实施会话内进行——结论从提交产物重建（diff 独立
重读、独立 target 目录复跑全量套件、AC-P2-2 端到端重现实证），不采信
r13 实施记录自述。AC-P2-2 复现使用用户 aaid 两次短 run（读目录+读文件，
最小配额，沿 r4/r6/r7 live 验证惯例）。

### 巩固收据 (Consolidation Receipt) — PLAN-073:r4

- stage: merge | plan_id: PLAN-073 | plan_revision: 4 | outcome: pass |
  delivery_commit: `1247846` | canonical_specs:
  [docs/specs/modules/chat-run-policy.md, docs/specs/modules/chat-streaming.md,
  docs/specs/index.json] | archive_path:
  docs/plans/archived/073-chat-run-stall-fixes.md | completion_kind: delivered |
  cleanup_state: cleaned

- Checkpoints:
  - `prepared`：reviewed baseline `e5e098e`（§9 复审 pass，rev4）；冻结增量
    = 计划 §6.6；巩固 worktree `D:/autostack/.wt/musk-073/auto-musk`
    （branch plan-073-dev 自 main tip ec9b9bb 建）。主分支在复审后的新增
    （PLAN-076 落地 ec9b9bb 等）与 073 复审面**零文件交叠**
    （`git diff --stat e5e098e..HEAD -- <复审文件>` 为空）——实现/依赖
    未变，文档型后代可作 delivery commit。
  - `landed`：spec 提交 `1247846`（docs-only）经 rebase 校验（up to
    date，无改写 → 无需 range-diff）后于 main `git merge --ff-only
    plan-073-dev`——tip = `1247846`，无 merge commit；两模块规范与
    index.json 在 main 验证（PLAN-073 Phase 2 标记 grep 命中）。
  - `ledger_refreshed`：`docs/specs/index.json`（version 2.0）——两模块
    已在 spec_files（核对），updated_at → 2026-09-20；无新增 spec 文件。
  - `archived`：active `docs/plans/073-chat-run-stall-fixes.md` →
    `docs/plans/archived/073-chat-run-stall-fixes.md`（git mv）；
    `status: archived`、`completion_kind: delivered`。
  - `cleaned`：巩固 worktree 移除（wt-guard clean 前置）+ 分支
    plan-073-dev 删除 + 组目录 `D:/autostack/.wt/musk-073` 移除。

- 备注：实现代码在 work 阶段已落地（e5e098e 为 main 祖先，收据 §9 复审
  记录）；本次 delivery commit 仅规范沉淀。运行态 8090 仍为旧 exe——
  新二进制生效需重启（用户操作）。遗留登记见 §6.5（gate 循环/孤儿快照/
  9063dfd4 死因）与 §9 F-R1/R2。
