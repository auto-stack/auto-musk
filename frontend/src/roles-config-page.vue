<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'

const API_BASE = (() => {
  try {
    const url = new URL(import.meta.url)
    return `${url.protocol}//${url.host}`
  } catch {
    return 'http://127.0.0.1:8080'
  }
})()

interface RoleSummary {
  name: string
  description: string
  tier: string
  allowed_tiers: string[]
  skills: string[]
  skill_count: number
  token_budget: number | null
  is_builtin: boolean
}

interface RoleDetail extends RoleSummary {
  soul: string
  soul_from_file: boolean
  temperature: number | null
  max_turns: number | null
  inherit: string | null
  tools: string[]
  model: string | null
  soul_file: string | null
}

const TIERS = ['min', 'lite', 'mid', 'pro', 'max'] as const
const TIER_COLORS: Record<string, string> = {
  min: '#9ca3af', lite: '#38bdf8', mid: '#6366f1', pro: '#a855f7', max: '#ec4899',
}

const roles = ref<RoleSummary[]>([])
const allSkills = ref<string[]>([])
const selected = ref<RoleDetail | null>(null)
const editing = ref(false)
const errorMsg = ref('')
const loaded = ref(false)
const filter = ref('')
const saving = ref(false)
const saveNote = ref('')

// Edit form state (bound to inputs)
const form = ref<RoleDetail | null>(null)

const filteredRoles = computed(() => {
  const q = filter.value.trim().toLowerCase()
  if (!q) return roles.value
  return roles.value.filter(
    (r) => r.name.toLowerCase().includes(q) || r.description.toLowerCase().includes(q),
  )
})

async function load() {
  errorMsg.value = ''
  loaded.value = false
  try {
    const [rolesResp, skillsResp] = await Promise.all([
      fetch(`${API_BASE}/api/roles`),
      fetch(`${API_BASE}/api/skills`),
    ])
    if (!rolesResp.ok) throw new Error(`GET /api/roles → ${rolesResp.status}`)
    const rolesData = await rolesResp.json()
    const skillsData = await skillsResp.json().catch(() => ({ skills: [] }))
    roles.value = rolesData.roles || []
    allSkills.value = (skillsData.skills || []).map((s: any) => s.name)
  } catch (e: any) {
    errorMsg.value = e.message || String(e)
  } finally {
    loaded.value = true
  }
}

async function selectRole(name: string) {
  editing.value = false
  saveNote.value = ''
  try {
    const resp = await fetch(`${API_BASE}/api/roles/${encodeURIComponent(name)}`)
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`)
    selected.value = await resp.json()
  } catch (e: any) {
    errorMsg.value = e.message
  }
}

function startEdit() {
  if (!selected.value) return
  // builtin roles: prompt to duplicate as a new user role (they're read-only)
  if (selected.value.is_builtin) {
    const newName = prompt(`Built-in role "${selected.value.name}" is read-only. Save a copy as a new role named:`, `${selected.value.name}-custom`)
    if (!newName) return
    form.value = { ...selected.value, name: newName, is_builtin: false, inherit: selected.value.name }
  } else {
    form.value = JSON.parse(JSON.stringify(selected.value))
  }
  editing.value = true
  saveNote.value = ''
}

function cancelEdit() {
  editing.value = false
  form.value = null
}

function toggleTier(tier: string) {
  if (!form.value) return
  const idx = form.value.allowed_tiers.indexOf(tier)
  if (idx >= 0) form.value.allowed_tiers.splice(idx, 1)
  else form.value.allowed_tiers.push(tier)
}

function toggleSkill(skill: string) {
  if (!form.value) return
  const idx = form.value.skills.indexOf(skill)
  if (idx >= 0) form.value.skills.splice(idx, 1)
  else form.value.skills.push(skill)
}

async function saveRole() {
  if (!form.value) return
  saving.value = true
  saveNote.value = ''
  const f = form.value
  const body = {
    description: f.description,
    tier: f.tier,
    allowed_tiers: f.allowed_tiers,
    skills: f.skills,
    token_budget: f.token_budget,
    temperature: f.temperature,
    max_turns: f.max_turns,
    inherit: f.inherit,
    tools: f.tools,
    model: f.model,
    soul: f.soul,
  }
  try {
    const resp = await fetch(`${API_BASE}/api/roles/${encodeURIComponent(f.name)}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    })
    const data = await resp.json()
    if (resp.ok && data.status === 'saved') {
      saveNote.value = `Saved as "${f.name}"`
      editing.value = false
      await load()
      await selectRole(f.name)
    } else {
      saveNote.value = typeof data === 'string' ? data : (data.statusText || `HTTP ${resp.status}`)
    }
  } catch (e: any) {
    saveNote.value = e.message
  } finally {
    saving.value = false
  }
}

