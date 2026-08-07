// forge_stream.ts — Forge 聊天流 SSE 消费（对应 useForge.ts 的 streamResponse）
//
// Plan 022 Phase 7b: forge 流 20+ 事件共享单结构（type discriminator）。
// 回写机制：直接 import store 的模块级 ref（singleton），在 SSE onmessage 里
// 操作 current_draft/messages/thinking/streaming/error，绕开 store 不能传回调
// 的限制。widget 通过 reactive(useForgeStoreStore()) 自动响应 ref 变化。
//
// Plan 022 Phase 7c: 补全 errand/relay/task_plan 14 事件回写 + tool_call 真正
// 累积到消息的 tool_calls 数组（7b 只拼文本，卡片渲染依赖结构化 tool_calls）。

// Plan 022 Phase 7b: 直接调 useForgeStoreStore() 拿 singleton store 对象
// (store 的模块级 ref 不 export，但 useForgeStoreStore() 返回同一份 singleton)，
// 在 SSE onmessage 里操作其 ref，绕开 store 不能传回调的限制。
import { useForgeStoreStore } from '@/stores/useForgeStoreStore'

const _store = useForgeStoreStore()

export interface ForgeStreamEvent {
  type: string
  text?: string
  thinking?: string
  id?: string
  name?: string
  arguments?: unknown
  result?: string
  status?: string
  phase?: string
  output?: string
  turns?: number
  tool_calls?: unknown[]
  message?: string
  // errand fields
  errand_id?: string
  profession_id?: string
  tool_call_id?: string
  task?: string
  token_usage?: number
  // relay fields
  run_id?: string
  flow_id?: string
  step_id?: string
  summary?: string
  tokens_used?: number
  title?: string
  // task plan fields
  instance_id?: string
  task_plan_id?: string
}

let currentEs: EventSource | null = null

/**
 * Start consuming the forge chat stream for a session. Closes any prior stream.
 * Events are dispatched directly into the store's singleton refs (current_draft,
 * thinking, streaming, error, messages.tool_calls, errands, relays, task_plans),
 * which the widget observes reactively.
 */
export function startForgeStream(
  sessionId: string,
  workspace: string | null,
  token: string | null,
): void {
  stopForgeStream()
  let path = `/api/chats/session/${encodeURIComponent(sessionId)}/stream`
  const params: string[] = []
  if (workspace) params.push(`workspace=${encodeURIComponent(workspace)}`)
  if (token) params.push(`token=${encodeURIComponent(token)}`)
  if (params.length) path += '?' + params.join('&')

  // 防御：forge_store.at 的 `Value = {}` 初值 codegen 成 null（对象字面量未识别），
  // 这里在流开始前确保三个 record 是可写对象。
  if (!_store.errands.value) _store.errands.value = {}
  if (!_store.relays.value) _store.relays.value = {}
  if (!_store.task_plans.value) _store.task_plans.value = {}

  const es = new EventSource(path)
  currentEs = es
  es.onmessage = (ev) => {
    try {
      const data = JSON.parse(ev.data) as ForgeStreamEvent
      if (!data || !data.type) return
      handleForgeEvent(data)
    } catch {
      // ignore malformed chunks
    }
  }
  es.onerror = () => {
    _store.error.value = 'stream error'
    _store.streaming.value = false
  }
}

/**
 * Dispatch a forge stream event into the store's singleton refs.
 *
 * Phase 7b handles the core event types (delta, thinking, tool_call, tool_result,
 * done, error). Phase 7c 补全 errand/relay/task_plan 14 事件，并修复 tool_call
 * 真正累积到消息的 tool_calls 数组（而非拼文本）。
 */
