<template>
  <span class="status-badge" :class="`tone-${tone}`" :title="status">{{ label }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { PlanStatus } from '@/types/plans'
import { STATUS_TONE, planStatusKey } from '@/types/plans'

const props = defineProps<{ status: PlanStatus }>()
const { t } = useI18n()

const tone = computed(() => STATUS_TONE[props.status])
// PLAN-033: 标签走 i18n（中文模式显示中文）；title 保留原始枚举值便于排查
const label = computed(() => t(planStatusKey(props.status)))
</script>

<style scoped>
.status-badge {
  font-size: 0.68rem;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  font-weight: 500;
  letter-spacing: 0.02em;
  white-space: nowrap;
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
