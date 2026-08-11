<template>
  <span class="status-badge" :class="`tone-${tone}`" :title="status">{{ label }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { PlanStatus } from '@/types/plans'
import { STATUS_TONE } from '@/types/plans'

const props = defineProps<{ status: PlanStatus }>()

const tone = computed(() => STATUS_TONE[props.status])
// 显示时把 snake_case 转成更紧凑的形式（execution_done → exec_done）
const label = computed(() =>
  props.status === 'execution_done' ? 'exec_done' : props.status,
)
</script>

<style scoped>
.status-badge {
  font-size: 0.68rem;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  font-weight: 500;
  letter-spacing: 0.02em;
  white-space: nowrap;
  text-transform: lowercase;
}
.tone-muted {
  background: hsl(var(--muted-foreground) / 0.14);
  color: var(--af-muted);
}
.tone-info {
  background: hsl(var(--primary) / 0.14);
  color: var(--af-primary);
}
.tone-warn {
  background: hsl(38 92% 50% / 0.16);
  color: hsl(38 92% 38%);
}
.tone-success {
  background: hsl(142 76% 36% / 0.16);
  color: hsl(142 76% 28%);
}
</style>
