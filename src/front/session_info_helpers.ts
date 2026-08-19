// session_info_helpers.ts — SessionInfo 剪贴板逻辑逃生舱（仅剩 copy 部分）
//
// Plan 029 T7：token 求和/消息计数已迁 session_info_helpers.at（单一真源）。
// 本文件仅保留 navigator.clipboard + setTimeout 回调闭包（copy 完成态共享
// ref），待 Phase C（T21）dom.copy 落地后并入 .at 并删除本文件。

import { ref } from 'vue'

/** 复制完成态（共享 ref——.at 无法存 setTimeout 回调闭包，computed 读取）。 */
export const sessionCopied = ref(false)

/** 复制 Chat ID（navigator.clipboard + 2s 后自动复位）。 */
export function sessionCopyId(sessionId: string): void {
  if (!sessionId) return
  navigator.clipboard.writeText(sessionId).then(() => {
    sessionCopied.value = true
    setTimeout(() => {
      sessionCopied.value = false
    }, 2000)
  })
}
