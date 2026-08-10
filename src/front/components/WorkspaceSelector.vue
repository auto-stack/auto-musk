<!--
  WorkspaceSelector.vue — 工作区选择器（逃生舱，简化版）
  Plan 022 后续：原生版 WorkspaceSelector 的精简移植。

  走逃生舱的理由：AutoUI .at 无法表达下拉面板 + browse 自动建议 + 条件渲染交互。
  本简化版只做"当前工作区指示器 + 最近列表切换"，不实现 browse/自动建议/empty-onboarding。
  KNOWN-DEBT: 完整切换面板（browse 自动建议、empty-opened onboarding）后续补齐。

  参照 web/src/components/WorkspaceSelector.vue。
-->
<template>
  <div class="workspace-selector">
    <button class="ws-btn" @click="toggle" :title="current?.path || '选择工作目录'">
      <Folder :size="14" />
      <span class="ws-name">{{ current?.name ?? '选择工作目录' }}</span>
      <ChevronUp :size="12" v-if="open" />
      <ChevronDown :size="12" v-else />
    </button>
    <div v-if="open" class="ws-panel">
      <div class="ws-panel-header">
        <span>切换 Workspace</span>
        <button class="ws-close" @click="open = false"><X :size="12" /></button>
      </div>
      <div class="ws-section-label">最近打开</div>
      <button
        v-for="w in recent"
        :key="w.id"
        class="ws-item"
        :class="{ active: w.id === current?.id }"
        @click="choose(w)"
      >
        <Folder :size="13" />
        <span class="ws-item-name">{{ w.name }}</span>
        <span class="ws-item-path">{{ w.path }}</span>
      </button>
      <div v-if="recent.length === 0" class="ws-empty">暂无工作区</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Folder, ChevronUp, ChevronDown, X } from 'lucide-vue-next'
import { workspace_list, workspace_status } from '@/lib/api'

/** WorkspaceMeta（与 api.ts WorkspaceMeta 对齐） */
interface WorkspaceMeta {
  id: string
  path: string
  name: string
  is_empty?: boolean
}

const open = ref(false)
const current = ref<WorkspaceMeta | null>(null)
const recent = ref<WorkspaceMeta[]>([])

async function loadRecent() {
  try {
    const data = await workspace_list()
    recent.value = data.workspaces || []
    // 同步当前 workspace 元信息
    const wid = localStorage.getItem('musk_workspace') || ''
    if (wid) {
      current.value = recent.value.find((w) => w.id === wid) || null
    }
  } catch {
    // 静默失败
  }
}

async function loadStatus() {
  const wid = localStorage.getItem('musk_workspace') || ''
  if (!wid) return
  try {
    const data = await workspace_status(wid)
    current.value = data.workspace || (data as any)
  } catch {
    // 静默失败
  }
}

function toggle() {
  open.value = !open.value
  if (open.value && recent.value.length === 0) loadRecent()
}

async function choose(w: WorkspaceMeta) {
  localStorage.setItem('musk_workspace', w.id)
  current.value = w
  open.value = false
  // 切换后刷新页面以让各 store 重新 Init 加载新 workspace 数据
  location.reload()
}

onMounted(() => {
  loadStatus()
  loadRecent()
})
</script>

<style scoped>
.workspace-selector {
  position: relative;
  margin-top: auto;
}
.ws-btn {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  width: 100%;
  padding: 0.35rem 0.5rem;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  background: transparent;
  color: hsl(var(--foreground));
  font-size: 0.75rem;
  cursor: pointer;
  transition: background 0.15s;
}
.ws-btn:hover {
  background: hsl(var(--accent));
}
.ws-name {
  flex: 1;
  text-align: left;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.ws-panel {
  position: absolute;
  bottom: 100%;
  left: 0;
  right: 0;
  margin-bottom: 4px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  padding: 0.5rem;
  z-index: 50;
  max-height: 320px;
  overflow-y: auto;
}
.ws-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.75rem;
  font-weight: 600;
  color: hsl(var(--muted-foreground));
  margin-bottom: 0.4rem;
}
.ws-close {
  border: none;
  background: transparent;
  cursor: pointer;
  color: hsl(var(--muted-foreground));
  padding: 2px;
}
.ws-close:hover {
  color: hsl(var(--foreground));
}
.ws-section-label {
  font-size: 0.7rem;
  color: hsl(var(--muted-foreground));
  padding: 0.2rem 0.3rem;
}
.ws-item {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  width: 100%;
  padding: 0.35rem 0.4rem;
  border: none;
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  font-size: 0.75rem;
  text-align: left;
}
.ws-item:hover {
  background: hsl(var(--accent));
}
.ws-item.active {
  background: hsl(var(--primary) / 0.1);
  color: hsl(var(--primary));
}
.ws-item-name {
  flex-shrink: 0;
  font-weight: 500;
}
.ws-item-path {
  flex: 1;
  font-size: 0.68rem;
  color: hsl(var(--muted-foreground));
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.ws-empty {
  padding: 0.5rem;
  text-align: center;
  font-size: 0.72rem;
  color: hsl(var(--muted-foreground));
}
</style>
