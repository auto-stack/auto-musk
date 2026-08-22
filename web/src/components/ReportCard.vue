<template>
  <!-- PLAN-031 T5：PPT 风格 Run 工作汇总（hero 渐变头 → 摘要 AutoDown →
       指标格 → 交付物 chips → 操作行）。默认展开——终态报告是给客户看的结果 -->
  <div class="report-card" :class="{ collapsed: !expanded }">
    <div class="report-hero" @click="expanded = !expanded">
      <span class="report-status">✅</span>
      <div class="report-title-block">
        <span class="report-title">{{ title }}</span>
        <span class="report-subtitle">{{ report.runId }}</span>
      </div>
      <span class="report-duration">{{ durationLabel }}</span>
      <span class="report-confidence" :class="confidence.toLowerCase()">{{ report.confidence }}</span>
      <ChevronDown v-if="!expanded" :size="14" class="report-chevron" />
      <ChevronUp v-else :size="14" class="report-chevron" />
    </div>

    <div v-if="expanded" class="report-body">
      <!-- 摘要：AutoDown/Markdown（PPT 内容页） -->
      <div v-if="report.summary" class="report-summary">
        <StreamingRenderer :source="report.summary" :streaming="false" />
      </div>

      <!-- 指标格：PPT 数据页 -->
      <div class="report-metrics">
        <div class="metric-cell">
          <span class="metric-value">{{ report.goalsMet }}</span>
          <span class="metric-label">步骤完成</span>
        </div>
        <div class="metric-cell">
          <span class="metric-value">{{ report.toolCalls }}</span>
          <span class="metric-label">工具调用</span>
        </div>
        <div class="metric-cell">
          <span class="metric-value">{{ report.cost }}</span>
          <span class="metric-label">令牌消耗</span>
        </div>
        <div class="metric-cell">
          <span class="metric-value">{{ durationLabel }}</span>
          <span class="metric-label">总用时</span>
        </div>
      </div>

      <!-- PLAN-032 deck 层：PPT 风格汇报预览（sandbox iframe，无脚本无同源） -->
      <div v-if="report.report && reportHtml" class="deck-wrap">
        <div class="deck-head">
          <span class="deck-title">📊 {{ report.report.title || 'Run 汇报报告' }}</span>
          <button class="report-btn" @click.stop="openDeck">
            <Maximize2 :size="13" />
            新窗口打开
          </button>
        </div>
        <iframe class="deck-frame" sandbox="" :srcdoc="reportHtml"></iframe>
      </div>

      <div v-if="report.deliverables?.length" class="report-deliverables">
        <div class="section-title">Deliverables</div>
        <div class="deliverable-chips">
          <span v-for="(d, i) in report.deliverables" :key="i" class="deliverable-chip">{{ d }}</span>
        </div>
      </div>

      <div class="report-actions">
        <button class="report-btn" @click.stop="$emit('view-full')">
          <FileText :size="13" />
          查看完整报告
        </button>
        <button class="report-btn" @click.stop="$emit('download')">
          <Download :size="13" />
          下载 Markdown
        </button>
        <button class="report-btn" @click.stop="$emit('open-files')">
          <FolderOpen :size="13" />
          打开变更文件
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ChevronDown, ChevronUp, FileText, Download, FolderOpen, Maximize2 } from 'lucide-vue-next'
import StreamingRenderer from '@/components/StreamingRenderer.vue'
import { useRelay } from '@/composables/useRelay'

export interface ReportData {
  runId: string
  title?: string
  summary?: string
  goalsMet: string
  testsPass: string
  driftDetected: string
  cost: string
  confidence: 'High' | 'Medium' | 'Low'
  deliverables: string[]
  filesChanged?: string[]
  toolCalls?: number
  durationS?: number
  /** PLAN-032: 汇报报告元数据（deck 层数据源；None=未生成）。 */
  report?: { format: string; title: string; path: string }
}

const props = defineProps<{ report: ReportData }>()

const { fetchRunReport } = useRelay()

// PLAN-032: 报告元数据存在时拉取 HTML 全文（useRelay 内缓存）
const reportHtml = ref<string | null>(null)
watch(
  () => props.report.report?.path,
  async (path) => {
    reportHtml.value = null
    if (path && props.report.runId) {
      reportHtml.value = await fetchRunReport(props.report.runId)
    }
  },
  { immediate: true },
)

function openDeck() {
  if (!reportHtml.value) return
  const blob = new Blob([reportHtml.value], { type: 'text/html' })
  const url = URL.createObjectURL(blob)
  window.open(url, '_blank')
  setTimeout(() => URL.revokeObjectURL(url), 60_000)
}

defineEmits<{
  (e: 'view-full'): void
  (e: 'download'): void
  (e: 'open-files'): void
}>()

const expanded = ref(true)

const title = computed(() => props.report.title || 'Run 工作汇总')

