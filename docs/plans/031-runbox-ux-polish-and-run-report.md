---
plan_id: PLAN-031
status: executing
feature_name: RunBox 体验细化（批次二）+ Run 完成报告（Report v1）
author: [zhaopuming]
created_at: 2026-08-21T21:00:00+08:00
updated_at: 2026-08-21T21:00:00+08:00

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 8
total_steps: 8
---

# [PLAN-031] RunBox 体验细化（批次二）+ Run 完成报告（Report v1）

## 0. 变更摘要

批次二聚焦两件事：**(A) RunBox 内外视觉/交互与普通 Block 全面对齐**（预览条排版、
折叠 UI 统一、正文 AutoDown 渲染）；**(B) Run 完成闭环**——完成时后端确定性组装
Run 报告（步骤/令牌/文件/时长/摘要）随 `run_completed` 事件下发，激活双轨沉睡的
ReportCard（PPT 风格），并向**父 chat 会话**追加总结消息（此前 run 完成后 chat
无任何总结输出——spawn_relay 异步脱离，watcher 只镜像状态到 run 自身会话）。

## 1. 前情：已完成（批次零/一，2026-08-21，试用反馈驱动）

| # | 事项 | 提交 |
|---|------|------|
| 1 | SSE 具名事件全链路修复（三处未命名帧 hw+ag 双轨；019 同款复发） | 5b42ccf |
| 2 | .at store 链式吞噬三处（StartRun/AdvanceRun/ResolveGate 花括号包裹） | 5b42ccf |
| 3 | gate_resolved 进 SSE 总线 | 5b42ccf |
| 4 | 1214 悬案：memory 裁剪吞首条 user → 裁剪器补锚点 + aaid 兜底（auto-ai） | ef13058→ |
| 5 | RunBox 收起态：SSE 挂载即订阅/展开即刷新/内联审批条/工具卡对齐聊天侧 | 6cd7d63, d817d29 |
| 6 | RunBox 活性指示：运行态图标旋转 + 收起态最新动态预览条 | c812903 |
| 7 | 消息块展示顺序统一（文字/思考在前、工具块在后） | 8d74dcd |
| 8 | 预览条 3 行（流式文本=尾部 3 行）；进度徽标 1-based + 迷你分段条 + hover 步骤清单；工具卡全宽（.entry-tool 行级 flex 收缩根因）；终态停闪（圆点常亮着色 + tool_call "已中断"）；gen 轨 .missing 未定义暗 bug；持久化 events 补生命周期映射；substring 两参语义陷阱 | b9704e1 + 修正提交 |
| 9 | 环境：8080 被 demo 进程劫持致 agent E2E 死循环（循环保护停车=正确行为）——已清进程释放端口 | — |

## 2. 新需求（本批实施）

### T1 预览条排版 v2（截图反馈 1/2/3）
- 圆点对齐**最末一行**（`align-items: flex-end` + 末行居中边距），替代现顶部。
- 行距加大：`gap 0.08rem → 0.2rem`，`line-height 1.35 → 1.45`。
- 预览行**结构化着色**（不再纯灰文本）：`relayPreviewLines` 返回行对象
  `{mark, name, target, text, text_class}`；工具行 = 🔧 + 工具名（前景色/500）+
  目标（青色 monospace，对齐 Block 头 `.tool-name`/`.tool-target` 口径）；
  错误行红、完成行绿、步骤行 muted。不做标题栏底色（仅内部排版对齐 Block 头）。

### T2 RunBox 头部与其它 Block 统一（截图反馈 4）
- **决策：右侧上下箭头**（ChevronDown=收起 / ChevronUp=展开）——采纳多数派
  （聊天侧工具卡/ErrandCard/ReportCard 全是右侧上下箭头）；移除左侧右/下箭头。
- Orbit 运行图标留左（Block 身份位）但改**主题色**（`hsl(var(--primary))`）。
- 职业图标映射补 `plan-dev`（现 fallback ⚙️ 出现在每条文本前）。

