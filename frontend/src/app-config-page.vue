<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'

const API_BASE = (() => {
  try {
    const url = new URL(import.meta.url)
    return `${url.protocol}//${url.host}`
  } catch {
    return 'http://127.0.0.1:8080'
  }
})()

interface EffectiveConfig {
  daemon_url: string
  default_mode: string
  context_file: string
  serve_addr: string
  auto_start_daemon: boolean
}

const effective = ref<EffectiveConfig | null>(null)
const daemonStatus = ref<'unknown' | 'checking' | 'ok' | 'fail'>('unknown')
const daemonLatency = ref<number | null>(null)

// Edit form (only the persisted fields; effective values are read-only display)
const form = reactive({
  daemon_url: '' as string,
  default_mode: 'superpowers',
  context_file: '',
  serve_addr: '127.0.0.1:8080',
  auto_start_daemon: true,
})

const loaded = ref(false)
const errorMsg = ref('')
const saving = ref(false)
const saveNote = ref('')

async function load() {
  errorMsg.value = ''
  loaded.value = false
  try {
    const resp = await fetch(`${API_BASE}/api/app-config`)
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`)
    const data = await resp.json()
    effective.value = data.effective
    // Seed the form with stored values (or effective defaults)
    const s = data.stored || {}
    form.daemon_url = s.daemon_url ?? data.effective.daemon_url
    form.default_mode = s.default_mode ?? data.effective.default_mode
    form.context_file = s.context_file ?? ''
    form.serve_addr = s.serve_addr ?? data.effective.serve_addr
    form.auto_start_daemon = s.auto_start_daemon ?? data.effective.auto_start_daemon
    // Probe daemon reachability
    testDaemon(data.effective.daemon_url)
  } catch (e: any) {
    errorMsg.value = e.message || String(e)
  } finally {
    loaded.value = true
  }
}

async function testDaemon(url?: string) {
  const target = url || effective.value?.daemon_url || ''
  if (!target) return
  daemonStatus.value = 'checking'
  daemonLatency.value = null
  try {
    const t0 = performance.now()
    const resp = await fetch(`${target}/v1/status`, { signal: AbortSignal.timeout(3000) })
    const t1 = performance.now()
    if (resp.ok) {
      daemonStatus.value = 'ok'
      daemonLatency.value = Math.round(t1 - t0)
    } else {
      daemonStatus.value = 'fail'
    }
  } catch {
    daemonStatus.value = 'fail'
  }
}

async function save() {
  saving.value = true
  saveNote.value = ''
  try {
    const resp = await fetch(`${API_BASE}/api/app-config`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        daemon_url: form.daemon_url || null,
        default_mode: form.default_mode || null,
        context_file: form.context_file || null,
        serve_addr: form.serve_addr || null,
        auto_start_daemon: form.auto_start_daemon,
      }),
    })
    const data = await resp.json()
    if (resp.ok && data.status === 'saved') {
      saveNote.value = `Saved → ${data.path}`
      effective.value = data.effective
    } else {
      saveNote.value = data.statusText || `HTTP ${resp.status}`
    }
  } catch (e: any) {
    saveNote.value = e.message
  } finally {
    saving.value = false
  }
}

onMounted(() => load())
</script>

<template>
  <div class="musk-config">
    <div v-if="errorMsg" class="state-msg error">
      ✗ Failed to load: {{ errorMsg }}
      <span class="hint">Make sure <code>musk serve</code> is running on {{ API_BASE }}</span>
    </div>
    <div v-else-if="!loaded" class="state-msg">Loading…</div>

    <template v-if="loaded && !errorMsg">
      <!-- Connection status -->
      <div class="card">
        <div class="card-head">
          <h2>Daemon Connection</h2>
          <span class="status-pill" :class="daemonStatus">
            <template v-if="daemonStatus === 'ok'">● Connected <span v-if="daemonLatency !== null" class="muted">({{ daemonLatency }}ms)</span></template>
            <template v-else-if="daemonStatus === 'checking'">○ Checking…</template>
            <template v-else-if="daemonStatus === 'fail'">✕ Unreachable</template>
            <template v-else>— Unknown</template>
          </span>
        </div>
        <p class="card-sub">
          musk talks to the AutoOS AI daemon (aaid) for all LLM calls. The daemon
          arbitrates concurrency, manages API keys, and resolves model tiers.
        </p>

        <div class="field">
          <label>Daemon URL</label>
          <input v-model="form.daemon_url" type="text" placeholder="http://127.0.0.1:17654" />
          <span class="field-hint">Effective: <code>{{ effective?.daemon_url }}</code> · env <code>AAID_URL</code> overrides</span>
        </div>

        <div class="field">
          <label class="check">
            <input type="checkbox" v-model="form.auto_start_daemon" />
            Auto-start daemon if unreachable
          </label>
          <span class="field-hint">ssh-agent model: spawn aaid on demand when an agent needs it</span>
        </div>

        <button class="btn-sm" :disabled="daemonStatus === 'checking'" @click="testDaemon(form.daemon_url)">
          Re-test connection
        </button>
      </div>

      <!-- Defaults -->
      <div class="card">
        <h2>Defaults</h2>
        <div class="grid">
          <div class="field">
            <label>Default mode</label>
            <input v-model="form.default_mode" type="text" placeholder="superpowers" />
            <span class="field-hint">Used by <code>musk run</code> / <code>musk chat</code> without <code>--mode</code></span>
          </div>
          <div class="field">
            <label>Serve address</label>
            <input v-model="form.serve_addr" type="text" placeholder="127.0.0.1:8080" />
            <span class="field-hint">Bind address for <code>musk serve</code></span>
          </div>
          <div class="field span-2">
            <label>Context file</label>
            <input v-model="form.context_file" type="text" placeholder="(auto: .musk.md / CLAUDE.md)" />
            <span class="field-hint">Explicit context file; empty = auto-discover upward from CWD</span>
          </div>
        </div>
      </div>

      <!-- Save bar -->
      <div class="save-bar">
        <button class="btn-sm primary" :disabled="saving" @click="save">
          {{ saving ? 'Saving…' : 'Save' }}
        </button>
        <button class="btn-sm" @click="load">Reload</button>
      </div>
      <div v-if="saveNote" class="save-note" :class="{ ok: saveNote.startsWith('Saved'), fail: !saveNote.startsWith('Saved') }">{{ saveNote }}</div>

      <p class="where">
        Persisted to <code>~/.config/autoos/apps/musk/config.at</code>.
        Capability configuration (Agents, Skills, Roles) lives in the shared
        OS-level modules, not here.
      </p>
    </template>
  </div>
</template>

<style scoped>
.musk-config { max-width: 680px; }
.card { background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius, 8px); padding: 20px 22px; margin-bottom: 16px; }
.card-head { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 4px; }
.card h2 { font-size: 15px; font-weight: 600; margin: 0; }
.card-sub { font-size: 12px; color: var(--text-muted); margin: 0 0 16px 0; line-height: 1.5; }

.status-pill { font-size: 12px; font-weight: 500; padding: 4px 12px; border-radius: 12px; }
.status-pill.ok { background: var(--accent-light); color: var(--accent); }
.status-pill.checking { background: var(--bg-hover); color: var(--text-secondary); }
.status-pill.fail { background: rgba(196,43,28,.1); color: var(--danger); }
.muted { color: var(--text-muted); font-weight: 400; }

.grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px 16px; }
.field { display: flex; flex-direction: column; gap: 4px; margin-bottom: 12px; }
.field.span-2 { grid-column: span 2; }
.field label { font-size: 11px; color: var(--text-muted); font-weight: 500; text-transform: uppercase; letter-spacing: .03em; }
.field label.check { text-transform: none; letter-spacing: 0; font-size: 13px; color: var(--text-primary); display: flex; align-items: center; gap: 8px; cursor: pointer; }
.field label.check input { margin: 0; }
.field input[type="text"] { padding: 7px 10px; border: 1px solid var(--border); border-radius: var(--radius-sm, 4px); font-size: 13px; background: var(--bg-input); outline: none; transition: border-color .15s, box-shadow .15s; }
.field input[type="text"]:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-light); }
.field-hint { font-size: 11px; color: var(--text-muted); }
.field-hint code { background: var(--bg-hover); padding: 1px 5px; border-radius: 3px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }

.btn-sm { padding: 6px 14px; font-size: 12px; border: 1px solid var(--border); border-radius: var(--radius-sm, 4px); background: var(--bg-card); cursor: pointer; font-weight: 500; transition: background .15s, border-color .15s; margin-top: 4px; }
.btn-sm:hover:not(:disabled) { background: var(--bg-hover); }
.btn-sm:disabled { opacity: .6; cursor: not-allowed; }
.btn-sm.primary { background: var(--accent); color: var(--accent-foreground, #fff); border-color: var(--accent); }
.btn-sm.primary:hover:not(:disabled) { background: var(--accent-hover); }

.save-bar { display: flex; gap: 8px; margin-bottom: 8px; }
.save-note { padding: 8px 12px; border-radius: var(--radius-sm, 4px); font-size: 12px; }
.save-note.ok { background: var(--accent-light); color: var(--accent); }
.save-note.fail { background: rgba(196,43,28,.08); color: var(--danger); }
.save-note code { background: rgba(0,0,0,.08); padding: 1px 5px; border-radius: 3px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }

.where { font-size: 11px; color: var(--text-muted); line-height: 1.6; margin-top: 12px; }
.where code { background: var(--bg-hover); padding: 1px 5px; border-radius: 3px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }

.state-msg { padding: 14px; border-radius: var(--radius, 8px); background: var(--bg-hover); color: var(--text-secondary); font-size: 13px; }
.state-msg.error { background: rgba(196,43,28,.08); color: var(--danger); }
.state-msg.error .hint { display: block; margin-top: 6px; font-size: 12px; opacity: .85; }
.state-msg.error code { background: rgba(0,0,0,.08); padding: 1px 5px; border-radius: 3px; }
</style>
