<!-- NativeSelect.vue — 原生 select 平台实现(PLAN-041 债务收口)。
     auto-lang schema 无原生 select 元素(select→shadcn Select);
     表单/状态控件的 native select 形态经本 ext 组件对齐 web DOM。
     props: value(当前值) options({value,label,disabled}[]) cls(class 串)。
     emit: change(value)。 -->
<script setup lang="ts">
defineProps<{
  value: string
  options: { value: string; label: string; disabled?: boolean }[]
  cls?: string
}>()
defineEmits<{ change: [value: string] }>()
</script>
<template>
  <select :value="value" :class="cls" @change="$emit('change', ($event.target as HTMLSelectElement).value)">
    <option v-for="o in options" :key="o.value" :value="o.value" :disabled="o.disabled">{{ o.label }}</option>
  </select>
</template>