async function deleteRole() {
  if (!selected.value || selected.value.is_builtin) return
  if (!confirm(`Delete role "${selected.value.name}"?`)) return
  const name = selected.value.name
  const resp = await fetch(`${API_BASE}/api/roles/${encodeURIComponent(name)}`, { method: 'DELETE' })
  if (resp.ok) {
    selected.value = null
    await load()
  } else {
    saveNote.value = await resp.text()
  }
}

function newRole() {
  form.value = {
    name: 'new-role', description: '', tier: 'mid', allowed_tiers: [], skills: [],
    skill_count: 0, token_budget: null, is_builtin: false,
    soul: '# Soul of the New Role\n\n## Personality\nYou are ...',
    soul_from_file: false, temperature: 0.3, max_turns: 20,
    inherit: '', tools: [], model: null, soul_file: null,
  }
  editing.value = true
  selected.value = null
  saveNote.value = ''
}

onMounted(() => load())
</script>

<template>
  <div class="roles-config">
    <div v-if="errorMsg && !loaded" class="state-msg error">
      ✗ Failed to load: {{ errorMsg }}
      <span class="hint">Make sure <code>musk serve</code> is running on {{ API_BASE }}</span>
    </div>
    <div v-else-if="!loaded" class="state-msg">Loading…</div>

    <template v-else>
      <!-- Two-pane: list (left) + detail/editor (right) -->
      <div class="layout">
        <!-- List pane -->
        <div class="list-pane">
          <div class="pane-head">
            <input v-model="filter" class="filter-input" placeholder="Filter roles…" />
            <button class="btn-sm primary" @click="newRole">+ New</button>
          </div>
          <div class="stat">{{ roles.length }} roles ({{ roles.filter(r => r.is_builtin).length }} built-in)</div>
          <div
            v-for="r in filteredRoles"
            :key="r.name"
            class="role-item"
            :class="{ active: selected?.name === r.name }"
            @click="selectRole(r.name)"
          >
            <span v-if="r.is_builtin" class="lock" title="built-in (read-only)">🔒</span>
            <span v-else class="lock user" title="user role">👤</span>
            <span class="role-name">{{ r.name }}</span>
            <span class="tier-dot" :style="{ background: TIER_COLORS[r.tier] }" :title="r.tier"></span>
            <span v-if="r.skill_count" class="badge muted">{{ r.skill_count }} skills</span>
          </div>
          <div v-if="filteredRoles.length === 0" class="empty">No roles match.</div>
        </div>

        <!-- Detail / editor pane -->
        <div class="detail-pane">
          <div v-if="!selected && !editing" class="placeholder">
            Select a role on the left, or click <strong>+ New</strong>.
          </div>

          <!-- Read-only detail -->
          <div v-else-if="selected && !editing" class="detail">
            <div class="detail-head">
              <h2>{{ selected.name }} <span v-if="selected.is_builtin" class="tag">built-in</span></h2>
              <div class="actions">
                <button class="btn-sm primary" @click="startEdit">
                  {{ selected.is_builtin ? 'Duplicate…' : 'Edit' }}
                </button>
                <button v-if="!selected.is_builtin" class="btn-sm danger" @click="deleteRole">Delete</button>
              </div>
            </div>
            <p v-if="selected.description" class="desc">{{ selected.description }}</p>

            <div class="kv">
              <div class="kv-row"><span class="k">Tier</span><span class="tier-dot" :style="{ background: TIER_COLORS[selected.tier] }"></span>{{ selected.tier }}</div>
              <div v-if="selected.allowed_tiers.length" class="kv-row"><span class="k">Allowed tiers</span>
                <span v-for="t in selected.allowed_tiers" :key="t" class="badge">{{ t }}</span>
              </div>
              <div v-if="selected.inherit" class="kv-row"><span class="k">Inherits</span><span class="mono">{{ selected.inherit }}</span></div>
              <div v-if="selected.temperature !== null" class="kv-row"><span class="k">Temperature</span>{{ selected.temperature }}</div>
              <div v-if="selected.max_turns !== null" class="kv-row"><span class="k">Max turns</span>{{ selected.max_turns }}</div>
              <div v-if="selected.token_budget !== null" class="kv-row"><span class="k">Token budget</span>{{ selected.token_budget.toLocaleString() }} <span class="muted">(stored, not enforced)</span></div>
            </div>

            <div v-if="selected.skills.length" class="section">
              <h3>Skills ({{ selected.skills.length }})</h3>
              <span v-for="s in selected.skills" :key="s" class="skill-chip">{{ s }}</span>
            </div>

            <div class="section">
              <h3>Soul <span v-if="selected.soul_from_file" class="muted">(from {{ selected.soul_file }})</span></h3>
              <pre class="soul-preview">{{ selected.soul }}</pre>
            </div>
          </div>

          <!-- Editor -->
          <div v-else-if="form" class="editor">
            <div class="detail-head">
              <h2>{{ form.is_builtin ? 'New role' : (selected ? `Edit ${selected.name}` : 'New role') }}</h2>
              <div class="actions">
                <button class="btn-sm primary" :disabled="saving" @click="saveRole">{{ saving ? 'Saving…' : 'Save' }}</button>
                <button class="btn-sm" @click="cancelEdit">Cancel</button>
              </div>
            </div>
            <div v-if="saveNote" class="save-note" :class="{ ok: saveNote.startsWith('Saved'), fail: !saveNote.startsWith('Saved') }">{{ saveNote }}</div>

            <div class="grid">
              <div class="field span-2">
                <label>Name</label>
                <input v-model="form.name" type="text" />
              </div>
              <div class="field span-2">
                <label>Description</label>
                <input v-model="form.description" type="text" />
              </div>
              <div class="field">
                <label>Default tier</label>
                <select v-model="form.tier">
                  <option v-for="t in TIERS" :key="t" :value="t">{{ t }}</option>
                </select>
              </div>
              <div class="field">
                <label>Token budget <span class="muted">(not enforced)</span></label>
                <input v-model.number="form.token_budget" type="number" placeholder="(unbounded)" />
              </div>
              <div class="field">
                <label>Temperature</label>
                <input v-model.number="form.temperature" type="number" step="0.05" min="0" max="2" />
              </div>
              <div class="field">
                <label>Max turns</label>
                <input v-model.number="form.max_turns" type="number" />
              </div>
              <div class="field span-2" v-if="form.inherit">
                <label>Inherits (built-in base)</label>
                <input v-model="form.inherit" type="text" />
              </div>
            </div>

            <!-- Allowed tiers: multi-select toggles -->
            <div class="section">
              <h3>Allowed tiers <span class="muted">(empty = no restriction)</span></h3>
              <div class="tier-toggles">
                <button
                  v-for="t in TIERS" :key="t"
                  class="tier-toggle"
                  :class="{ on: form.allowed_tiers.includes(t) }"
                  :style="form.allowed_tiers.includes(t) ? { background: TIER_COLORS[t], borderColor: TIER_COLORS[t] } : {}"
                  @click="toggleTier(t)"
                >{{ t }}</button>
              </div>
            </div>

            <!-- Skills: checkbox list -->
            <div class="section">
              <h3>Skills <span class="muted">(whitelist — empty = all when enabled)</span></h3>
              <div v-if="allSkills.length === 0" class="muted">No skills installed.</div>
              <div v-else class="skill-grid">
                <label v-for="s in allSkills" :key="s" class="skill-check" :class="{ on: form.skills.includes(s) }">
                  <input type="checkbox" :checked="form.skills.includes(s)" @change="toggleSkill(s)" />
                  {{ s }}
                </label>
              </div>
            </div>

            <!-- Soul editor -->
            <div class="section">
              <h3>Soul (markdown)</h3>
              <textarea v-model="form.soul" class="soul-edit" rows="14" placeholder="# Soul of the Role&#10;&#10;## Personality&#10;You are ..."></textarea>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
