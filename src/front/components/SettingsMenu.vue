<!--
  SettingsMenu.vue — 设置菜单（逃生舱组件，Plan 022 视觉对齐）
  精简移植自原版 web/src/components/SettingsMenu.vue。

  含四个分区：强调色 / 外观（light/dark/auto）/ 语言（EN/中文）/ AutoOS 设置链接。
  去掉 useForgeMode（GSD/Check，后端无对应端点）。
-->
<template>
  <div ref="menuRef" class="settings-menu-wrapper">
    <button
      class="settings-trigger"
      :class="{ open: isOpen }"
      :title="t('settings.title')"
      @click="isOpen = !isOpen"
    >
      <Settings :size="16" />
    </button>
    <transition name="fade">
      <div v-if="isOpen" class="settings-panel">
        <!-- Accent Section -->
        <div class="settings-section">
          <div class="settings-section-title">{{ t('settings.accent') }}</div>
          <div class="accent-swatches">
            <button
              v-for="opt in accentOptions"
              :key="opt.name"
              class="accent-swatch"
              :class="{ active: accentCurrent === opt.name }"
              :style="{ background: opt.brand1 }"
              :title="opt.label"
              @click="setAccent(opt.name)"
            >
              <Check v-if="accentCurrent === opt.name" :size="12" />
            </button>
          </div>
        </div>

        <!-- Theme Section -->
        <div class="settings-section">
          <div class="settings-section-title">{{ t('settings.theme') }}</div>
          <div class="theme-options">
            <button
              v-for="opt in themeOptions"
              :key="opt.value"
              class="theme-option"
              :class="{ active: themeMode === opt.value }"
              @click="setMode(opt.value)"
            >
              <component :is="opt.icon" :size="14" />
              <span>{{ opt.label }}</span>
              <Check v-if="themeMode === opt.value" :size="12" class="check" />
            </button>
          </div>
        </div>

        <!-- Language Section -->
        <div class="settings-section">
          <div class="settings-section-title">{{ t('settings.language') }}</div>
          <div class="language-options">
            <button
              class="language-option"
              :class="{ active: currentLocale === 'en' }"
              @click="changeLocale('en')"
            >
              <span class="lang-code">EN</span>
              <span class="lang-name">English</span>
              <Check v-if="currentLocale === 'en'" :size="12" class="check" />
            </button>
            <button
              class="language-option"
              :class="{ active: currentLocale === 'zh' }"
              @click="changeLocale('zh')"
            >
              <span class="lang-code">中</span>
              <span class="lang-name">中文</span>
              <Check v-if="currentLocale === 'zh'" :size="12" class="check" />
            </button>
          </div>
        </div>

        <!-- AutoOS Settings deep-link -->
        <div class="settings-section">
          <div class="settings-section-title">AutoOS</div>
          <button class="deep-link-btn" @click="openAutoOsConfig">
            <ExternalLink :size="14" />
            <span>{{ t('settings.openSystemSettings') }}</span>
          </button>
          <div v-if="autoOsError" class="deep-link-error">{{ autoOsError }}</div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Settings, Check, Sun, Moon, Monitor, ExternalLink } from 'lucide-vue-next'
import { useTheme } from '../composables/useTheme'
import { useAccentColor } from '../composables/useAccentColor'

const { t, locale } = useI18n()
const { mode: themeMode, setMode } = useTheme()
const { current: accentCurrent, setAccent, options: accentOptions } = useAccentColor()

const isOpen = ref(false)
const menuRef = ref<HTMLDivElement>()
const currentLocale = ref(locale.value)

function changeLocale(l: string) {
  locale.value = l
  currentLocale.value = l
  localStorage.setItem('musk-language', l)
}

const autoOsError = ref('')

async function openAutoOsConfig() {
  isOpen.value = false
  autoOsError.value = ''
  try {
    const resp = await fetch('/api/settings-link', { method: 'POST' })
    const data = await resp.json()
    if (data.status === 'running' && data.url) {
      window.open(data.url + '/#ai-musk', '_blank')
    } else {
      autoOsError.value = data.error || 'Service not available'
    }
  } catch (e: any) {
    autoOsError.value = e.message || 'Could not reach settings service'
  }
}

