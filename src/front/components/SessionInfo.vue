<!--
  SessionInfo.vue — 会话信息 tooltip（逃生舱组件，Plan 022 视觉对齐）
  对齐原版 ChatsView.vue 的 .session-info-tooltip。

  显示三行：Chat ID（可复制）/ Messages 消息数 / Token Cost token 消耗。
  点击 info 按钮切换显隐，点击外部关闭。
-->
<template>
  <div ref="wrapperRef" class="session-info-wrapper">
    <button
      class="session-info-btn"
      :title="t('chat.sessionInfo')"
      @click="open = !open"
    >
      <Info :size="15" />
    </button>
    <div v-if="open" class="session-info-tooltip" @click.stop>
      <div class="session-info-row">
        <span class="session-info-label">{{ t('chat.chatId') }}</span>
        <code class="session-info-value session-info-id">{{ sessionId || '—' }}</code>
        <button class="session-info-copy" :title="t('chat.copyChatId')" @click="copyId">
          <CopyCheck v-if="copied" :size="12" />
          <Copy v-else :size="12" />
        </button>
      </div>
      <div class="session-info-row">
        <span class="session-info-label">{{ t('chat.messages') }}</span>
        <span class="session-info-value">{{ messageCount }}</span>
      </div>
      <div class="session-info-row">
        <span class="session-info-label">{{ t('chat.tokenCost') }}</span>
        <span class="session-info-value">{{ tokenCost }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Info, Copy, CopyCheck } from 'lucide-vue-next'
import { useForgeStoreStore } from '@/stores/useForgeStoreStore'

const { t } = useI18n()
const store = reactive(useForgeStoreStore())

const open = ref(false)
const copied = ref(false)
const wrapperRef = ref<HTMLDivElement>()

const sessionId = computed(() => store.session_id || '')
const messageCount = computed(() => store.messages?.length || 0)
const tokenCost = computed(() => {
  let total = 0
  const errands = store.errands
  if (errands && typeof errands === 'object') {
    for (const key in errands) {
      total += (errands as any)[key]?.token_usage || 0
    }
  }
  return total
})

function copyId() {
  if (!sessionId.value) return
  navigator.clipboard.writeText(sessionId.value).then(() => {
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  })
}

function onDocClick(e: MouseEvent) {
  if (open.value && wrapperRef.value && !wrapperRef.value.contains(e.target as Node)) {
    open.value = false
  }
}

onMounted(() => document.addEventListener('click', onDocClick))
onUnmounted(() => document.removeEventListener('click', onDocClick))
</script>

<style scoped>
.session-info-wrapper {
  position: relative;
}
.session-info-btn {
  display: flex; align-items: center; justify-content: center;
  width: 28px; height: 28px; border: none; border-radius: 6px;
  background: transparent; color: hsl(var(--muted-foreground)); cursor: pointer;
}
.session-info-btn:hover { background: hsl(var(--accent)); color: hsl(var(--foreground)); }
.session-info-tooltip {
  position: absolute;
  top: calc(100% + 0.5rem);
  right: 0;
  min-width: 280px;
  background: hsl(var(--background));
  border: 1px solid hsl(var(--border));
  border-radius: 0.5rem;
  padding: 0.75rem;
  box-shadow: 0 4px 12px rgba(0,0,0,0.15);
  z-index: 100;
  display: flex; flex-direction: column; gap: 0.5rem;
}
.session-info-row { display: flex; align-items: center; gap: 0.5rem; }
.session-info-label { font-size: 0.75rem; color: hsl(var(--muted-foreground)); min-width: 5rem; }
.session-info-value { font-size: 0.82rem; color: hsl(var(--foreground)); flex: 1; }
.session-info-id { font-family: monospace; font-size: 0.75rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.session-info-copy {
  display: flex; align-items: center; justify-content: center;
  width: 24px; height: 24px; border: none; border-radius: 4px;
  background: transparent; color: hsl(var(--muted-foreground)); cursor: pointer; flex-shrink: 0;
}
.session-info-copy:hover { background: hsl(var(--accent)); color: hsl(var(--foreground)); }
</style>
