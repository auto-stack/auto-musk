// secretary_helpers.ts — SecretaryMessage 时间格式化逃生舱（Plan 023 队列 B4）
//
// 对标 src/front/components/SecretaryMessage.vue（逃生舱，已删除）的 formatElapsed。
// .at 无法表达 Date.now() 时间差计算与多层分支，放逃生舱 fn。

/** 距 since 的时间标签（just now / Xm / Xh / Xd）。 */
export function secretaryFormatElapsed(since: number): string {
  const mins = Math.floor((Date.now() - since) / 60000)
  if (mins < 1) return 'just now'
  if (mins < 60) return `${mins}m`
  const hrs = Math.floor(mins / 60)
  if (hrs < 24) return `${hrs}h`
  return `${Math.floor(hrs / 24)}d`
}
