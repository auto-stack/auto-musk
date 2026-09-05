// focus_composer.ts — PLAN-061 T16 (D22): composer 聚焦逃生舱（web）。
//
// 新建会话后自动聚焦输入框（此前点 + 后需再手点输入框才能打字）。
// .chats-input 是 composer textarea 的稳定类（mention_input.at:123,
// 常驻挂载——NewSession 不重建输入条,无需等重渲染即可 focus）。
// 时序注意:handler 内同步 focus() 会被点击链的默认焦点（聚焦被点的
// + 按钮本身）覆盖——让到下一任务拍(setTimeout 0;不用 rAF,IAB/后台
// 页 rAF 停帧会饿死)。
export function focusComposer(): void {
  setTimeout(() => {
    // 收窄 textarea.chats-input:类名 wrapper div 与 textarea 双持有,
    // 裸 .chats-input 首匹配落不可聚焦的 wrapper。
    document.querySelector<HTMLTextAreaElement>("textarea.chats-input")?.focus()
  }, 0)
}
