# 012 — UI 统一 Phase 1: Relay Box in Chat 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 relay run 从独立标签页（RelayView）移进对话流（ChatsView），渲染为可展开的 box，gate inline 审批。后端 API 不变，纯前端改动。

**Architecture:** ChatsView 已有 `spawn_relay` tool card（模板 237 行）和 useForge 的 `relay_spawned`/`relay_update` SSE 事件处理。Phase 1 做两件事：(1) 把薄卡片升级为可展开 box，内嵌 relay 的 session log（步骤 + agent 对话 + gate）；(2) 移除独立 RelayView 标签页。box 内容通过 `useRelay().subscribeToRun(runId)` 订阅 relay SSE 驱动，复用 RelayView 的 session log 渲染逻辑。

**Tech Stack:** Vue 3.5 + composables（无 vue-router），TypeScript，lucide-vue-next。

**Spec:** `designs/007-unified-workflow-architecture.md`（Phase 1 部分）

---

## File Structure

### Frontend — new
- **Create `web/src/components/RelayRunBox.vue`** — 可展开的 relay run 嵌入式卡片。默认折叠（显示摘要 + 状态徽章），展开后显示 session log（步骤序列 + agent 对话流 + 工具调用 + gate inline 审批）。内部调 `useRelay().subscribeToRun` 订阅实时事件，用 `useRelay().sessionLog` 渲染。

### Frontend — modify
- **`web/src/views/ChatsView.vue`** — 把 `spawn_relay` tool card 区域（模板 ~237 行）从薄卡片替换为 `<RelayRunBox :run-id="..." />`。移除 `goToRelayRun`（window.open 跳转）和 `openRelayView`。
- **`web/src/App.vue`** — tabs 从 4 个减为 3 个（移除 `agents`/RelayView tab）。Ctrl 快捷键调整。
- **`web/src/composables/useRelay.ts`** — `sessionLog` 改为按 runId 隔离的 map（当前只跟踪单个 run），使 ChatsView 里同时展开多个 relay box 不会互相干扰。`subscribeToRun` 写入对应 runId 的 log。

### Frontend — 不删除（保留兼容）
- `web/src/views/RelayView.vue` — 保留文件但不再从 tab 入口可达（Phase 2 可能复用其渲染逻辑）。

---

## Task 1: useRelay sessionLog 按 runId 隔离

**Files:**
- Modify: `web/src/composables/useRelay.ts`

当前 `sessionLog` 是单个 `ref<SessionLogEntry[]>([])`，`selectRun` 会清空它。改为按 runId 隔离的 map，使多个 relay box 能同时各自持有自己的 log。

- [ ] **Step 1: 改 sessionLog 数据结构**

在 `useRelay.ts` 中，把单例 `_sessionLog` 从 `ref([])` 改为 `ref<Record<string, SessionLogEntry[]>>({})`（key = runId）。

增加一个 `sessionLogFor(runId)` 计算属性/方法，返回指定 run 的 log 数组：

```ts
const _sessionLogs = ref<Record<string, SessionLogEntry[]>>({})

// 兼容：保留 sessionLog 名但改为 computed，指向"当前选中 run"的 log
const sessionLog = computed(() => {
  const id = _currentRun.value?.run_id
  return id ? _sessionLogs.value[id] ?? [] : []
})

// 新增：按 runId 获取 log（给 RelayRunBox 用）
function sessionLogFor(runId: string): SessionLogEntry[] {
  return _sessionLogs.value[runId] ?? []
}
```

- [ ] **Step 2: 改 subscribeToRun 写入对应 runId 的 log**

`subscribeToRun` 里所有 `_sessionLog.value.push(...)` 改为往 `_sessionLogs.value[runId]` 数组 push（不存在则先初始化为 `[]`）。`selectRun` 里的 `sessionLog.value = []` 改为切换 `_currentRun`（不清空 log map）。

具体：在 `subscribeToRun` 函数开头初始化 `_sessionLogs.value[runId] = _sessionLogs.value[runId] ?? []`，然后所有 push 改为 `_sessionLogs.value[runId].push(...)`。

- [ ] **Step 3: 改 loadRun 的事件重建**

