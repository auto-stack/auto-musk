import { ref } from 'vue'
import { apiFetch } from './useAuth'

export interface ToolCall { tool: string; args: any; result: string }
export interface ChatMessage {
  id: string
  role: 'user' | 'assistant' | 'tool'
  content: string
  tool_calls?: ToolCall[]
  created_at: number
}
export interface ChatSessionSummary {
  id: string; name: string; mode: string;
  message_count: number; preview: string; updated_at: number;
}
export interface ChatSession {
  id: string; name: string; mode: string;
  messages: ChatMessage[]; created_at: number; updated_at: number;
}

const sessions = ref<ChatSessionSummary[]>([])
const activeSession = ref<ChatSession | null>(null)
const loading = ref(false)
const streaming = ref(false)
// the in-progress assistant text being streamed
const streamingText = ref('')
const streamingTools = ref<ToolCall[]>([])

async function loadSessions() {
  const resp = await apiFetch('/api/chats/sessions')
  if (resp.ok) {
    const data = await resp.json()
    sessions.value = data.sessions || []
  }
}

async function selectSession(id: string) {
  loading.value = true
  const resp = await apiFetch(`/api/chats/session/${id}`)
  if (resp.ok) {
    activeSession.value = (await resp.json()).session
  }
  loading.value = false
}

async function newSession(mode = 'superpowers') {
  const resp = await apiFetch('/api/chats/session', {
    method: 'POST',
    body: JSON.stringify({ mode }),
  })
  if (resp.ok) {
    const data = await resp.json()
    await loadSessions()
    await selectSession(data.session.id)
  }
}

async function renameSession(id: string, name: string) {
  await apiFetch(`/api/chats/session/${id}`, {
    method: 'PATCH',
    body: JSON.stringify({ name }),
  })
  await loadSessions()
  if (activeSession.value?.id === id) activeSession.value.name = name
}

async function deleteSession(id: string) {
  await apiFetch(`/api/chats/session/${id}`, { method: 'DELETE' })
  if (activeSession.value?.id === id) activeSession.value = null
  await loadSessions()
}

/** Send a message: persist the user turn, then stream the agent reply via SSE.
 * Accumulates streamed deltas into `streamingText` and tool calls into
 * `streamingTools`, then reloads the session so the persisted assistant turn
 * is authoritative. */
async function sendMessage(content: string) {
  if (!activeSession.value || streaming.value) return
  const id = activeSession.value.id

  // 1. Persist the user message.
  const postResp = await apiFetch(`/api/chats/session/${id}/message`, {
    method: 'POST',
    body: JSON.stringify({ content }),
  })
  if (!postResp.ok) return
  // Optimistically show the user message immediately.
  const data = await postResp.json()
  activeSession.value = data.session

  // 2. Stream the agent turn via SSE (EventSource can't set headers, but the
  //    musk auth layer accepts the token as a ?token= query fallback — or, if
  //    auth is permissive in dev, no token needed).
  streaming.value = true
  streamingText.value = ''
  streamingTools.value = []

  await new Promise<void>((resolve) => {
    const url = `/api/chats/session/${id}/stream${tokenQuery()}`
    const es = new EventSource(url)
    es.onmessage = (ev) => {
      try {
        const msg = JSON.parse(ev.data)
        if (msg.type === 'delta' && msg.text) {
          streamingText.value += msg.text
        } else if (msg.type === 'tool') {
          streamingTools.value.push({ tool: msg.tool, args: msg.args, result: msg.result })
        } else if (msg.type === 'done' || msg.type === 'error') {
          es.close()
          resolve()
        }
      } catch { /* ignore parse hiccups */ }
    }
    es.onerror = () => { es.close(); resolve() }
  })

  streaming.value = false
  // 3. Reload the session to get the persisted assistant message.
  await selectSession(id)
  await loadSessions()
}

function tokenQuery(): string {
  const t = localStorage.getItem('musk_jwt')
  return t ? `?token=${encodeURIComponent(t)}` : ''
}

export function useChats() {
  return {
    sessions, activeSession, loading, streaming, streamingText, streamingTools,
    loadSessions, selectSession, newSession, renameSession, deleteSession, sendMessage,
  }
}
