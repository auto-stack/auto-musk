<!--
  StreamingRenderer.vue — 流式文档渲染器（逃生舱，从 web/ 整体移植）
  Plan 022 Phase 7.3: 依赖 useStreamingDocument(增量 JSON 解析) + StreamingTable +
  markstream-vue。把 source 文本流切成 markdown/table 等 segment 渲染。
  import 路径改为相对（逃生舱目录结构：components/ + composables/）。
-->
<template>
  <div class="streaming-document">
    <template v-for="(segment, idx) in segments" :key="segment.type + '-' + idx">
      <MarkdownRender
        v-if="segment.type === 'markdown'"
        :content="segment.text"
        :final="!streaming"
        :max-live-nodes="streaming ? 0 : 320"
        :batch-rendering="streaming"
        :render-batch-size="16"
        :render-batch-delay="8"
        :typewriter="streaming && idx === lastMarkdownIndex"
        :fade="false"
      />
      <component
        v-else-if="segment.type === 'component'"
        :is="registry[segment.componentType]"
        v-bind="segment.props"
        :final="segment.final"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { MarkdownRender, setCustomComponents } from 'markstream-vue'
import { useStreamingDocument } from '../composables/useStreamingDocument'
import StreamingTable from '@/components/StreamingTable.vue'
import PrismCodeBlock from './PrismCodeBlock.vue'

// code_block → prism 语法高亮（默认 PreCodeNode 无高亮；Monaco/Shiki 未安装）。
// 模块级注册一次（全局 mapping）。
setCustomComponents({ code_block: PrismCodeBlock })

const props = defineProps<{
  source: string
  streaming?: boolean
}>()

const sourceRef = computed(() => props.source)
const { segments } = useStreamingDocument(sourceRef)

const lastMarkdownIndex = computed(() => {
  for (let i = segments.value.length - 1; i >= 0; i--) {
    if (segments.value[i].type === 'markdown') return i
  }
  return -1
})

const registry: Record<string, any> = {
  table: StreamingTable,
  // Future: chart: StreamingChart, form: StreamingForm, ...
}
</script>

<style>
.streaming-document {
  /* markstream-vue already scopes its styles under .markstream-vue */
}

.streaming-document > * + * {
  margin-top: 0.75rem;
}
</style>
