<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useSpecs, statusColor, type SpecStatus, type SpecItem } from '../composables/useSpecs'

const { doc, overview, loading, error, loadDoc, loadOverview, transitionItem, deleteItem } = useSpecs()

const activeSectionId = ref('goals')
const activeSection = computed(() =>
  doc.value?.sections.find(s => s.id === activeSectionId.value) ?? null
)

onMounted(async () => {
  await loadDoc()
  await loadOverview()
})

function badgeClass(s: SpecStatus) {
  const c = statusColor(s)
  const map: Record<string, string> = {
    gray: 'badge-gray', yellow: 'badge-yellow', blue: 'badge-blue',
    green: 'badge-green', red: 'badge-red',
  }
  return map[c] || 'badge-gray'
}

// status transition options per section type (mirrors backend SectionConfig)
const transitions: Record<string, SpecStatus[]> = {
  goals: ['Empty', 'Proposed', 'Analysed', 'Approved', 'InProgress', 'Implemented', 'Verified', 'Done', 'Archived'],
  architecture: ['Empty', 'Draft', 'UnderReview', 'Approved', 'Rejected', 'Superseded', 'Outdated'],
  designs: ['Empty', 'Draft', 'UnderReview', 'Approved', 'Rejected', 'Superseded', 'Outdated'],
  plans: ['Empty', 'Draft', 'Approved', 'InProgress', 'Done', 'Obsolete'],
  tests: ['Empty', 'Draft', 'Implemented', 'Done', 'Verified', 'Blocked'],
  reviews: ['Empty', 'Draft', 'Published'],
  reports: ['Empty', 'Draft', 'Published'],
}

async function onStatusChange(item: SpecItem, newStatus: SpecStatus) {
  if (item.status === newStatus) return
  await transitionItem(activeSectionId.value, item.id, newStatus)
}

async function onDelete(item: SpecItem) {
  if (confirm(`Delete ${item.id}?`)) {
    await deleteItem(activeSectionId.value, item.id)
  }
}
</script>

<template>
  <div class="specs-view">
    <aside class="sidebar">
      <h3>Specs</h3>
      <div v-if="overview" class="overview-summary">
        <span>{{ overview.total_items }} items</span>
      </div>
      <ul class="section-list">
        <li
          v-for="s in overview?.sections ?? []"
          :key="s.id"
          :class="['section-item', { active: s.id === activeSectionId }]"
          @click="activeSectionId = s.id"
        >
          <span class="section-title">{{ s.title }}</span>
          <span class="section-count">{{ s.item_count }}</span>
        </li>
      </ul>
    </aside>

    <main class="content">
      <div v-if="loading" class="empty">Loading…</div>
      <div v-else-if="error" class="empty error">{{ error }}</div>
      <div v-else-if="!activeSection" class="empty">No section selected.</div>
      <div v-else>
        <header class="section-header">
          <h2>{{ activeSection.title }}</h2>
          <span :class="['badge', badgeClass(activeSection.status)]">{{ activeSection.status }}</span>
        </header>

        <div v-if="activeSection.items.length === 0" class="empty">
          No items in this section yet.
        </div>

        <div v-else class="item-list">
          <div v-for="item in activeSection.items" :key="item.id" class="item-card">
            <div class="item-head">
              <span class="item-id">{{ item.id }}</span>
              <span class="item-title">{{ item.title }}</span>
              <span :class="['badge', badgeClass(item.status)]">{{ item.status }}</span>
              <select
                class="status-select"
                :value="item.status"
                @change="onStatusChange(item, ($event.target as HTMLSelectElement).value as SpecStatus)"
              >
                <option
                  v-for="s in transitions[activeSectionId] ?? []"
                  :key="s"
                  :value="s"
                >{{ s }}</option>
              </select>
              <button class="btn-danger-sm" @click="onDelete(item)">✕</button>
            </div>
            <p v-if="item.content" class="item-content">{{ item.content }}</p>
            <div v-if="item.depends_on.length || item.related.length" class="item-relations">
              <span v-if="item.depends_on.length" class="rels">
                <strong>depends:</strong> {{ item.depends_on.join(', ') }}
              </span>
              <span v-if="item.related.length" class="rels">
                <strong>related:</strong> {{ item.related.join(', ') }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.specs-view { display: flex; height: 100%; }
.sidebar {
  width: 220px; flex-shrink: 0; padding: 12px;
  border-right: 1px solid var(--border); background: var(--bg-panel); overflow-y: auto;
}
.sidebar h3 { margin: 0 0 8px; font-size: 14px; }
.overview-summary { font-size: 12px; color: var(--text-muted); margin-bottom: 12px; }
.section-list { list-style: none; padding: 0; margin: 0; }
.section-item {
  display: flex; justify-content: space-between; align-items: center;
  padding: 6px 8px; border-radius: var(--radius-sm); cursor: pointer; font-size: 13px;
}
.section-item:hover { background: var(--accent-light); }
.section-item.active { background: var(--accent-light); color: var(--accent); font-weight: 600; }
.section-count {
  font-size: 11px; background: var(--bg-elevated); padding: 1px 6px; border-radius: 8px; color: var(--text-muted);
}
.content { flex: 1; padding: 16px 24px; overflow-y: auto; }
.section-header { display: flex; align-items: center; gap: 10px; margin-bottom: 16px; }
.section-header h2 { margin: 0; font-size: 18px; }
.empty { color: var(--text-muted); padding: 32px; text-align: center; }
.empty.error { color: var(--danger); }
.item-list { display: flex; flex-direction: column; gap: 10px; }
.item-card {
  border: 1px solid var(--border); border-radius: var(--radius); padding: 12px; background: var(--bg-panel);
}
.item-head { display: flex; align-items: center; gap: 8px; }
.item-id {
  font-family: monospace; font-size: 12px; font-weight: 700; color: var(--accent);
  background: var(--accent-light); padding: 2px 6px; border-radius: 4px;
}
.item-title { font-weight: 600; flex: 1; font-size: 14px; }
.status-select {
  font-size: 11px; padding: 2px 4px; border: 1px solid var(--border);
  border-radius: var(--radius-sm); background: var(--bg-elevated); color: var(--text);
}
.btn-danger-sm {
  border: none; background: none; color: var(--text-muted); cursor: pointer; font-size: 14px;
}
.btn-danger-sm:hover { color: var(--danger); }
.item-content { margin: 8px 0 0; font-size: 13px; color: var(--text-secondary); white-space: pre-wrap; }
.item-relations { margin-top: 6px; font-size: 11px; color: var(--text-muted); display: flex; gap: 12px; }
.badge {
  font-size: 11px; padding: 2px 8px; border-radius: 10px; font-weight: 500;
  border: 1px solid transparent; white-space: nowrap;
}
.badge-gray { background: #f3f4f6; color: #6b7280; border-color: #e5e7eb; }
.badge-yellow { background: #fef3c7; color: #92400e; border-color: #fcd34d; }
.badge-blue { background: #dbeafe; color: #1e40af; border-color: #93c5fd; }
.badge-green { background: #d1fae5; color: #065f46; border-color: #6ee7b7; }
.badge-red { background: #fee2e2; color: #991b1b; border-color: #fca5a5; }
</style>
