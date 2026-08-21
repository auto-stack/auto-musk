<template>
  <div class="relay-box" :class="`status-${statusClass}`">
    <!-- Collapsed header -->
    <div class="box-header" @click="toggle">
      <Orbit :size="14" :class="{ spinning: isLiveRunning }" class="run-icon" />
      <span class="box-title">{{ boxTitle }}</span>
      <!-- 进度区：1-based 徽标 + 迷你分段条；hover 弹出步骤清单
           （4 步各是什么、当前在哪——不占独立行） -->
      <div v-if="run" class="progress-wrap">
        <span class="box-progress">{{ badgePos }}/{{ run.total_steps }}</span>
        <div class="progress-segs">
          <span v-for="sv in stepViews" :key="sv.label" :class="sv.seg_class"></span>
        </div>
        <div class="progress-pop">
          <div v-for="sv in stepViews" :key="'pop-' + sv.label" class="pop-row">
            <span :class="sv.mark_class">{{ sv.mark }}</span>
            <span class="pop-label">{{ sv.label }}</span>
          </div>
        </div>
      </div>
      <span class="box-status" :class="`badge-${statusClass}`">{{ statusLabel }}</span>
      <!-- 折叠 UI 与其它 Block 统一：右侧上下箭头（ChevronDown=收起/Up=展开） -->
      <component :is="expanded ? ChevronUp : ChevronDown" :size="14" class="head-chevron" />
    </div>

    <!-- 收起态最新动态预览（最多 3 行；流式文本时为其尾部 3 行）。
         行内排版对齐 Block 头（工具名/目标分色）；停靠审批时由审批条接管 -->
    <div v-if="!expanded && !waitingGate && !missing && previewRows.length" class="live-preview">
      <span :class="dotClass"></span>
      <div class="live-preview-lines">
        <div v-for="(row, i) in previewRows" :key="i" class="preview-line">
          <span class="preview-mark">{{ row.mark }}</span>
          <span v-if="row.name" class="preview-tool-name">{{ row.name }}</span>
          <span v-if="row.target" class="preview-tool-target">{{ row.target }}</span>
          <span v-if="row.text" :class="row.text_class">{{ row.text }}</span>
        </div>
      </div>
    </div>

    <!-- 收起态审批条：停靠人工 gate 且未展开时，标题栏下方直接内联审批
         （不自动展开长日志、不产生滚动跳变——展开后由底部完整审批区接管） -->
    <div v-if="waitingGate && !expanded && !missing" class="gate-strip" @click.stop>
      <span class="gate-strip-prompt">⏸ 等待审批：{{ waitingGate.step_id }}</span>
      <button class="gate-btn approve" @click="approve" :disabled="gateBusy">批准</button>
      <button class="gate-btn reject" @click="reject" :disabled="gateBusy">拒绝</button>
    </div>

    <!-- Expanded body -->
    <div v-if="expanded" class="box-body">
      <!-- Run 不存在（服务重启清空内存 run / 已删除）的失效态 -->
      <div v-if="missing" class="missing-note">
        ⚠️ Run 已失效（run 为内存态，服务重启后清空；活动日志见对应会话记录）。重新发送需求即可续跑——计划文件会幂等复用。
      </div>
      <!-- Session log entries -->
      <div v-else class="log-entries" ref="logRef">
        <div v-for="entry in logEntries" :key="entry.id" class="log-entry" :class="`entry-${entry.type}`">
          <template v-if="entry.type === 'text'">
            <!-- 与 chat 一致：AutoDown/Markdown 渲染，全宽与工具块左对齐（不再
                 加职业图标——单独占行且缩进正文）。PLAN_FILE 行/长文本 → 折叠文档块 -->
            <div v-if="textPlanFile(entry.content)" class="plan-file-row">
              📄 <span class="plan-file-chip">{{ textPlanFile(entry.content) }}</span>
            </div>
            <div v-if="isDoc(entry.content)" class="doc-block">
              <div class="doc-head" @click="entry._docOpen = !entry._docOpen">
                <span class="doc-icon">📄</span>
                <span class="doc-title">{{ docTitle(entry.content) }}</span>
                <component :is="entry._docOpen ? ChevronUp : ChevronDown" :size="12" class="tool-chevron" />
              </div>
              <div v-if="entry._docOpen" class="doc-body">
                <StreamingRenderer :source="textBody(entry.content)" :streaming="false" />
              </div>
            </div>
            <div v-else-if="textBody(entry.content)" class="entry-md">
              <StreamingRenderer :source="textBody(entry.content)" :streaming="false" />
            </div>
          </template>
          <template v-else-if="entry.type === 'thinking'">
            <!-- 思考条目：muted 斜体（此前无分支不渲染） -->
            <div class="entry-thinking">{{ entry.content }}</div>
          </template>
          <!-- 与聊天侧工具卡一致：默认收起、显示操作目标、点击展开。
               文档读取型（read_plan 等/结果长）：📄 图标 + 结果 Markdown 子窗（小一号） -->
          <template v-else-if="entry.type === 'tool' || entry.type === 'tool_call'">
            <div class="entry-tool-card">
              <div class="entry-tool-head" @click="entry._expanded = !entry._expanded">
                <span v-if="toolIsDocKind(entry)" class="tool-icon">📄</span>
                <Wrench v-else :size="12" />
                <span class="tool-name">{{ entry.tool_name }}</span>
                <span class="tool-target">{{ toolTarget(entry) }}</span>
                <!-- 终态下仍无结果的工具调用 → 标记中断（此前收起态一直
                     留"进行中"观感，误导） -->
                <span v-if="entry.type === 'tool_call' && isTerminal" class="tool-interrupted">已中断</span>
                <span v-else-if="entry.type === 'tool_call'" class="tool-pending">…</span>
                <component
                  :is="entry._expanded ? ChevronUp : ChevronDown"
                  :size="12"
                  class="tool-chevron"
                />
              </div>
              <div v-if="entry._expanded" class="entry-tool-body">
                <pre v-if="entry.arguments" class="tool-args">{{ prettyArgs(entry.arguments) }}</pre>
                <div v-if="entry.result && toolIsDocKind(entry)" class="tool-doc-body">
                  <StreamingRenderer :source="entry.result" :streaming="false" />
                </div>
                <pre v-else-if="entry.result" class="tool-result">{{ entry.result }}</pre>
              </div>
            </div>
          </template>
          <!-- 阶段 Log 行：[icon] [类型·着色] [阶段名] [动作] [短尾线] -->
          <template v-else-if="PHASE_TYPES.has(entry.type)">
            <div :class="phaseClass(entry.type)">
              <span class="ph-icon">{{ phaseIcon(entry.type) }}</span>
              <span class="ph-kind">{{ phaseKind(entry.type) }}</span>
              <span v-if="phaseText(entry.type, entry.content)" class="ph-name">{{ phaseText(entry.type, entry.content) }}</span>
              <span class="ph-action">{{ phaseAction(entry.type) }}</span>
              <span v-if="phaseText(entry.type, entry.content) && (entry.type === 'run_failed' || entry.type === 'error')" class="ph-err-text">{{ phaseText(entry.type, entry.content) }}</span>
              <span class="ph-tail"></span>
            </div>
          </template>
        </div>
      </div>

      <!-- Inline gate actions（展开态） -->
      <div v-if="run?.waiting_for_gate" class="gate-actions">
        <span class="gate-prompt">等待审批：{{ run.waiting_for_gate.step_id }}</span>
        <button class="gate-btn approve" @click="approve" :disabled="gateBusy">批准</button>
        <button class="gate-btn reject" @click="reject" :disabled="gateBusy">拒绝</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { ChevronDown, ChevronRight, ChevronUp, Orbit, Wrench } from 'lucide-vue-next'
