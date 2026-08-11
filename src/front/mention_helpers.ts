// mention_helpers.ts — MentionDropdown 过滤/定位逻辑逃生舱（Plan 023 队列 B3）
//
// 对标 src/front/components/MentionDropdown.vue（逃生舱，已删除）。
// .at 无法表达：filter+includes、DOMRect 位置对象构造（window 宿主）、findIndex。

/** 按 filter 过滤 professions（id/name 大小写不敏感 contains，对齐原 filtered）。 */
export function mentionFiltered(professions: any[], filter: string): any[] {
  const f = (filter || '').toLowerCase()
  return (professions || []).filter(
    (p) =>
      (p.id || '').toLowerCase().includes(f) || (p.name || '').toLowerCase().includes(f),
  )
}

/**
 * 锚点定位（对齐原 position computed：anchorRect 为 null 返回 {}）。
 * .at 无法构造对象字面量 → helper 返回 { position, left, bottom }。
 */
export function mentionPosition(anchorRect: any): any {
  if (!anchorRect) return {}
  return {
    position: 'fixed',
    left: `${anchorRect.left}px`,
    bottom: `${window.innerHeight - anchorRect.top + 4}px`,
  }
}

/** 当前高亮项 id（filtered[index]?.id，无则空串）。 */
export function mentionCurrentId(filtered: any[], index: number): string {
  return filtered[index]?.id ?? ''
}

/** 键盘导航索引 clamp（moveUp/moveDown 共用）。 */
export function mentionClampIndex(index: number, len: number): number {
  if (len <= 0) return 0
  return Math.max(0, Math.min(index, len - 1))
}

/** 鼠标 hover 按 id 定位索引（.at 无 findIndex）。 */
export function mentionIndexOf(filtered: any[], id: string): number {
  const i = filtered.findIndex((p: any) => p.id === id)
  return Math.max(0, i)
}
