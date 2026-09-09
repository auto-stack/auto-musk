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

## 对齐检查表（每次输入面改动后过一遍）

- [ ] 深色/浅色下 IME 组合串可见（组词期实绘、结束还原）
- [ ] Ctrl+A/拖选的选区高亮可见（透明文字层需显式 ::selection）
- [ ] 输入链路控制台零错误零 Vue 告警（含 mention 检测、autoGrow）
- [ ] alert-dialog 系弹窗标题/描述/按钮全件渲染（导入齐全）
- [ ] aaid 存活（对话无响应先查 :17654/v1/status；启动顺序 aaid 先于 musk）
