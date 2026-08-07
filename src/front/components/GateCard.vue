<!--
  GateCard.vue — Spec 审批门卡片（逃生舱，从 web/ 移植）
  Plan 022 Phase 7.3: 显示待审批的 spec 变更（section_id/old→new/可编辑）。
  props: { title?, profession?, changes }, emit approve(editedSpecs)/reject。
  SpecChange 类型自包含（去 @/types/forge 依赖）。内态：expandedDiffs/editedSpecs/
  isReviewing/approveBtn 自动聚焦/watch changes 同步——全部组件内自管。
-->
<template>
  <div class="gate-card">
    <div class="gate-card-header">
      <span class="gate-icon">🔒</span>
      <span class="gate-title">{{ title }}</span>
      <span v-if="profession" class="gate-profession">{{ profession }}</span>
    </div>

    <div v-if="changes && changes.length > 0" class="gate-diff-list">
      <div
        v-for="change in changes"
        :key="change.section_id"
        class="diff-card"
      >
        <div class="diff-header" @click="toggleDiff(change.section_id)">
          <span class="diff-title">{{ change.section_id }}</span>
          <span class="diff-status" :class="change.new_status">
            {{ change.old_status }} → {{ change.new_status }}
          </span>
          <ChevronDown v-if="!expandedDiffs.has(change.section_id)" :size="14" class="diff-chevron" />
          <ChevronUp v-else :size="14" class="diff-chevron" />
        </div>
        <div v-if="expandedDiffs.has(change.section_id)" class="diff-body">
          <div class="diff-side">
            <div class="diff-label">Before</div>
            <pre class="diff-content old">{{ change.old_content }}</pre>
          </div>
          <div class="diff-side">
            <div class="diff-label">After</div>
            <textarea
              v-model="editedSpecs[change.section_id]"
              class="diff-editor"
              rows="6"
            />
          </div>
        </div>
      </div>
    </div>

    <div class="gate-actions">
      <button ref="approveBtnRef" class="approve-btn" @click="$emit('approve', editedSpecs)">
        <Check :size="14" /> Approve & Execute
      </button>
      <button class="reject-btn" @click="$emit('reject')">
        <X :size="14" /> Reject & Redraft
      </button>
      <button class="review-btn" @click="isReviewing = !isReviewing">
        <Eye :size="14" /> {{ isReviewing ? 'Hide' : 'Review' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue'
import { Check, X, Eye, ChevronDown, ChevronUp } from 'lucide-vue-next'

// SpecChange 类型自包含（对应 web/src/types/forge.ts，逃生舱无 @/types/forge）。
export interface SpecChange {
  section_id: string
  old_content: string
  new_content: string
  old_status: string
  new_status: string
}

const props = withDefaults(defineProps<{
  title?: string
  profession?: string
  changes: SpecChange[]
}>(), {
  title: 'Spec drafted. Review the proposed changes below.',
  profession: undefined,
})

defineEmits<{
  (e: 'approve', editedSpecs: Record<string, string>): void
  (e: 'reject'): void
}>()

const expandedDiffs = ref<Set<string>>(new Set())
const editedSpecs = ref<Record<string, string>>({})
const isReviewing = ref(false)
const approveBtnRef = ref<HTMLButtonElement | null>(null)

onMounted(() => {
  nextTick(() => {
    approveBtnRef.value?.focus()
  })
})

function toggleDiff(sectionId: string) {
  if (expandedDiffs.value.has(sectionId)) {
    expandedDiffs.value.delete(sectionId)
  } else {
    expandedDiffs.value.add(sectionId)
  }
}

watch(() => props.changes, (changes) => {
  for (const change of changes) {
    if (!(change.section_id in editedSpecs.value)) {
      editedSpecs.value[change.section_id] = change.new_content
    }
  }
}, { immediate: true, deep: true })
</script>

<style scoped>
.gate-card {
  display: flex; flex-direction: column; gap: 0.6rem;
  padding: 0.75rem 1.25rem; border-top: 1px solid var(--af-border, hsl(220 13% 91%));
  flex-shrink: 0; background: hsl(220 14% 50% / 0.02);
}
.gate-card-header {
  display: flex; align-items: center; gap: 0.5rem;
  font-size: 0.93rem; color: var(--af-fg, hsl(220 14% 10%));
}
.gate-icon { font-size: 1.18rem; }
.gate-title { flex: 1; }
.gate-profession {
  font-size: 0.78rem; padding: 0.15rem 0.4rem; border-radius: 4px;
  background: hsl(220 90% 56% / 0.08); color: hsl(220 90% 56%); font-weight: 500;
}
.gate-diff-list {
  display: flex; flex-direction: column; gap: 0.35rem;
  max-height: 300px; overflow-y: auto;
}
.diff-card { border: 1px solid var(--af-border, hsl(220 13% 91%)); border-radius: 8px; overflow: hidden; }
.diff-header {
  display: flex; align-items: center; gap: 0.5rem;
  padding: 0.4rem 0.6rem; cursor: pointer; user-select: none;
}
.diff-header:hover { background: hsl(220 14% 50% / 0.03); }
.diff-title {
  font-size: 0.88rem; font-weight: 500; color: var(--af-fg, hsl(220 14% 10%));
  text-transform: capitalize; flex: 1;
}
.diff-status { font-size: 0.73rem; font-weight: 500; color: hsl(38 92% 50%); }
.diff-status.approved { color: hsl(142 71% 45%); }
.diff-chevron { color: var(--af-muted, hsl(220 9% 46%)); }
.diff-body {
  display: grid; grid-template-columns: 1fr 1fr; gap: 0.5rem;
  padding: 0.5rem 0.6rem; background: hsl(220 14% 50% / 0.02);
  border-top: 1px solid var(--af-border, hsl(220 13% 91%));
}
.diff-side { display: flex; flex-direction: column; gap: 0.2rem; }
.diff-label {
  font-size: 0.73rem; font-weight: 500; text-transform: uppercase;
  color: var(--af-muted, hsl(220 9% 46%)); letter-spacing: 0.02em;
}
.diff-content {
  font-size: 0.83rem; font-family: 'JetBrains Mono', 'Fira Code', monospace;
  background: var(--af-bg, #fff); border: 1px solid var(--af-border, hsl(220 13% 91%));
  border-radius: 4px; padding: 0.35rem; overflow-x: auto;
  white-space: pre-wrap; word-break: break-word;
  color: var(--af-fg, hsl(220 14% 10%)); margin: 0;
}
.diff-content.old { color: var(--af-muted, hsl(220 9% 46%)); }
.diff-editor {
  font-size: 0.83rem; font-family: 'JetBrains Mono', 'Fira Code', monospace;
  background: var(--af-bg, #fff); border: 1px solid var(--af-border, hsl(220 13% 91%));
  border-radius: 4px; padding: 0.35rem; color: var(--af-fg, hsl(220 14% 10%));
  resize: vertical; outline: none; width: 100%; box-sizing: border-box;
}
.diff-editor:focus { border-color: hsl(220 90% 56% / 0.4); }
.gate-actions { display: flex; gap: 0.5rem; }
.approve-btn, .reject-btn, .review-btn {
  display: inline-flex; align-items: center; gap: 0.35rem;
  padding: 0.4rem 0.9rem; border: none; border-radius: 6px;
  font-size: 0.88rem; font-weight: 500; cursor: pointer; transition: opacity 0.15s;
}
.approve-btn { background: hsl(142 71% 45%); color: #fff; }
.reject-btn { background: transparent; color: var(--af-fg, hsl(220 14% 10%)); border: 1px solid var(--af-border, hsl(220 13% 91%)); }
.review-btn { background: hsl(220 14% 50% / 0.08); color: var(--af-fg, hsl(220 14% 10%)); border: 1px solid var(--af-border, hsl(220 13% 91%)); }
.approve-btn:hover, .reject-btn:hover, .review-btn:hover { opacity: 0.85; }
</style>