/* Theme-variable colors follow the host's accent picker automatically. */
.roles-config { max-width: 1000px; }
.layout { display: grid; grid-template-columns: 280px 1fr; gap: 16px; align-items: start; }

/* List pane */
.list-pane { background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius, 8px); padding: 12px; }
.pane-head { display: flex; gap: 8px; margin-bottom: 8px; }
.filter-input { flex: 1; padding: 6px 10px; border: 1px solid var(--border); border-radius: var(--radius-sm, 4px); font-size: 12px; background: var(--bg-input); outline: none; transition: border-color .15s, box-shadow .15s; }
.filter-input:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-light); }
.stat { font-size: 11px; color: var(--text-muted); margin-bottom: 8px; padding: 0 4px; }
.role-item { display: flex; align-items: center; gap: 8px; padding: 8px 10px; border-radius: var(--radius-sm, 5px); cursor: pointer; transition: background .12s; }
.role-item:hover { background: var(--bg-hover); }
.role-item.active { background: var(--accent-light); }
.role-item.active .role-name { color: var(--accent); font-weight: 600; }
.lock { font-size: 13px; }
.lock.user { opacity: 0.6; }
.role-name { font-size: 13px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; flex: 1; }
.tier-dot { width: 9px; height: 9px; border-radius: 50%; flex-shrink: 0; }
.empty { color: var(--text-muted); font-size: 12px; padding: 16px; text-align: center; }

