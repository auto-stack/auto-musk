<template>
  <!-- PLAN-032: 汇报报告 deck 预览（16:9 沙箱 iframe——无脚本/无同源/无表单） -->
  <iframe class="deck-frame" sandbox="" :srcdoc="srcdoc"></iframe>
</template>

<script setup lang="ts">
defineProps<{ srcdoc: string }>()
</script>

<style scoped>
.deck-frame {
  width: 100%;
  aspect-ratio: 16 / 9;
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  background: #fff;
}
</style>
