<!--
  TaskPlanCard.vue — Task Plan 内联卡片（逃生舱）
  Plan 022 Phase 7c: 显示 task_plan 工具调用的状态（instance_id/task_plan_id/
  status/phases）。状态来自 store.task_plans（task_plan_spawned 事件回写）。
-->
<template>
  <div class="task-plan-card" :class="plan?.status || 'idle'">
    <div class="tp-header" @click="expanded = !expanded">
      <span class="tp-icon">📋</span>
      <span class="tp-title">Task Plan</span>
      <span class="tp-status" :class="plan?.status || 'idle'">{{ statusLabel }}</span>
      <span v-if="plan?.phases?.length" class="tp-progress">{{ plan.phases.length }} phases</span>
      <span class="tool-chevron">{{ expanded ? '▲' : '▼' }}</span>
    </div>
    <div v-if="expanded" class="tp-body">
      <div v-if="plan?.task_plan_id" class="tp-field">
        <span class="tp-field-label">Plan:</span>
        <code class="tp-field-value">{{ plan.task_plan_id }}</code>
      </div>
      <div v-if="plan?.instance_id" class="tp-field">
        <span class="tp-field-label">Instance:</span>
        <code class="tp-field-value">{{ plan.instance_id }}</code>
      </div>
      <!-- phases 列表 -->
      <div v-if="plan?.phases?.length" class="tp-phases">
        <div v-for="(ph, i) in plan.phases" :key="i" class="tp-phase" :class="ph.status">
          <span class="tp-phase-icon">{{ phaseIcon(ph.status) }}</span>
          <span class="tp-phase-name">{{ ph.phase }}</span>
          <span class="tp-phase-status">{{ ph.status }}</span>
        </div>
      </div>
      <div v-if="!plan" class="tp-empty">等待 task plan 数据…</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  getTaskPlan,
  type ToolCallLike,
  type TaskPlanState,
} from '../forge_helpers'

const props = defineProps<{
  tc: ToolCallLike
  taskPlans: Record<string, TaskPlanState>
}>()

const expanded = ref(false)
const plan = computed(() => getTaskPlan(props.taskPlans, props.tc))

const statusLabel = computed(() => {
  const map: Record<string, string> = {
    started: '启动中', running: '运行中', completed: '已完成', failed: '失败', idle: '就绪',
  }
  return map[plan.value?.status ?? 'idle'] ?? plan.value?.status ?? '...'
})

function phaseIcon(status: string): string {
  if (status === 'completed') return '✓'
  if (status === 'running') return '▶'
  if (status === 'failed') return '✗'
  return '○'
}
</script>

<style scoped>
.task-plan-card {
  border: 1px solid var(--af-border, hsl(220 13% 91%));
  border-radius: 8px;
  margin: 0.5rem 0;
  overflow: hidden;
  background: hsl(280 60% 96% / 0.4);
}
.tp-header {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.5rem 0.75rem;
  cursor: pointer;
  font-size: 0.82rem;
}
.tp-header:hover { background: hsl(280 60% 96% / 0.7); }
.tp-icon { font-size: 0.9rem; }
.tp-title { font-weight: 500; flex: 1; }
.tp-status { font-size: 0.7rem; padding: 0.1rem 0.4rem; border-radius: 3px; }
.tp-status.running, .tp-status.started { background: hsl(220 90% 56% / 0.15); color: hsl(220 90% 56%); }
.tp-status.completed { background: hsl(142 71% 45% / 0.15); color: hsl(142 71% 45%); }
.tp-status.failed { background: hsl(0 72% 51% / 0.15); color: hsl(0 72% 51%); }
.tp-progress { font-size: 0.72rem; color: var(--af-muted, hsl(220 9% 46%)); }
.tool-chevron { font-size: 0.7rem; color: var(--af-muted, hsl(220 9% 46%)); }
.tp-body { padding: 0.5rem 0.75rem; border-top: 1px solid var(--af-border, hsl(220 13% 91%)); font-size: 0.8rem; }
.tp-field { display: flex; gap: 0.4rem; margin: 0.2rem 0; }
.tp-field-label { color: var(--af-muted, hsl(220 9% 46%)); min-width: 60px; }
.tp-field-value { font-family: monospace; font-size: 0.75rem; }
.tp-phases { margin: 0.4rem 0; display: flex; flex-direction: column; gap: 0.2rem; }
.tp-phase { display: flex; align-items: center; gap: 0.4rem; font-size: 0.75rem; padding: 0.15rem 0; }
.tp-phase-name { flex: 1; color: var(--af-fg, hsl(220 14% 10%)); }
.tp-phase-status { font-size: 0.68rem; color: var(--af-muted, hsl(220 9% 46%)); }
.tp-empty { font-size: 0.78rem; color: var(--af-muted, hsl(220 9% 46%)); font-style: italic; }
</style>