### T3 展开态正文 AutoDown 渲染（截图反馈 5）
- `text` 条目：`Markdown`（platform:markdown / StreamingRenderer）渲染，与
  chat 一致；保留职业图标前缀。
- `thinking` 条目：新增分支（muted 斜体摘要行），此前无分支不渲染。

### T4 Run 完成总结回流 chat（截图反馈 6 前半）
- `orch_tools.rs` spawn_relay 的 detached watcher 在 `completed` 分支：取 run
  报告（T5 的 store 访问器），向**父会话**（`ctx.parent_conversation_id` 捕获）
  `append_turn` 一条 assistant `Message`（总结文本，Markdown）。
- 依赖 conversation SSE 推送，chat 实时出现总结消息；持久化天然可回放。

### T5 Run Report v1（截图反馈 6 后半）
- 后端：`RunCompleted` 事件携带 `report` 载荷（`#[serde(default)]` 向后兼容），
  确定性组装（不加 LLM 调用）：`{run_id,title,summary(=末次 handoff 摘要,
  Markdown),goals_met("N/M"),tests_pass(=成功工具调用数),drift_detected("None"),
  cost(=累计 tokens),confidence,deliverables(=变更文件清单),files_changed,
  tool_calls,duration_s,completed_steps,total_steps}`。三个追加点
  （advance/submit_handoff/resolve_gate）统一走 `build_run_report(entry)`。
- store 增加 `run_report(run_id)` 访问器（watcher 用）。
- 前端激活：web 轨 ChatsView `onReport` 路由已存在（`run_completed` →
  ReportCard）——payload 即活；gen 轨 `OnRelayEvent(run_completed)` 存
  `run_reports[runId]` + LoadRun 回扫 events，RunBox 完成态底部内嵌 ReportCard。
- ReportCard **PPT 风格升级**（双轨）：hero 区（渐变底、标题、状态、时长）+
  摘要 Markdown 渲染 + 指标格（步骤/工具调用/令牌/时长）+ 交付物 chips +
  保留操作行（Download 用真实摘要内容）。

### T6 一致性原则（贯穿）
> RunBox 内外的 Block 展示应基本一致，唯一区别是多套一层可折叠的 Run 窗口。

### T7 交付
- `auto build --gen-only` + web dist 重建 + musk 后端重建重启（8090）。
- E2E：真实 run（小任务）→ gate 批准 → completed：验证 events 内报告字段、
  SSE 帧、父会话总结 turn、双轨 ReportCard 数据通路。
- 提交。

## 3. 验收标准

1. 收起态预览：圆点与末行齐平；行距舒适；工具行名称与路径分色。
2. RunBox 头部：右侧上下箭头折叠（与工具卡一致）；运行图标主题色。
3. 展开态文本为 Markdown 渲染；不再出现 ⚙️+纯文本。
4. run 完成后：chat 出现总结消息；出现 PPT 风格 ReportBlock（web 全局位 /
   gen RunBox 内嵌）；刷新后报告可回放（events 持久化）。
5. `cargo test -p musk` 绿；web `vue-tsc && vite build` 绿；gen 可构建。

## 4. 风险与边界

- `RunCompleted` 加字段：serde `#[serde(default)]` 保证旧持久化 events 反序列化
  兼容；SSE 消费方（eventRouter）只读新增字段，无破坏。
- watcher 追加父会话 turn：仅 `completed` 分支一次性追加（幂等性靠 watcher 单次
  退出语义）；失败分支本批不追加（v2 可加失败摘要）。
- Report v1 摘要取 document 相位 handoff 摘要（LLM 产物），无独立"漂亮报告"
  生成调用——PPT 化交给前端呈现层；v2 再考虑专用报告生成相位。

## 2b. 批次三（历史回放修复 + 阶段可视化，试用反馈截图×2）

