<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import { marked } from 'marked'
import { useChats } from '../composables/useChats'

const {
  sessions, activeSession, loading, streaming, streamingText, streamingTools,
  loadSessions, selectSession, newSession, renameSession, deleteSession, sendMessage,
} = useChats()

const input = ref('')
const canvas = ref<HTMLElement | null>(null)

const renderedMessages = computed(() => {
  if (!activeSession.value) return []
  return activeSession.value.messages.map((m) => ({
    ...m,
    html: m.role === 'user' ? escapeHtml(m.content) : marked.parse(m.content || '', { breaks: true }) as string,
  }))
})

function escapeHtml(s: string) {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/\n/g, '<br>')
}

const streamingHtml = computed(() => {
  if (!streamingText.value) return ''
  return marked.parse(streamingText.value, { breaks: true }) as string
})

async function send() {
  const text = input.value.trim()
  if (!text || streaming.value) return
  input.value = ''
  await sendMessage(text)
}

async function onNew() {
  await newSession('superpowers')
}

let renamingId: string | null = null
const renameInput = ref('')
function startRename(s: { id: string; name: string }) {
  renamingId = s.id
  renameInput.value = s.name
}
async function commitRename(id: string) {
  if (renameInput.value.trim()) await renameSession(id, renameInput.value.trim())
  renamingId = null
}

async function onDelete(id: string) {
  if (confirm('Delete this chat?')) await deleteSession(id)
}

function autoScroll() {
  nextTick(() => {
    if (canvas.value) canvas.value.scrollTop = canvas.value.scrollHeight
  })
}
watch([renderedMessages, streamingText], autoScroll)

onMounted(loadSessions)
</script>

<template>
  <div class="chats">
    <!-- Session sidebar -->
    <aside class="sidebar">
      <button class="new-btn" @click="onNew">+ New chat</button>
      <div class="session-list">
        <div
          v-for="s in sessions"
          :key="s.id"
          class="session-item"
          :class="{ active: activeSession?.id === s.id }"
          @click="selectSession(s.id)"
        >
          <template v-if="renamingId === s.id">
            <input
              v-model="renameInput"
              class="rename-input"
              @click.stop
              @keyup.enter="commitRename(s.id)"
              @blur="commitRename(s.id)"
              autofocus
            />
          </template>
          <template v-else>
            <div class="session-name" @dblclick.stop="startRename(s)">{{ s.name }}</div>
            <div class="session-preview">{{ s.preview }}</div>
            <button class="del-btn" @click.stop="onDelete(s.id)" title="Delete">✕</button>
          </template>
        </div>
        <div v-if="sessions.length === 0" class="empty-side">No chats yet.</div>
      </div>
    </aside>

    <!-- Chat area -->
    <section class="chat-area">
      <div v-if="!activeSession" class="placeholder">
        <p>Select a chat or start a new one.</p>
      </div>
      <template v-else>
        <div ref="canvas" class="canvas">
          <div
            v-for="m in renderedMessages"
            :key="m.id"
            class="msg"
            :class="m.role"
          >
            <div class="msg-avatar">{{ m.role === 'user' ? '🧑' : '🤖' }}</div>
            <div class="msg-body">
              <div v-if="m.role === 'user'" class="msg-content user-content" v-html="m.html"></div>
              <div v-else class="msg-content markdown" v-html="m.html"></div>
              <div v-if="m.tool_calls?.length" class="tool-calls">
                <details v-for="(tc, i) in m.tool_calls" :key="i" class="tool-call">
                  <summary>🔧 {{ tc.tool }}</summary>
                  <pre class="tool-result">{{ tc.result }}</pre>
                </details>
              </div>
            </div>
          </div>

          <!-- Streaming bubble -->
          <div v-if="streaming" class="msg assistant">
            <div class="msg-avatar">🤖</div>
            <div class="msg-body">
              <div v-if="streamingHtml" class="msg-content markdown" v-html="streamingHtml"></div>
              <div v-else class="typing"><span></span><span></span><span></span></div>
              <div v-if="streamingTools.length" class="tool-calls">
                <details v-for="(tc, i) in streamingTools" :key="i" class="tool-call">
                  <summary>🔧 {{ tc.tool }}</summary>
                  <pre class="tool-result">{{ tc.result }}</pre>
                </details>
              </div>
            </div>
          </div>
        </div>

        <!-- Input bar -->
        <div class="input-bar">
          <textarea
            v-model="input"
            class="input"
            placeholder="Message Auto Musk…  (Enter to send, Shift+Enter for newline)"
            @keydown.enter.exact.prevent="send"
            :disabled="streaming"
            rows="1"
          ></textarea>
          <button class="send-btn" @click="send" :disabled="!input.trim() || streaming">➤</button>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.chats { display: grid; grid-template-columns: 260px 1fr; height: 100%; }