import StreamingRenderer from '@/components/StreamingRenderer.vue'
import { useRelay } from '@/composables/useRelay'

const props = defineProps<{ runId: string }>()

const { runs, currentRun, loadRun, loadRunHistory, subscribeToRun, resolveGate, sessionLogFor } = useRelay()
import type { RunHistory } from '@/composables/useRelay'

const expanded = ref(false)
const gateBusy = ref(false)
const logRef = ref<HTMLElement | null>(null)
let unsubscribe: (() => void) | null = null

// Find this run: check currentRun first, then search runs list.
// （修复：原 `?? currentRun.value` 兜底会错拿无关 run；严格按 runId 匹配，
// 找不到就走历史回放/失效态——例如历史消息引用的 run 已随 serve 重启清空。）
const run = computed(() => {
  if (currentRun.value?.run_id === props.runId) return currentRun.value
  return runs.value.find(r => r.run_id === props.runId) as any ?? null
})

// 历史回放（PLAN-030 试用反馈）：run 内存态被清空后，从持久化 run 日志
// （Flow 会话，id=run_id）重建只读视图——run 块由此获得持久性。
const history = ref<RunHistory | null>(null)
const loaded = ref(false)
const missing = computed(() => loaded.value && run.value == null && history.value == null)
const historyMode = computed(() => loaded.value && run.value == null && history.value != null)

