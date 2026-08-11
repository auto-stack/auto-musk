// gate_helpers.ts — GateCard 审批门数据逻辑逃生舱（Plan 023 队列 B2）
//
// 对标 src/front/components/GateCard.vue（逃生舱，已删除）。
// .at 无法表达：Set（expandedDiffs 用 obj 代替）+ 对象 key 存在性检查 + 动态键
// 记录初始化。放逃生舱 fn。

/** 展开态判断（.at 无 Set，用 obj 记录；动态键 obj 访问用 fn 收敛）。 */
export function gateExpanded(expanded: any, sid: string): boolean {
  return !!expanded[sid]
}

/** toggle 展开态（返回新值）。 */
export function gateToggleExpanded(expanded: any, sid: string): boolean {
  return !expanded[sid]
}

/**
 * 把展开态扁平化到每个 change（_expanded 字段）。view 级 if 不支持索引表达式
 * （.expanded[sid] 或多参 fn 调用），用字段访问 .change._expanded 替代。
 */
export function gateWithExpanded(changes: any[], expanded: any): any[] {
  return (changes || []).map((c: any) => ({ ...c, _expanded: !!expanded[c.section_id] }))
}

/**
 * 初始化 editedSpecs：为缺失的 section_id 填入 new_content（对齐原 watch
 * changes immediate+deep）。返回新记录（不就地修改）。
 */
export function gateInitEditedSpecs(changes: any[], edited: any): any {
  const result: any = { ...edited }
  for (const c of changes) {
    if (!(c.section_id in result)) result[c.section_id] = c.new_content
  }
  return result
}

/** title 兜底（对齐 withDefaults 默认文案）。 */
export function gateTitleText(title: string): string {
  return title || 'Spec drafted. Review the proposed changes below.'
}