`loadRun` 里从持久化 events 重建 session log 的逻辑（`eventsToSessionLog`），写入 `_sessionLogs.value[runId]` 而非 `_sessionLog`。

- [ ] **Step 4: 导出 sessionLogFor**

在 `useRelay()` 的 return 对象里加 `sessionLogFor`。

- [ ] **Step 5: 验证 RelayView 仍工作**

`npx vite build` 通过（RelayView 用 `sessionLog` computed，不受影响）。

- [ ] **Step 6: Commit**

```bash
cd D:/autostack/auto-musk
git add web/src/composables/useRelay.ts
git commit -m "feat(ui-unify): useRelay sessionLog isolated per-runId for multi-box support"
```

---

## Task 2: RelayRunBox 组件

**Files:**
- Create: `web/src/components/RelayRunBox.vue`

可展开的嵌入式 relay run 卡片。props: `runId`。功能：
- 默认折叠：显示 run 标题 + 状态徽章（running/waiting_gate/completed/failed）+ 进度（current_step/total_steps）
- 展开后：显示 session log（步骤 + agent 消息 + 工具调用 + gate inline 审批）
- gate 等待时：显示 inline 批准/拒绝按钮
- 订阅实时 SSE（`subscribeToRun`），组件卸载时取消订阅

- [ ] **Step 1: 创建 RelayRunBox.vue**

```vue
<template>
  <div class="relay-box" :class="`status-${statusClass}`">
    <!-- 折叠头部 -->
    <div class="box-header" @click="toggle">
      <component :is="expanded ? ChevronDown : ChevronRight" :size="14" />
      <Orbit :size="14" />
      <span class="box-title">{{ run?.title || runId }}</span>
      <span class="box-progress" v-if="run">{{ run.current_step }}/{{ run.total_steps }}</span>
      <span class="box-status" :class="`badge-${statusClass}`">{{ statusLabel }}</span>
    </div>

    <!-- 展开内容 -->
    <div v-if="expanded" class="box-body">
      <!-- Session log entries -->
      <div class="log-entries" ref="logRef">
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

const { runs, currentRun, loadRun, subscribeToRun, resolveGate, sessionLogFor } = useRelay()

const expanded = ref(false)
const gateBusy = ref(false)
const logRef = ref<HTMLElement | null>(null)
let unsubscribe: (() => void) | null = null

// Find this run in the runs list
const run = computed(() => {
  // Check currentRun first (if it's us), then search runs list
  if (currentRun.value?.run_id === props.runId) return currentRun.value
  return runs.value.find(r => r.run_id === props.runId) as any ?? currentRun.value
})

const statusClass = computed(() => {
  const s = run.value?.status ?? 'idle'
  if (s === 'completed') return 'completed'
  if (s === 'failed') return 'failed'
  if (s === 'waiting_approval') return 'gate'
  return 'running'
})

const statusLabel = computed(() => {
  const map: Record<string, string> = {
    running: '运行中', completed: '已完成', failed: '失败',
    waiting_approval: '待审批', idle: '就绪', paused: '已暂停',
  }
  return map[run.value?.status ?? 'idle'] ?? run.value?.status ?? '...'
})

const logEntries = computed(() => sessionLogFor(props.runId))

function toggle() {
  expanded.value = !expanded.value
  if (expanded.value && !unsubscribe) {
    subscribe()
  }
}

function subscribe() {
  unsubscribe = subscribeToRun(props.runId)
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

// Load run data on mount
onMounted(() => {
  loadRun(props.runId)
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
```

- [ ] **Step 2: Build 验证**

`cd D:/autostack/auto-musk/web && npx vite build 2>&1 | tail -3` — build OK。

- [ ] **Step 3: Commit**

```bash
cd D:/autostack/auto-musk
git add web/src/components/RelayRunBox.vue
git commit -m "feat(ui-unify): RelayRunBox — expandable relay run embedded in chat"
```

---

## Task 3: ChatsView 接入 RelayRunBox

**Files:**
- Modify: `web/src/views/ChatsView.vue`

把 `spawn_relay` tool card 区域（模板 ~237 行）从薄卡片替换为 `<RelayRunBox>`。

- [ ] **Step 1: 找到 spawn_relay card 位置**