const statusClass = computed(() => {
  if (missing.value) return 'missing'
  if (historyMode.value) {
    if (history.value!.status === 'completed') return 'completed'
    if (history.value!.status === 'failed') return 'failed'
    return 'missing' // interrupted
  }
  const s = run.value?.status ?? 'idle'
  if (s === 'completed') return 'completed'
  if (s === 'failed') return 'failed'
  if (s === 'waiting_approval') return 'gate'
  return 'running'
})

const statusLabel = computed(() => {
  if (missing.value) return '已失效'
  if (historyMode.value) {
    const m: Record<string, string> = { completed: '已完成', failed: '失败', interrupted: '已中断' }
    return (m[history.value!.status] ?? '已中断') + '（历史）'
  }
  const map: Record<string, string> = {
    running: '运行中', completed: '已完成', failed: '失败',
    waiting_approval: '待审批', idle: '就绪', paused: '已暂停',
  }
  return map[run.value?.status ?? 'idle'] ?? run.value?.status ?? '...'
})

const boxTitle = computed(() => run.value?.title || history.value?.title || props.runId)

const logEntries = computed(() =>
  historyMode.value ? history.value!.entries : sessionLogFor(props.runId)
)

const waitingGate = computed(() => run.value?.waiting_for_gate ?? null)

// 运行中（实时 run 且状态 running）→ 图标旋转的活性指示
const isLiveRunning = computed(
  () => run.value != null && !historyMode.value && run.value.status === 'running'
)

// 终态：失败/完成/失效——不再显示任何"进行中"指示
const isTerminal = computed(
  () => statusClass.value === 'failed' || statusClass.value === 'completed' || statusClass.value === 'missing'
)

// 徽标 1-based 位置（后端 current_step 是 0-based"即将执行"索引，直接展示
// 会把第 2 步显示成 1/4；完成态钉在 total 防越界）
const badgePos = computed(() => {
  const total = run.value?.total_steps ?? 0
  if (total <= 0) return 0
  if (run.value!.status === 'completed') return total
  return Math.min((run.value?.current_step ?? 0) + 1, total)
})

// 步骤 id → 中文标签（进度悬浮列表用）
function stepLabel(id: string): string {
  const m: Record<string, string> = { plan: '方案', execute: '执行', review: '审查', document: '文档' }
  return m[id] ?? id
}

// run.steps → 进度视图行（悬浮列表 + 迷你分段条共用）。失败 run 的当前步
// 后端标 pending，这里按 run status 修正为 ✗。
const stepViews = computed(() => {
  const steps: any[] = run.value?.steps ?? []
  const cur = run.value?.current_step ?? 0
  const st = run.value?.status ?? ''
  return steps.map((s, idx) => {
    let cls = 'pending'
    let mark = '○'
    if (idx < cur) { cls = 'done'; mark = '✓' }
    if (idx === cur) {
      if (st === 'completed') { cls = 'done'; mark = '✓' }
      if (st === 'running') { cls = 'active'; mark = '▶' }
      if (st === 'waiting_approval') { cls = 'gate'; mark = '⏸' }
      if (st === 'failed') { cls = 'failed'; mark = '✗' }
    }
    return { label: stepLabel(s.id ?? ''), mark, seg_class: 'seg ' + cls, mark_class: 'pop-mark ' + cls }
  })
})

// 收起态预览（最多 3 行，旧→新）：最新为流式文本/思考时取其尾部 3 行；
// 否则取最近 3 条动态。行内排版对齐 Block 头（工具名/目标分色）。
const previewRows = computed<any[]>(() => {
  const entries = logEntries.value as any[]
  let latest: any = null
  for (let i = entries.length - 1; i >= 0; i--) {
    const e = entries[i]
    if (e.type === 'tool' || e.type === 'tool_call' || (e.content ?? '') !== '') { latest = e; break }
  }
  if (!latest) return []
  if (latest.type === 'text' || latest.type === 'thinking') {
    const lines = String(latest.content ?? '').split('\n').filter((l) => l !== '').slice(-3)
    return lines.map((l) => ({ mark: '', name: '', target: '', text: l.length > 100 ? '…' + l.slice(-100) : l, text_class: 'preview-text' }))
  }
  const items: any[] = []
  for (let i = entries.length - 1; i >= 0 && items.length < 3; i--) {
    const row = entryPreviewRow(entries[i])
    if (row) items.push(row)
  }
  return items.reverse()
})

