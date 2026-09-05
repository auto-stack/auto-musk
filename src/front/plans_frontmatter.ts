// plans_frontmatter.ts — PLAN-063 T7 (D4): 计划 frontmatter 拆分真源
// (自 gen/front/vue/src/utils/frontmatter.ts 迁入,PLAN-033 原作;
// 解析逻辑逐字保留,断言见 __tests__/frontmatter.spec.ts 改址后的同套)
// + T8 meta 整形薄壳(供 plans_view.at 经 use.web 消费)。
// 单文件持有避免 ext 镜像的传递拷贝问题(use.web 引本文件即整体拷贝)。

export type FrontmatterValue = string | string[]

export interface SplitResult {
  /** 键序保持文件书写顺序。 */
  meta: Record<string, FrontmatterValue>
  /** 去掉 frontmatter 围栏后的正文（保留首行空行之外的原文）。 */
  body: string
}

/** 文件头围栏 `---\n...\n---\n`（CRLF 兼容；必须从首行开始）。 */
const FENCE_RE = /^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/

/**
 * 拆分 markdown 的 frontmatter。无 frontmatter（或 `---` 只出现在正文中）
 * 时返回 null。
 */
export function splitFrontmatter(content: string): SplitResult | null {
  const m = FENCE_RE.exec(content)
  if (!m) return null
  const meta: Record<string, FrontmatterValue> = {}
  let currentKey: string | null = null
  let currentList: string[] | null = null
  for (const rawLine of m[1].split(/\r?\n/)) {
    const line = rawLine.trim()
    if (!line || line.startsWith('#')) continue // 空行 / 注释
    const listMatch = /^-\s+(.*)$/.exec(line)
    if (listMatch) {
      if (currentList) currentList.push(unquote(listMatch[1]))
      continue
    }
    const kv = /^([A-Za-z0-9_]+)\s*:\s*(.*)$/.exec(line)
    if (!kv) continue
    currentKey = kv[1]
    const rest = kv[2].trim()
    if (rest === '') {
      // 块列表头（`key:` 后跟 `- item` 行）
      currentList = []
      meta[currentKey] = currentList
    } else if (rest.startsWith('[') && rest.endsWith(']')) {
      const inner = rest.slice(1, -1).trim()
      currentList = null
      meta[currentKey] = inner
        ? inner.split(',').map((s) => unquote(s.trim()))
        : []
    } else {
      currentList = null
      meta[currentKey] = stripComment(unquote(rest))
    }
  }
  // 去掉正文开头的空行（MetaBlock 与首个标题之间的空隙来源之一）
  return { meta, body: content.slice(m[0].length).replace(/^(?:[ \t]*\r?\n)+/, '') }
}

/** 去掉成对的包裹引号。 */
function unquote(s: string): string {
  if (
    s.length >= 2 &&
    ((s.startsWith('"') && s.endsWith('"')) ||
      (s.startsWith("'") && s.endsWith("'")))
  ) {
    return s.slice(1, -1)
  }
  return s
}

/** 去掉行尾 ` # comment`（仅未加引号的值；对齐后端 parse_frontmatter 行为）。 */
function stripComment(s: string): string {
  if (s.startsWith('"') || s.startsWith("'")) return s
  const idx = s.indexOf(' #')
  return idx === -1 ? s : s.slice(0, idx).trimEnd()
}

// ── PLAN-063 T8: plans_view.at 消费面(meta chips + 正文) ──────────────

export interface PlansMetaBar {
  plan_id: string
  status: string
  feature_name: string
  step_label: string
}

/** frontmatter meta → 详情 chips 行四字段;无 frontmatter 回退空串族。 */
export function plansFrontmatterMeta(content: string): PlansMetaBar {
  const r = splitFrontmatter(content)
  if (!r) return { plan_id: '', status: '', feature_name: '', step_label: '' }
  const meta = r.meta
  const cur = typeof meta['current_step'] === 'string' ? Number(meta['current_step']) : 0
  const total = typeof meta['total_steps'] === 'string' ? Number(meta['total_steps']) : 0
  const step_label = total > 0 ? `${cur}/${total}` : ''
  return {
    plan_id: typeof meta['plan_id'] === 'string' ? meta['plan_id'] : '',
    status: typeof meta['status'] === 'string' ? meta['status'] : '',
    feature_name: typeof meta['feature_name'] === 'string' ? meta['feature_name'] : '',
    step_label,
  }
}

/** 正文(去 frontmatter 后);无 frontmatter 时原文返回(裸 md/纯文本回退)。 */
export function plansFrontmatterBody(content: string): string {
  const r = splitFrontmatter(content)
  return r ? r.body : content
}
