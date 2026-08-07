// forge_stream.ts — Forge 聊天流 SSE 消费（对应 useForge.ts 的 streamResponse）
//
// Plan 022 Phase 7b: forge 流 20+ 事件共享单结构（type discriminator）。
// 回写机制：直接 import store 的模块级 ref（singleton），在 SSE onmessage 里
// 操作 current_draft/messages/thinking/streaming/error，绕开 store 不能传回调
// 的限制。widget 通过 reactive(useForgeStoreStore()) 自动响应 ref 变化。

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
  errand_id?: string
  run_id?: string
  instance_id?: unknown
}

let currentEs: EventSource | null = null

/**
 * Start consuming the forge chat stream for a session. Closes any prior stream.
 * Events are dispatched directly into the store's singleton refs (current_draft,
 * thinking, streaming, error), which the widget observes reactively.
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
 * Phase 7b handles the core event types (delta, thinking, tool_call, done, error).
 * errand / relay / task_plan events 留 7c (当前只 log)。
 */
function handleForgeEvent(event: ForgeStreamEvent): void {
  const t = event.type
  if (t === 'delta') {
    if (event.text) _store.current_draft.value += event.text
  } else if (t === 'thinking') {
    if (event.thinking) _store.thinking.value = event.thinking
  } else if (t === 'tool_call') {
    if (event.name) _store.current_draft.value += `\n\n[tool: ${event.name}]\n\n`
  } else if (t === 'tool_result') {
    if (event.name) _store.current_draft.value += `\n\n[result: ${event.name}]\n\n`
  } else if (t === 'done') {
    _store.streaming.value = false
    stopForgeStream()
  } else if (t === 'error') {
    _store.error.value = event.message || 'unknown error'
    _store.streaming.value = false
    stopForgeStream()
  } else {
    console.log('[forge stream]', t)
  }
}

/** Stop the current forge stream (idempotent). */
export function stopForgeStream(): void {
  if (currentEs) {
    currentEs.close()
    currentEs = null
  }
}