/* Detail pane */
.detail-pane { min-height: 400px; }
.placeholder { color: var(--text-muted); font-size: 14px; padding: 60px 20px; text-align: center; background: var(--bg-card); border: 1px dashed var(--border); border-radius: var(--radius, 8px); }
.detail-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
.detail-head h2 { font-size: 16px; font-weight: 600; margin: 0; }
.tag { font-size: 10px; padding: 1px 7px; border-radius: 8px; background: var(--bg-hover); color: var(--text-secondary); font-weight: 500; vertical-align: middle; }
.actions { display: flex; gap: 6px; }
.desc { font-size: 13px; color: var(--text-secondary); margin: 0 0 14px 0; }

.kv { border: 1px solid var(--border); border-radius: var(--radius-sm, 6px); padding: 4px 14px; margin-bottom: 16px; }
.kv-row { display: flex; align-items: center; gap: 8px; padding: 7px 0; border-bottom: 1px solid var(--border); font-size: 13px; }
.kv-row:last-child { border-bottom: none; }
.k { width: 130px; color: var(--text-muted); font-size: 12px; }
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 12px; }
.muted { color: var(--text-muted); font-size: 11px; font-weight: 400; }

.badge { font-size: 11px; padding: 2px 9px; border-radius: 10px; background: var(--accent-light); color: var(--accent); font-weight: 500; }
.badge.muted { background: var(--bg-hover); color: var(--text-secondary); }