// 预览圆点：仅运行中脉动；gate 琥珀/失败红/完成绿/失效灰——终态常亮
// （失败后圆点仍在闪烁会让用户误以为命令还在跑）
const dotClass = computed(() =>
  isLiveRunning.value ? 'live-preview-dot live' : `live-preview-dot dot-${statusClass.value}`
)

/** 单条日志 → 预览行对象（无可展示内容返回 null）。
 *  工具行 name+target 分色——对齐 Block 头 tool-name/tool-target 口径。 */
function entryPreviewRow(e: any): any | null {
  if (e.type === 'tool' || e.type === 'tool_call') {
    return { mark: '🔧', name: e.tool_name ?? '', target: toolTarget(e), text: '', text_class: '' }
  }
  const c = e.content ?? ''
  if (!c) return null
  if (e.type === 'step_started') return { mark: '▶', name: '', target: '', text: c, text_class: 'preview-text' }
  if (e.type === 'step_completed') return { mark: '✓', name: '', target: '', text: c, text_class: 'preview-ok' }
  if (e.type === 'gate_waiting') return { mark: '⏸', name: '', target: '', text: c, text_class: 'preview-warn' }
  if (e.type === 'run_completed') return { mark: '✅', name: '', target: '', text: c, text_class: 'preview-ok' }
  if (e.type === 'run_failed' || e.type === 'error') return { mark: '❌', name: '', target: '', text: c, text_class: 'preview-err' }
  if (e.type === 'complete' || e.type === 'budget_warning' || e.type === 'budget_exceeded') return null
  if (e.type === 'thinking') return { mark: '💭', name: '', target: '', text: c.length > 90 ? '…' + c.slice(-90) : c, text_class: 'preview-text' }
  return { mark: '', name: '', target: '', text: c.length > 100 ? '…' + c.slice(-100) : c, text_class: 'preview-text' }
}

/** 工具条目的操作目标（与聊天侧工具卡的展示口径一致）。 */
function toolTarget(entry: any): string {
  const a = entry.arguments || {}
  const clip = (s: string, n = 60) => (s.length > n ? s.slice(0, n - 1) + '…' : s)
  if (a.path) return clip(String(a.path))
  if (a.cmd) return clip(String(a.cmd))
  if (a.seq != null) return `plan ${String(a.seq).padStart(3, '0')}`
  if (a.section_id) return String(a.section_id)
  if (a.task) return clip(String(a.task))
  if (a.query) return clip(String(a.query))
  if (a.pattern) return clip(String(a.pattern))
  return ''
}

function prettyArgs(args: any): string {
  if (args == null) return ''
  if (typeof args === 'string') return args
  try { return JSON.stringify(args, null, 2) } catch { return String(args) }
}

function toggle() {
  expanded.value = !expanded.value
  if (expanded.value) {
    // 展开即重拉最新状态（审批按钮/进度不滞留旧快照）
    loadRun(props.runId)
    if (!unsubscribe) unsubscribe = subscribeToRun(props.runId)
  }
}

async function approve() {
  gateBusy.value = true
  await resolveGate(props.runId, 'approve')
  gateBusy.value = false
}
async function reject() {
  gateBusy.value = true
  await resolveGate(props.runId, 'reject', '需要修改')
  gateBusy.value = false
}

function professionIcon(id: string): string {
  const map: Record<string, string> = {
    assistant: '📥', advisor: '💡', architect: '🏗️', planner: '📝',
    coder: '💻', tester: '🧪', reviewer: '🔍', documenter: '📚',
    'plan-dev': '📝',
  }
  return map[id] ?? '⚙️'
}

// Auto-scroll log to bottom on NEW entries only. 试用修复：原 {deep: true}
// 在点击工具条目展开（_expanded 变更）时也触发滚底——展开内容被滚出视口，
// 表现为"点了没展开、跳到最后一段"。仅监听长度变化。
watch(() => logEntries.value.length, async () => {
  if (expanded.value) {
    await nextTick()
    logRef.value?.scrollTo({ top: logRef.value.scrollHeight, behavior: 'smooth' })
  }
})

/** 'Step "xxx" started/completed' → 中文阶段名（分割线标签）。 */
function stepIdOf(content: string): string {
  const a = content.indexOf('"')
  if (a === -1) return ''
  const rest = content.slice(a + 1)
  const b = rest.indexOf('"')
  return b === -1 ? rest : rest.slice(0, b)
}

const STEP_LABELS: Record<string, string> = { plan: '方案', execute: '执行', review: '审查', document: '文档' }

// ─── 阶段 Log 行（[icon][类型·着色][阶段名][动作][短尾线]）──────────────────
const PHASE_TYPES = new Set(['step_started', 'step_completed', 'gate_waiting', 'run_completed', 'run_failed', 'error'])