const themeOptions = computed(() => [
  { value: 'light' as const, label: t('settings.themeLight'), icon: Sun },
  { value: 'dark' as const, label: t('settings.themeDark'), icon: Moon },
  { value: 'auto' as const, label: t('settings.themeSystem'), icon: Monitor },
])

function onDocClick(e: MouseEvent) {
  if (isOpen.value && menuRef.value && !menuRef.value.contains(e.target as Node)) {
    isOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', onDocClick)
  // 恢复保存的语言
  const savedLang = localStorage.getItem('musk-language')
  if (savedLang && savedLang !== locale.value) {
    locale.value = savedLang
    currentLocale.value = savedLang
  }
})
onUnmounted(() => document.removeEventListener('click', onDocClick))
</script>

<style scoped>
.settings-menu-wrapper {
  position: relative;
}
.settings-trigger {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: hsl(var(--muted-foreground));
  cursor: pointer;
  transition: all 0.15s;
}
.settings-trigger:hover {
  background: hsl(var(--accent));
  color: hsl(var(--foreground));
}
.settings-trigger.open {
  background: hsl(var(--accent));
  color: hsl(var(--primary));
}
.settings-panel {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 8px;
  min-width: 220px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  padding: 0.6rem;
  z-index: 100;
}
.settings-section {
  padding: 0.4rem 0;
}
.settings-section + .settings-section {
  border-top: 1px solid hsl(var(--border));
}
.settings-section-title {
  font-size: 0.7rem;
  font-weight: 600;
  color: hsl(var(--muted-foreground));
  text-transform: uppercase;
  letter-spacing: 0.04em;
  margin-bottom: 0.4rem;
  padding: 0 0.3rem;
}
/* Accent swatches */
.accent-swatches {
  display: flex;
  gap: 0.5rem;
  padding: 0 0.3rem;
}
.accent-swatch {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  transition: transform 0.1s;
}
.accent-swatch:hover { transform: scale(1.1); }
.accent-swatch.active { border-color: hsl(var(--foreground)); }
/* Theme options */
.theme-options {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.theme-option {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.35rem 0.5rem;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: hsl(var(--foreground));
  font-size: 0.82rem;
  cursor: pointer;
  text-align: left;
  width: 100%;
}
.theme-option:hover { background: hsl(var(--accent)); }
.theme-option.active { background: hsl(var(--primary) / 0.08); color: hsl(var(--primary)); }
.theme-option .check { margin-left: auto; }
/* Language options */
.language-options { display: flex; flex-direction: column; gap: 2px; }
.language-option {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.35rem 0.5rem;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: hsl(var(--foreground));
  font-size: 0.82rem;
  cursor: pointer;
  text-align: left;
  width: 100%;
}
.language-option:hover { background: hsl(var(--accent)); }
.language-option.active { background: hsl(var(--primary) / 0.08); color: hsl(var(--primary)); }
.lang-code { font-weight: 700; min-width: 1.5rem; }
.lang-name { flex: 1; }
.language-option .check { margin-left: auto; }
/* AutoOS deep-link */
.deep-link-btn {
  display: flex; align-items: center; gap: 0.5rem;
  width: 100%; padding: 0.35rem 0.5rem;
  border: 1px solid hsl(var(--border)); border-radius: 6px;
  background: transparent; color: hsl(var(--foreground));
  font-size: 0.78rem; cursor: pointer; text-align: left;
  transition: background 0.15s;
}
.deep-link-btn:hover { background: hsl(var(--accent)); }
.deep-link-error {
  margin-top: 0.3rem; padding: 0.2rem 0.5rem;
  font-size: 0.7rem; color: hsl(var(--destructive));
  background: hsl(var(--destructive) / 0.08); border-radius: 4px;
}
/* Transition */
.fade-enter-active, .fade-leave-active { transition: opacity 0.15s, transform 0.15s; }
.fade-enter-from, .fade-leave-to { opacity: 0; transform: translateY(4px); }
</style>
