<template>
  <!-- PLAN-031 T5：PPT 风格 Run 工作汇总。PLAN-035 v2：structured 存在时
       body 渲染原生 blocks（目标+Goal chips / 流程+成果方框链 / 指标格 /
       交付物 badges+详情展开）；旧数据回退摘要渲染。默认展开。 -->
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
      <!-- ── blocks（`.ad` frontmatter 或 v2 数据驱动；有任一 block 数据即走卡片 UI）── -->
      <template v-if="hasBlocks">
        <div v-if="objectiveText" class="rb-section">
          <div class="rb-label">目标</div>
          <div class="rb-objective">{{ objectiveText }}</div>
          <div v-if="goalLinks.length" class="rb-chips">
            <button
              v-for="g in goalLinks"
              :key="g.id || g.label"
              class="rb-chip"
              title="在 Specs 中查看"
              @click.stop="goSpecs"
            >{{ g.label || g.id }}</button>
          </div>
        </div>

        <div v-if="stages.length" class="rb-section">
          <div class="rb-label">实现流程 · 各阶段成果</div>
          <div class="rb-flow">
            <template v-for="(s, i) in stages" :key="i">
              <div v-if="i > 0" class="rb-arrow">→</div>
              <div class="rb-stage">
                <div class="rb-stage-title">{{ s.title }}</div>
                <div v-if="s.outcome" class="rb-stage-outcome">{{ s.outcome }}</div>
              </div>
            </template>
          </div>
        </div>
      </template>

      <!-- 指标格（v1/v2 共用；数据机械采集） -->
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

      <!-- ── v2 blocks：交付物 badges（点击展开详情） ── -->
      <template v-if="structured">
        <div v-if="deliverables.length" class="rb-section">
          <div class="rb-label">交付物</div>
          <div class="rb-deliverables">
            <template v-for="(d, i) in deliverables" :key="i">
              <button class="rb-dl" :class="chgClass(d.change)" title="点击预览" @click.stop="openPreview(i)">
                <span class="rb-dl-icon">{{ kindIcon(d.kind) }}</span>
                <span class="rb-dl-name">{{ d.name }}</span>
                <span class="rb-dl-chg">{{ d.change }}</span>
              </button>
            </template>
          </div>
        </div>
      </template>

      <!-- 正文仅在无任何 block 数据时回退渲染（blocks 已承载主信息，避免重复） -->
      <div v-if="!hasBlocks" class="report-summary ad-body">
        <StreamingRenderer :source="(structured?.body || report.summary) || ''" :streaming="false" />
      </div>

      <!-- 交付物预览弹窗（路径形态尝试取文件内容，否则显示 detail） -->
      <Teleport to="body">
        <div v-if="preview" class="rb-preview-overlay" @click.self="closePreview">
          <div class="rb-preview">
            <div class="rb-preview-head">
              <span class="rb-preview-title">{{ preview.name }}</span>
              <button class="rb-preview-close" @click="closePreview">✕</button>
            </div>
            <pre v-if="preview.content" class="rb-preview-content">{{ preview.content }}</pre>
            <div v-else-if="preview.loading" class="rb-preview-text">加载中…</div>
            <div v-else class="rb-preview-text">{{ preview.detail || '无详情' }}</div>
          </div>
        </div>
      </Teleport>

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
import { ref, computed } from 'vue'
import { ChevronDown, ChevronUp, FileText, Download, FolderOpen } from 'lucide-vue-next'
import StreamingRenderer from '@/components/StreamingRenderer.vue'
import { useViewState } from '@/composables/useViewState'
import { useProject } from '@/composables/useProject'

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
  report?: { format: string; title: string; path: string; structured?: Record<string, any> }
  /** PLAN-035 v2: 结构化报告数据（objective/goal_links/stages/deliverables）。 */
  structured?: Record<string, any>
}

const props = defineProps<{ report: ReportData }>()

