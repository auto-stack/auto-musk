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

interface Skill {
  name: string
  description: string
}

const skills = ref<Skill[]>([])
const errorMsg = ref('')
const loaded = ref(false)
const filter = ref('')

const filteredSkills = computed(() => {
  const q = filter.value.trim().toLowerCase()
  if (!q) return skills.value
  return skills.value.filter(
    (s) =>
      s.name.toLowerCase().includes(q) ||
      s.description.toLowerCase().includes(q),
  )
})

async function load() {
  errorMsg.value = ''
  loaded.value = false
  try {
    const resp = await fetch(`${API_BASE}/api/skills`)
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`)
    const data = await resp.json()
    // sort alphabetically by name for stable display
    skills.value = (data.skills || []).slice().sort((a: Skill, b: Skill) =>
      a.name.localeCompare(b.name),
    )
  } catch (e: any) {
    errorMsg.value = e.message || String(e)
  } finally {
    loaded.value = true
  }
}

onMounted(() => load())
</script>

<template>
  <div class="skills-config">
    <div v-if="errorMsg" class="state-msg error">
      ✗ Failed to load: {{ errorMsg }}
      <span class="hint">Make sure <code>musk serve</code> is running on {{ API_BASE }}</span>
    </div>

    <div v-if="!loaded && !errorMsg" class="state-msg">Loading…</div>

    <template v-if="loaded && !errorMsg">
      <div class="card">
        <div class="card-header">
          <h2>Skill Registry</h2>
          <input
            v-model="filter"
            class="filter-input"
            placeholder="Filter skills…"
          />
        </div>
        <p class="card-sub">
          Skills are Markdown prompts auto-discovered from
          <code>~/.config/autoos/skills/&lt;name&gt;/SKILL.md</code>. When the skill
          system is enabled for a mode, the model invokes them autonomously via the
          <code>skill</code> tool. Add a skill by creating a new folder with a
          <code>SKILL.md</code> (frontmatter: <code>name</code>, <code>description</code>).
        </p>

        <div class="stat-row">
          <span class="stat">
            <span class="stat-num">{{ skills.length }}</span>
            <span class="stat-label">installed</span>
          </span>
          <span v-if="filter" class="stat muted">
            {{ filteredSkills.length }} match{{ filteredSkills.length === 1 ? '' : 'es' }}
          </span>
        </div>

        <div v-if="filteredSkills.length === 0" class="empty">
          <template v-if="skills.length === 0">
            No skills installed. Create a directory under
            <code>~/.config/autoos/skills/</code> with a <code>SKILL.md</code>.
          </template>
          <template v-else>No skills match "{{ filter }}".</template>
        </div>

        <div v-for="s in filteredSkills" :key="s.name" class="skill-card">
          <div class="skill-head">
            <span class="skill-icon">🧩</span>
            <span class="skill-name">{{ s.name }}</span>
          </div>
          <div class="skill-desc">{{ s.description }}</div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
/* All colors reference the host's theme variables (set on :root), so this page
   follows the sidebar accent picker automatically. */
.skills-config { max-width: 760px; }
.card { background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius, 8px); padding: 20px 22px; }
.card h2 { font-size: 15px; font-weight: 600; margin: 0; }
.card-header { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
.card-sub { font-size: 12px; color: var(--text-muted); margin: 8px 0 16px 0; line-height: 1.5; }
.card-sub code { background: var(--bg-hover); padding: 1px 6px; border-radius: var(--radius-sm, 3px); font-size: 11px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.filter-input { padding: 6px 11px; border: 1px solid var(--border); border-radius: var(--radius-sm, 4px); font-size: 12px; width: 180px; background: var(--bg-input); outline: none; transition: border-color 0.15s, box-shadow 0.15s; }
.filter-input:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-light); }

.stat-row { display: flex; align-items: baseline; gap: 16px; margin-bottom: 16px; }
.stat { display: flex; align-items: baseline; gap: 6px; }
.stat-num { font-size: 22px; font-weight: 700; color: var(--accent); }
.stat-label { font-size: 12px; color: var(--text-muted); }
.stat.muted { font-size: 12px; color: var(--text-muted); font-weight: 400; }

.skill-card {
  border: 1px solid var(--border);
  border-radius: var(--radius-sm, 6px);
  padding: 12px 14px;
  margin-bottom: 8px;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.skill-card:hover { border-color: var(--accent); box-shadow: 0 1px 4px var(--accent-light); }
.skill-head { display: flex; align-items: center; gap: 8px; }
.skill-icon { font-size: 14px; }
.skill-name { font-weight: 600; font-size: 13px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; color: var(--text-primary); }
.skill-desc { font-size: 12px; color: var(--text-secondary); margin-top: 6px; line-height: 1.5; }

.empty { color: var(--text-muted); font-size: 13px; padding: 20px 0; text-align: center; }
.empty code { background: var(--bg-hover); padding: 1px 5px; border-radius: 3px; font-size: 11px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }

.state-msg { padding: 14px; border-radius: var(--radius, 8px); background: var(--bg-hover); color: var(--text-secondary); font-size: 13px; }
.state-msg.error { background: rgba(196,43,28,0.08); color: var(--danger); }
.state-msg.error .hint { display: block; margin-top: 6px; font-size: 12px; opacity: 0.85; }
.state-msg.error code { background: rgba(0,0,0,0.08); padding: 1px 5px; border-radius: 3px; }
</style>