function phaseKind(ty: string): string {
  if (ty === 'step_started' || ty === 'step_completed') return '阶段'
  if (ty === 'gate_waiting') return '审批'
  if (ty === 'run_completed') return 'Run'
  return '事件'
}

function phaseAction(ty: string): string {
  const m: Record<string, string> = {
    step_started: '开始', step_completed: '完成', gate_waiting: '等待',
    run_completed: '完成', run_failed: '失败',
  }
  return m[ty] ?? '记录'
}

function phaseIcon(ty: string): string {
  const m: Record<string, string> = {
    step_started: '▶', step_completed: '✓', gate_waiting: '⏸',
    run_completed: '✅', run_failed: '✗',
  }
  return m[ty] ?? '•'
}

function phaseClass(ty: string): string {
  if (ty === 'step_started') return 'phase-line ph-step'
  if (ty === 'step_completed') return 'phase-line ph-step done'
  if (ty === 'gate_waiting') return 'phase-line ph-gate'
  if (ty === 'run_completed') return 'phase-line ph-done'
  if (ty === 'run_failed' || ty === 'error') return 'phase-line ph-fail'
  return 'phase-line ph-step'
}

function phaseText(ty: string, content: string): string {
  if (ty === 'step_started' || ty === 'step_completed') {
    const id = stepIdOf(content ?? '')
    return STEP_LABELS[id] ?? id
  }
  if (ty === 'run_failed' || ty === 'error') {
    const c = String(content ?? '')
    return c.length > 60 ? c.slice(0, 59) + '…' : c
  }
  return ''
}

/** 文档读取型工具（📄 图标 + 结果 Markdown 子窗，字号小一号）。 */
function toolIsDocKind(entry: any): boolean {
  const n = entry.tool_name ?? ''
  if (['read_plan', 'read_file', 'list_plans', 'get_plan'].includes(n)) return true
  return String(entry.result ?? '').length > 400
}

/** 文本条目 → 文档块视图：PLAN_FILE 行抽为独立文件 chip；正文 ≥600 字符
 *  或含 PLAN_FILE → 折叠文档块（头部标题=首个 # 标题，展开渲染 Markdown）。 */
function textPlanFile(content: string): string {
  for (const ln of String(content ?? '').split('\n')) {
    if (ln.trim().startsWith('PLAN_FILE:')) return ln.trim().slice('PLAN_FILE:'.length).trim()
  }
  return ''
}

function textBody(content: string): string {
  return String(content ?? '')
    .split('\n')
    .filter((ln) => !ln.trim().startsWith('PLAN_FILE:'))
    .join('\n')
}

function isDoc(content: string): boolean {
  return textPlanFile(content) !== '' || textBody(content).length > 600
}

function docTitle(content: string): string {
  for (const ln of String(content ?? '').split('\n')) {
    const t = ln.trim()
    if (t.startsWith('# ')) {
      const title = t.slice(2).trim()
      if (title) return title.length > 56 ? title.slice(0, 55) + '…' : title
    }
  }
  return '文档'
}

// Load run data on mount；内存 run 不在 → 回退到持久化 run 日志（Flow 会话）
// 试用修复：挂载即订阅 SSE——折叠态也要收状态事件（停靠审批/完成/失败）。
// gate 到达不自动展开：收起态由标题下方内联审批条承接（无滚动跳变）。
onMounted(async () => {
  await loadRun(props.runId)
  if (!run.value) history.value = await loadRunHistory(props.runId)
  loaded.value = true
  if (!unsubscribe) unsubscribe = subscribeToRun(props.runId)
})

onUnmounted(() => {
  if (unsubscribe) unsubscribe()
})
</script>

<style scoped>
.relay-box {
  border: 1px solid var(--af-border);
  border-radius: 8px;
  margin: 0.5rem 0;
  overflow: visible;
  background: hsl(var(--muted-foreground) / 0.03);
}
/* overflow 改 visible（进度悬浮列表要伸出盒外）；子元素圆角补位防溢角 */
.relay-box > :first-child { border-radius: 8px 8px 0 0; }
.relay-box > :last-child { border-radius: 0 0 8px 8px; }
.relay-box > :only-child { border-radius: 8px; }
/* 工具卡全宽（左右边缘齐平，与聊天侧工具卡同款）——行级 flex 会把它收缩
   成内容宽度，正是收起态"长短不一"的根因 */