const { setView } = useViewState()
const { workspaceId } = useProject()

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

// ── v2/`.ad` 结构化 blocks ──
const structured = computed(() => props.report.structured ?? null)
const goalLinks = computed<any[]>(() => structured.value?.goal_links ?? [])
const stages = computed<any[]>(() => structured.value?.stages ?? [])
const deliverables = computed<any[]>(() => structured.value?.deliverables ?? [])
/** PLAN-036：`.ad` 模式的引导句（objective 或 summary）。 */
const objectiveText = computed(
  () => structured.value?.objective || structured.value?.summary || '',
)
/** 有任一 block 数据即走卡片 UI；正文仅在无 blocks 时回退渲染（避免重复）。 */
const hasBlocks = computed(
  () =>
    !!(stages.value.length || deliverables.value.length || goalLinks.value.length || objectiveText.value),
)

function kindIcon(kind: string): string {
  switch (kind) {
    case 'code': return '⌨'
    case 'spec': return '🧩'
    case 'doc': return '📝'
    case 'report': return '📊'
    default: return '📦'
  }
}

/** change（+/-/M）→ 合法 CSS 类后缀。 */
function chgClass(change: string): string {
  if (change === '+') return 'chg-add'
  if (change === '-') return 'chg-del'
  return 'chg-M'
}

function toggleDl(i: number) {
  openPreview(i)
}

function goSpecs() {
  setView('specs')
}

// ── 交付物预览弹窗（路径形态尝试取文件内容；否则显示 detail） ──
const preview = ref<{ name: string; content: string | null; detail: string; loading: boolean } | null>(null)

async function openPreview(i: number) {
  const d = deliverables.value[i]
  if (!d) return
  const name = String(d.name || '')
  const detail = String(d.detail || '')
  preview.value = { name, content: null, detail, loading: true }
  const looksLikePath = name.includes('/') || (name.includes('.') && !name.includes(' '))
  const wid = workspaceId.value
  if (looksLikePath && wid) {
    try {
      const resp = await fetch(
        `/api/files/${encodeURIComponent(wid)}/${name.split('/').map(encodeURIComponent).join('/')}`,
      )
      if (resp.ok) {
        const text = await resp.text()
        preview.value = { name, content: text.slice(0, 80_000), detail, loading: false }
        return
      }
    } catch {
      // 回退 detail
    }
  }
  preview.value = { name, content: null, detail, loading: false }
}

function closePreview() {
  preview.value = null
}
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
/* 摘要 */
.report-summary { font-size: 0.88rem; line-height: 1.6; color: var(--af-fg); }
/* PLAN-036：`.ad` 正文（StreamingRenderer 全宽文档流） */
.ad-body { width: 100%; }
/* 指标格 */
.report-metrics { display: grid; grid-template-columns: repeat(4, 1fr); gap: 0.5rem; }
.metric-cell {
  display: flex; flex-direction: column; gap: 0.1rem;
  padding: 0.45rem 0.5rem; background: hsl(220 14% 50% / 0.05); border-radius: 8px;
}
.metric-value { font-size: 1.02rem; font-weight: 600; color: hsl(142 70% 38%); }
.metric-label { font-size: 0.7rem; color: var(--af-muted); }

/* ── v2 blocks ── */
.rb-section { display: flex; flex-direction: column; gap: 0.3rem; }
.rb-label {
  font-size: 0.73rem; font-weight: 600; text-transform: uppercase;
  color: var(--af-muted); letter-spacing: 0.03em;
}
.rb-objective { font-size: 0.88rem; line-height: 1.6; color: var(--af-fg); }
.rb-chips { display: flex; flex-wrap: wrap; gap: 0.3rem; margin-top: 0.15rem; }
.rb-chip {
  font-size: 0.73rem; padding: 0.16rem 0.55rem; border-radius: 999px;
  background: hsl(160 70% 40% / 0.12); color: hsl(160 70% 28%);
  border: 1px solid hsl(160 70% 40% / 0.25); cursor: pointer;
}
.rb-chip:hover { background: hsl(160 70% 40% / 0.2); }

