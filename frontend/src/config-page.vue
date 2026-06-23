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

interface Skill {
  name: string
  description: string
}

interface ConfigOverview {
  modes: Mode[]
  professions: Profession[]
  skills: Skill[]
}

const config = ref<ConfigOverview>({ modes: [], professions: [], skills: [] })
const errorMsg = ref('')
const loaded = ref(false)
const filter = ref('')

const filteredModes = computed(() => {
  const q = filter.value.trim().toLowerCase()
  if (!q) return config.value.modes
  return config.value.modes.filter(
    (m) =>
      m.name.toLowerCase().includes(q) ||
      m.profession.toLowerCase().includes(q) ||
      m.description.toLowerCase().includes(q),
  )
})

async function loadConfig() {
  errorMsg.value = ''
  loaded.value = false
  try {
    const resp = await fetch(`${API_BASE}/api/config`)
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`)
    const data = await resp.json()
    config.value = {
      modes: data.modes || [],
      professions: data.professions || [],
      skills: data.skills || [],
    }
  } catch (e: any) {
    errorMsg.value = e.message || String(e)
  } finally {
    loaded.value = true
  }
}

onMounted(() => loadConfig())
</script>

<template>
  <div class="musk-config">
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
          Modes are declared in <code>modes/*.at</code>. Select one with <code>--mode &lt;name&gt;</code> on
          <code>musk run</code> / <code>musk chat</code>.
        </p>
        <div v-if="filteredModes.length === 0" class="empty">No modes match.</div>
        <div v-for="m in filteredModes" :key="m.name" class="mode-card">
          <div class="mode-header">
            <span class="mode-name">{{ m.name }}</span>
            <span class="badge">{{ m.profession }}</span>
            <span class="badge muted">{{ m.tool_count }} tools</span>
            <span v-if="m.skills" class="badge muted">{{ m.skills }} skills</span>
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
            <tr v-for="p in config.professions" :key="p.name">
              <td class="mono">{{ p.name }}</td>
              <td><span class="badge">{{ p.tier }}</span></td>
              <td>{{ p.temperature }}</td>
              <td>{{ p.max_turns }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Skills -->
      <div class="card">
        <h2>Skills</h2>
        <p class="card-sub">
          Skills are Markdown prompts in <code>~/.config/autoos/skills/*/SKILL.md</code>.
          The model invokes them autonomously via the <code>skill</code> tool.
        </p>
        <div v-if="config.skills.length === 0" class="empty">No skills installed.</div>
        <div v-for="s in config.skills" :key="s.name" class="skill-row">
          <span class="skill-name">{{ s.name }}</span>
          <span class="skill-desc">{{ s.description }}</span>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.musk-config { max-width: 760px; }
.card { background: var(--bg-card, #fff); border: 1px solid var(--border, #e0e0e0); border-radius: 8px; padding: 20px; margin-bottom: 16px; }
.card h2 { font-size: 16px; margin: 0 0 4px 0; border-bottom: 1px solid #eee; padding-bottom: 8px; }
.card-header { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
.card-header h2 { border: none; padding: 0; margin: 0; }
.card-sub { font-size: 12px; color: #777; margin: 4px 0 14px 0; }
.card-sub code { background: #f4f4f4; padding: 1px 5px; border-radius: 3px; font-size: 11px; }
.filter-input { padding: 5px 10px; border: 1px solid #ccc; border-radius: 4px; font-size: 12px; width: 180px; }
.mode-card { border: 1px solid #eee; border-radius: 8px; padding: 12px 14px; margin-bottom: 10px; }
.mode-header { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.mode-name { font-weight: 600; font-size: 14px; }
.mode-desc { font-size: 12px; color: #666; margin-top: 6px; }
.badge { font-size: 11px; padding: 2px 8px; border-radius: 10px; background: #e3f2fd; color: #1565c0; }
.badge.muted { background: #f0f0f0; color: #666; }
table { width: 100%; border-collapse: collapse; margin-top: 8px; }
th, td { text-align: left; padding: 6px 10px; border-bottom: 1px solid #f0f0f0; font-size: 13px; }
th { font-size: 11px; text-transform: uppercase; color: #999; }
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.skill-row { display: flex; gap: 12px; padding: 8px 0; border-bottom: 1px solid #f6f6f6; align-items: baseline; }
.skill-name { font-weight: 600; font-size: 13px; min-width: 180px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.skill-desc { font-size: 12px; color: #666; flex: 1; }
.empty { color: #999; font-size: 13px; padding: 12px 0; }
.state-msg { padding: 14px; border-radius: 8px; background: #f4f4f4; color: #555; font-size: 13px; }
.state-msg.error { background: #f8d7da; color: #721c24; }
.state-msg.error .hint { display: block; margin-top: 6px; font-size: 12px; opacity: 0.85; }
.state-msg.error code { background: rgba(0,0,0,0.08); padding: 1px 5px; border-radius: 3px; }
</style>
