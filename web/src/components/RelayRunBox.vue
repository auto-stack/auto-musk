<template>
  <div class="relay-box" :class="`status-${statusClass}`">
    <!-- Collapsed header -->
    <div class="box-header" @click="toggle">
      <component :is="expanded ? ChevronDown : ChevronRight" :size="14" />
      <Orbit :size="14" />
      <span class="box-title">{{ boxTitle }}</span>
      <span class="box-progress" v-if="run">{{ run.current_step }}/{{ run.total_steps }}</span>
      <span class="box-status" :class="`badge-${statusClass}`">{{ statusLabel }}</span>
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
            <span class="entry-prof">{{ professionIcon(entry.profession_id) }}</span>
            <span class="entry-text">{{ entry.content }}</span>
          </template>
          <template v-else-if="entry.type === 'tool' || entry.type === 'tool_call'">
            <div class="entry-tool">
              <Wrench :size="12" />
              <span class="tool-name">{{ entry.tool_name }}</span>
            </div>
          </template>
          <template v-else-if="entry.type === 'step_started'">
            <div class="entry-step">▶ {{ entry.content }}</div>
          </template>
          <template v-else-if="entry.type === 'step_completed'">
            <div class="entry-step done">✓ {{ entry.content }}</div>
          </template>
          <template v-else-if="entry.type === 'gate_waiting'">
            <div class="entry-gate">⏸️ {{ entry.content }}</div>
          </template>
          <template v-else-if="entry.type === 'error'">
            <div class="entry-error">❌ {{ entry.content }}</div>
          </template>
          <template v-else-if="entry.type === 'run_completed'">
            <div class="entry-done">✅ {{ entry.content }}</div>
          </template>
        </div>
      </div>

      <!-- Inline gate actions -->
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
import { ChevronDown, ChevronRight, Orbit, Wrench } from 'lucide-vue-next'
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
  }
  return map[id] ?? '⚙️'
}

// Auto-scroll log to bottom on new entries
watch(logEntries, async () => {
  if (expanded.value) {
    await nextTick()
    logRef.value?.scrollTo({ top: logRef.value.scrollHeight, behavior: 'smooth' })
  }
}, { deep: true })

// Load run data on mount；内存 run 不在 → 回退到持久化 run 日志（Flow 会话）
// 试用修复：挂载即订阅 SSE——折叠态也要收状态事件（停靠审批/完成/失败），
// 否则除页面手刷外状态永不更新。
onMounted(async () => {
  await loadRun(props.runId)
  if (!run.value) history.value = await loadRunHistory(props.runId)
  loaded.value = true
  if (!unsubscribe) unsubscribe = subscribeToRun(props.runId)
})

// 停靠人工审批时自动展开（需要用户交互的块不能默认折叠）
watch(() => run.value?.waiting_for_gate, (g) => {
  if (g) expanded.value = true
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
  overflow: hidden;
  background: hsl(var(--muted-foreground) / 0.03);
}
.status-running { border-left: 3px solid hsl(var(--primary)); }
.status-completed { border-left: 3px solid hsl(142 71% 45%); }
.status-failed { border-left: 3px solid hsl(var(--af-error)); }
.status-gate { border-left: 3px solid hsl(38 92% 50%); }
.status-missing { border-left: 3px solid hsl(var(--af-border)); opacity: 0.75; }
.missing-note { padding: 0.5rem 0.75rem; font-size: 0.78rem; color: var(--af-fg-secondary, #888); }
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
.entry-text { color: var(--af-fg); }
.entry-tool { display: flex; align-items: center; gap: 0.3rem; color: var(--af-muted); padding-left: 1rem; }
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
