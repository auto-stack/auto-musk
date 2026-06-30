<template>
  <div class="onboarding-overlay" @click.self="$emit('cancel')">
    <div class="onboarding-dialog">
      <!-- confirm step -->
      <template v-if="step === 'confirm'">
        <div class="ob-icon"><FolderOpen :size="40" /></div>
        <h2 class="ob-title">{{ t('onboarding.emptyTitle') }}</h2>
        <p class="ob-message">{{ t('onboarding.emptyMessage') }}</p>
        <p class="ob-path">{{ workspace?.path }}</p>
        <div class="ob-actions">
          <button class="ob-btn ob-btn-ghost" @click="$emit('cancel')">{{ t('onboarding.no') }}</button>
          <button class="ob-btn ob-btn-primary" @click="step = 'describe'">{{ t('onboarding.yes') }}</button>
        </div>
      </template>

      <!-- describe step -->
      <template v-else>
        <div class="ob-icon"><Flame :size="40" /></div>
        <h2 class="ob-title">{{ t('onboarding.describePrompt') }}</h2>
        <textarea
          ref="descRef"
          v-model="description"
          class="ob-textarea"
          :placeholder="t('onboarding.describePlaceholder')"
          rows="4"
          :disabled="starting"
          @keydown.ctrl.enter="startProject"
        />
        <div class="ob-actions">
          <button class="ob-btn ob-btn-ghost" :disabled="starting" @click="step = 'confirm'">{{ t('onboarding.back') }}</button>
          <button
            class="ob-btn ob-btn-primary"
            :disabled="!description.trim() || starting"
            @click="startProject"
          >
            {{ starting ? t('onboarding.starting') : t('onboarding.startProject') }}
          </button>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { FolderOpen, Flame } from 'lucide-vue-next'
import { useForge } from '@/composables/useForge'
import { useViewState } from '@/composables/useViewState'
import type { WorkspaceMeta } from '@/composables/useProject'

const props = defineProps<{ workspace: WorkspaceMeta | null }>()
const emit = defineEmits<{
  (e: 'done'): void
  (e: 'cancel'): void
}>()

const { t } = useI18n()
const { clearSession, sendMessage } = useForge()
const { setView } = useViewState()

const step = ref<'confirm' | 'describe'>('confirm')
const description = ref('')
const starting = ref(false)
const descRef = ref<HTMLTextAreaElement | null>(null)

// Auto-focus the textarea when entering the describe step.
function focusTextarea() {
  nextTick(() => descRef.value?.focus())
}

// Watch step to focus — simpler: call on button click via @click.
import { watch } from 'vue'
watch(step, (s) => { if (s === 'describe') focusTextarea() })

async function startProject() {
  const desc = description.value.trim()
  if (!desc || starting.value) return
  starting.value = true
  try {
    // Fresh session for the new project, then send the description as the
    // first message — the agent initializes the project via its tools.
    await clearSession(props.workspace?.path)
    await sendMessage(desc)
    setView('chats')
    emit('done')
  } catch {
    // sendMessage/clearSession set their own error state; just re-enable.
    starting.value = false
  }
}
</script>

<style scoped>
.onboarding-overlay {
  position: fixed; inset: 0; z-index: 200;
  background: rgba(0, 0, 0, 0.5);
  display: flex; align-items: center; justify-content: center;
}
.onboarding-dialog {
  background: var(--af-card); border: 1px solid var(--af-border);
  border-radius: 12px; padding: 2rem; max-width: 480px; width: 90%;
  display: flex; flex-direction: column; align-items: center; gap: 0.75rem;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}
.ob-icon {
  color: var(--af-primary); display: flex; align-items: center; justify-content: center;
  margin-bottom: 0.25rem;
}
.ob-title { font-size: 1.15rem; font-weight: 600; color: var(--af-fg); margin: 0; text-align: center; }
.ob-message { font-size: 0.92rem; color: var(--af-muted); margin: 0; text-align: center; line-height: 1.5; }
.ob-path {
  font-size: 0.78rem; color: var(--af-muted); opacity: 0.7;
  font-family: monospace; margin: 0; word-break: break-all; text-align: center;
  max-width: 100%;
}
.ob-textarea {
  width: 100%; box-sizing: border-box;
  background: var(--af-bg); border: 1px solid var(--af-border); border-radius: 6px;
  padding: 0.6rem; color: var(--af-fg); font-size: 0.9rem; font-family: inherit;
  resize: vertical; min-height: 80px; outline: none;
}
.ob-textarea:focus { border-color: hsl(var(--primary) / 0.5); }
.ob-actions { display: flex; gap: 0.6rem; justify-content: center; margin-top: 0.5rem; width: 100%; }
.ob-btn {
  padding: 0.5rem 1.2rem; border-radius: 6px; border: 1px solid var(--af-border);
  cursor: pointer; font-size: 0.88rem; transition: all 0.15s;
}
.ob-btn-ghost { background: transparent; color: var(--af-fg); }
.ob-btn-ghost:hover { background: hsl(var(--muted-foreground) / 0.08); }
.ob-btn-primary {
  background: hsl(var(--primary)); color: #fff; border-color: transparent; font-weight: 500;
}
.ob-btn-primary:hover { filter: brightness(1.1); }
.ob-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
