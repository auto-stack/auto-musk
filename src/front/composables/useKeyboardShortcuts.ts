// useKeyboardShortcuts.ts — 全局键盘快捷键 composable（Plan 022 技术债）
//
// 注册 Ctrl+Shift+S（聚焦搜索框）+ Ctrl+Shift+N（新建会话）。
// 对齐原版 ChatsView.vue 的 handleGlobalKeydown。
// 内部直接操作 DOM（无参 composable，适配 codegen const x = useX() 模式）。
// codegen: const keyboardShortcuts = useKeyboardShortcuts()

import { onMounted, onUnmounted } from 'vue'

export function useKeyboardShortcuts() {
  function handleKeydown(e: KeyboardEvent) {
    if (!(e.ctrlKey || e.metaKey) || !e.shiftKey) return

    const key = e.key.toLowerCase()
    if (key === 's') {
      // Ctrl+Shift+S: 聚焦搜索框
      e.preventDefault()
      const input = document.querySelector('.search-input') as HTMLInputElement | null
      input?.focus()
    } else if (key === 'n') {
      // Ctrl+Shift+N: 点击新建会话按钮
      e.preventDefault()
      const btn = document.querySelector('.sidebar-new-btn') as HTMLButtonElement | null
      btn?.click()
    }
  }

  onMounted(() => document.addEventListener('keydown', handleKeydown))
  onUnmounted(() => document.removeEventListener('keydown', handleKeydown))
}
