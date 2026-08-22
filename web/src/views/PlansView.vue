<template>
  <div class="plans-view">
    <aside class="plans-nav">
      <div class="nav-header">
        <span class="nav-title">{{ t('plans.title') }}</span>
        <button class="nav-icon-btn" :title="t('plans.newPlan')" @click="startCreate">
          <Plus :size="16" />
        </button>
      </div>
      <div class="nav-filter-group">
        <button
          class="filter-btn"
          :class="{ active: filterMode === 'active' }"
          @click="filterMode = 'active'"
        >{{ t('plans.filterActive') }}</button>
        <button
          class="filter-btn"
          :class="{ active: filterMode === 'all' }"
          @click="filterMode = 'all'"
        >{{ t('plans.filterAll') }}</button>
      </div>
      <div class="nav-list">
        <div v-if="isLoading" class="nav-empty">{{ t('plans.loading') }}</div>
        <div v-else-if="plans.length === 0" class="nav-empty">{{ t('plans.empty') }}</div>
        <button
          v-for="p in plans"
          :key="p.seq"
          class="plan-item"
          :class="{ active: selectedSeq === p.seq, archived: p.archived }"
          @click="selectPlan(p.seq)"
        >
          <span class="plan-seq">{{ String(p.seq).padStart(3, '0') }}</span>
          <span class="plan-name">{{ p.feature_name || p.title || p.id }}</span>
          <PlanStatusBadge :status="p.status" />
        </button>
      </div>
    </aside>

    <section class="plans-content">
      <div v-if="!current" class="content-empty">
        {{ isLoading ? t('plans.loading') : t('plans.selectPlan') }}
      </div>
      <template v-else>
        <header class="content-header">
          <div class="header-left">
            <span class="plan-id">{{ current.id }}</span>
            <PlanStatusBadge :status="current.status" />
            <span class="plan-feature">{{ current.feature_name }}</span>
          </div>
          <div class="header-actions">
            <template v-if="!editing">
              <!-- 状态转移由 AI Agent（auto-plan 技能/relay plan_flow）驱动，UI 不提供手动转移（PLAN-033 复审后调整） -->
              <button class="action-btn" @click="startEdit">{{ t('plans.edit') }}</button>
              <!-- PLAN-033 单一终态：reviewed 只显示"沉淀到 Spec"（沉淀即归档）；
                   非 reviewed 未归档显示"归档"（搁置不沉淀）；两者互斥。 -->
              <button
                v-if="!current.archived && current.status !== 'reviewed'"
                class="action-btn accent"
                @click="onArchive"
              >{{ t('plans.archive') }}</button>
              <button
                v-if="!current.archived && current.status === 'reviewed'"
                class="action-btn primary"
                @click="onMerge"
              >{{ t('plans.mergeToSpec') }}</button>
            </template>
            <template v-else>
              <button class="action-btn primary" @click="onSave">{{ t('plans.save') }}</button>
              <button class="action-btn" @click="cancelEdit">{{ t('plans.cancel') }}</button>
            </template>
          </div>
        </header>
        <div class="content-scroll">
          <div v-if="editing" class="edit-area">
            <textarea
              v-model="editBody"
              class="edit-textarea"
              :placeholder="t('plans.contentPlaceholder')"
            ></textarea>
          </div>
          <template v-else>
            <PlanMetaBlock v-if="parsed" :meta="parsed.meta" />
            <MarkdownContent :content="parsed ? parsed.body : current.content" />
          </template>
        </div>
      </template>
    </section>

    <!-- New-plan dialog -->
    <div v-if="creating" class="modal-overlay" @click.self="creating = false">
      <div class="modal">
        <h3 class="modal-title">{{ t('plans.newPlan') }}</h3>
        <label class="modal-label">{{ t('plans.featureName') }}</label>
        <input
          v-model="newName"
          class="modal-input"
          :placeholder="t('plans.featureNamePlaceholder')"
          @keyup.enter="doCreate"
        />
        <div class="modal-actions">
          <button class="action-btn" @click="creating = false">{{ t('plans.cancel') }}</button>
          <button class="action-btn primary" :disabled="!newName.trim()" @click="doCreate">
            {{ t('plans.create') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Plus } from 'lucide-vue-next'
import MarkdownContent from '@/components/MarkdownContent.vue'
import PlanStatusBadge from '@/components/plan/PlanStatusBadge.vue'
import PlanMetaBlock from '@/components/plan/PlanMetaBlock.vue'
import { splitFrontmatter } from '@/utils/frontmatter'
import { usePlans } from '@/composables/usePlans'

const { t } = useI18n()
const {
  plans,
  current,
  isLoading,
  loadPlans,
  loadPlan,
  createPlan,
  updatePlan,
  archivePlan,
  mergePlan,
} = usePlans()

/** 过滤档位持久化键（与语言键同 autoforge- 前缀风格）。 */
const FILTER_KEY = 'autoforge-plans-filter'
type FilterMode = 'active' | 'all'
const filterMode = ref<FilterMode>(
  localStorage.getItem(FILTER_KEY) === 'all' ? 'all' : 'active',
)

const selectedSeq = ref<number | null>(null)
const editing = ref(false)
const editBody = ref('')
const creating = ref(false)
const newName = ref('')

/** 拆出 frontmatter：渲染区只喂正文，meta 交给 PlanMetaBlock（PLAN-033 T9）。 */
const parsed = computed(() => (current.value ? splitFrontmatter(current.value.content) : null))

async function refresh() {
  await loadPlans(filterMode.value === 'all')
}

async function selectPlan(seq: number) {
  selectedSeq.value = seq
  editing.value = false
  await loadPlan(seq)
}

function startCreate() {
  newName.value = ''
  creating.value = true
}

async function doCreate() {
  const name = newName.value.trim()
  if (!name) return
  const p = await createPlan(name)
  if (p) {
    creating.value = false
    await refresh()
    await selectPlan(p.seq)
  }
}

function startEdit() {
  if (current.value) editBody.value = current.value.content
  editing.value = true
}

function cancelEdit() {
  editing.value = false
  if (current.value) editBody.value = current.value.content
}

async function onSave() {
  if (!current.value) return
  const updated = await updatePlan(current.value.seq, editBody.value)
  if (updated) editing.value = false
}

async function onArchive() {
  if (!current.value) return
  if (!confirm(t('plans.archiveConfirm', { id: current.value.id }))) return
  const p = await archivePlan(current.value.seq)
  if (p) {
    await refresh()
    await selectPlan(p.seq)
  }
}

async function onMerge() {
  if (!current.value) return
  if (!confirm(t('plans.mergeConfirm'))) return
  const result = await mergePlan(current.value.seq)
  if (result) {
    alert(
      t('plans.mergeSuccess', {
        count: result.items_created,
        sections: result.sections_touched.length,
      }),
    )
    await refresh()
    await selectPlan(current.value.seq)
  }
}

// keep edit body in sync when current plan changes
watch(current, (c) => {
  if (c) editBody.value = c.content
})

// 过滤档位切换：持久化 + 重新加载列表
watch(filterMode, (m) => {
  localStorage.setItem(FILTER_KEY, m)
  refresh()
})

onMounted(() => {
  refresh()
})
</script>

<style scoped>
.plans-view {
  display: flex;
  height: 100%;
  overflow: hidden;
}

/* ── Sidebar ─────────────────────────────────────────────── */
.plans-nav {
  width: 240px;
  flex-shrink: 0;
  border-right: 1px solid var(--af-border);
  display: flex;
  flex-direction: column;
  background: var(--af-bg);
}
.nav-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.6rem 0.75rem;
  border-bottom: 1px solid var(--af-border);
}
.nav-title {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--af-fg);
}
.nav-icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  background: transparent;
  border: 1px solid var(--af-border);
  border-radius: 5px;
  color: var(--af-muted);
  cursor: pointer;
}
.nav-icon-btn:hover {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
}
.nav-filter-group {
  display: inline-flex;
  margin: 0.5rem 0.75rem;
}
.filter-btn {
  font-size: 0.78rem;
  padding: 0.25rem 0.65rem;
  background: var(--af-bg);
  color: var(--af-muted);
  border: 1px solid var(--af-border);
  cursor: pointer;
  white-space: nowrap;
}
.filter-btn + .filter-btn {
  border-left: none;
}
.filter-btn:first-child {
  border-radius: 5px 0 0 5px;
}
.filter-btn:last-child {
  border-radius: 0 5px 5px 0;
}
.filter-btn:hover {
  background: hsl(var(--muted-foreground) / 0.07);
}
.filter-btn.active {
  background: hsl(var(--primary) / 0.12);
  color: var(--af-primary);
  border-color: hsl(var(--primary) / 0.4);
}
.filter-btn.active + .filter-btn {
  border-left: 1px solid hsl(var(--primary) / 0.4);
}
.nav-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.25rem;
}
.nav-empty {
  padding: 1rem 0.75rem;
  font-size: 0.82rem;
  color: var(--af-muted);
}
.plan-item {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  width: 100%;
  padding: 0.4rem 0.5rem;
  background: transparent;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  text-align: left;
  color: var(--af-fg);
  font-size: 0.83rem;
}
.plan-item:hover {
  background: hsl(var(--muted-foreground) / 0.06);
}
.plan-item.active {
  background: hsl(var(--primary) / 0.1);
  color: var(--af-primary);
}
.plan-item.archived {
  opacity: 0.6;
}
.plan-seq {
  font-family: ui-monospace, SFMono-Regular, monospace;
  font-size: 0.74rem;
  color: var(--af-muted);
  flex-shrink: 0;
}
.plan-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ── Content ─────────────────────────────────────────────── */
.plans-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.content-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--af-muted);
  font-size: 0.9rem;
}
.content-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding: 0.55rem 1rem;
  border-bottom: 1px solid var(--af-border);
  flex-shrink: 0;
  flex-wrap: wrap;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}
