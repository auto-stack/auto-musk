// session_info_helpers.ts — SessionInfo 数据/宿主 API 逻辑逃生舱（Plan 023 队列 B 续）
//
// 对标 src/front/components/SessionInfo.vue（逃生舱，已删除）。
// .at 无法表达：errands 对象遍历求和、navigator.clipboard + setTimeout 回调闭包
// （copy 完成态用共享 ref）。放逃生舱 fn。

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

/** errands 对象 token_usage 求和（对齐原 tokenCost）。 */
export function sessionTokenCost(errands: any): number {
  let total = 0
  if (errands && typeof errands === 'object') {
    for (const key in errands) {
      total += (errands as any)[key]?.token_usage || 0
    }
  }
  return total
}

/** messages 数量兜底（?.length || 0）。 */
export function sessionMessageCount(messages: any): number {
  return messages?.length || 0
}
