import { ref } from 'vue'
import { useEventRouter, type SSEEvent } from './useEventRouter'
import { authFetch } from './useAuth'

const API_BASE = '/api/forge/relay'

// ─── Singleton state ────────────────────────────────────────────────────────
// PLAN-030 T12: converged to the chat-inline surface (RelayRunBox + slash
// commands + event router). RelayView-only members (professions/souls
// browsers, rerun/handoff/delete/rename, run-list progress computeds) are
// retired along with the standalone relay view.
const _runs = ref<RunSummary[]>([])
const _currentRun = ref<RunState | null>(null)
const _loading = ref(false)
const _error = ref<string | null>(null)
const _liveLog = ref<Array<{ time: string; profession: string; action: string }>>([])
const _professionTokens = ref<Record<string, number>>({})
const _sessionLogs = ref<Record<string, SessionLogEntry[]>>({})

function sessionLogFor(runId: string): SessionLogEntry[] {
  return _sessionLogs.value[runId] ?? []
}

export interface RunHistory {
  title: string
  status: 'completed' | 'failed' | 'interrupted'
  entries: SessionLogEntry[]
  /** PLAN-032: 汇报报告元数据（会话镜像 system turn 提取；重启回放恢复）。 */
  report?: { format: string; title: string; path: string }
}

/**
 * Run 历史回放（PLAN-030 试用反馈）：run 是内存态、serve 重启即清空，
 * 但 run 日志持久化在 Flow 会话（id=run_id）里。内存 run 不在时从会话
 * turns 重建只读视图：流式 message 按来源合并、system turns 推导终态。
 */
async function loadRunHistory(runId: string): Promise<RunHistory | null> {
  try {
    const resp = await authFetch(`/api/conversations/${runId}`)
    if (!resp.ok) return null
    const conv = await resp.json()
    const entries: SessionLogEntry[] = []
    let bufProf = ''
    let bufText = ''
    let sawCompleted = false
    let sawFailed = false
    const flush = () => {
      if (bufText) {
        entries.push({ id: `h${entries.length}`, time: '', profession_id: bufProf || 'unknown', type: 'text', content: bufText })
        bufProf = ''
        bufText = ''
      }
    }
    for (const t of conv.turns ?? []) {
      const k = t.kind ?? ''
      if (k === 'message') {
        const prof = t.from ?? 'unknown'
        if (prof === bufProf) bufText += t.content ?? ''
        else {
          flush()
          bufProf = prof
          bufText = t.content ?? ''
        }
      } else {
        flush()
        if (k === 'tool_call') {
          // PLAN-031 T8: 与 .at 轨同口径——带 arguments（操作目标展示用），
          // 结果由紧随的 tool_result turn 配对合并。此前两者都缺：目标列空白、
          // 条目永远是 tool_call 型（终态下全部误显"已中断"）。
          entries.push({
            id: `h${entries.length}`, time: '', profession_id: t.from ?? 'unknown',
            type: 'tool_call', content: '',
            tool_name: t.tool?.name ?? '', arguments: t.tool?.args ?? undefined, result: '',
          })
        } else if (k === 'tool_result') {
          // 并入最近一个 tool 条目（展开态显示结果）。结果字段双兼容：
          // 新版镜像写 t.content；旧版会话写 t.tool.result（content 空）。
          const res = String(t.content ?? '') || String(t.tool?.result ?? '')
          for (let j = entries.length - 1; j >= 0; j--) {
            if (entries[j].type === 'tool_call' || entries[j].type === 'tool') {
              entries[j].result = res
              entries[j].type = 'tool'
              break
            }
          }
        } else if (k === 'gate') {
          // 审批标题可解析化：step_id 优先取 GateRecord，兜底从
          // "Waiting for gate approval: execute" 冒号后抽取
          let gs = t.gate?.step_id ?? ''
          if (!gs) {
            const ci = String(t.content ?? '').indexOf(':')
            if (ci !== -1) gs = String(t.content).slice(ci + 1).trim()
          }
          entries.push({ id: `h${entries.length}`, time: '', profession_id: 'system', type: 'gate_waiting', content: `Gate '${gs}' waiting` })
        } else if (k === 'system') {
          const c = t.content ?? ''
          if (c.startsWith("Step '") && c.includes(' completed')) {
            entries.push({ id: `h${entries.length}`, time: '', profession_id: 'system', type: 'step_completed', content: c })
          } else if (c.startsWith("Step '") && c.includes(' started')) {
            entries.push({ id: `h${entries.length}`, time: '', profession_id: 'system', type: 'step_started', content: c })
          } else if (c.includes('Flow completed')) {
            // T13: 后端曾双写 RunCompleted（advance 终态重复追加已修），
            // 回放侧仍去重防御
            if (!sawCompleted) {
              sawCompleted = true
              entries.push({ id: `h${entries.length}`, time: '', profession_id: 'system', type: 'run_completed', content: 'Run 已完成' })
            }
          } else if (c.includes('failed')) {
            sawFailed = true
            entries.push({ id: `h${entries.length}`, time: '', profession_id: 'system', type: 'error', content: c })
          }
        }
      }
    }
    flush()
    let status: RunHistory['status'] = 'interrupted'
    if (sawCompleted) status = 'completed'
    else if (sawFailed || conv.status === 'failed') status = 'failed'
    // PLAN-032: 会话镜像 system turn「汇报报告已生成：T（F，path：P）」→ 元数据
    let report: RunHistory['report']
    for (const t of conv.turns ?? []) {
      const c = String(t.content ?? '')
      const m = c.match(/汇报报告已生成：(.+?)（(.+?)，path：(.+?)）/)
      if (m) report = { format: m[2], title: m[1], path: m[3] }
    }
    if (report) {
      _reportMeta.value[runId] = report
    } else if (sawCompleted) {
      // 兜底：旧格式镜像 turn（PLAN-032 前无 path）→ 试拉一次（端点有磁盘
      // 回退）；有内容即构造元数据（title 取会话标题）。
      void fetchRunReport(runId).then((html) => {
        if (html) _reportMeta.value[runId] = { format: 'html', title: conv.title ?? '', path: '' }
      })
    }
  
  return { title: conv.title ?? '', status, entries, report }
  } catch {
    return null
  }
}