.section { margin-bottom: 16px; }
.section h3 { font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: .03em; color: var(--text-muted); margin: 0 0 8px 0; }
.skill-chip { display: inline-block; font-size: 11px; padding: 3px 9px; border-radius: 10px; background: var(--accent-light); color: var(--accent); margin: 0 4px 4px 0; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.soul-preview { background: var(--bg-app); border: 1px solid var(--border); border-radius: var(--radius-sm, 6px); padding: 12px; font-size: 12px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; white-space: pre-wrap; max-height: 300px; overflow: auto; line-height: 1.5; color: var(--text-primary); }

/* Editor */
.editor .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px 16px; margin-bottom: 16px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.field.span-2 { grid-column: span 2; }
.field label { font-size: 11px; color: var(--text-muted); font-weight: 500; text-transform: uppercase; letter-spacing: .03em; }
.field input, .field select { padding: 7px 10px; border: 1px solid var(--border); border-radius: var(--radius-sm, 4px); font-size: 13px; background: var(--bg-input); outline: none; transition: border-color .15s, box-shadow .15s; }
.field input:focus, .field select:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-light); }

.tier-toggles { display: flex; gap: 8px; flex-wrap: wrap; }
.tier-toggle { padding: 5px 14px; border: 1px solid var(--border); border-radius: var(--radius-sm, 4px); background: var(--bg-card); cursor: pointer; font-size: 12px; font-weight: 500; text-transform: capitalize; color: var(--text-secondary); transition: all .12s; }
.tier-toggle:hover { border-color: var(--accent); }
.tier-toggle.on { color: #fff; }

.skill-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; }
.skill-check { display: flex; align-items: center; gap: 6px; font-size: 12px; padding: 5px 8px; border-radius: var(--radius-sm, 4px); cursor: pointer; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.skill-check.on { background: var(--accent-light); color: var(--accent); }
.skill-check input { margin: 0; }

.soul-edit { width: 100%; padding: 12px; border: 1px solid var(--border); border-radius: var(--radius-sm, 6px); font-size: 12px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; background: var(--bg-input); outline: none; resize: vertical; line-height: 1.5; transition: border-color .15s, box-shadow .15s; box-sizing: border-box; }
.soul-edit:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-light); }

/* Buttons */
.btn-sm { padding: 6px 14px; font-size: 12px; border: 1px solid var(--border); border-radius: var(--radius-sm, 4px); background: var(--bg-card); cursor: pointer; font-weight: 500; transition: background .15s, border-color .15s; }
.btn-sm:hover:not(:disabled) { background: var(--bg-hover); }
.btn-sm:disabled { opacity: .6; cursor: not-allowed; }
.btn-sm.primary { background: var(--accent); color: var(--accent-foreground, #fff); border-color: var(--accent); }
.btn-sm.primary:hover:not(:disabled) { background: var(--accent-hover); }
.btn-sm.danger { color: var(--danger); }
.btn-sm.danger:hover { background: rgba(196,43,28,.08); border-color: var(--danger); }

.save-note { padding: 8px 12px; border-radius: var(--radius-sm, 4px); font-size: 12px; margin-bottom: 12px; }
.save-note.ok { background: var(--accent-light); color: var(--accent); }
.save-note.fail { background: rgba(196,43,28,.08); color: var(--danger); }

.state-msg { padding: 14px; border-radius: var(--radius, 8px); background: var(--bg-hover); color: var(--text-secondary); font-size: 13px; }
.state-msg.error { background: rgba(196,43,28,.08); color: var(--danger); }
.state-msg.error .hint { display: block; margin-top: 6px; font-size: 12px; opacity: .85; }
.state-msg.error code { background: rgba(0,0,0,.08); padding: 1px 5px; border-radius: 3px; }
</style>