.rb-flow { display: flex; align-items: stretch; gap: 0.35rem; flex-wrap: wrap; }
.rb-arrow { color: var(--af-muted); align-self: center; font-size: 0.9rem; flex-shrink: 0; }
.rb-stage {
  flex: 1; min-width: 110px; padding: 0.45rem 0.6rem;
  border: 1px solid var(--af-border); border-radius: 10px;
  background: hsl(220 14% 50% / 0.04);
  display: flex; flex-direction: column; gap: 0.2rem;
}
.rb-stage-title { font-size: 0.8rem; font-weight: 600; color: var(--af-primary); }
.rb-stage-outcome { font-size: 0.72rem; color: var(--af-muted); line-height: 1.45; }

.rb-deliverables { display: flex; flex-direction: column; gap: 0.2rem; }
.rb-dl {
  display: inline-flex; align-items: center; gap: 0.45rem;
  align-self: flex-start; max-width: 100%;
  font-size: 0.78rem; padding: 0.28rem 0.6rem;
  border: 1px solid var(--af-border); border-radius: 7px;
  background: hsl(220 14% 50% / 0.04); cursor: pointer;
}
.rb-dl:hover { background: hsl(220 14% 50% / 0.09); }
.rb-dl-name {
  color: var(--af-fg); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  font-family: 'Geist Mono', 'Fira Code', monospace; font-size: 0.74rem;
}
.rb-dl-chg { font-weight: 700; flex-shrink: 0; }
.rb-dl.chg-add .rb-dl-chg { color: hsl(142 70% 38%); }
.rb-dl.chg-del .rb-dl-chg { color: hsl(0 70% 45%); }
.rb-dl.chg-M .rb-dl-chg { color: hsl(210 70% 45%); }
.rb-dl.chg-add { border-color: hsl(142 70% 45% / 0.3); }
.rb-dl-detail {
  font-size: 0.74rem; color: var(--af-muted); line-height: 1.5;
  padding: 0.25rem 0.6rem; border-left: 2px solid var(--af-border); margin-left: 0.4rem;
}

/* 交付物预览弹窗 */
.rb-preview-overlay {
  position: fixed; inset: 0; z-index: 100;
  background: hsl(0 0% 0% / 0.45);
  display: flex; align-items: center; justify-content: center;
}
.rb-preview {
  width: min(760px, 90vw); max-height: 80vh;
  display: flex; flex-direction: column;
  background: var(--af-bg); border: 1px solid var(--af-border); border-radius: 10px;
  box-shadow: 0 12px 40px hsl(0 0% 0% / 0.3); overflow: hidden;
}
.rb-preview-head {
  display: flex; align-items: center; gap: 0.6rem;
  padding: 0.6rem 0.9rem; border-bottom: 1px solid var(--af-border);
}
.rb-preview-title {
  flex: 1; font-size: 0.85rem; font-weight: 600; color: var(--af-fg);
  font-family: 'Geist Mono', 'Fira Code', monospace;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.rb-preview-close {
  border: 1px solid var(--af-border); border-radius: 5px; background: transparent;
  color: var(--af-muted); cursor: pointer; font-size: 0.8rem; padding: 0.15rem 0.45rem;
}
.rb-preview-content {
  margin: 0; padding: 0.8rem 0.9rem; overflow: auto;
  font-family: 'Geist Mono', 'Fira Code', monospace; font-size: 0.75rem; line-height: 1.5;
  color: var(--af-fg); white-space: pre-wrap; word-break: break-all;
}
.rb-preview-text { padding: 1rem 0.9rem; font-size: 0.85rem; color: var(--af-muted); line-height: 1.6; }

/* deck 层（已移除模板；样式保留无引用） */
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
