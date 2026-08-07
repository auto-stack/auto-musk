<!--
  GenericToolCard.vue — 通用工具调用卡片（逃生舱，从 web/src/views/ChatsView.vue:295-324 移植）
  Plan 022 Phase 7c: 兜底显示所有非 errand/relay/task_plan 的工具调用
  （write_file/run_command/bring_in/读文件/搜索等）。展开/折叠 + arguments JSON + result。
  write_file 的代码高亮、run_command 的 shell 展开等专用卡片留 5c 同批。
-->
<template>
  <div class="tool-card" :class="tc.status">
    <div class="tool-header" @click="expanded = !expanded">
      <span class="tool-icon">🔧</span>
      <span class="tool-name">{{ tc.name }}</span>
      <template v-for="(seg, i) in summary" :key="i">
        <span class="tool-seg" :class="'seg-' + seg.type">{{ seg.text }}</span>
      </template>
      <span class="tool-status" :class="tc.status">{{ tc.status || 'running' }}</span>
      <span class="tool-chevron">{{ expanded ? '▲' : '▼' }}</span>
    </div>
    <div v-if="expanded" class="tool-body">
      <div class="tool-section">
        <div class="tool-section-title">Arguments</div>
        <pre class="tool-code">{{ JSON.stringify(tc.arguments, null, 2) }}</pre>
      </div>
      <div v-if="tc.result" class="tool-section">
        <div class="tool-section-title">Result</div>
        <pre class="tool-result">{{ tc.result }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { getToolSummary, type ToolCallLike } from '../forge_helpers'

const props = defineProps<{
  tc: ToolCallLike
}>()

const expanded = ref(false)
const summary = computed(() => getToolSummary(props.tc))
</script>

<style scoped>
.tool-card {
  border: 1px solid var(--af-border, hsl(220 13% 91%));
  border-radius: 8px;
  margin: 0.5rem 0;
  overflow: hidden;
}
.tool-header {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.5rem 0.75rem;
  cursor: pointer;
  font-size: 0.82rem;
}
.tool-header:hover { background: hsl(220 14% 96%); }
.tool-icon { font-size: 0.9rem; }
.tool-name { font-weight: 500; font-family: monospace; }
.tool-seg { font-size: 0.75rem; color: var(--af-muted, hsl(220 9% 46%)); max-width: 300px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.seg-path { color: hsl(190 80% 40%); font-family: monospace; }
.seg-pattern { color: hsl(280 60% 45%); }
.seg-desc { color: var(--af-fg, hsl(220 14% 10%)); }
.tool-status { font-size: 0.7rem; padding: 0.1rem 0.4rem; border-radius: 3px; margin-left: auto; }
.tool-status.running { background: hsl(220 90% 56% / 0.15); color: hsl(220 90% 56%); }
.tool-status.success, .tool-status.completed { background: hsl(142 71% 45% / 0.15); color: hsl(142 71% 45%); }
.tool-status.error, .tool-status.failed { background: hsl(0 72% 51% / 0.15); color: hsl(0 72% 51%); }
.tool-chevron { font-size: 0.7rem; color: var(--af-muted, hsl(220 9% 46%)); }
.tool-body { padding: 0.5rem 0.75rem; border-top: 1px solid var(--af-border, hsl(220 13% 91%)); }
.tool-section { margin: 0.3rem 0; }
.tool-section-title { font-size: 0.72rem; color: var(--af-muted, hsl(220 9% 46%)); margin-bottom: 0.2rem; text-transform: uppercase; letter-spacing: 0.05em; }
.tool-code, .tool-result { font-size: 0.75rem; white-space: pre-wrap; background: hsl(220 14% 97%); padding: 0.4rem; border-radius: 4px; max-height: 300px; overflow-y: auto; }
</style>