### T8 回放重建 web 轨补齐（反馈 1/3：已中断误显 + 目标消失）
- 根因：web `loadRunHistory` 的 tool_call 条目**不带 arguments、不合并
  tool_result**（.at 轨有、web 漏）——目标列空 + 条目恒为 `tool_call` 型，
  终态下全部误显"已中断"。补齐与 .at 同口径。

### T9 展开点击跳滚修复（反馈 2）
- 根因：web `watch(logEntries, {deep:true})` 在 `_expanded` 变更时也触发
  自动滚底——展开内容被滚出视口，表现为"没打开+跳到最后一段"。
  改为仅监听条目**长度**变化。

### T10 文本行去图标 + 全宽对齐（反馈 4）
- 去掉 📝 职业图标（单独占行）；Markdown 全宽，与工具块左对齐。

### T11 阶段分割线（反馈 7）
- step_started/completed/gate_waiting/run_completed/run_failed/error 改为
  分割线风格：`── ▶ 方案 ──`（::before/::after 横线 + 中文阶段名，完成绿/
  门禁琥珀/失败红）。'Step "x" started' 解析 → relayStepIdOf + relayStepLabel。

### T12 PLAN_FILE/长文本 → 折叠文档块（反馈 5）
- PLAN_FILE 行抽为独立文件 chip（📄 + mono 青色路径）；
  正文 ≥600 字符或含 PLAN_FILE → 折叠 Markdown 文档块（头部 📄+首个 #
  标题，默认收起，展开渲染 AutoDown）。

### T13 RunCompleted 重复追加修复（反馈 6："Run 已完成"×2）
- 后端根因：driver 循环在 submit_handoff 返回 Completed 后的下一轮
  advance 对已 Completed 的 run 再次追加 RunCompleted（会话双写
  "Flow completed"×2 实证）。advance() 加终态幂等守卫；
  双轨回放转换器加 sawCompleted 去重防御；标签去"（历史回放）"尾巴。

### T14 附带
- parity_relay_driver 过时断言修正（PLAN-030 置败停车改造遗留：期望
  error→handoff，现行 error→fail_run；按现行语义断言 RunFailed 事件标记）。

## 5. 执行步骤（批次三）

- [x] b1. 根因定位（会话 turns 解剖 + 双轨转换器对照 + "Flow completed"×2 实证）。
- [x] b2. T8-T12 前端（.at + web 双轨）+ codegen + web dist 重建。
- [x] b3. T13 后端 advance 终态守卫 + T14 测试修正；cargo test 全绿（31 组）。
- [x] b4. 真实会话数据模拟验证：25/25 工具合并（不再误显中断）、12 条带
       操作目标、run_completed 恰 1 条。
- [x] b5. musk 重建重启（8090）+ 提交。

## 5x. 执行步骤（批次一/二）

- [x] 1. 侦察：ReportCard 双轨现状（onReport 路由在、无发射方）、spawn_relay
       watcher、RunEvent 枚举、Turn/append_turn、originating_chat_session（未填充→
       用 ctx 捕获父会话）。
- [x] 2. 本计划文档。
- [x] 3. T1+T2+T3 前端（.at 源 + web 手写镜像）。
- [x] 4. T5 后端（RunReport + 三追加点 + 访问器）+ T4 watcher 回流。
- [x] 5. ReportCard PPT 升级（web 手写 + report_card.at）+ gen 激活接线
       （store run_reports + RunBox 内嵌）。
- [x] 6. codegen + web dist 重建 + musk 重建重启（8090）。
- [x] 7. E2E（真实小 run 全流程）+ cargo test（294+ 绿，含 tool_atoms 预存
       编译损坏顺手修复）。E2E 实证：4/4 步骤完成，run_completed 事件与 SSE
       实时帧均携带完整报告载荷（标题/摘要=文档相位 handoff/25 工具调用/
       2784 令牌/168s/2 个变更文件）。**T4（父 chat 总结 turn）为代码审读
       级验证**——REST E2E 无父会话，待用户下次 chat 发起 run 自然验证。
- [x] 8. 提交并更新本文件状态。
