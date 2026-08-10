<!--
  WikiNav.vue — Wiki 二级导航（逃生舱组件，Plan 022 视觉对齐）
  对齐原版 WikiView.vue 的 wiki-nav：双树（Raw + Wiki）+ 搜索 + 图标。

  简化：原版用 TreeView 递归组件，这里用扁平列表（后端树节点无 children）。
  保留：可折叠 section + 图标 + 搜索过滤 + 空状态 + 新建按钮。
-->
<template>
  <div class="wiki-nav" :class="{ collapsed: collapsed }">
    <!-- header -->
    <div class="wiki-nav-header">
      <span class="wiki-nav-title">{{ t('wiki.title') }}</span>
      <div class="wiki-nav-actions">
        <button class="nav-icon-btn" @click="$emit('new-page')" :title="t('wiki.newPage')">
          <Plus :size="14" />
        </button>
        <button class="nav-icon-btn" @click="collapsed = !collapsed" :title="t('wiki.toggleSidebar')">
          <PanelLeft :size="14" />
        </button>
      </div>
    </div>

    <!-- 搜索框 -->
    <div class="wiki-search">
      <Search :size="13" />
      <input v-model="query" type="text" class="wiki-search-input" :placeholder="t('wiki.searchPlaceholder')" />
    </div>

    <!-- tree list -->
    <div class="wiki-nav-list" v-if="!collapsed">
      <!-- Raw section -->
      <div class="tree-section">
        <div class="tree-section-header" @click="rawExpanded = !rawExpanded">
          <component :is="rawExpanded ? ChevronDown : ChevronRight" :size="12" />
          <FolderInput :size="13" />
          <span class="tree-section-title">{{ t('wiki.raw') }}</span>
        </div>
        <div v-if="rawExpanded" class="tree-section-body">
          <button
            v-for="node in filteredRawTree"
            :key="node.path"
            class="tree-item"
            :class="{ active: activeRawPath === node.path }"
            @click="$emit('select-raw', node.path)"
          >
            <FileIcon :size="12" />
            <span class="tree-item-name">{{ node.name }}</span>
          </button>
          <div v-if="filteredRawTree.length === 0" class="tree-empty">
            <FileText :size="14" />
            <span>{{ t('wiki.noPages') }}</span>
          </div>
        </div>
      </div>

      <!-- Wiki section -->
      <div class="tree-section">
        <div class="tree-section-header" @click="wikiExpanded = !wikiExpanded">
          <component :is="wikiExpanded ? ChevronDown : ChevronRight" :size="12" />
          <BookOpen :size="13" />
          <span class="tree-section-title">{{ t('wiki.title') }}</span>
        </div>
        <div v-if="wikiExpanded" class="tree-section-body">
          <button
            v-for="node in filteredWikiTree"
            :key="node.path"
            class="tree-item"
            :class="{ active: activeWikiPath === node.path }"
            @click="$emit('select-wiki', node.path)"
          >
            <FileText :size="12" />
            <span class="tree-item-name">{{ node.name }}</span>
          </button>
          <div v-if="filteredWikiTree.length === 0" class="tree-empty">
            <FileText :size="14" />
            <span>{{ t('wiki.noPages') }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Plus, PanelLeft, Search, BookOpen, FolderInput,
  ChevronDown, ChevronRight, FileText, File as FileIcon,
} from 'lucide-vue-next'

const { t } = useI18n()

interface TreeNode {
  name: string
  path: string
  type: string
  size?: number
  modified?: number
}

const props = defineProps<{
  wikiTree: TreeNode[]
  rawTree: TreeNode[]
  activeWikiPath?: string
  activeRawPath?: string
}>()

defineEmits<{
  'new-page': []
  'select-wiki': [path: string]
  'select-raw': [path: string]
}>()

const collapsed = ref(false)
const query = ref('')
const rawExpanded = ref(true)
const wikiExpanded = ref(true)

const filteredWikiTree = computed(() => {
  if (!query.value) return props.wikiTree || []
  const q = query.value.toLowerCase()
  return (props.wikiTree || []).filter(n => n.name.toLowerCase().includes(q))
})

const filteredRawTree = computed(() => {
  if (!query.value) return props.rawTree || []
  const q = query.value.toLowerCase()
  return (props.rawTree || []).filter(n => n.name.toLowerCase().includes(q))
})
</script>

<style scoped>
.wiki-nav { display: flex; flex-direction: column; height: 100%; }
.wiki-nav.collapsed { width: 48px; }
.wiki-nav-header {
  display: flex; align-items: center; justify-content: space-between;
  height: 48px; padding: 0 0.75rem; flex-shrink: 0;
  border-bottom: 1px solid hsl(var(--border));
}
.wiki-nav-title {
  font-family: 'Noto Sans SC', sans-serif; font-size: 1rem; font-weight: 700;
  color: hsl(var(--foreground));
}
.wiki-nav-actions { display: flex; gap: 0.25rem; }
.nav-icon-btn {
  display: flex; align-items: center; justify-content: center;
  width: 26px; height: 26px; border: none; border-radius: 6px;
  background: transparent; color: hsl(var(--muted-foreground)); cursor: pointer;
}
.nav-icon-btn:hover { background: hsl(var(--accent)); color: hsl(var(--foreground)); }
/* 搜索 */
.wiki-search {
  display: flex; align-items: center; gap: 0.35rem;
  margin: 0.5rem 0.5rem; padding: 0.3rem 0.6rem;
  background: hsl(var(--muted-foreground) / 0.06);
  border: 1px solid hsl(var(--muted-foreground) / 0.12);
  border-radius: 6px; color: hsl(var(--muted-foreground));
}
.wiki-search:focus-within { border-color: hsl(var(--primary) / 0.35); }
.wiki-search-input {
  border: none; background: transparent; outline: none;
  font-size: 0.8rem; color: hsl(var(--foreground)); width: 100%;
}
.wiki-search-input::placeholder { color: hsl(var(--muted-foreground)); }
/* tree */
.wiki-nav-list { flex: 1; overflow-y: auto; padding: 0 0.25rem; }
.tree-section { margin-bottom: 0.25rem; }
.tree-section-header {
  display: flex; align-items: center; gap: 0.3rem;
  padding: 0.35rem 0.5rem; cursor: pointer; border-radius: 4px;
  color: hsl(var(--muted-foreground)); font-size: 0.75rem; font-weight: 600;
}
.tree-section-header:hover { background: hsl(var(--accent)); }
.tree-section-title { flex: 1; }
.tree-section-body { padding-left: 0.75rem; }
.tree-item {
  display: flex; align-items: center; gap: 0.35rem;
  width: 100%; padding: 0.25rem 0.5rem; border: none; border-radius: 4px;
  background: transparent; color: hsl(var(--foreground)); font-size: 0.8rem;
  cursor: pointer; text-align: left;
}
.tree-item:hover { background: hsl(var(--accent)); }
.tree-item.active { background: hsl(var(--primary) / 0.08); color: hsl(var(--primary)); }
.tree-item-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.tree-empty {
  display: flex; align-items: center; gap: 0.4rem;
  padding: 0.5rem; color: hsl(var(--muted-foreground)); font-size: 0.75rem;
}
</style>
