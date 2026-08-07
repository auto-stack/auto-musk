<!--
  SecretaryMessageWrapper.vue — SecretaryMessage 包装组件（逃生舱）
  Plan 022 B 类批4：规避 composable facade 的 ref 解包限制——codegen 把
  useGateInbox.currentSecretary 当普通值访问，但它是 Ref<PendingGate|null>，
  直接传给 SecretaryMessage 会类型不匹配。本 wrapper 内部用 useGateInbox
  （正确 .value 解包），.at 只声明 wrapper + 绑 emit。
-->
<template>
  <SecretaryMessage
    v-if="gate"
    :gate="gate"
    :queue-position="queuePosition"
    @approve="onApprove"
    @reject="onReject"
    @snooze="$emit('snooze')"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import SecretaryMessage from './SecretaryMessage.vue'
import { useGateInbox } from '../composables/useGateInbox'

defineEmits<{
  approve: []
  reject: []
  snooze: []
}>()

const { currentSecretary, badgeCount, resolveGate, dismissSecretary } = useGateInbox()

const gate = computed(() => currentSecretary.value)
const queuePosition = computed(() => badgeCount.value)

function onApprove() {
  if (currentSecretary.value) {
    resolveGate(currentSecretary.value.gateId, 'approved')
  }
}
function onReject() {
  if (currentSecretary.value) {
    resolveGate(currentSecretary.value.gateId, 'rejected')
  }
}
</script>
