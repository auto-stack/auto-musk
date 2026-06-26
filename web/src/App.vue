<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuth } from './composables/useAuth'
import LoginView from './views/LoginView.vue'
import ChatsView from './views/ChatsView.vue'
import SpecsView from './views/SpecsView.vue'

const { token, username, fetchMe, logout } = useAuth()
const checking = ref(true)
const currentView = ref<'chats' | 'specs'>('chats')

onMounted(async () => {
  if (token.value) await fetchMe()
  checking.value = false
})
</script>

<template>
  <div v-if="checking" class="boot">Loading…</div>
  <LoginView v-else-if="!token" />
  <div v-else class="app-shell">
    <header class="topbar">
      <span class="brand">🦌 Auto Musk</span>
      <span class="spacer"></span>
      <span
        :class="['nav-item', { active: currentView === 'chats' }]"
        @click="currentView = 'chats'"
      >Chats</span>
      <span
        :class="['nav-item', { active: currentView === 'specs' }]"
        @click="currentView = 'specs'"
      >Specs</span>
      <span class="nav-item muted" title="coming soon">Flows</span>
      <span class="nav-item muted" title="coming soon">Wikis</span>
      <span class="spacer"></span>
      <span class="user">{{ username }}</span>
      <button class="btn-link" @click="logout">Sign out</button>
    </header>
    <main class="main">
      <ChatsView v-if="currentView === 'chats'" />
      <SpecsView v-else-if="currentView === 'specs'" />
    </main>
  </div>
</template>

<style scoped>
.boot { display: flex; align-items: center; justify-content: center; height: 100vh; color: var(--text-muted); }
.app-shell { display: flex; flex-direction: column; height: 100vh; }
.topbar {
  display: flex; align-items: center; gap: 16px; height: 48px; padding: 0 16px;
  background: var(--bg-panel); border-bottom: 1px solid var(--border); flex-shrink: 0;
}
.brand { font-weight: 700; font-size: 15px; }
.spacer { flex: 1; }
.nav-item { font-size: 13px; color: var(--text-secondary); padding: 4px 10px; border-radius: var(--radius-sm); cursor: pointer; }
.nav-item:hover { background: var(--accent-light); }
.nav-item.active { color: var(--accent); background: var(--accent-light); font-weight: 600; }
.nav-item.muted { opacity: 0.4; cursor: not-allowed; }
.user { font-size: 12px; color: var(--text-secondary); }
.btn-link { font-size: 12px; color: var(--text-muted); }
.btn-link:hover { color: var(--danger); }
.main { flex: 1; overflow: hidden; }
</style>
