<!-- ChatMessage component - Auto-generated from Auto language -->
<script setup lang="ts">
import { computed } from 'vue'
import StreamingRenderer from '@/ext/src/front/components/StreamingRenderer.vue'
import UserMessage from '@/components/UserMessage.vue'
import { msgTimeLabel } from '@/ext/src/front/forge_helpers'


const role = computed<any>(() => props.msg.role)
const isUser = computed<boolean>(() => role.value === 'user')
const rowClass = computed<any>(() => isUser.value ? 'msg-row msg-row-user' : ('msg-row msg-row-ai'))
const badge = computed<any>(() => isUser.value ? '🧑 You' : ('🤖 AI'))
const hasContent = computed<boolean>(() => props.msg.content !== '')
const content = computed<any>(() => props.msg.content)
const time = computed<any>(() => msgTimeLabel(props.msg.created_at))
const hasTime = computed<boolean>(() => time.value !== '')
const hasThinking = computed<boolean>(() => !!props.msg.thinking)
const thinkingText = computed<any>(() => hasThinking.value ? props.msg.thinking : (''))
const hasError = computed<boolean>(() => !!props.msg.error)
const errorText = computed<any>(() => hasError.value ? props.msg.error : (''))

const props = defineProps<{
  msg: any
  is_streaming: boolean
}>()


</script>

<template>
    <div :class="rowClass" class="flex flex-col">
      <div class="flex flex-row msg-header">
        <span class="msg-role-badge">{{ badge }}</span>
        <template v-if="hasTime">
          <span class="msg-time">{{ time }}</span>
        </template>
      </div>
      <template v-if="isUser">
        <div class="flex flex-col msg-bubble msg-bubble-user">
          <UserMessage :content="content" :key="'UserMessage-1'" />
        </div>
      </template>
      <template v-else-if="hasContent">
        <div class="flex flex-col msg-bubble msg-bubble-ai">
          <StreamingRenderer :source="content" :streaming="is_streaming" :key="'StreamingRenderer-2'" />
        </div>
      </template>
      <template v-if="hasThinking">
        <span class="msg-thinking">{{ thinkingText }}</span>
      </template>
      <template v-if="hasError">
        <span class="msg-error">{{ errorText }}</span>
      </template>
    </div>

</template>

<style>
/* ChatMessage bubble layout — gen track (PLAN-025 调查发现 .at 生成器
   丢弃了 `class: .rowClass` 绑定 + index.css 无 msg-* 规则，导致气泡撑满。
   这里补齐对齐 + 自适应宽度。TODO: 沉淀到 .at 生成器或 ext CSS 使其持久。）*/
.msg-row {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  max-width: 85%;
}
.msg-row-user {
  align-self: flex-end;
  align-items: flex-end;
}
.msg-row-ai {
  align-self: flex-start;
  align-items: flex-start;
}
.msg-bubble {
  padding: 0.6rem 0.9rem;
  border-radius: 0.75rem;
  word-wrap: break-word;
  line-height: 1.5;
}
.msg-bubble-user {
  background: hsl(var(--primary) / 0.1);
  border: 1px solid hsl(var(--primary) / 0.2);
}
.msg-bubble-ai {
  background: hsl(var(--muted-foreground) / 0.05);
  border: 1px solid hsl(var(--border));
}
.msg-role-badge {
  font-size: 0.8rem;
  font-weight: 500;
  color: hsl(var(--muted-foreground));
}
.msg-time {
  font-size: 0.75rem;
  color: hsl(var(--muted-foreground));
}
</style>
