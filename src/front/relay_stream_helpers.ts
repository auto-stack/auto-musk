// relay_run_helpers.ts — RelayRunBox 数据/订阅逻辑逃生舱（Plan 023 队列 A3）
//
// 对标 src/front/components/RelayRunBox.vue（逃生舱，已删除）。useRelay 是模块级
// 单例状态，helper 内 useRelay() 与 component fn 的 composable facade 共享同一 state。
//
// .at 无法表达：订阅闭包存储（unsubscribe 按 runId 存模块 Map）、数组 find、字典
// 映射（statusLabel/professionIcon）。放逃生舱 fn，component fn 经 use { fn } 引入。

import { useRelay } from './composables/useRelay'

// 订阅取消函数按 runId 存模块级 Map（.at 无法存闭包；component fn 用
// subscribed 标记 + .Destroy 调用 unsubscribe）。
const unsubMap = new Map<string, () => void>()

/** 订阅 run 的 SSE 日志流（幂等：已订阅则不重复订阅）。 */
export function relayRunSubscribe(runId: string): void {
  if (unsubMap.has(runId)) return
  unsubMap.set(runId, useRelay().subscribeToRun(runId))
}

/** 取消订阅（幂等）。 */
export function relayRunUnsubscribe(runId: string): void {
  const u = unsubMap.get(runId)
  if (u) {
    u()
    unsubMap.delete(runId)
  }
}

/** 加载 run 详情（内部 try/catch，不抛异常——.at 侧无需再包 try）。 */
export async function relayRunLoad(runId: string): Promise<void> {
  await useRelay().loadRun(runId)
}

/** 审批通过（resolveGate 内部 try/catch，不抛异常）。 */
export async function relayRunApprove(runId: string): Promise<void> {
  await useRelay().resolveGate(runId, 'approve')
}

/** 拒绝（带 '需要修改' feedback）。 */
export async function relayRunReject(runId: string): Promise<void> {
  await useRelay().resolveGate(runId, 'reject', '需要修改')
}

/** 取 run 的会话日志（.at 无此法直接调 composable 方法时的响应式保证）。 */
export function relaySessionLogFor(runId: string): any[] {
  return useRelay().sessionLogFor(runId)
}
