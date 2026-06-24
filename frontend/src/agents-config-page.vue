<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'

// Base URL for API calls — the origin of wherever this module was loaded from.
// This ensures fetch() hits the correct server (musk :8080) even when the
// component is loaded remotely by auto-os-config (:17700).
const API_BASE = (() => {
  try {
    const url = new URL(import.meta.url)
    return `${url.protocol}//${url.host}`
  } catch {
    return 'http://127.0.0.1:8080'
  }
})()

interface Mode {
  name: string
  description: string
  profession: string
  skills: string[]
  tool_count: number
}

interface Profession {
  name: string
  tier: string
  temperature: number
  max_turns: number
}

const modes = ref<Mode[]>([])
const professions = ref<Profession[]>([])
const errorMsg = ref('')
const loaded = ref(false)
const filter = ref('')

const filteredModes = computed(() => {
  const q = filter.value.trim().toLowerCase()
  if (!q) return modes.value
  return modes.value.filter(
    (m) =>
      m.name.toLowerCase().includes(q) ||
      m.profession.toLowerCase().includes(q) ||
      m.description.toLowerCase().includes(q),
  )
})

async function load() {
  errorMsg.value = ''
  loaded.value = false
  try {
    // Two independent endpoints — modes come from .at files, professions from
    // compiled-in built-ins. They are unrelated data sources (see README).
    const [modesResp, profsResp] = await Promise.all([
      fetch(`${API_BASE}/api/modes`),
      fetch(`${API_BASE}/api/professions`),
    ])
    if (!modesResp.ok) throw new Error(`GET /api/modes → HTTP ${modesResp.status}`)
    if (!profsResp.ok) throw new Error(`GET /api/professions → HTTP ${profsResp.status}`)
    const modesData = await modesResp.json()
    const profsData = await profsResp.json()
    modes.value = modesData.modes || []
    professions.value = profsData.professions || []
  } catch (e: any) {
    errorMsg.value = e.message || String(e)
  } finally {
    loaded.value = true
  }
}

onMounted(() => load())
</script>

<template>
  <div class="agents-config">
    <div v-if="errorMsg" class="state-msg error">
      ✗ Failed to load: {{ errorMsg }}
      <span class="hint">Make sure <code>musk serve</code> is running on {{ API_BASE }}</span>
    </div>

    <div v-if="!loaded && !errorMsg" class="state-msg">Loading…</div>

    <template v-if="loaded && !errorMsg">
      <!-- Modes -->
      <div class="card">
        <div class="card-header">
          <h2>Agent Modes</h2>
          <input
            v-model="filter"
            class="filter-input"
            placeholder="Filter modes…"
          />
        </div>
        <p class="card-sub">
          Modes are declared in <code>modes/*.at</code> (built-ins) and
          <code>~/.config/autoos/modes/*.at</code> (user). Select one with
          <code>--mode &lt;name&gt;</code> on <code>musk run</code> / <code>musk chat</code>.
        </p>
        <div v-if="filteredModes.length === 0" class="empty">No modes match.</div>
        <div v-for="m in filteredModes" :key="m.name" class="mode-card">
          <div class="mode-header">
            <span class="mode-name">{{ m.name }}</span>
            <span class="badge">{{ m.profession }}</span>
            <span class="badge muted">{{ m.tool_count }} tools</span>
            <span v-if="m.skills" class="badge muted">skills on</span>
          </div>
          <div v-if="m.description" class="mode-desc">{{ m.description }}</div>
        </div>
      </div>

      <!-- Professions -->
      <div class="card">
        <h2>Professions</h2>
        <p class="card-sub">
          Built-in professions define the model tier, temperature and turn cap.
          The daemon resolves each tier to a concrete model.
        </p>
        <table>
          <thead>
            <tr><th>Name</th><th>Tier</th><th>Temp</th><th>Max Turns</th></tr>
          </thead>
          <tbody>
            <tr v-for="p in professions" :key="p.name">
              <td class="mono">{{ p.name }}</td>
              <td><span class="badge">{{ p.tier }}</span></td>
              <td>{{ p.temperature }}</td>
              <td>{{ p.max_turns }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </div>
</template>

<style scoped>
/* All colors reference the host's theme variables (set on :root), so this page
   follows the sidebar accent picker automatically. */
.agents-config { max-width: 760px; }
.card { background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius, 8px); padding: 20px 22px; margin-bottom: 16px; }
.card h2 { font-size: 15px; font-weight: 600; margin: 0 0 4px 0; }
.card-header { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 4px; }
.card-sub { font-size: 12px; color: var(--text-muted); margin: 0 0 16px 0; }
.card-sub code { background: var(--bg-hover); padding: 1px 6px; border-radius: var(--radius-sm, 3px); font-size: 11px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.filter-input { padding: 6px 11px; border: 1px solid var(--border); border-radius: var(--radius-sm, 4px); font-size: 12px; width: 180px; background: var(--bg-input); outline: none; transition: border-color 0.15s, box-shadow 0.15s; }
.filter-input:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-light); }
.mode-card { border: 1px solid var(--border); border-radius: var(--radius-sm, 6px); padding: 12px 14px; margin-bottom: 10px; transition: border-color 0.15s; }
.mode-card:hover { border-color: var(--accent); }
.mode-header { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.mode-name { font-weight: 600; font-size: 14px; }
.mode-desc { font-size: 12px; color: var(--text-secondary); margin-top: 6px; }
.badge { font-size: 11px; padding: 2px 9px; border-radius: 10px; background: var(--accent-light); color: var(--accent); font-weight: 500; }
.badge.muted { background: var(--bg-hover); color: var(--text-secondary); }
table { width: 100%; border-collapse: collapse; margin-top: 8px; }
th, td { text-align: left; padding: 7px 10px; border-bottom: 1px solid var(--border); font-size: 13px; }
th { font-size: 11px; text-transform: uppercase; letter-spacing: 0.03em; color: var(--text-muted); font-weight: 600; }
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.empty { color: var(--text-muted); font-size: 13px; padding: 12px 0; }
.state-msg { padding: 14px; border-radius: var(--radius, 8px); background: var(--bg-hover); color: var(--text-secondary); font-size: 13px; }
.state-msg.error { background: rgba(196,43,28,0.08); color: var(--danger); }
.state-msg.error .hint { display: block; margin-top: 6px; font-size: 12px; opacity: 0.85; }
.state-msg.error code { background: rgba(0,0,0,0.08); padding: 1px 5px; border-radius: 3px; }
</style>
