<!--
  MentionInput.vue — @mention 输入框（逃生舱，完整版）
  Plan 022 Phase 7c: 封装 textarea + v-html backdrop + MentionDropdown（键盘导航）+
  /relay /superpower /spec1 命令解析。emit submit(text)/stop。

  走逃生舱的理由（plan 7c 探索结论）：.at 无法表达 v-html backdrop（mention 高亮
  视觉核心），且牵连 ref+keydown+5 交互函数，整体封装内聚度最高。

  参照 web/src/views/ChatsView.vue:368-402（模板）+ 646-727（mention 交互）+
  1087-1160（命令解析）。
-->
<template>
  <div class="chats-input-bar">
    <div class="input-inner">
      <div class="input-row">
        <div class="input-compose">
          <!-- v-html backdrop：@mention 高亮层（.at 无法表达 v-html） -->
          <div class="input-backdrop" v-html="backdropHtml"></div>
          <textarea
            ref="textareaRef"
            v-model="text"
            class="chats-input"
            :placeholder="t('chat.inputPlaceholder')"
            :disabled="disabled"
            @input="handleInput"
            @keydown="handleKeydown"
            @keydown.enter.exact.prevent="submitText"
          ></textarea>
        </div>
        <button
          v-if="disabled"
          class="send-btn stop"
          @click="$emit('cancel')"
        >
          <Square :size="16" />
        </button>
        <button
          v-else
          class="send-btn"
          :disabled="!text.trim()"
          @click="submitText"
        >
          <Send :size="16" />
        </button>
      </div>
      <MentionDropdown
        ref="mentionRef"
        :professions="professionsList"
        :visible="mentionVisible"
        :filter="mentionFilter"
        :anchor-rect="mentionAnchor"
        @select="handleMentionSelect"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { Send, Square } from 'lucide-vue-next'
import MentionDropdown from './MentionDropdown.vue'
import { useAgentConfigs } from '../composables/useAgentConfigs'
import {
  DEFAULT_PROFESSIONS,
  renderInputMentions,
  buildMentionNames,
} from '../forge_helpers'

// B 类批1：动态 profession 列表（useAgentConfigs singleton，App/chats_view Init 时 loadConfigs）。
const { configs: agentConfigs } = useAgentConfigs()
const { t } = useI18n()

interface ProfessionOption {
  id: string
  name: string
}

const props = defineProps<{
  disabled?: boolean
  professions?: ProfessionOption[]
}>()

const emit = defineEmits<{
  send: [text: string]
  cancel: []
}>()

const text = ref('')
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const mentionRef = ref<InstanceType<typeof MentionDropdown> | null>(null)

// @mention state
const mentionVisible = ref(false)
const mentionFilter = ref('')
const mentionAnchor = ref<DOMRect | null>(null)

const professionsList = computed(() => {
  if (props.professions && props.professions.length) return props.professions
  // 动态 profession 列表（loadConfigs 后填充）；未加载时 fallback DEFAULT_PROFESSIONS
  const dynamic = agentConfigs.value.map((c) => ({ id: c.profession_id || c.id, name: c.name }))
  return dynamic.length > 0 ? dynamic : DEFAULT_PROFESSIONS
})
const mentionNames = computed(() => buildMentionNames(professionsList.value))
const backdropHtml = computed(() => renderInputMentions(text.value, mentionNames.value))

function handleInput(e: Event) {
  const el = e.target as HTMLTextAreaElement
  const val = el.value
  const pos = el.selectionStart

  // 从光标向前找 @（必须是行首或空白后）
  const textBeforeCursor = val.slice(0, pos)
  const atIdx = textBeforeCursor.lastIndexOf('@')
  if (atIdx >= 0) {
    const charBefore = atIdx > 0 ? val[atIdx - 1] : ''
    if (charBefore === '' || /\s/.test(charBefore)) {
      const afterAt = textBeforeCursor.slice(atIdx + 1)
      // @ 后无空格才显示下拉（仍在输入名字）
      if (!afterAt.includes(' ')) {
        mentionFilter.value = afterAt
        mentionAnchor.value = el.getBoundingClientRect()
        mentionVisible.value = true
        return
      }
    }
  }
  mentionVisible.value = false
}

