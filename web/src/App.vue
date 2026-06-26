<template>
  <!-- Auth guard: show LoginView when unauthenticated -->
  <LoginView v-if="!isAuthenticated" @auth-success="onAuthSuccess" />
  <div v-else class="musk-app">
    <nav class="view-rail">
      <div class="rail-brand">
        <span class="brand-text">{{ t('app.brandName') }}</span>
        <span class="version">{{ t('app.version') }}</span>
      </div>
      <div class="rail-divider"></div>
      <div class="rail-tabs" data-testid="nav-rail">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="rail-tab"
          :class="{ active: currentView === tab.id }"
          :data-testid="`nav-tab-${tab.id}`"
          @click="setView(tab.id)"
        >
          <component :is="tab.icon" :size="16" class="tab-icon" />
          <span class="tab-label">{{ tab.label }}</span>
          <span v-if="tab.id === 'chats' && gateBadgeCount > 0" class="tab-badge">
            {{ gateBadgeCount }}
          </span>
        </button>
      </div>
      <div class="rail-divider"></div>
      <div class="rail-footer">
        <SettingsMenu />
      </div>
    </nav>
    <main class="view-main">
      <ChatsView v-if="currentView === 'chats'" />
      <SpecsView v-else-if="currentView === 'specs'" />
      <WikiView v-else-if="currentView === 'wiki'" />
      <RelayView v-else-if="currentView === 'agents'" />
      <!-- default: chats -->
      <ChatsView v-else />
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { MessageSquare, Scroll, BookOpen, Orbit } from 'lucide-vue-next'
import { useGateInbox } from '@/composables/useGateInbox'
import { useAuth } from '@/composables/useAuth'
import { useViewState } from '@/composables/useViewState'
import SettingsMenu from '@/components/SettingsMenu.vue'
import LoginView from './views/LoginView.vue'
import ChatsView from './views/ChatsView.vue'
import SpecsView from './views/SpecsView.vue'
import WikiView from './views/WikiView.vue'
import RelayView from './views/RelayView.vue'

const { t } = useI18n()
const { badgeCount: gateBadgeCount } = useGateInbox()
const { isAuthenticated } = useAuth()
const { currentView, setView } = useViewState()

function onAuthSuccess() {
  // Auth state is reactive in useAuth; this acknowledges the LoginView event.
}

onMounted(() => {
  document.addEventListener('keydown', onKeyDown)
})
onUnmounted(() => {
  document.removeEventListener('keydown', onKeyDown)
})

function onKeyDown(e: KeyboardEvent) {
  if (!e.ctrlKey) return
  switch (e.key) {
    case '1': e.preventDefault(); setView('chats'); break
    case '2': e.preventDefault(); setView('specs'); break
    case '3': e.preventDefault(); setView('agents'); break
    case '4': e.preventDefault(); setView('wiki'); break
  }
}

type ViewId = 'chats' | 'specs' | 'wiki' | 'agents'

const tabs = computed(() => {
  const base: { id: ViewId; i18nKey: string; icon: unknown }[] = [
    { id: 'chats', i18nKey: 'nav.chat', icon: MessageSquare },
    { id: 'agents', i18nKey: 'nav.relay', icon: Orbit },
    { id: 'specs', i18nKey: 'nav.specs', icon: Scroll },
    { id: 'wiki', i18nKey: 'nav.wiki', icon: BookOpen },
  ]
  return base.map((tab) => ({ ...tab, label: t(tab.i18nKey) }))
})
</script>

<style>
* { box-sizing: border-box; margin: 0; padding: 0; }

html, body, #app {
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
  background: var(--af-bg);
  color: var(--af-fg);
}

.musk-app { display: flex; height: 100vh; overflow: hidden; }

.view-rail {
  width: 180px; display: flex; flex-direction: column;
  background: hsl(var(--secondary)); border-right: 1px solid var(--af-border);
  padding: 0 0 1rem 0; flex-shrink: 0;
}
.rail-brand {
  display: flex; align-items: center; gap: 0.5rem;
  color: var(--af-primary); padding: 0.75rem 1rem; height: 48px; flex-shrink: 0;
}
.brand-text { font-size: 1.05rem; font-weight: 600; }
.rail-divider { height: 1px; background: hsl(var(--muted-foreground) / 0.2); margin: -1px 0 0 0; flex-shrink: 0; }
.rail-tabs { display: flex; flex-direction: column; gap: 0.15rem; flex: 1; padding: 0 0.5rem; }
.rail-tab {
  display: flex; align-items: center; gap: 0.6rem; width: 100%;
  padding: 0.5rem 0.6rem; background: transparent; border: none; border-radius: 6px;
  color: var(--af-muted); cursor: pointer; transition: all 0.15s; font-size: 0.88rem;
}
.rail-tab:hover { background: hsl(var(--muted-foreground) / 0.06); color: var(--af-fg); }
.rail-tab.active { background: hsl(var(--primary) / 0.08); color: var(--af-primary); font-weight: 500; }
.rail-tab.active .tab-icon { color: var(--af-primary); stroke: var(--af-primary); }
.tab-label { font-size: 0.88rem; }
.rail-footer { margin-top: auto; display: flex; align-items: center; justify-content: flex-end; gap: 0.25rem; padding: 0 1rem; color: var(--af-muted); }
.version { font-size: 0.73rem; color: var(--af-muted); font-weight: 400; margin-left: 0.35rem; }
.view-main { flex: 1; min-width: 0; overflow: hidden; display: flex; flex-direction: column; }
.tab-badge {
  font-size: 0.68rem; font-weight: 600; min-width: 16px; height: 16px; padding: 0 4px;
  border-radius: 8px; background: hsl(var(--af-error)); color: #fff;
  display: inline-flex; align-items: center; justify-content: center; margin-left: auto;
}
</style>