export interface SessionLogEntry {
  id: string
  time: string
  profession_id: string
  /** 展示态（RunBox 内工具条目的收起/展开），非协议字段 */
  _expanded?: boolean
  /** 展示态（长文本/计划文档块的收起/展开），非协议字段 */
  _docOpen?: boolean
  step_id?: string
  type: 'text' | 'thinking' | 'tool_call' | 'tool_result' | 'tool' | 'complete' | 'error' | 'budget_warning' | 'budget_exceeded' | 'step_started' | 'step_completed' | 'gate_waiting' | 'run_completed' | 'run_failed'
  content: string
  tool_name?: string
  tool_id?: string
  arguments?: any
  result?: string
  remaining?: number
}

// ─── Types (mirroring Rust structs) ─────────────────────────────────────────

export interface RunSummary {
  run_id: string
  status: string
  current_step: number
  total_steps: number
  current_profession: string | null
  cumulative_tokens: number
  created_at: number
  updated_at: number
  title?: string
  task?: string
}

export interface RunEventDto {
  type: string
  timestamp?: number
  step_id?: string
  profession_id?: string
  handoff_summary?: string
  gate?: string
  decision?: string
  error?: string
  cumulative?: number
  step_tokens?: number
  text?: string
  tool_id?: string
  tool_name?: string
  arguments?: any
  result?: string
  message?: string
  remaining?: number
  thinking?: string
}

export interface RunState {
  run_id: string
  status: string
  current_step: number
  total_steps: number
  current_profession: string | null
  steps: StepState[]
  step_history: StepRecord[]
  cumulative_tokens: number
  budget_limit: number
  budget_remaining: number
  waiting_for_gate: GateState | null
  parallel_estimate: number
  savings: number
  savings_ratio: number
  events: RunEventDto[]
  title?: string
  current_step_started_at?: number
  profession_tokens?: Record<string, number>
}

export interface StepState {
  id: string
  profession_id: string
  status: string
  gate: string
}

export interface StepRecord {
  step_id: string
  profession_id: string
  started_at: number
  completed_at: number
  iteration: number
}

export interface GateState {
  step_id: string
  profession_id: string
  since: number
}

export interface ProfessionDto {
  id: string
  name: string
  phase: string
  owned_sections: string[]
  allowed_tools: string[]
}

export interface SoulDto {
  id: string
  name: string
}

