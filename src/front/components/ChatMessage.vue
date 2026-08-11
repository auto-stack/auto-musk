<!--
  ChatMessage.vue — 单条聊天消息渲染（逃生舱组件）
  Plan 022 B 阶段：统一消息渲染为独立可复用组件。

  走逃生舱的理由：view fn 的内联展开路径在 ChatsView 通过
  `use chat_message: ChatMessage` 声明引用时不生效（extract.rs 把它
  当外部组件跳过 VIEW_FRAGMENTS 查找）。改为真正的 .vue 组件最直接。

  对标 chat_message.at 的 view fn ChatMessage(msg, is_streaming)。
-->
<template>
  <div :class="msg.role === 'user' ? 'msg-row msg-row-user' : 'msg-row msg-row-ai'">
    <!-- header: 角色标识 + 时间 -->
    <div class="msg-header">
      <span class="msg-role-badge">{{ msg.role === 'assistant' ? '🤖 AI' : '🧑 You' }}</span>
      <span class="msg-time">{{ msg.created_at ? new Date(msg.created_at * 1000).toLocaleTimeString() : '' }}</span>
    </div>
    <!-- 气泡内容（按 role 分支） -->
    <template v-if="msg.role === 'assistant'">
      <div v-if="msg.content" class="msg-bubble msg-bubble-ai">
        <StreamingRenderer :source="msg.content" :streaming="is_streaming" />
      </div>
    </template>
    <template v-else>
      <div class="msg-bubble msg-bubble-user">
        <UserMessage :content="msg.content" />
      </div>
    </template>
    <!-- thinking 推理链（仅 assistant） -->
    <span v-if="msg.thinking" class="msg-thinking">{{ msg.thinking }}</span>
  </div>
</template>

<script setup lang="ts">
import StreamingRenderer from './StreamingRenderer.vue'
import UserMessage from '@/components/UserMessage.vue'

export interface ChatMsg {
  id?: string | number
  role: string
  content: string
  created_at?: number
  thinking?: string
}

const props = defineProps<{
  msg: ChatMsg
  is_streaming?: boolean
}>()
</script>
