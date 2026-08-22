<template>
  <div class="meta-block">
    <button class="meta-header" type="button" @click="expanded = !expanded">
      <ChevronDown v-if="expanded" :size="14" class="meta-chevron" />
      <ChevronRight v-else :size="14" class="meta-chevron" />
      <span class="meta-summary">{{ summary || t('plans.metaShow') }}</span>
      <span class="meta-toggle-text">
        {{ expanded ? t('plans.metaHide') : t('plans.metaShow') }}
      </span>
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
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronDown, ChevronRight } from 'lucide-vue-next'
import type { FrontmatterValue } from '@/utils/frontmatter'

const props = defineProps<{ meta: Record<string, FrontmatterValue> }>()
const { t } = useI18n()

const expanded = ref(false)

/** 折叠态概要：feature_name · 创建~更新 · 步骤进度（字段缺失则跳过）。 */
const summary = computed(() => {
  const parts: string[] = []
  const feature = toText(props.meta['feature_name'])
  if (feature) parts.push(feature)
  const created = toText(props.meta['created_at'])
  const updated = toText(props.meta['updated_at'])
  if (created || updated) parts.push(`${created || '?'} ~ ${updated || '?'}`)
  const step = toText(props.meta['current_step'])
  const total = toText(props.meta['total_steps'])
  if (step || total) parts.push(`${step || '0'}/${total || '?'}`)
  return parts.join(' · ')
})

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
  font-size: 0.78rem;
  text-align: left;
}
.meta-header:hover {
  background: hsl(var(--muted-foreground) / 0.09);
}
.meta-chevron {
  flex-shrink: 0;
}
.meta-summary {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--af-fg);
}
.meta-toggle-text {
  flex-shrink: 0;
  font-size: 0.72rem;
  color: var(--af-muted);
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