读 ChatsView.vue 模板，找到 `tc.name === 'spawn_relay'` 的渲染分支（约 237 行）。当前是 `relay-card`，显示 status/summary + "Monitor →" 按钮。

- [ ] **Step 2: 替换为 RelayRunBox**

在该分支里，从 tool call 的 args 里提取 `run_id`（或 result 里解析），渲染 `<RelayRunBox :run-id="runId" />`：

```vue
<!-- spawn_relay: 展开为 relay run box -->
<div v-else-if="tc.name === 'spawn_relay'" class="relay-inline">
  <RelayRunBox :run-id="extractRunId(tc)" />
</div>
```

script 里加：
```ts
import RelayRunBox from '@/components/RelayRunBox.vue'

function extractRunId(tc: ToolCallInfo): string {
  // run_id 在 args 或 result 里
  try {
    const args = typeof tc.args === 'string' ? JSON.parse(tc.args) : tc.args
    if (args?.run_id) return args.run_id
    const result = typeof tc.result === 'string' ? JSON.parse(tc.result) : tc.result
    return result?.run_id || ''
  } catch {
    return ''
  }
}
```

- [ ] **Step 3: 移除 goToRelayRun / openRelayView**

删除 `goToRelayRun`（window.open 跳转）和 header 里的 `openRelayView` 按钮（或保留 badge 但改为滚动到 box）。

- [ ] **Step 4: Build + 手动验证**

`npx vite build` 通过。

- [ ] **Step 5: Commit**

```bash
cd D:/autostack/auto-musk
git add web/src/views/ChatsView.vue
git commit -m "feat(ui-unify): ChatsView renders RelayRunBox for spawn_relay tool calls"
```

---

## Task 4: App.vue 移除 RelayView 标签

**Files:**
- Modify: `web/src/App.vue`

4 标签 → 3 标签（移除 `agents`/流水线 tab）。

- [ ] **Step 1: 从 tabs 数组移除 agents**

在 `tabs` computed 里移除 `{ id: 'agents', i18nKey: 'nav.relay', icon: Orbit }`。移除 `Orbit` import（如不再使用）。

- [ ] **Step 2: 移除 RelayView 渲染分支**

模板里移除 `<RelayView v-else-if="currentView === 'agents'" />` 和 `import RelayView`。

- [ ] **Step 3: 调整快捷键**

Ctrl+1→chats, Ctrl+2→specs, Ctrl+3→wiki（移除 Ctrl+3→agents 和 Ctrl+4→wiki）。更新 `onKeyDown` switch。

- [ ] **Step 4: ViewId 类型调整**

从 `'chats' | 'specs' | 'wiki' | 'agents'` 改为 `'chats' | 'specs' | 'wiki'`（如果 ViewId 在 useViewState.ts 里定义，也改那里）。

- [ ] **Step 5: Build 验证**

`npx vite build` 通过。

- [ ] **Step 6: Commit**

```bash
cd D:/autostack/auto-musk
git add web/src/App.vue web/src/composables/useViewState.ts
git commit -m "feat(ui-unify): remove RelayView tab — 4 tabs → 3 (relay merged into chat)"
```

---

## Task 5: Playwright 验证 + 收尾

- [ ] **Step 1: 构建前端**

```bash
cd D:/autostack/auto-musk/web && npx vite build 2>&1 | tail -3
```

- [ ] **Step 2: Playwright 验证**

启动服务（如未运行），写 Playwright 脚本：
- 登录 → 进入对话标签
- 确认只有 3 个标签（无"流水线"）
- 如果有 relay run（通过 curl 启动一个），确认 chat 里出现 RelayRunBox（可展开）

- [ ] **Step 3: 清理 + 最终 commit（如有）**

---

## Self-Review

- **Spec coverage**: Design 007 Phase 1 要求（relay box in chat + gate inline + 3 tabs）→ Task 2-4 覆盖。sessionLog 隔离（Task 1）是技术前置。
- **Placeholder scan**: 无 TBD。每个 Task 有完整代码。
- **Type consistency**: `sessionLogFor(runId)` 在 Task 1 定义、Task 2 使用。`RelayRunBox` props `runId` 在 Task 2 定义、Task 3 传入。`extractRunId` 在 Task 3 定义使用。