.plan-id {
  font-family: ui-monospace, SFMono-Regular, monospace;
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--af-primary);
}
.plan-feature {
  font-size: 0.82rem;
  color: var(--af-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.header-actions {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}
.action-btn {
  font-size: 0.78rem;
  padding: 0.25rem 0.6rem;
  border: 1px solid var(--af-border);
  border-radius: 4px;
  background: transparent;
  color: var(--af-fg);
  cursor: pointer;
  white-space: nowrap;
}
.action-btn:hover {
  background: hsl(var(--muted-foreground) / 0.08);
}
.action-btn.primary {
  background: hsl(var(--primary) / 0.12);
  border-color: hsl(var(--primary) / 0.4);
  color: var(--af-primary);
}
.action-btn.primary:hover {
  background: hsl(var(--primary) / 0.2);
}
/* 归档按钮主题色（弱于 primary，避免与"沉淀到 Spec"抢焦点） */
.action-btn.accent {
  background: hsl(var(--primary) / 0.07);
  border-color: hsl(var(--primary) / 0.35);
  color: var(--af-primary);
}
.action-btn.accent:hover {
  background: hsl(var(--primary) / 0.15);
}
.action-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.content-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 1rem 1.25rem;
}
/* 计划正文的 H1/H2 用主题色（仅本页，不影响 chat 等其它 Markdown 渲染） */
.content-scroll :deep(h1),
.content-scroll :deep(h2) {
  color: var(--af-primary);
}
/* 正文首块不设上 margin（MetaBlock 与首个标题之间的空隙来源；
   markstream 的块可能有包装层，覆盖三层嵌套内的首元素链） */
.content-scroll :deep(.markdown-content > :first-child),
.content-scroll :deep(.markdown-content > :first-child > :first-child),
.content-scroll :deep(.markdown-content > :first-child > :first-child > :first-child) {
  margin-top: 0;
}
.edit-area {
  height: 100%;
  display: flex;
}
.edit-textarea {
  flex: 1;
  width: 100%;
  resize: none;
  border: 1px solid var(--af-border);
  border-radius: 6px;
  padding: 0.75rem;
  font-family: ui-monospace, SFMono-Regular, monospace;
  font-size: 0.85rem;
  line-height: 1.5;
  background: var(--af-bg);
  color: var(--af-fg);
  outline: none;
}
.edit-textarea:focus {
  border-color: hsl(var(--primary) / 0.5);
}

/* ── Modal ───────────────────────────────────────────────── */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: hsl(0 0% 0% / 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
}
.modal {
  background: var(--af-bg);
  border: 1px solid var(--af-border);
  border-radius: 8px;
  padding: 1.25rem;
  width: 380px;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  box-shadow: 0 8px 24px hsl(0 0% 0% / 0.2);
}
.modal-title {
  font-size: 1rem;
  font-weight: 600;
  margin-bottom: 0.25rem;
}
.modal-label {
  font-size: 0.78rem;
  color: var(--af-muted);
}
.modal-input {
  padding: 0.4rem 0.5rem;
  border: 1px solid var(--af-border);
  border-radius: 4px;
  font-size: 0.85rem;
  background: var(--af-bg);
  color: var(--af-fg);
  outline: none;
}
.modal-input:focus {
  border-color: hsl(var(--primary) / 0.5);
}
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.4rem;
  margin-top: 0.5rem;
}
</style>