function handleForgeEvent(event: ForgeStreamEvent): void {
  const t = event.type

  // ─── 核心事件（7b 引入，7c 修正 tool_call 累积） ─────────────────────────
  if (t === 'delta') {
    if (event.text) {
      const msg = ensureAssistantMsg()
      msg.content += event.text
      _store.current_draft.value += event.text
    }
    return
  }
  if (t === 'thinking') {
    if (event.thinking) {
      const msg = ensureAssistantMsg()
      msg.thinking = (msg.thinking || '') + event.thinking
      _store.thinking.value = msg.thinking
    }
    return
  }
  if (t === 'tool_call') {
    const msg = ensureAssistantMsg()
    const call = {
      id: event.id || `tc-${Date.now()}`,
      name: event.name || 'unknown',
      arguments: (event.arguments as Record<string, unknown>) ?? {},
      status: 'running',
    }
    msg.tool_calls = msg.tool_calls ?? []
    msg.tool_calls.push(call)
    return
  }
  if (t === 'tool_result') {
    const msg = currentAssistantMsg()
    if (msg) {
      const calls = msg.tool_calls ?? []
      // 先按 id 匹配；找不到则回退到最后一个同名 running 调用。
      let call = event.id ? calls.find((c: any) => c.id === event.id) : undefined
      if (!call) {
        const name = event.name
        for (let i = calls.length - 1; i >= 0; i--) {
          const c = calls[i] as any
          if (c.status === 'running' && (!name || c.name === name)) {
            call = c
            break
          }
        }
      }
      if (call) {
        ;(call as any).result = event.result ?? ''
        ;(call as any).status =
          event.status === 'error' ? 'error' : 'success'
      }
    }
    return
  }
  if (t === 'done') {
    _store.streaming.value = false
    stopForgeStream()
    return
  }
  if (t === 'error') {
    _store.error.value = event.message || 'unknown error'
    _store.streaming.value = false
    stopForgeStream()
    return
  }

  // ─── errand 事件（7c） ──────────────────────────────────────────────────
  if (t === 'errand_start' && event.errand_id) {
    _store.errands.value[event.errand_id] = {
      errand_id: event.errand_id,
      profession_id: event.profession_id || 'gofer',
      tool_call_id: event.tool_call_id || '',
      task: event.task || '',
      content: '',
      tool_calls: [],
      status: 'running',
    }
    return
  }
  if (t === 'errand_delta' && event.errand_id && event.text) {
    const e = _store.errands.value[event.errand_id]
    if (e) e.content += event.text
    return
  }
  if (t === 'errand_tool_call' && event.errand_id) {
    const e = _store.errands.value[event.errand_id]
    if (e) {
      e.tool_calls.push({
        id: event.id || `etc-${Date.now()}`,
        name: event.name || 'unknown',
        arguments: (event.arguments as Record<string, unknown>) ?? {},
        status: 'running',
      })
    }
    return
  }
  if (t === 'errand_tool_result' && event.errand_id) {
    const e = _store.errands.value[event.errand_id]
    if (e) {
      const tc = e.tool_calls.find((c: any) => c.id === event.id)
      if (tc) {
        ;(tc as any).result = event.result ?? ''
        ;(tc as any).status = 'success'
      }
    }
    return
  }
  if (t === 'errand_complete' && event.errand_id) {
    const e = _store.errands.value[event.errand_id]
    if (e) {
      e.status = event.status || 'completed'
      e.result = event.result || e.content
      e.token_usage = event.token_usage
    }
    return
  }

  // ─── relay 事件（7c） ───────────────────────────────────────────────────
  if (t === 'relay_spawned' && event.run_id) {
    _store.relays.value[event.run_id] = {
      run_id: event.run_id,
      flow_id: event.flow_id || 'standard',
      status: 'started',
      steps: [],
      title: event.title,
    }
    return
  }
  if (t === 'relay_update' && event.run_id) {
    const r = _store.relays.value[event.run_id]
    if (r) {
      r.steps.push({
        step_id: event.step_id || '',
        profession_id: event.profession_id || '',
      })
      r.status = 'running'
    }
    return
  }
  if (t === 'relay_gate_waiting' && event.run_id) {
    const r = _store.relays.value[event.run_id]
    if (r) r.status = 'gate_waiting'
    return
  }
  if (t === 'relay_complete' && event.run_id) {
    const r = _store.relays.value[event.run_id]
    if (r) {
      r.status = event.status === 'failed' ? 'failed' : 'completed'
      r.summary = event.summary || ''
      r.tokens_used = event.tokens_used || 0
    }
    return
  }

  // ─── task_plan 事件（7c） ───────────────────────────────────────────────
  if (t === 'task_plan_spawned' && event.instance_id) {
    _store.task_plans.value[event.instance_id] = {
      instance_id: event.instance_id,
      task_plan_id: event.task_plan_id || '',
      status: 'started',
      phases: [],
    }
    return
  }

  // 未识别事件（turn_start/phase_change/agent_handoff/gate_reached/run_completed 等）
  console.log('[forge stream]', t)
}

/**
 * Get the current (last) assistant message, or null if the last message
 * isn't an assistant message. Used by tool_result to update an existing call.
 */
function currentAssistantMsg(): any | null {
  const msgs = _store.messages.value as any[]
  if (!msgs || !msgs.length) return null
  const last = msgs[msgs.length - 1]
  return last && last.role === 'assistant' ? last : null
}

/**
 * Ensure there is a "current" assistant message to accumulate deltas/tool_calls
 * into. If the last message isn't an assistant message, push a fresh one.
 * Mirrors useForge.ts ensureAssistantMsg.
 */
function ensureAssistantMsg(): any {
  const msgs = _store.messages.value as any[]
  let last = msgs && msgs.length ? msgs[msgs.length - 1] : null
  if (!last || last.role !== 'assistant') {
    last = {
      id: `a-${Date.now()}`,
      role: 'assistant',
      content: '',
      timestamp: Date.now(),
      tool_calls: [],
    }
    _store.messages.value.push(last)
  }
  return last
}

/** Stop the current forge stream (idempotent). */
export function stopForgeStream(): void {
  if (currentEs) {
    currentEs.close()
    currentEs = null
  }
}