function handleMentionSelect(id: string) {
  const name = professionsList.value.find((p) => p.id === id)?.name || id
  const val = text.value
  const ta = textareaRef.value
  const pos = ta?.selectionStart ?? val.length
  const textBeforeCursor = val.slice(0, pos)
  const atIdx = textBeforeCursor.lastIndexOf('@')
  if (atIdx >= 0) {
    const before = val.slice(0, atIdx)
    const after = val.slice(pos)
    text.value = `${before}@${name} ${after}`
  } else {
    text.value = `@${name} ${val}`
  }
  mentionVisible.value = false
  nextTick(() => {
    textareaRef.value?.focus()
  })
}

function handleKeydown(e: KeyboardEvent) {
  if (!mentionVisible.value || !mentionRef.value?.hasItems()) return
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    mentionRef.value.moveDown()
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    mentionRef.value.moveUp()
  } else if (e.key === 'Enter' || e.key === 'Tab') {
    e.preventDefault()
    const id = mentionRef.value.currentId()
    if (id) handleMentionSelect(id)
  } else if (e.key === 'Escape') {
    mentionVisible.value = false
  }
}

/** 发送文本。命令解析（/relay//superpower//spec1）需 startRun（useRelay 未接线），
 *  本 phase 透传原始文本给父组件（父走 store.Send + startForgeStream）；
 *  命令路由待 relay 后端接线计划补齐，登记 KNOWN-DEBT。 */
function submitText() {
  const t = text.value.trim()
  if (!t) return
  text.value = ''
  mentionVisible.value = false
  emit('send', t)
}
</script>

<style scoped>
.chats-input-bar {
  border-top: 1px solid var(--af-border, hsl(220 13% 91%));
  padding: 0.75rem;
  background: var(--af-card, #fff);
}
.input-inner { position: relative; }
.input-row { display: flex; align-items: flex-end; gap: 0.5rem; }
.input-compose { position: relative; flex: 1; }
.input-backdrop {
  position: absolute;
  top: 0; left: 0;
  width: 100%;
  min-height: 1.5rem;
  padding: 0.5rem 0.75rem;
  font-family: inherit;
  font-size: 0.88rem;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
  pointer-events: none;
  color: transparent;
  background: var(--af-input, hsl(220 14% 96%));
  border: 1px solid var(--af-border, hsl(220 13% 91%));
  border-radius: 6px;
}
.input-backdrop :deep(.inline-mention) {
  background: hsl(220 90% 56% / 0.12);
  color: hsl(220 90% 56%);
  border-radius: 3px;
  padding: 0 0.2rem;
  font-weight: 500;
}
.chats-input {
  position: relative;
  width: 100%;
  min-height: 1.5rem;
  max-height: 200px;
  padding: 0.5rem 0.75rem;
  font-family: inherit;
  font-size: 0.88rem;
  line-height: 1.5;
  background: transparent;
  border: 1px solid var(--af-border, hsl(220 13% 91%));
  border-radius: 6px;
  color: var(--af-fg, hsl(220 14% 10%));
  resize: none;
  outline: none;
}
.chats-input:focus {
  border-color: hsl(220 90% 56%);
  box-shadow: 0 0 0 2px hsl(220 90% 56% / 0.15);
}
.chats-input::placeholder { color: var(--af-muted, hsl(220 9% 46%)); }
.send-btn {
  flex-shrink: 0;
  width: 36px; height: 36px;
  border: none;
  border-radius: 6px;
  background: hsl(220 90% 56%);
  color: #fff;
  cursor: pointer;
  font-size: 0.9rem;
  display: flex;
  align-items: center;
  justify-content: center;
}
.send-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.send-btn.stop { background: hsl(0 72% 51%); }
</style>
