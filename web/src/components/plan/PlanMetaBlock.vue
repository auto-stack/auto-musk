<template>
  <div class="meta-block">
    <button
      class="meta-header"
      type="button"
      :title="expanded ? t('plans.metaHide') : t('plans.metaShow')"
      @click="expanded = !expanded"
    >
      <TableProperties :size="14" class="meta-icon" />
      <span class="meta-badge">{{ t('plans.metaTitle') }}</span>
      <ChevronUp v-if="expanded" :size="14" class="meta-chevron" />
      <ChevronDown v-else :size="14" class="meta-chevron" />
    </button>
    <table v-if="expanded" class="meta-table">
      <tbody>
        <tr v-for="(value, key) in meta" :key="key">
          <td class="meta-key">{{ key }}</td>
          <td class="meta-value">{{ formatValue(value) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronDown, ChevronUp, TableProperties } from 'lucide-vue-next'
import type { FrontmatterValue } from '@/utils/frontmatter'

defineProps<{ meta: Record<string, FrontmatterValue> }>()
const { t } = useI18n()

const expanded = ref(false)

function toText(v: FrontmatterValue | undefined): string {
  if (v == null) return ''
  return Array.isArray(v) ? v.join('、') : v
}

function formatValue(v: FrontmatterValue): string {
  return toText(v)
}
</script>

<style scoped>
.meta-block {
  border: 1px solid var(--af-border);
  border-radius: 6px;
  margin-bottom: 1rem;
  overflow: hidden;
}
.meta-header {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  width: 100%;
  padding: 0.45rem 0.7rem;
  background: hsl(var(--muted-foreground) / 0.05);
  border: none;
  cursor: pointer;
  color: var(--af-muted);
  text-align: left;
}
.meta-header:hover {
  background: hsl(var(--muted-foreground) / 0.09);
}
.meta-icon {
  flex-shrink: 0;
}
.meta-badge {
  font-size: 0.72rem;
  font-weight: 500;
  padding: 0.1rem 0.45rem;
  border-radius: 4px;
  background: hsl(var(--muted-foreground) / 0.14);
  color: var(--af-muted);
  white-space: nowrap;
}
.meta-chevron {
  flex-shrink: 0;
  margin-left: auto;
}
.meta-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.78rem;
}
.meta-table td {
  padding: 0.3rem 0.7rem;
  border-top: 1px solid var(--af-border);
  vertical-align: top;
}
.meta-key {
  width: 220px;
  font-family: ui-monospace, SFMono-Regular, monospace;
  font-size: 0.74rem;
  color: var(--af-muted);
  white-space: nowrap;
}
.meta-value {
  color: var(--af-fg);
  word-break: break-all;
}
</style>
