<!--
  RelayCard.vue — Relay run 内联卡片（逃生舱，精简状态展示版）
  Plan 022 Phase 7c: 显示 spawn_relay 工具调用的 run 状态（run_id/flow/status/
  steps/summary/tokens）。完整交互（subscribeToRun/resolveGate/startRun）依赖
  useRelay composable 未接线，登记 KNOWN-DEBT，本 phase 仅做状态展示。
-->
<template>
  <div class="relay-card" :class="`status-${statusClass}`">
    <div class="relay-header" @click="expanded = !expanded">
      <span class="relay-icon">🛰️</span>
      <span class="relay-title">{{ run?.title || runId || 'Relay Run' }}</span>
      <span class="relay-progress" v-if="run">{{ run.steps.length }} steps</span>
      <span class="relay-status" :class="`badge-${statusClass}`">{{ statusLabel }}</span>
      <span class="tool-chevron">{{ expanded ? '▲' : '▼' }}</span>
    </div>
    <div v-if="expanded" class="relay-body">
      <div v-if="runId" class="relay-field">
        <span class="relay-field-label">Run ID:</span>
        <code class="relay-field-value">{{ runId }}</code>
      </div>
      <div v-if="run?.flow_id" class="relay-field">
        <span class="relay-field-label">Flow:</span>
        <span class="relay-field-value">{{ run.flow_id }}</span>
      </div>
      <!-- steps 列表 -->
      <div v-if="run?.steps?.length" class="relay-steps">
        <div
          v-for="(step, i) in run.steps"
          :key="i"
          class="relay-step"
        >
          <span class="relay-step-icon">{{ professionIcon(step.profession_id) }}</span>
          <span class="relay-step-name">{{ step.profession_id || step.step_id }}</span>
        </div>
      </div>
      <!-- summary -->
      <div v-if="run?.summary" class="relay-summary">
        <div class="relay-summary-label">Summary</div>
        <pre class="relay-summary-text">{{ run.summary }}</pre>
      </div>
      <div v-if="run?.tokens_used" class="relay-cost">{{ run.tokens_used }} tokens</div>
      <div v-if="!run" class="relay-empty">等待 relay 数据…</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  extractRunId,
  getRelayStatus,
  type ToolCallLike,
  type RelayRunState,
} from '../forge_helpers'

const props = defineProps<{
  tc: ToolCallLike
  relays: Record<string, RelayRunState>
}>()

const expanded = ref(false)

const runId = computed(() => extractRunId(props.tc))
const run = computed(() => getRelayStatus(props.relays, props.tc))

const statusClass = computed(() => {
  const s = run.value?.status ?? 'idle'
  if (s === 'completed') return 'completed'
  if (s === 'failed') return 'failed'
  if (s === 'gate_waiting') return 'gate'
  return 'running'
})

const statusLabel = computed(() => {
  const map: Record<string, string> = {
    running: '运行中',
    started: '启动中',
    completed: '已完成',
    failed: '失败',
    gate_waiting: '待审批',
    idle: '就绪',
  }
  return map[run.value?.status ?? 'idle'] ?? run.value?.status ?? '...'
})

function professionIcon(id: string): string {
  const map: Record<string, string> = {
    assistant: '📥', advisor: '💡', architect: '🏗️', planner: '📝',
    coder: '💻', tester: '🧪', reviewer: '🔍', documenter: '📚',
  }
  return map[id] ?? '⚙️'
}
</script>

<style scoped>
.relay-card {
  border: 1px solid var(--af-border, hsl(220 13% 91%));
  border-radius: 8px;
  margin: 0.5rem 0;
  overflow: hidden;
  background: hsl(220 14% 96% / 0.5);
}
.status-running { border-left: 3px solid hsl(220 90% 56%); }
.status-completed { border-left: 3px solid hsl(142 71% 45%); }
.status-failed { border-left: 3px solid hsl(0 72% 51%); }
.status-gate { border-left: 3px solid hsl(38 92% 50%); }
.relay-header {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.5rem 0.75rem;
  cursor: pointer;
  font-size: 0.82rem;
}
.relay-header:hover { background: hsl(220 14% 96% / 0.8); }
.relay-icon { font-size: 0.9rem; }
.relay-title { font-weight: 500; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.relay-progress { font-size: 0.72rem; color: var(--af-muted, hsl(220 9% 46%)); }
.relay-status { font-size: 0.7rem; padding: 0.1rem 0.4rem; border-radius: 3px; }
.badge-running, .badge-started { background: hsl(220 90% 56% / 0.15); color: hsl(220 90% 56%); }
.badge-completed { background: hsl(142 71% 45% / 0.15); color: hsl(142 71% 45%); }
.badge-failed { background: hsl(0 72% 51% / 0.15); color: hsl(0 72% 51%); }
.badge-gate { background: hsl(38 92% 50% / 0.15); color: hsl(38 92% 50%); }
.tool-chevron { font-size: 0.7rem; color: var(--af-muted, hsl(220 9% 46%)); }
.relay-body { padding: 0.5rem 0.75rem; border-top: 1px solid var(--af-border, hsl(220 13% 91%)); font-size: 0.8rem; }
.relay-field { display: flex; gap: 0.4rem; margin: 0.2rem 0; }
.relay-field-label { color: var(--af-muted, hsl(220 9% 46%)); min-width: 60px; }
.relay-field-value { font-family: monospace; font-size: 0.75rem; }
.relay-steps { margin: 0.4rem 0; display: flex; flex-direction: column; gap: 0.2rem; }
.relay-step { display: flex; align-items: center; gap: 0.3rem; font-size: 0.75rem; padding: 0.15rem 0; }
.relay-step-name { color: var(--af-fg, hsl(220 14% 10%)); }
.relay-summary { margin-top: 0.4rem; }
.relay-summary-label { font-size: 0.72rem; color: var(--af-muted, hsl(220 9% 46%)); margin-bottom: 0.2rem; }
.relay-summary-text { font-size: 0.76rem; white-space: pre-wrap; max-height: 200px; overflow-y: auto; }
.relay-cost { font-size: 0.72rem; color: var(--af-muted, hsl(220 9% 46%)); margin-top: 0.3rem; }
.relay-empty { font-size: 0.78rem; color: var(--af-muted, hsl(220 9% 46%)); font-style: italic; }
</style>
