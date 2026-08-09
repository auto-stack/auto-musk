<!--
  UserMessage.vue — user 消息气泡 + @mention 高亮（逃生舱）
  Plan 022 Phase 7.3/7.4：原生 ChatsView 用 v-html="renderMentions(msg.content)"
  做 @agent 高亮。.at 无法表达 v-html（map_tag 无 html/raw tag），故封此组件。
  renderMentions helper 已在 forge_helpers.ts（转义 HTML + @word 包高亮 span）。
-->
<template>
  <div class="user-text" v-html="html"></div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { renderMentions } from '../forge_helpers'

const props = defineProps<{
  content: string
}>()

const html = computed(() => renderMentions(props.content))
</script>

<style scoped>
.user-text {
  color: inherit;
  line-height: 1.5;
  word-break: break-word;
}
.user-text :deep(.inline-mention) {
  background: hsl(220 90% 56% / 0.12);
  color: hsl(220 90% 56%);
  border-radius: 3px;
  padding: 0 0.2rem;
  font-weight: 500;
}
</style>
