# PLAN-065 冒烟清单——六件历史人工冒烟合并执行

> 服务拉起（dev-stack 惯例）：`musk serve --addr 127.0.0.1:9247`，CWD=tmp/musk-demo。
> 本清单由 PLAN-065 T7 建立；S1-S3/S6 需在**普通浏览器**执行（会话内置浏览器有
> webview 挂载限制，033 W2 两次复现）；S4-S5 需 **VM 实机**（iced）。结果回填
> ✅（通过）/ 🔶（残留，注记原因）。

## S1 · 033 W2 · Plans 视图 MetaBlock 渲染（浏览器 :9247）

- [ ] 步骤：打开 Plans 视图，任选一个计划进入详情；观察 MetaBlock 布局、
      状态徽标文案、按钮组排布。
- [ ] 预期：布局正常、徽标为中文、按钮组换行不破版。
- 顺带（PLAN-065 T5 目验）：切到 Specs 视图打开任一测试项详情，fixture
  代码块**首行无多余缩进**（pre→div + pre-wrap 修复）。
- 结果：＿

## S2 · 042 T10 · 真实 LLM 会话工具卡 details（浏览器）

- [ ] 步骤：发起真实 LLM 会话；让 agent edit 改一个文件、读一个大文件
      （触发截断）；刷新页面回放会话。
- [ ] 预期：工具卡 details 区正确显示（edit 显示 diff、read 显示截断信息），
      刷新后回放一致。
- 结果：＿

## S3 · 040-1 · run_command 流式进度（浏览器，可与 S2 同会话）

- [ ] 步骤：让 agent 跑一个长命令（如 sleep/ping 类），观察 tool_update
      流式进度。
- [ ] 预期：流式进度实时渲染。
- 结果：＿

## S4 · 493 · @mention 着色像素终验（VM 实机）

- [ ] 步骤：`AUTO_DEBUG_MENTIONS=1` 起实例，聊天输入 @ 触发 mention
      下拉并选中。
- [ ] 预期：日志出现 `[493-MENTIONS]` 段 + mention 文字蓝色着色像素目验。
- 结果：＿

## S5 · 050 #1/#3 · rail 导航像素目验（VM 实机）

- [ ] 步骤：观察 rail 导航栏；对照 050 修复点 items-baseline 居中与
      mt-auto 弹性占位。
- [ ] 预期：图标-文字纵向居中、底部项贴边、无破版。
- 结果：＿

## S6 · 061 D29 · IAB 语言切换复验（浏览器）

- [ ] 步骤：设置区切换界面语言 中↔英。
- [ ] 预期：全界面文案翻转（D29 i18n-instance 根修的活体复验）。
- 结果：＿

---

## 冒烟过程新发现（2026-09-07，执行 S2/S3 时实测）

均为**既有缺陷**（主检出同款生成物/源码一致，非 PLAN-065 T1-T6 引入）：

