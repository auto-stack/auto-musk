<template>
  <div class="workspace-selector">
    <button class="ws-btn" @click="open = !open" :title="current?.path">
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
        v-for="w in recent" :key="w.id"
        class="ws-item" :class="{ active: w.id === current?.id }"
        @click="choose(w)"
      >
        <Folder :size="13" />
        <span class="ws-item-name">{{ w.name }}</span>
        <span class="ws-item-path">{{ w.path }}</span>
      </button>
      <div class="ws-divider" />
      <div class="ws-section-label">打开其他文件夹</div>
      <input class="ws-input" v-model="customPath" placeholder="D:\path\to\project"
        @keydown.enter="openCustom" @input="onInput" />
      <div v-if="suggestions.length" class="ws-suggest">
        <button v-for="s in suggestions" :key="s.path" class="ws-suggest-item" @click="customPath = s.path">
          📁 {{ s.name }}
        </button>
      </div>
      <button class="ws-open-btn" @click="openCustom" :disabled="!customPath.trim()">
        <FolderOpen :size="13" /> 打开
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Folder, FolderOpen, ChevronUp, ChevronDown, X } from 'lucide-vue-next'
import { useProject } from '@/composables/useProject'

const { currentWorkspace: current, recentWorkspaces: recent, openWorkspace, loadRecent, browse } = useProject()
const open = ref(false)
const customPath = ref('')
const suggestions = ref<{ name: string; path: string }[]>([])

loadRecent()

let debounce: ReturnType<typeof setTimeout> | null = null
function onInput() {
  if (debounce) clearTimeout(debounce)
  debounce = setTimeout(async () => {
    const v = customPath.value
    if (v && (v.includes('/') || v.includes('\\'))) {
      suggestions.value = await browse(v)
    } else {
      suggestions.value = []
    }
  }, 300)
}

async function choose(w: { path: string }) {
  await openWorkspace(w.path)
  open.value = false
}
async function openCustom() {
  const p = customPath.value.trim()
  if (!p) return
  await openWorkspace(p)
  customPath.value = ''
  suggestions.value = []
  open.value = false
}
</script>

<style scoped>
.workspace-selector { position: relative; }
.ws-btn {
  display: flex; align-items: center; gap: 0.4rem;
  background: hsl(var(--muted-foreground) / 0.06); border: none; border-radius: 6px;
  padding: 0.35rem 0.6rem; color: var(--af-fg); cursor: pointer; font-size: 0.8rem;
  max-width: 140px;
}
.ws-btn:hover { background: hsl(var(--muted-foreground) / 0.12); }
.ws-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ws-panel {
  position: absolute; bottom: 100%; left: 0; margin-bottom: 6px;
  width: 280px; background: var(--af-card); border: 1px solid var(--af-border);
  border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.2); padding: 0.5rem;
  z-index: 100;
}
.ws-panel-header { display: flex; justify-content: space-between; align-items: center; padding: 0.25rem 0.5rem; font-size: 0.85rem; font-weight: 600; }
.ws-close { background: none; border: none; color: var(--af-muted); cursor: pointer; }
.ws-section-label { font-size: 0.7rem; text-transform: uppercase; color: var(--af-muted); padding: 0.4rem 0.5rem 0.2rem; }
.ws-item { display: flex; align-items: center; gap: 0.4rem; width: 100%; background: none; border: none; padding: 0.4rem 0.5rem; border-radius: 4px; cursor: pointer; color: var(--af-fg); text-align: left; }
.ws-item:hover { background: hsl(var(--muted-foreground) / 0.08); }
.ws-item.active { background: hsl(var(--primary) / 0.1); color: var(--af-primary); }
.ws-item-name { font-size: 0.82rem; }
.ws-item-path { font-size: 0.68rem; color: var(--af-muted); margin-left: auto; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 120px; }
.ws-divider { height: 1px; background: var(--af-border); margin: 0.4rem 0; }
.ws-input { width: 100%; box-sizing: border-box; background: var(--af-bg); border: 1px solid var(--af-border); border-radius: 4px; padding: 0.35rem 0.5rem; color: var(--af-fg); font-size: 0.8rem; }
.ws-suggest { max-height: 120px; overflow-y: auto; }
.ws-suggest-item { display: block; width: 100%; background: none; border: none; padding: 0.3rem 0.5rem; text-align: left; cursor: pointer; color: var(--af-fg); font-size: 0.78rem; border-radius: 4px; }
.ws-suggest-item:hover { background: hsl(var(--muted-foreground) / 0.08); }
.ws-open-btn { display: flex; align-items: center; gap: 0.4rem; width: 100%; justify-content: center; margin-top: 0.4rem; background: hsl(var(--primary)); color: #fff; border: none; border-radius: 4px; padding: 0.4rem; cursor: pointer; font-size: 0.82rem; }
.ws-open-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
