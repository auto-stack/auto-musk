// mention_helpers.ts — MentionInput/MentionDropdown 过滤/定位/输入逻辑逃生舱（Plan 023 队列 B3/B5）
//
// 对标 src/front/components/MentionDropdown.vue + MentionInput.vue（逃生舱，已删除）。
// .at 无法表达：filter+includes、DOMRect 位置对象构造（window 宿主）、findIndex、
// @mention 检测（regex/slice/lastIndexOf）、插入文本拼接、DEFAULT_PROFESSIONS 兜底。

// Plan 028 T9: forge_helpers.ts 已原生化（forge_helpers.at）。mention 域
// （DEFAULT_PROFESSIONS/buildMentionNames/renderMentions/renderInputMentions/
// resolveMention）依赖回调式正则 replace，超出 F4 可移植子集，就近留在本文件
// （归后续 G-对话壳/输入组立项迁移）。

/** 默认职业列表（useAgentConfigs 当前是空 stub，故内置）。 */
export const DEFAULT_PROFESSIONS: { id: string; name: string }[] = [
  { id: 'assistant', name: 'Assistant Agent' },
  { id: 'advisor', name: 'Advisor' },
  { id: 'architect', name: 'Architect' },
  { id: 'planner', name: 'Planner' },
  { id: 'coder', name: 'Coder' },
  { id: 'tester', name: 'Tester' },
  { id: 'reviewer', name: 'Reviewer' },
  { id: 'documenter', name: 'Documenter' },
  { id: 'gofer', name: 'Gofer' },
]

/** id → 显示名映射（用于 @mention 高亮）。 */
export function buildMentionNames(
  professions: { id: string; name: string }[] = DEFAULT_PROFESSIONS,
): Map<string, string> {
  const names = new Map<string, string>()
  for (const p of professions) {
    names.set(p.id.toLowerCase(), p.name)
    names.set(p.name.toLowerCase(), p.name)
  }
  return names
}

/** 转义 HTML，然后把 @mention 包成高亮 span。 */
export function renderMentions(
  text: string,
  names: Map<string, string> = buildMentionNames(),
): string {
  const escaped = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
  return escaped.replace(/@(\w+)/g, (match, name: string) => {
    const displayName = names.get(name.toLowerCase())
    if (displayName) {
      return `<span class="inline-mention">@${displayName}</span>`
    }
    return match
  })
}

/** 同 renderMentions，但末尾加换行（输入框 backdrop 用）。 */
export function renderInputMentions(
  text: string,
  names?: Map<string, string>,
): string {
  if (!text) return ''
  return renderMentions(text, names) + '\n'
}

/** 把 @mention 词解析为 profession_id。 */
export function resolveMention(
  word: string,
  professions: { id: string; name: string }[] = DEFAULT_PROFESSIONS,
): string | undefined {
  const lower = word.toLowerCase()
  if (professions.some((c) => c.id.toLowerCase() === lower)) return lower
  const match = professions.find((c) => c.name.toLowerCase() === lower)
  return match?.id
}

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

// ─── MentionInput 输入逻辑（Plan 023 队列 B5）───

/** 发送按钮可用性（文本非空白）。 */
export function mentionCanSend(text: string): boolean {
  return !!text.trim()
}

/** trim（.at 无法在 view 事件参数里调用宿主方法）。 */
export function mentionTrim(text: string): string {
  return text.trim()
}

/** 按键名（.at handler 参数字段访问不可靠，用 fn 收敛）。 */
export function mentionKeyAction(e: any): string {
  return e?.key || ''
}

/** professions 列表兜底（props.professions > 动态 configs > DEFAULT_PROFESSIONS）。 */
export function mentionProfessionsList(propProf: any, configs: any): any[] {
  if (propProf && propProf.length) return propProf
  const dynamic = (configs || []).map((c: any) => ({ id: c.profession_id || c.id, name: c.name }))
  return dynamic.length > 0 ? dynamic : DEFAULT_PROFESSIONS
}

/**
 * @mention 检测（对齐原 handleInput：光标前找 @ → 行首/空白后 → @后无空格 →
 * 记录 filter/anchor/visible/pos）。.at 无法表达 DOM 读取 + regex + slice。
 */
function mentionDetect(e: any): { filter: string; anchor: any; visible: boolean; pos: number } {
  const el = e?.target
  const val = el?.value || ''
  const pos = el?.selectionStart ?? val.length
  const textBeforeCursor = val.slice(0, pos)
  const atIdx = textBeforeCursor.lastIndexOf('@')
  if (atIdx >= 0) {
    const charBefore = atIdx > 0 ? val[atIdx - 1] : ''
    if (charBefore === '' || /\s/.test(charBefore)) {
      const afterAt = textBeforeCursor.slice(atIdx + 1)
      if (!afterAt.includes(' ')) {
        return { filter: afterAt, anchor: el.getBoundingClientRect(), visible: true, pos }
      }
    }
  }
  return { filter: '', anchor: null, visible: false, pos }
}

export function mentionDetectFilter(e: any): string {
  return mentionDetect(e).filter
}
export function mentionDetectAnchor(e: any): any {
  return mentionDetect(e).anchor
}
export function mentionDetectVisible(e: any): boolean {
  return mentionDetect(e).visible
}
export function mentionDetectPos(e: any): number {
  return mentionDetect(e).pos
}

/**
 * 插入 mention 文本（对齐原 handleMentionSelect：@ 位置前插 @name + 保留光标后文本）。
 * pos 在 input 时捕获（.at 无 template ref 取实时 selectionStart）。
 */
export function mentionInsert(professions: any[], id: string, val: string, pos: number): string {
  const name = professions.find((p: any) => p.id === id)?.name || id
  const textBeforeCursor = val.slice(0, pos)
  const atIdx = textBeforeCursor.lastIndexOf('@')
  if (atIdx >= 0) {
    const before = val.slice(0, atIdx)
    const after = val.slice(pos)
    return `${before}@${name} ${after}`
  }
  return `@${name} ${val}`
}