.entry-tool { display: block; padding-left: 0; }
.tool-interrupted {
  flex-shrink: 0; font-size: 0.68rem; color: hsl(var(--af-error));
  border: 1px solid hsl(var(--af-error) / 0.35); border-radius: 3px;
  padding: 0 0.25rem;
}

/* 进度区：1-based 徽标 + 迷你分段条；hover 悬浮步骤清单（CSS-only） */
.progress-wrap { position: relative; display: flex; align-items: center; gap: 0.3rem; cursor: help; }
.progress-segs { display: flex; gap: 2px; }
.progress-segs .seg { width: 10px; height: 4px; border-radius: 2px; background: hsl(var(--af-border)); }
.progress-segs .seg.done { background: hsl(142 71% 45%); }
.progress-segs .seg.active { background: hsl(var(--primary)); animation: rb-pulse 1.4s ease-in-out infinite; }
.progress-segs .seg.gate { background: hsl(38 92% 50%); }
.progress-segs .seg.failed { background: hsl(var(--af-error)); }
.progress-pop {
  display: none; position: absolute; top: 100%; right: 0; margin-top: 4px;
  background: var(--af-bg, #fff); border: 1px solid var(--af-border);
  border-radius: 6px; padding: 0.4rem 0.6rem; min-width: 128px; z-index: 30;
  box-shadow: 0 4px 12px hsl(0 0% 0% / 0.12);
}
.progress-wrap:hover .progress-pop { display: block; }
.pop-row { display: flex; align-items: center; gap: 0.4rem; font-size: 0.75rem; padding: 0.1rem 0; }
.pop-mark { width: 1em; flex-shrink: 0; text-align: center; }
.pop-mark.done { color: hsl(142 71% 45%); }
.pop-mark.active { color: hsl(var(--primary)); }
.pop-mark.gate { color: hsl(38 92% 50%); }
.pop-mark.failed { color: hsl(var(--af-error)); }
.pop-mark.pending { color: var(--af-muted); }
.pop-label { color: var(--af-fg); white-space: nowrap; }
.status-running { border-left: 3px solid hsl(var(--primary)); }
.status-completed { border-left: 3px solid hsl(142 71% 45%); }
.status-failed { border-left: 3px solid hsl(var(--af-error)); }
.status-gate { border-left: 3px solid hsl(38 92% 50%); }
.status-missing { border-left: 3px solid hsl(var(--af-border)); opacity: 0.75; }
.missing-note { padding: 0.5rem 0.75rem; font-size: 0.78rem; color: var(--af-fg-secondary, #888); }

/* 收起态审批条：停靠 gate 且未展开时，标题栏正下方的内联审批 */
.gate-strip {
  display: flex; align-items: center; gap: 0.5rem;
  padding: 0.4rem 0.75rem; border-top: 1px dashed hsl(38 92% 50%);
  background: hsl(38 92% 50% / 0.06);
}
.gate-strip-prompt { flex: 1; font-size: 0.78rem; color: hsl(38 60% 35%); }

@keyframes rb-spin { to { transform: rotate(360deg); } }
.spinning { animation: rb-spin 1.2s linear infinite; }
/* Orbit 身份图标：主题色（此前黑色） */
.run-icon { color: hsl(var(--primary)); }
.head-chevron { color: var(--af-muted); flex-shrink: 0; }
@keyframes rb-pulse { 0%, 100% { opacity: 0.35; } 50% { opacity: 1; } }
/* 预览条 v2：圆点对齐末行（flex-end + 末行居中边距），行距 0.2rem */
.live-preview {
  display: flex; align-items: flex-end; gap: 0.5rem;
  padding: 0.32rem 0.75rem 0.38rem;
  border-top: 1px solid var(--af-border); background: hsl(var(--primary) / 0.03);
}
.live-preview-dot {
  width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0;
  margin-bottom: 0.36rem; background: var(--af-muted, #999);
}
/* 仅运行中脉动；终态常亮（失败红/完成绿/gate 琥珀/失效灰） */
.live-preview-dot.live { background: hsl(var(--primary)); animation: rb-pulse 1.4s ease-in-out infinite; }
.live-preview-dot.dot-gate { background: hsl(38 92% 50%); }
.live-preview-dot.dot-failed { background: hsl(var(--af-error)); }
.live-preview-dot.dot-completed { background: hsl(142 71% 45%); }
.live-preview-lines { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 0.2rem; }
/* 预览行：排版对齐 Block 头（工具名 500/前景色 + 目标 青色 monospace） */
.preview-line { display: flex; align-items: center; gap: 0.35rem; min-width: 0; font-size: 0.74rem; line-height: 1.45; }
.preview-mark { width: 1.1em; flex-shrink: 0; font-size: 0.72rem; }
.preview-tool-name { flex-shrink: 0; font-weight: 500; color: var(--af-fg); }
.preview-tool-target {
  flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  color: hsl(190 80% 40%); font-family: 'Geist Mono', 'Fira Code', monospace; font-size: 0.72rem;
}
.preview-text {
  flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  color: var(--af-fg-secondary, #777);
}
.preview-err { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: hsl(var(--af-error)); }
.preview-ok { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: hsl(142 71% 45%); }
.preview-warn { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: hsl(38 60% 35%); }

/* 工具条目卡片（与聊天侧工具卡同款交互：默认收起/显示目标/点击展开） */
.entry-tool-card { border: 1px solid var(--af-border); border-radius: 6px; overflow: hidden; }
.entry-tool-head {
  display: flex; align-items: center; gap: 0.4rem; padding: 0.25rem 0.5rem;
  cursor: pointer; font-size: 0.75rem; background: hsl(var(--af-bg-secondary, 0 0% 97%));
}
.entry-tool-head .tool-name { font-weight: 500; }
.entry-tool-head .tool-target {
  flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  color: hsl(190 80% 35%); font-family: monospace;
}
.entry-tool-head .tool-pending { color: #999; }
.tool-chevron { color: #999; flex-shrink: 0; }
.entry-tool-body { border-top: 1px solid var(--af-border); }
.entry-tool-body pre {
  margin: 0; padding: 0.4rem 0.5rem; font-size: 0.72rem; line-height: 1.4;
  white-space: pre-wrap; word-break: break-all; max-height: 260px; overflow-y: auto;
}
.entry-tool-body .tool-args { background: hsl(0 0% 96%); }
.entry-tool-body .tool-result { background: hsl(140 30% 96%); border-top: 1px dashed var(--af-border); }
.badge-missing { background: hsl(var(--af-border)); color: hsl(var(--af-fg)); }

.box-header {
  display: flex; align-items: center; gap: 0.4rem;
  padding: 0.5rem 0.75rem; cursor: pointer; font-size: 0.82rem;
  color: var(--af-fg);
}
.box-header:hover { background: hsl(var(--muted-foreground) / 0.06); }
.box-title { font-weight: 500; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.box-progress { font-size: 0.72rem; color: var(--af-muted); }
.box-status { font-size: 0.7rem; padding: 0.1rem 0.4rem; border-radius: 3px; }
.badge-running { background: hsl(var(--primary) / 0.15); color: hsl(var(--primary)); }
.badge-completed { background: hsl(142 71% 45% / 0.15); color: hsl(142 71% 45%); }
.badge-failed { background: hsl(var(--af-error) / 0.15); color: hsl(var(--af-error)); }
.badge-gate { background: hsl(38 92% 50% / 0.15); color: hsl(38 92% 50%); }

.box-body { padding: 0.5rem 0.75rem; border-top: 1px solid var(--af-border); }
.log-entries { max-height: 400px; overflow-y: auto; font-size: 0.78rem; line-height: 1.5; }
.log-entry { padding: 0.15rem 0; }
.entry-prof { margin-right: 0.3rem; }
/* 展开态正文：Markdown 全宽（与工具块左对齐，无图标前缀） */
.entry-md { width: 100%; min-width: 0; }
.entry-thinking {
  color: var(--af-muted); font-style: italic; font-size: 0.78rem;
  padding: 0.15rem 0; white-space: pre-wrap; word-break: break-word;
}
/* 阶段 Log 行：[icon] [类型·着色] [阶段名] [动作] [短尾线]（左对齐 log 风格） */
.phase-line { display: flex; align-items: center; gap: 0.4rem; margin: 0.35rem 0 0.2rem; font-size: 0.75rem; }
.phase-line .ph-icon { flex-shrink: 0; font-size: 0.78rem; }
.phase-line .ph-kind {
  flex-shrink: 0; font-weight: 600; font-size: 0.7rem; letter-spacing: 0.05em;
  padding: 0.02rem 0.35rem; border-radius: 3px;
}
.phase-line .ph-name { flex-shrink: 0; font-weight: 500; color: var(--af-fg); }
.phase-line .ph-action { flex-shrink: 0; font-weight: 500; }
.phase-line .ph-err-text { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.phase-line .ph-tail { flex: 1; max-width: 2.2rem; height: 1px; border-bottom: 1px dashed currentColor; opacity: 0.4; margin-left: 0.3rem; }
/* 类型着色：阶段蓝 / 审批琥珀 / 完成绿 / 失败红 */
.phase-line.ph-step { color: var(--af-muted); }
.phase-line.ph-step .ph-icon, .phase-line.ph-step .ph-action { color: hsl(var(--primary)); }
.phase-line.ph-step .ph-kind { background: hsl(var(--primary) / 0.12); color: hsl(var(--primary)); }
.phase-line.ph-step.done { color: var(--af-muted); }
.phase-line.ph-step.done .ph-icon, .phase-line.ph-step.done .ph-action { color: hsl(142 71% 45%); }
.phase-line.ph-step.done .ph-kind { background: hsl(142 71% 45% / 0.12); color: hsl(142 71% 45%); }
.phase-line.ph-gate { color: var(--af-muted); }
.phase-line.ph-gate .ph-icon, .phase-line.ph-gate .ph-action { color: hsl(38 92% 50%); }
.phase-line.ph-gate .ph-kind { background: hsl(38 92% 50% / 0.15); color: hsl(38 80% 40%); }
.phase-line.ph-done { color: var(--af-muted); }
.phase-line.ph-done .ph-icon, .phase-line.ph-done .ph-action { color: hsl(142 71% 45%); }
.phase-line.ph-done .ph-kind { background: hsl(142 71% 45% / 0.12); color: hsl(142 71% 45%); }
.phase-line.ph-fail { color: var(--af-muted); }
.phase-line.ph-fail .ph-icon, .phase-line.ph-fail .ph-action { color: hsl(var(--af-error)); }
.phase-line.ph-fail .ph-kind { background: hsl(var(--af-error) / 0.12); color: hsl(var(--af-error)); }
.phase-line.ph-fail .ph-err-text { color: hsl(var(--af-error)); }
/* 工具文档型结果子窗：Markdown 渲染，比 Run 普通文档（0.85rem）小一号 */
.tool-doc-body {
  padding: 0.4rem 0.55rem; font-size: 0.78rem; line-height: 1.5;
  background: hsl(140 30% 97%); border-top: 1px dashed var(--af-border);
  max-height: 360px; overflow-y: auto;
}
.tool-icon { font-size: 0.85rem; line-height: 1; flex-shrink: 0; }
/* PLAN_FILE 行 → 独立文件 chip */
.plan-file-row { display: flex; align-items: center; gap: 0.3rem; padding: 0.25rem 0; font-size: 0.78rem; }
.plan-file-chip {
  font-family: 'Geist Mono', 'Fira Code', monospace; font-size: 0.74rem;
  color: hsl(190 80% 32%); background: hsl(190 80% 45% / 0.08);
  border: 1px solid hsl(190 80% 45% / 0.25); border-radius: 5px; padding: 0.14rem 0.5rem;
}
/* 长文本/计划文档 → 折叠文档块（Markdown 文件 Block） */
.doc-block { border: 1px solid var(--af-border); border-radius: 8px; margin: 0.3rem 0; overflow: hidden; background: hsl(var(--muted-foreground) / 0.02); }
.doc-head { display: flex; align-items: center; gap: 0.4rem; padding: 0.4rem 0.6rem; cursor: pointer; font-size: 0.8rem; }
.doc-head:hover { background: hsl(var(--muted-foreground) / 0.05); }
.doc-icon { flex-shrink: 0; }
.doc-title { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 500; color: var(--af-fg); }
.doc-body { border-top: 1px solid var(--af-border); padding: 0.5rem 0.7rem; font-size: 0.85rem; max-height: 420px; overflow-y: auto; }
.tool-name { font-family: monospace; font-size: 0.74rem; }
.entry-step { color: var(--af-muted); font-size: 0.75rem; padding: 0.2rem 0; }
.entry-step.done { color: hsl(142 71% 45%); }
.entry-gate { color: hsl(38 92% 50%); padding: 0.3rem 0; font-weight: 500; }
.entry-error { color: hsl(var(--af-error)); }
.entry-done { color: hsl(142 71% 45%); font-weight: 500; padding: 0.3rem 0; }

.gate-actions {
  display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem 0;
  border-top: 1px solid var(--af-border); margin-top: 0.5rem;
}
.gate-prompt { font-size: 0.78rem; color: hsl(38 92% 50%); flex: 1; }
.gate-btn {
  padding: 0.25rem 0.8rem; border-radius: 4px; border: 1px solid var(--af-border);
  cursor: pointer; font-size: 0.78rem;
}
.gate-btn.approve { background: hsl(142 71% 45%); color: #fff; border-color: transparent; }
.gate-btn.reject { background: hsl(var(--af-error) / 0.1); color: hsl(var(--af-error)); }
.gate-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
