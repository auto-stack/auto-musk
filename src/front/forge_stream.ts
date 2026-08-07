// forge_stream.ts — Forge 聊天流 SSE 消费（对应 useForge.ts 的 streamResponse）
//
// Plan 022 Phase 7: forge 流有 20+ 事件类型，共享单 JSON 结构（type discriminator
// + 可选字段），不是 enum 每变体独立结构。Phase 1 的自动 SSE dispatch 按变体→
// 独立 action 设计，不适合这里。故用逃生舱手动消费 SSE，把事件回调给 store。
//
// 用法（forge_store 的 Init/Send 后调用）:
//   startForgeStream(sessionId, workspace, token, (event) => { store action })

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
 *
 * Plan 022 Phase 7a: AutoUI store handlers cannot receive a JS callback fn, so
 * this opens the EventSource and logs events to the console. Phase 7b will wire
 * events back into the store (either via a generated SSE action or a global
 * event bus the widget polls). For now this establishes the SSE connection so
 * the backend side proceeds; UI reaction to streamed chunks comes later.
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
      // Phase 7b TODO: dispatch data into the ForgeStore. For now, log so the
      // connection is observable and the backend stream is consumed.
      if (data && data.type) console.log('[forge stream]', data.type, data.text ?? '')
    } catch {
      // ignore malformed chunks
    }
  }
  es.onerror = () => {
    console.error('[forge stream] error')
  }
}

/** Stop the current forge stream (idempotent). */
export function stopForgeStream(): void {
  if (currentEs) {
    currentEs.close()
    currentEs = null
  }
}