- **F1 · S1 · gen 轨徽标中文未落地**：`i18n/zh.json` 有全套状态键（statusDrafting=草拟 等）但 `plans_view.at` 零消费——详情 meta 芯片渲染原始英文 status（plans_view.at:198），四个状态流转按钮硬编码英文（:126-142）。033 #3 当年只在 web 轨落地（PlanStatusBadge t(planStatusKey)），063 T8 移植时未带 i18n。
- **F2 · gen 轨 composer @mention 检测链每键抛 TypeError**：codegen（051 P3-② v-model 优化）把 `oninput: .Input($event)` 编译为 `@input="Input(($event.target).value)"` 传**字符串**，而 `.Input(e)` 处理器（mention_input.at:64-72）期望**DOM 事件**（mention_detect_filter 读 e.target.value/getBoundingClientRect）→ `reading 'value' of undefined` 每键必抛。输入/发送不受影响（v-model 直更 text），坏的是 @mention 下拉检测。主检出 gen 生成物一字不差，同款既有。
- **F3 · gen 轨直播流不渲染（刷新才见回复）**：PollStream 每 500ms `chats_get_session` 快照**整体覆盖** .messages（forge_store.at:337-380"以轮询为最终态"），而 musk 后端 assistant 消息**完成时才持久化**（server.rs run_stream 收尾 append_message）——直播期间快照恒为 [user msg]，SSE delta 增量每拍被抹；done 事件随即关 deadman 窗（StopStream pop），完成后无补拍 → 界面停留空态直至手动刷新。VM 轨纯轮询无此问题；web 轨"轮询+SSE 共存"设计（051 T10）与后端持久化时序相悖。修法建议：streaming 期间不覆盖式回填（或 done 臂补一次最终回填）。
- **F4 · gen 轨无 tool_update 消费臂**：`[forge stream] tool_update` 落入 OnStreamEvent 末尾兜底日志（forge_store.at:617）——P040-2 的 run_command 流式进度在 gen 轨无渲染路径（S3 预期项），web 轨 useForge.ts 有（冻结不动）。
- **F5 · details 载荷不持久化（musk 后端契约缺口，双轨共有）**：`chats.rs` ToolCall 结构无 details 字段（实测落盘 JSON tool_calls 键仅 `arguments/id/name/result`）——SSE 直播路径 store 里有 details（generic_tool_card 的 diff/截断徽标/全量输出三区 computed 就绪），回放路径全部丢失。042 的"刷新后回放一致"对 details 面只成立一半：截断信息因 auto-ai read 自带分页提示而落在 result **文本**里可见，diff 专用区回放消失。修法：ToolCall 增 Option<details> 字段（serde skip_if None）+ 双写路径透传。
- **F6 · 一级导航文案硬编码中文（048 存量债，双轨共有）**：`app.at:93-113` 四个 rail 项为字面量 `text "会话"/"计划"/"规范"/"知识库"`——PLAN-048 因"VM 视图文本无裸调用臂,t() 恒空"字面量化，字符串从未接 t()，D29 的 i18n-instance 根修无法覆盖。S6 实测：二级菜单（走 t()）翻转正常，一级导航恒中文。

---

## 汇总（T8 回填后填写）

| # | 出处 | 面 | 结果 | 注记 |
|---|------|----|------|------|
| S1 | 033 W2 | 浏览器 | 🔶 | 芯片行(MetaBlock 对应物)布局/按钮组正常（截图）；徽标中文未落地=F1。附件项(T5 fixture 目验):全库零 `**Fixture:**` 测试项,区块从不渲染属预期,无活体标本——生成物级验证充分(div+pre-wrap;伪影根源=Vue 编译器仅 pre 内保留模板缩进),待台账登记首个 Fixture 项时顺带目验销注 |
| S2 | 042 T10 | 浏览器 | 🔶 | ARGUMENTS/RESULT 渲染正确且回放一致（截图实证）；big-dump 截断提示在 result 文本可见（50KB 分页+offset，auto-ai read 自身机制）；042 结构化 details 区回放丢失=F5；RESULT 无语法高亮（小注记，纯文本区不走 markdown 管线） |
| S3 | 040-1 | 浏览器 | 🔶 | 服务端 tool_update 流式事件已证实到达（console `[forge stream] tool_update`）；gen 轨无消费臂（F4）+ 直播视图被轮询覆盖（F3）→ 实时渲染不可达；web 轨当年已验收（冻结） |
| S4 | 493 | VM | 🔶 | 输入 @ 无反应 + 全程零 `[493-MENTIONS]` 日志（该日志每帧视图构建即打,零日志=链路未武装）。候选根因:env 未达 VM 进程 / mentionNames 空（AgentConfigs 拉取失败→降级无高亮）。归 VM 专项复验,残留 |
| S5 | 050 | VM | ✅ | rail 导航（左侧竖条:标题行+四菜单项）图标-文字纵向居中、底部贴边、缩放不破版——用户目验通过（2026-09-07） |
| S6 | 061 D29 | 浏览器 | 🔶 | 二级菜单翻转正常（D29 根修工作）;一级导航四项恒中文=F6（048 字面量化存量,字符串未接 t(),根修管不到） |