export interface StartRunRequest {
  run_id?: string
  flow_id: string
  steps?: { id: string; profession_id: string; gate?: string }[]
  task?: string
}

// ─── Composable ─────────────────────────────────────────────────────────────

// PLAN-032: 汇报报告 HTML 缓存（runId → html 全文；deck 层数据源）。
const _reportHtml = new Map<string, string>()
// PLAN-032: 报告元数据（runId → {format,title,path}；SSE 载荷与会话回放双通道）。
const _reportMeta = ref<Record<string, { format: string; title: string; path: string }>>({})

  /** PLAN-032: 拉取 run 汇报报告 HTML 全文（带缓存；404/未生成返回 null）。 */
async function fetchRunReport(runId: string): Promise<string | null> {
  const cached = _reportHtml.get(runId)
  if (cached != null) return cached
  try {
    const ws = localStorage.getItem('musk_workspace')
    const qs = ws ? `?workspace=${encodeURIComponent(ws)}` : ''
    const resp = await authFetch(`${API_BASE}/runs/${runId}/report${qs}`)
    if (!resp.ok) return null
    const html = await resp.text()
    if (html) _reportHtml.set(runId, html)
    return html || null
  } catch {
    return null
  }
}

export function useRelay() {
  const runs = _runs
  const currentRun = _currentRun
  const loading = _loading
  const error = _error

  async function loadRuns(projectPath?: string) {
    try {
      const query = projectPath ? `?project_path=${encodeURIComponent(projectPath)}` : ''
      const resp = await authFetch(`${API_BASE}/runs${query}`)
      if (!resp.ok) throw new Error(`Failed: ${resp.status}`)
      const raw = await resp.json()
      // Backend returns { runs: [...] }; tolerate a bare array too.
      const data: RunSummary[] = Array.isArray(raw) ? raw : (raw?.runs ?? [])
      runs.value = data.sort((a: RunSummary, b: RunSummary) => b.updated_at - a.updated_at)
      // Clear stale currentRun if it's no longer in the list
      if (currentRun.value && !data.find((r: RunSummary) => r.run_id === currentRun.value!.run_id)) {
        currentRun.value = null
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      runs.value = []
    }
  }

  function formatTimestamp(ts: number): string {
    return new Date(ts * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  }

  function eventsToSessionLog(runId: string, events: RunEventDto[]): SessionLogEntry[] {
    const result: SessionLogEntry[] = []
    for (const ev of events) {
      const time = ev.timestamp ? formatTimestamp(ev.timestamp) : new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
      const prof = ev.profession_id || 'unknown'
      switch (ev.type) {
        case 'turn_delta':
          if (result.length > 0 && result[result.length - 1].type === 'text' && result[result.length - 1].profession_id === prof) {
            result[result.length - 1].content += ev.text || ''
          } else {
            result.push({ id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`, time, profession_id: prof, type: 'text', content: ev.text || '' })
          }
          break
        case 'turn_thinking':
          if (result.length > 0 && result[result.length - 1].type === 'thinking' && result[result.length - 1].profession_id === prof) {
            result[result.length - 1].content += ev.thinking || ''
          } else {
            result.push({ id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`, time, profession_id: prof, type: 'thinking', content: ev.thinking || '' })
          }
          break
        case 'turn_tool_call':
          result.push({ id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`, time, profession_id: prof, type: 'tool_call', content: '', tool_name: ev.tool_name, tool_id: ev.tool_id, arguments: ev.arguments })
          break
        case 'turn_tool_result': {
          const last = result[result.length - 1]
          if (last && last.type === 'tool_call' && last.tool_id === ev.tool_id) {
            // Merge into a single tool widget
            last.type = 'tool'
            last.result = ev.result || ''
          } else {
            result.push({ id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`, time, profession_id: prof, type: 'tool_result', content: ev.result || '', tool_id: ev.tool_id })
          }
          break
        }
        case 'turn_complete':
          result.push({ id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`, time, profession_id: prof, type: 'complete', content: 'Turn completed' })
          break
        case 'turn_error':
          result.push({ id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`, time, profession_id: prof, type: 'error', content: ev.message || 'Unknown error' })
          break
        case 'turn_budget_warning':
          result.push({ id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`, time, profession_id: prof, type: 'budget_warning', content: `Budget warning: ${ev.remaining} tokens remaining`, remaining: ev.remaining })
          break
        case 'turn_budget_exceeded':
          result.push({ id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`, time, profession_id: prof, type: 'budget_exceeded', content: 'Budget exceeded — turn stopped' })
          break
      }
    }
    return result
  }

  async function loadRun(runId: string) {
    try {
      const resp = await authFetch(`${API_BASE}/runs/${runId}`)
      if (!resp.ok) {
        if (resp.status === 404) {
          currentRun.value = null
          delete _sessionLogs.value[runId]
        }
        throw new Error(`Failed: ${resp.status}`)
      }
      const data = await resp.json()
      currentRun.value = data
      // Populate session log from persisted events
      if (data.events && data.events.length > 0) {
        _sessionLogs.value[runId] = eventsToSessionLog(runId, data.events)
      }
      // Populate profession tokens for cost breakdown when viewing historical runs
      if (data.profession_tokens) {
        _professionTokens.value = data.profession_tokens
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function startRun(req: StartRunRequest) {
    loading.value = true
    error.value = null
    try {
      const resp = await authFetch(`${API_BASE}/runs`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(req),
      })
      if (!resp.ok) throw new Error(`Failed: ${resp.status}`)
      const data = await resp.json()
      currentRun.value = data.state
      await loadRuns()
      return data.run_id as string
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  async function advanceRun(runId: string) {
    try {
      const resp = await authFetch(`${API_BASE}/runs/${runId}/advance`, { method: 'POST' })
      if (!resp.ok) throw new Error(`Failed: ${resp.status}`)
      await loadRun(runId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function resolveGate(runId: string, decision: 'approve' | 'reject' | 'edit', feedback?: string) {
    try {
      const body: any = { decision }
      if (feedback) body.feedback = feedback
      const resp = await authFetch(`${API_BASE}/runs/${runId}/gate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      })
      if (!resp.ok) throw new Error(`Failed: ${resp.status}`)
      await loadRun(runId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function submitHandoff(runId: string, handoff: any) {
    try {
      const resp = await authFetch(`${API_BASE}/runs/${runId}/handoff`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ handoff }),
      })
      if (!resp.ok) throw new Error(`Failed: ${resp.status}`)
      await loadRun(runId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  // SSE for live updates
  function subscribeToRun(runId: string, onEvent?: (event: any) => void) {
    const eventRouter = useEventRouter()
    if (!_sessionLogs.value[runId]) _sessionLogs.value[runId] = []
    const es = new EventSource(`${API_BASE}/runs/${runId}/events`)
    es.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        if (onEvent) onEvent(data)
        // Route through event router for cross-view coordination
        const sseEvent: SSEEvent = {
          type: data.event_type || data.type,
          runId,
          payload: data,
        }
        eventRouter.handleEvent(sseEvent, 'relay')
        // Append to live log
        if (data.event_type === 'handoff_submitted') {
          _liveLog.value.push({
            time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
            profession: data.profession_id || data.from_profession || 'unknown',
            action: `Handoff to ${data.to_profession || 'next'}`,
          })
        }
        if (data.event_type === 'step_advanced') {
          _liveLog.value.push({
            time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
            profession: data.profession_id || 'system',
            action: `Step advanced: ${data.step_id || ''}`,
          })
        }
        // Session log: turn events
        const time = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
        const prof = data.payload?.profession_id || 'unknown'
        if (data.event_type === 'turn_delta') {
          const last = _sessionLogs.value[runId][_sessionLogs.value[runId].length - 1]
          if (last && last.type === 'text' && last.profession_id === prof) {
            last.content += data.payload.text || ''
          } else {
            _sessionLogs.value[runId].push({
              id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
              time,
              profession_id: prof,
              type: 'text',
              content: data.payload.text || '',
            })
          }
        }
        if (data.event_type === 'turn_thinking') {
          const last = _sessionLogs.value[runId][_sessionLogs.value[runId].length - 1]
          if (last && last.type === 'thinking' && last.profession_id === prof) {
            last.content += data.payload.thinking || ''
          } else {
            _sessionLogs.value[runId].push({
              id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
              time,
              profession_id: prof,
              type: 'thinking',
              content: data.payload.thinking || '',
            })
          }
        }
        if (data.event_type === 'turn_tool_call') {
          _sessionLogs.value[runId].push({
            id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
            time,
            profession_id: prof,
            type: 'tool_call',
            content: '',
            tool_name: data.payload.tool_name,
            tool_id: data.payload.tool_id,
            arguments: data.payload.arguments,
          })
        }
        if (data.event_type === 'turn_tool_result') {
          const last = _sessionLogs.value[runId][_sessionLogs.value[runId].length - 1]
          if (last && last.type === 'tool_call' && last.tool_id === data.payload.tool_id) {
            last.type = 'tool'
            last.result = data.payload.result || ''
          } else {
            _sessionLogs.value[runId].push({
              id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
              time,
              profession_id: prof,
              type: 'tool_result',
              content: data.payload.result || '',
              tool_id: data.payload.tool_id,
            })
          }
        }
        if (data.event_type === 'turn_complete') {
          _sessionLogs.value[runId].push({
            id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
            time,
            profession_id: prof,
            type: 'complete',
            content: 'Turn completed',
          })
        }
        if (data.event_type === 'turn_error') {
          _sessionLogs.value[runId].push({
            id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
            time,
            profession_id: prof,
            type: 'error',
            content: data.payload.message || 'Unknown error',
          })
        }
        if (data.event_type === 'turn_budget_warning') {
          _sessionLogs.value[runId].push({
            id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
            time,
            profession_id: prof,
            type: 'budget_warning',
            content: `Budget warning: ${data.payload.remaining} tokens remaining`,
            remaining: data.payload.remaining,
          })
        }
        if (data.event_type === 'turn_budget_exceeded') {
          _sessionLogs.value[runId].push({
            id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
            time,
            profession_id: prof,
            type: 'budget_exceeded',
            content: 'Budget exceeded — turn stopped',
          })
        }
        // Step lifecycle events
        if (data.event_type === 'step_started') {
          _sessionLogs.value[runId].push({
            id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
            time,
            profession_id: data.payload?.profession_id || 'system',
            type: 'step_started',
            content: `Step "${data.payload?.step_id || ''}" started`,
          })
        }
        if (data.event_type === 'step_completed') {
          _sessionLogs.value[runId].push({
            id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
            time,
            profession_id: data.payload?.profession_id || 'system',
            type: 'step_completed',
            content: `Step "${data.payload?.step_id || ''}" completed`,
          })
        }
        if (data.event_type === 'gate_waiting') {
          _sessionLogs.value[runId].push({
            id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
            time,
            profession_id: data.payload?.profession_id || 'system',
            type: 'gate_waiting',
            // 审批标题可解析化（与 Step 同构）——阶段块标题栏显示审批的阶段名
            content: `Gate '${data.payload?.step_id ?? ''}' waiting`,
          })
        }
        if (data.event_type === 'run_completed' && data.payload?.report) {
          _reportMeta.value[runId] = data.payload.report
        }
        if (data.event_type === 'run_completed') {
          _sessionLogs.value[runId].push({
            id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
            time,
            profession_id: 'system',
            type: 'run_completed',
            content: 'Run completed successfully',
          })
        }
        if (data.event_type === 'run_failed') {
          _sessionLogs.value[runId].push({
            id: `${runId}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
            time,
            profession_id: 'system',
            type: 'run_failed',
            content: data.payload?.error || 'Run failed',
          })
        }
        // Track per-profession tokens (best-effort from event data)
        if (data.tokens_used && data.profession_id) {
          const prev = _professionTokens.value[data.profession_id] || 0
          _professionTokens.value[data.profession_id] = prev + (data.tokens_used as number)
        }
        // Auto-refresh run state on relevant events
        // PLAN-030 试用修复：对齐 RunEvent 真实枚举——原列表含不存在的事件名，
        // gate_waiting/run_failed/run_completed 缺席，停靠/完成/失败从不刷新。
        if (['step_started', 'step_completed', 'gate_waiting', 'gate_resolved', 'run_failed', 'run_completed'].includes(data.event_type)) {
          loadRun(runId)
        }
      } catch {
        // ignore parse errors
      }
    }
    es.onerror = () => {
      // Will auto-reconnect or close
    }
    return () => es.close()
  }

  return {
    runs,
    currentRun,
    loading,
    error,
    loadRun,
    loadRunHistory,
    fetchRunReport,
    runReports: _reportMeta,
    startRun,
    advanceRun,
    resolveGate,
    subscribeToRun,
    sessionLogFor,
  }
}
