<!--
  ErrandCard.vue — Errand 内联卡片（逃生舱，从 web/src/views/ChatsView.vue:259-294 移植）
  Plan 022 Phase 7c: 显示 dispatch 工具调用的 errand 状态（task/status/token_usage/
  content/子工具/result）。展开/折叠交互在组件内部（.at 无法表达 @click toggle）。
-->
<template>
  <div class="errand-card" :class="errandStatus?.status || tc.status || 'running'">
    <div class="errand-header" @click="expanded = !expanded">
      <span class="errand-icon">🔍</span>
      <span class="errand-name">Errand: {{ task }}</span>
      <span class="errand-status" :class="errandStatus?.status || 'running'">
        {{ errandStatus?.status || 'running' }}
      </span>
      <span v-if="errandStatus?.token_usage" class="errand-cost">
        {{ errandStatus.token_usage }} tok
      </span>
      <span class="tool-chevron">{{ expanded ? '▲' : '▼' }}</span>
    </div>
    <div v-if="expanded" class="errand-body">
      <div class="errand-task">{{ task }}</div>
      <!-- 实时 errand content -->
      <pre v-if="errandStatus?.content" class="errand-content">{{ errandStatus.content }}</pre>
      <!-- errand 子工具 -->
      <div v-if="errandStatus?.tool_calls?.length" class="errand-tool-calls">
        <div
          v-for="(etc, i) in errandStatus.tool_calls"
          :key="i"
          class="errand-sub-tool"
        >
          <div class="errand-sub-tool-header">
            <span class="errand-sub-tool-name">{{ etc.name }}</span>
            <span class="errand-sub-tool-status" :class="etc.status">{{ etc.status }}</span>
          </div>
          <pre v-if="etc.result" class="errand-sub-tool-result">{{ etc.result }}</pre>
        </div>
      </div>
      <!-- 最终 result -->
      <div
        v-if="errandStatus?.result && errandStatus?.status !== 'running'"
        class="errand-result"
      >
        <div class="errand-result-label">Result</div>
        <pre class="errand-result-text">{{ errandStatus.result }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  getErrandTask,
  getErrandState,
  type ToolCallLike,
  type ErrandState,
} from '../forge_helpers'

const props = defineProps<{
  tc: ToolCallLike
  errands: Record<string, ErrandState>
}>()

const expanded = ref(false)

const task = computed(() => getErrandTask(props.tc))
const errandStatus = computed(() => getErrandState(props.errands, props.tc))
</script>

<style scoped>
.errand-card {
  border: 1px solid var(--af-border, hsl(220 13% 91%));
  border-radius: 8px;
  margin: 0.5rem 0;
  overflow: hidden;
  background: hsl(38 92% 50% / 0.03);
}
.errand-header {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.5rem 0.75rem;
  cursor: pointer;
  font-size: 0.82rem;
}
.errand-header:hover {
  background: hsl(38 92% 50% / 0.06);
}
.errand-icon { font-size: 0.9rem; }
.errand-name { font-weight: 500; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.errand-status { font-size: 0.7rem; padding: 0.1rem 0.4rem; border-radius: 3px; }
.errand-status.running { background: hsl(220 90% 56% / 0.15); color: hsl(220 90% 56%); }
.errand-status.completed { background: hsl(142 71% 45% / 0.15); color: hsl(142 71% 45%); }
.errand-status.failed { background: hsl(0 72% 51% / 0.15); color: hsl(0 72% 51%); }
.errand-cost { font-size: 0.7rem; color: var(--af-muted, hsl(220 9% 46%)); }
.tool-chevron { font-size: 0.7rem; color: var(--af-muted, hsl(220 9% 46%)); }
.errand-body { padding: 0.5rem 0.75rem; border-top: 1px solid var(--af-border, hsl(220 13% 91%)); }
.errand-task { font-size: 0.8rem; margin-bottom: 0.3rem; color: var(--af-fg, hsl(220 14% 10%)); }
.errand-content { font-size: 0.78rem; white-space: pre-wrap; margin: 0.3rem 0; max-height: 300px; overflow-y: auto; }
.errand-tool-calls { margin-top: 0.4rem; }
.errand-sub-tool { padding: 0.2rem 0; border-left: 2px solid var(--af-border, hsl(220 13% 91%)); padding-left: 0.6rem; margin: 0.2rem 0; }
.errand-sub-tool-header { display: flex; align-items: center; gap: 0.4rem; font-size: 0.75rem; }
.errand-sub-tool-name { font-family: monospace; color: var(--af-muted, hsl(220 9% 46%)); }
.errand-sub-tool-status { font-size: 0.68rem; padding: 0.05rem 0.3rem; border-radius: 3px; }
.errand-sub-tool-status.running { background: hsl(220 90% 56% / 0.15); color: hsl(220 90% 56%); }
.errand-sub-tool-status.success, .errand-sub-tool-status.completed { background: hsl(142 71% 45% / 0.15); color: hsl(142 71% 45%); }
.errand-sub-tool-status.error, .errand-sub-tool-status.failed { background: hsl(0 72% 51% / 0.15); color: hsl(0 72% 51%); }
.errand-sub-tool-result { font-size: 0.72rem; white-space: pre-wrap; margin: 0.2rem 0; max-height: 150px; overflow-y: auto; }
.errand-result { margin-top: 0.5rem; }
.errand-result-label { font-size: 0.72rem; color: var(--af-muted, hsl(220 9% 46%)); margin-bottom: 0.2rem; }
.errand-result-text { font-size: 0.78rem; white-space: pre-wrap; max-height: 400px; overflow-y: auto; }
</style>
