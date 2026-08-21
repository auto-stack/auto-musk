<!-- AutoDownEditor stub — Phase 4 (Wikis) will port the real TipTap editor.
     Specs category cards + WikiView import this with v-model + several event
     handlers (onLinkClick/onCancel/onSave). The stub accepts all of them via a
     permissive interface and renders a plain textarea so they compile + work. -->
<script setup lang="ts">
// PLAN-030 复审修复：按头注释自述的 permissive 意图实现——同时接受
// v-model(modelValue) 与旧式 content 绑定（7 处调用点仍用 content；
// 此前 modelValue 必填导致调用点类型错 + 运行时编辑框为空）。
defineProps<{ modelValue?: string; content?: string; placeholder?: string }>()
defineEmits<{
  'update:modelValue': [value: string]
  linkClick: [link: string]
  cancel: []
  save: [content: string]
}>()
</script>
<template>
  <textarea
    :value="modelValue ?? content ?? ''"
    class="autodown-stub"
    :placeholder="placeholder || 'Edit content...'"
    @input="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
  />
</template>
<style scoped>
.autodown-stub {
  width: 100%; min-height: 200px; padding: 8px;
  border: 1px solid var(--af-border); border-radius: 6px;
  background: var(--af-bg); color: var(--af-fg);
  font-family: monospace; font-size: 0.9rem; resize: vertical;
}
</style>
