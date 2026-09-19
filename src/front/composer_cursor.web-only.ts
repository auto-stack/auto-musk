// composer_cursor.web-only.ts — r10（PLAN-073 续）：引用 token 原子删除后
// 的光标恢复。受控 textarea 的 value 由 Vue patch **异步**回写（光标被
// 重置到末尾），恢复动作必须发生在 flush 之后——setTimeout(0) 宏任务兜住。
// 定位用 .chats-input 稳定类（focus_composer.ts 同款口径）。VM 轨无此概念
// （text_editor 光标自管），端口侧 no-op。

export function restoreComposerCursor(pos: number): void {
  window.setTimeout(() => {
    const ta = document.querySelector('textarea.chats-input') as HTMLTextAreaElement | null
    if (ta) ta.setSelectionRange(pos, pos)
  }, 0)
}
