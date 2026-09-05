// focus_composer.ts — PLAN-061 T16 (D22): composer 聚焦逃生舱（web）。
//
// 新建会话后自动聚焦输入框（此前点 + 后需再手点输入框才能打字）。
// .chats-input 是 composer textarea 的稳定类（mention_input.at:123,
// 常驻挂载——NewSession 不重建输入条,无需等重渲染即可 focus）。
export function focusComposer(): void {
  const el = document.querySelector<HTMLTextAreaElement>('.chats-input')
  el?.focus()
}
