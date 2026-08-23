<!--
  MarkdownRender.vue — renderer 域 @autodown/vue 消费适配器（PLAN-038 T13）
  //
  // @autodown/vue 实测导出面仅 StreamingRenderer/StreamingTable/useStreamingDocument
  //（无 MarkdownRender 命名导出）——本适配器保留 musk 端口消费面（content/final
  // props,raw_preview 等调用方零改动），内部映射到上游 StreamingRenderer：
  //     content → source；final（默认 true，静态渲染语境）→ streaming = !final
  // 超集语义差异（容器 div.streaming-document / IAL table segment / codeBlockProps
  // / katex·mermaid 启用 / :::details 变换）经 render-switch 对拍白名单登记。
  // 上游内部 markstream 语义不变——切换前后核心 markdown DOM 一致（对拍保障）。
-->
<script setup lang="ts">
import { StreamingRenderer } from '@autodown/vue'

const props = withDefaults(
  defineProps<{
    content: string
    final?: boolean
  }>(),
  { final: true },
)
</script>

<template>
  <StreamingRenderer :source="props.content" :streaming="!props.final" />
</template>