const durationLabel = computed(() => {
  const s = props.report.durationS ?? 0
  if (s < 60) return `${s}s`
  const m = Math.floor(s / 60)
  const r = s % 60
  return `${m}m ${String(r).padStart(2, '0')}s`
})

const confidence = computed(() => props.report.confidence || 'Medium')
</script>

<style scoped>
.report-card {
  border: 1px solid hsl(142 70% 45% / 0.3);
  border-radius: 12px;
  background: hsl(0 0% 100%);
  margin: 0.5rem 0;
  overflow: hidden;
  box-shadow: 0 2px 10px hsl(142 70% 30% / 0.06);
}
/* hero：PPT 封面式渐变头 */
.report-hero {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  padding: 0.75rem 0.9rem;
  cursor: pointer;
  user-select: none;
  background: linear-gradient(135deg, hsl(142 70% 45% / 0.14), hsl(174 60% 45% / 0.06) 55%, transparent);
  border-bottom: 1px solid hsl(142 70% 45% / 0.15);
}
.report-hero:hover {
  background: linear-gradient(135deg, hsl(142 70% 45% / 0.2), hsl(174 60% 45% / 0.08) 55%, transparent);
}
.report-status { font-size: 1.25rem; flex-shrink: 0; }
.report-title-block { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 0.1rem; }
.report-title {
  font-size: 0.95rem; font-weight: 600; color: var(--af-fg);
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.report-subtitle {
  font-size: 0.72rem; color: var(--af-muted);
  font-family: 'Geist Mono', 'Fira Code', monospace;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.report-duration {
  font-size: 0.7rem; padding: 0.14rem 0.45rem; border-radius: 999px; font-weight: 500;
  flex-shrink: 0; background: hsl(220 15% 50% / 0.1); color: var(--af-muted);
}
.report-confidence {
  font-size: 0.7rem; padding: 0.14rem 0.45rem; border-radius: 999px;
  font-weight: 500; flex-shrink: 0; text-transform: uppercase;
}
.report-confidence.high { background: hsl(142 70% 45% / 0.15); color: hsl(142 70% 35%); }
.report-confidence.medium { background: hsl(38 90% 50% / 0.15); color: hsl(38 80% 40%); }
.report-confidence.low { background: hsl(0 70% 50% / 0.15); color: hsl(0 70% 45%); }
.report-chevron { color: var(--af-muted); flex-shrink: 0; }

.report-body { padding: 0.65rem 0.9rem 0.8rem; display: flex; flex-direction: column; gap: 0.65rem; }
/* 摘要：AutoDown/Markdown（PPT 内容页） */
.report-summary { font-size: 0.88rem; line-height: 1.6; color: var(--af-fg); }
/* 指标格：PPT 数据页 */
.report-metrics { display: grid; grid-template-columns: repeat(4, 1fr); gap: 0.5rem; }
.metric-cell {
  display: flex; flex-direction: column; gap: 0.1rem;
  padding: 0.45rem 0.5rem; background: hsl(220 14% 50% / 0.05); border-radius: 8px;
}
.metric-value { font-size: 1.02rem; font-weight: 600; color: hsl(142 70% 38%); }
.metric-label { font-size: 0.7rem; color: var(--af-muted); }
/* 交付物 chips */
.section-title {
  font-size: 0.73rem; font-weight: 600; text-transform: uppercase;
  color: var(--af-muted); letter-spacing: 0.03em; margin-bottom: 0.25rem;
}
.deliverable-chips { display: flex; flex-wrap: wrap; gap: 0.3rem; }
.deliverable-chip {
  font-size: 0.73rem; padding: 0.18rem 0.5rem; border-radius: 5px;
  background: hsl(190 80% 45% / 0.08); color: hsl(190 80% 32%);
  border: 1px solid hsl(190 80% 45% / 0.2);
  font-family: 'Geist Mono', 'Fira Code', monospace;
}
/* PLAN-032 deck 层：16:9 沙箱预览 */
.deck-wrap { display: flex; flex-direction: column; gap: 0.35rem; }
.deck-head { display: flex; align-items: center; gap: 0.5rem; }
.deck-title { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 0.82rem; font-weight: 600; color: var(--af-fg); }
.deck-frame {
  width: 100%; aspect-ratio: 16 / 9; border: 1px solid var(--af-border);
  border-radius: 8px; background: #fff;
}
.report-actions {
  display: flex; flex-wrap: wrap; gap: 0.35rem;
  border-top: 1px solid hsl(var(--af-border) / 0.6); padding-top: 0.55rem;
}
.report-btn {
  display: inline-flex; align-items: center; gap: 0.3rem;
  padding: 0.32rem 0.6rem; border: 1px solid var(--af-border); border-radius: 5px;
  background: transparent; color: var(--af-fg); font-size: 0.8rem; font-weight: 500;
  cursor: pointer; transition: all 0.15s;
}
.report-btn:hover { background: hsl(220 14% 50% / 0.06); border-color: hsl(var(--primary) / 0.3); }
</style>
