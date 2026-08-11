// forge_helpers.ts — Forge 卡片渲染纯函数（对应 web/src/views/ChatsView.vue 的 helper 区）
//
// Plan 022 Phase 7c: 逃生舱纯函数，供 ErrandCard/RelayCard/TaskPlanCard/
// GenericToolCard/MentionInput 等 .vue 组件 import。从 web/ 移植，去掉对
// useForge/useAgentConfigs 的 composable 依赖（状态经 props 传入）。

/** 工具调用信息（对应 web/src/types/tool.ts 的 ToolCallInfo，宽松版）。 */
export interface ToolCallLike {
  id: string
  name: string
  arguments?: Record<string, unknown>
  result?: string
  status?: string
  _expanded?: boolean
}

/** errand 状态（对应 web/src/types/forge.ts 的 ErrandState）。 */
export interface ErrandState {
  errand_id: string
  profession_id: string
  tool_call_id: string
  task: string
  content: string
  tool_calls: ToolCallLike[]
  status: 'running' | 'completed' | 'failed' | 'truncated'
  result?: string
  token_usage?: number
}

/** relay run 状态（对应 web/src/types/forge.ts 的 RelayRunState）。 */
export interface RelayRunState {
  run_id: string
  flow_id: string
  status: 'started' | 'running' | 'gate_waiting' | 'completed' | 'failed'
  steps: { step_id: string; profession_id: string }[]
  summary?: string
  tokens_used?: number
  title?: string
}

/** task plan 状态（对应 web/src/types/forge.ts 的 TaskPlanState）。 */
export interface TaskPlanState {
  instance_id: string
  task_plan_id: string
  status: 'started' | 'running' | 'completed' | 'failed'
  phases: { phase: string; status: string }[]
}

type RecordMap<T> = Record<string, T>

// ─── Errand helpers ──────────────────────────────────────────────────────────

/** 按 tool_call_id 在 errands record 里查 errand 状态。 */
export function getErrandByToolCallId(
  errands: RecordMap<ErrandState>,
  toolCallId: string,
): ErrandState | null {
  return (
    Object.values(errands).find((e) => e.tool_call_id === toolCallId) || null
  )
}

export function getErrandTask(tc: ToolCallLike): string {
  return (tc.arguments?.task as string) || 'Research task'
}

export function getErrandState(
  errands: RecordMap<ErrandState>,
  tc: ToolCallLike,
): ErrandState | null {
  return getErrandByToolCallId(errands, tc.id)
}

export function getErrandContent(
  errands: RecordMap<ErrandState>,
  tc: ToolCallLike,
): string {
  return getErrandState(errands, tc)?.content || ''
}

export function getErrandToolCalls(
  errands: RecordMap<ErrandState>,
  tc: ToolCallLike,
): ToolCallLike[] {
  return getErrandState(errands, tc)?.tool_calls || []
}

// ─── Relay helpers ───────────────────────────────────────────────────────────

/** 从 tool_call 的 arguments.run_id 或 result JSON 里提取 run_id。 */
export function extractRunId(tc: ToolCallLike): string {
  try {
    const args = tc.arguments as any
    if (args?.run_id) return args.run_id as string
    if (tc.result) {
      const result = JSON.parse(tc.result)
      if (result?.run_id) return result.run_id as string
    }
    return ''
  } catch {
    return ''
  }
}

export function getRelayStatus(
  relays: RecordMap<RelayRunState>,
  tc: ToolCallLike,
): RelayRunState | null {
  const runId = extractRunId(tc)
  if (!runId) return null
  return relays[runId] || null
}

// ─── Task plan helpers ───────────────────────────────────────────────────────

/** task_plan status 中文 label（Plan 023 P3：component fn 嵌套 if 引用 computed
 *  触发缺陷 8 嵌套残留，用 fn 绕过——字典映射在 .at 无表达，fn 内部承载）。 */
export function taskPlanStatusLabel(status: string): string {
  const map: Record<string, string> = {
    started: '启动中', running: '运行中', completed: '已完成', failed: '失败', idle: '就绪',
  }
  return map[status] ?? status
}

export function getTaskPlan(
  taskPlans: RecordMap<TaskPlanState>,
  tc: ToolCallLike,
): TaskPlanState | null {
  const instanceId = (tc.arguments?.instance_id as string) || ''
  if (!instanceId) return null
  return taskPlans[instanceId] || null
}

// ─── Generic tool card helpers ───────────────────────────────────────────────

export interface ToolSeg {
  type: string
  text: string
}

/** 提取工具调用的摘要段（path/pattern/desc），用于通用卡片 header 显示。 */
export function getToolSummary(tc: ToolCallLike): ToolSeg[] {
  const args = tc.arguments ?? {}
  const segs: ToolSeg[] = []

  const path = (args.path as string) || ''
  const slug = (args.slug as string) || ''
  const sectionId = (args.section_id as string) || ''
  const pattern = (args.pattern as string) || ''
  const query = (args.query as string) || ''
  const task = (args.task as string) || ''
  const command = (args.command as string) || ''
  const limit = args.limit as number | undefined
  const offset = args.offset as number | undefined

  if (path) {
    segs.push({ type: 'path', text: path })
    if (limit !== undefined || offset !== undefined) {
      segs.push({ type: 'loc', text: `:${limit ?? ''}:${offset ?? ''}` })
    }
  }
  if (slug) segs.push({ type: 'path', text: slug })
  if (sectionId) segs.push({ type: 'path', text: sectionId })
  if (pattern) {
    const s = pattern.length > 60 ? pattern.slice(0, 57) + '…' : pattern
    segs.push({ type: 'pattern', text: `"${s}"` })
  }
  if (query) {
    const s = query.length > 60 ? query.slice(0, 57) + '…' : query
    segs.push({ type: 'pattern', text: `"${s}"` })
  }
  if (task && !segs.length) {
    const s = task.length > 60 ? task.slice(0, 57) + '…' : task
    segs.push({ type: 'desc', text: s })
  }
  if (command && !segs.length) {
    const s = command.length > 80 ? command.slice(0, 77) + '…' : command
    segs.push({ type: 'desc', text: s })
  }

  return segs
}

/** 文件扩展名 → 代码围栏语言提示。 */
export function langFromPath(path: unknown): string {
  if (typeof path !== 'string') return ''
  const ext = path.split('.').pop()?.toLowerCase() || ''
  const map: Record<string, string> = {
    rs: 'rust', py: 'python', js: 'javascript', ts: 'typescript',
    vue: 'vue', go: 'go', java: 'java', rb: 'ruby', php: 'php',
    c: 'c', h: 'c', cpp: 'cpp', cc: 'cpp', hpp: 'cpp',
    sh: 'bash', bash: 'bash', zsh: 'bash',
    json: 'json', yaml: 'yaml', yml: 'yaml', toml: 'toml',
    html: 'html', xml: 'xml', css: 'css', scss: 'scss',
    md: 'markdown', sql: 'sql',
  }
  return map[ext] || ext || ''
}

// ─── Mention helpers ─────────────────────────────────────────────────────────

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
