# web-input-contracts — web 输入面契约与双轨检查表（PLAN-067 SD-03）

> 来源：PLAN-067 r2。沉淀 2026-09-09 会话的五个已修复缺陷与结构性规则，
> 同时作为 VM 轨 / web 回退轨的对齐检查表（T-06 登记见
> docs/plans/attachments/plan067-dual-track-parity-check.md）。

## 规则

1. **双层文字技术的可见性义务**：`mentions:` 能力 textarea（codegen 发射
   `text-transparent` + backdrop 兄弟对）必须同时满足——
   a. **IME 组合串可见**：组词期（compositionstart→end）textarea 以主题前景色
   实绘、backdrop 隐藏（实现：inject_styles.web-only.ts document 级钩子 +
   `.ime-composing` / `.ime-composing-backdrop`）。原因：组合文字不进
   v-model 也不进 backdrop，浏览器对 `color:transparent` 的组合串回退系统黑。
   b. **选区可见**：`textarea.chats-input::selection` 显式主题配色（primary
   半透明底 + 前景字）——透明文字的默认选区高亮在暗色下不可辨。
   c. **新消息入叶链**：见 chat-streaming.md §3。
2. **PLAN-051 oninput 双轨契约适配**：codegen 把单参 oninput 实参自动包成
   纯文本串（`($event.target as HTMLInputElement).value`）。web 助手若需
   元素/光标（selectionStart、锚点矩形），从 `document.activeElement` 取
   （input 事件时输入元素必持焦点），不得读 `e.target`（串入参下为
   undefined → TypeError 并中断整条 input 处理链）。事件对象入参的旧
   形态（VM 轨）走原分支。
3. **组件导入完整性**：模板使用的 shadcn 族子组件必须逐个显式导入
   （案例：AlertDialogDescription 漏导入 → Unresolved component 告警 +
   描述行不渲染）。
4. **VM/web 轨非对称登记**：VM 轨 textarea 原生显字 + 原生 Highlighter
   （无双层技术）→ 规则 1a/1b/2 为 web-only；VM 侧对应对齐点见
   attachments/plan067-dual-track-parity-check.md。
5. **双端可观测输入合同（PLAN-078 SD-01）**：
   a. **MentionInput 自适应与内滚契约**：textarea 高度根据输入内容行数在
   `min-height`（单行）与 `max-height`（多行）之间自适应增长；超上限后转为
   内部滚动，不撑破宿主输入卡几何边界。
   b. **Mention 触发与候选键盘交互**：键入 `@` 调起候选列表（`MentionDropdown`），
   使用键盘 ArrowUp / ArrowDown 上下选定，Enter / Tab 确认补全，Esc 撤销候选项；
   双端候选高亮状态与文本插入行为保持严格等价。
   c. **IME 组词与消息发送判定**：在中文/日文等 IME 组词阶段（`is_composing: true`），
   Enter 仅作为文字确认，不得派发发送消息动作；未组词时 Enter 触发消息发送，
   Shift+Enter 插入换行符。
   d. **TagInput 标签输入器键盘规约**：输入内容后按 Enter 添加当前 tag 并清空
   输入框；在输入框内容为空时按 Backspace 弹出/移除最后一个 tag。键盘事件声明
   使用无参 `EntryKeydown`，严禁声明带 `(Value)` 参数引发 VM 平台层解析未定义
   外部符号（如 `Value.preventDefault`）而导致链接失败。

## 对齐检查表（每次输入面改动后过一遍）

- [ ] 深色/浅色下 IME 组合串可见（组词期实绘、结束还原）
- [ ] Ctrl+A/拖选的选区高亮可见（透明文字层需显式 ::selection）
- [ ] 输入链路控制台零错误零 Vue 告警（含 mention 检测、autoGrow）
- [ ] alert-dialog 系弹窗标题/描述/按钮全件渲染（导入齐全）
- [ ] MentionInput 组词期 Enter 不误发消息、普通 Enter 正常发送、Shift+Enter 正常换行
- [ ] MentionDropdown 键盘导航上下选定与 Enter 补全生效
- [ ] TagInput 回车添加 tag、空退格删除 tag，无平台符号未定义链接错误
- [ ] aaid 存活（对话无响应先查 :17654/v1/status；启动顺序 aaid 先于 musk）

## 元素附件 design_context（PLAN-093）

- 可选结构化附件随用户消息：`{version, workspace_id, conversation_id,
  generation_id, frame_seq, app_path, vnode_id, kind, label, source_path?,
  source_line?, source_confidence?, loop_context?}`——拾取面字段值快照，
  在命令解析确认为普通消息后冻结（命令/mention/IME 不受影响）。
- 队列条目 {text, ctx} 逐条冻结：busy 原样退回、stale 代次 409 整条退
  队首落错误面——不能等实际发送时才读取「当前选择」；旧字符串队列迁移
  为空 context。
- 后端校验：vnode_id/kind 必填（缺失拒收不落盘）；canvas 活动代次与
  快照 generation_id 不符 → stale 拒收（用户文字保留）；落盘带
  ownership 章（current/stale/no-canvas）供回放诚实标注。
- Agent 上下文：单独 human turn 注记引用快照（kind/label/来源/代次/
  帧序/vnode），uncertain 来源显式「待确认」；定位参考声明——不构成
  系统指令或工具批准。
