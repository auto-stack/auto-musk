// useTheme.ts — 主题切换 composable（Plan 022 视觉对齐，移植自原版 web/）
//
// mode: 'auto' | 'dark' | 'light'
// 切换 <html> 的 class="dark"，持久化到 localStorage。
// codegen composable 机制：const theme = useTheme()（去 use 前缀小写）。

import { ref, readonly, onMounted } from 'vue'

const STORAGE_KEY = 'musk-theme'

type ThemeMode = 'auto' | 'dark' | 'light'

const _mode = ref<ThemeMode>('auto')

function systemPrefersDark(): boolean {
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

function applyMode(mode: ThemeMode) {
  const html = document.documentElement
  const isDark = mode === 'dark' || (mode === 'auto' && systemPrefersDark())
  if (isDark) {
    html.classList.add('dark')
  } else {
    html.classList.remove('dark')
  }
}

export function useTheme() {
  const mode = readonly(_mode)

  function setMode(next: ThemeMode) {
    _mode.value = next
    localStorage.setItem(STORAGE_KEY, next)
    applyMode(next)
  }

  function init() {
    const stored = localStorage.getItem(STORAGE_KEY) as ThemeMode | null
    const initial: ThemeMode = stored ?? 'auto'
    _mode.value = initial
    applyMode(initial)

    // React to system changes when in auto mode
    const mql = window.matchMedia('(prefers-color-scheme: dark)')
    mql.addEventListener?.('change', () => {
      if (_mode.value === 'auto') applyMode('auto')
    })
  }

  onMounted(init)

  return { mode, setMode, init }
}