/* Sidebar */
.sidebar { background: var(--bg-panel); border-right: 1px solid var(--border); display: flex; flex-direction: column; }
.new-btn { margin: 12px; padding: 9px; background: var(--accent); color: var(--accent-foreground); border-radius: var(--radius-sm); font-weight: 600; font-size: 13px; transition: background .15s; }
.new-btn:hover { background: var(--accent-hover); }
.session-list { flex: 1; overflow-y: auto; padding: 0 8px 8px; }
.session-item { padding: 10px 12px; border-radius: var(--radius-sm); cursor: pointer; position: relative; transition: background .12s; margin-bottom: 2px; }
.session-item:hover { background: var(--bg-elevated); }
.session-item.active { background: var(--accent-light); }
.session-item.active .session-name { color: var(--accent); }
.session-name { font-size: 13px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; padding-right: 20px; }
.session-preview { font-size: 11px; color: var(--text-muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-top: 2px; }
.del-btn { position: absolute; top: 10px; right: 8px; font-size: 11px; color: var(--text-muted); opacity: 0; transition: opacity .12s; }
.session-item:hover .del-btn { opacity: 1; }
.del-btn:hover { color: var(--danger); }
.rename-input { width: 100%; padding: 4px 6px; background: var(--bg-input); border: 1px solid var(--accent); border-radius: 4px; color: var(--text-primary); font-size: 13px; outline: none; }
.empty-side { color: var(--text-muted); font-size: 12px; padding: 20px; text-align: center; }

/* Chat area */
.chat-area { display: flex; flex-direction: column; min-width: 0; }
.placeholder { flex: 1; display: flex; align-items: center; justify-content: center; color: var(--text-muted); }
.canvas { flex: 1; overflow-y: auto; padding: 24px; max-width: 900px; margin: 0 auto; width: 100%; }

.msg { display: flex; gap: 12px; margin-bottom: 24px; }
.msg-avatar { width: 32px; height: 32px; border-radius: 50%; background: var(--bg-elevated); display: flex; align-items: center; justify-content: center; font-size: 16px; flex-shrink: 0; }
.msg.user .msg-avatar { background: var(--accent-light); }
.msg-body { flex: 1; min-width: 0; }
.msg-content { font-size: 14px; line-height: 1.6; }
.user-content { background: var(--bg-elevated); padding: 10px 14px; border-radius: var(--radius-sm); display: inline-block; }
.markdown :deep(p) { margin-bottom: 8px; }
.markdown :deep(p:last-child) { margin-bottom: 0; }
.markdown :deep(pre) { background: var(--bg-app); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; overflow-x: auto; margin: 8px 0; }
.markdown :deep(code) { font-size: 13px; }
.markdown :deep(pre code) { background: none; padding: 0; }
.markdown :deep(:not(pre) > code) { background: var(--bg-elevated); padding: 1px 5px; border-radius: 3px; }
.markdown :deep(ul), .markdown :deep(ol) { margin: 8px 0; padding-left: 24px; }
.markdown :deep(h1), .markdown :deep(h2), .markdown :deep(h3) { margin: 12px 0 8px; }

.tool-calls { margin-top: 8px; }
.tool-call { background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--radius-sm); margin-bottom: 6px; }
.tool-call summary { padding: 6px 10px; font-size: 12px; cursor: pointer; color: var(--text-secondary); }
.tool-result { padding: 8px 10px; font-size: 11px; color: var(--text-secondary); white-space: pre-wrap; max-height: 200px; overflow: auto; }

.typing { display: flex; gap: 4px; padding: 8px 0; }
.typing span { width: 8px; height: 8px; border-radius: 50%; background: var(--text-muted); animation: blink 1.4s infinite both; }
.typing span:nth-child(2) { animation-delay: .2s; }
.typing span:nth-child(3) { animation-delay: .4s; }
@keyframes blink { 0%, 80%, 100% { opacity: .3; } 40% { opacity: 1; } }

/* Input */
.input-bar { display: flex; gap: 8px; padding: 16px 24px; border-top: 1px solid var(--border); background: var(--bg-panel); max-width: 900px; margin: 0 auto; width: 100%; }
.input { flex: 1; padding: 10px 14px; background: var(--bg-input); border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 14px; outline: none; resize: none; transition: border-color .15s; }
.input:focus { border-color: var(--accent); }
.send-btn { width: 40px; background: var(--accent); color: var(--accent-foreground); border-radius: var(--radius-sm); font-size: 16px; transition: background .15s; }
.send-btn:hover:not(:disabled) { background: var(--accent-hover); }
.send-btn:disabled { opacity: .4; cursor: not-allowed; }
</style>
