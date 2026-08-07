<!--
  QuestionnaireMessage.vue — 问卷消息包装组件（逃生舱）
  Plan 022 Phase 7.3: 包装 QuestionnaireCard，内部调 questionnaireFor 解析 msg。
  存在理由：.at 的组件 props 绑定不支持 fn 调用表达式（getQuestions(.msg) 会被
  codegen 丢弃成 null），故把 fn 调用移入逃生舱，.at 只传 msg（简单字段访问）。
-->
<template>
  <QuestionnaireCard
    v-if="questions.length > 0"
    :questions="questions"
    @submit="(answers) => $emit('submit', answers)"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import QuestionnaireCard from './QuestionnaireCard.vue'
import { questionnaireFor, type Question } from '../questionnaire'

const props = defineProps<{
  msg: { role: string; content: string }
}>()

defineEmits<{
  submit: [answers: Record<string, string | string[]>]
}>()

const questions = computed<Question[]>(() => {
  return questionnaireFor(props.msg)?.questions ?? []
})
</script>
