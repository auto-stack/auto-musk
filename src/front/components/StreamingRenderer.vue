<!--
  StreamingRenderer.vue — 流式文档渲染器逃生舱（PLAN-038 T12 对齐上游）
  Plan 022 Phase 7.3 初版为 markstream-vue 骨架的本地移植；PLAN-038 收敛为
  @autodown/vue StreamingRenderer（同源超集）的再导出 + musk 专属注册：

  - 上游特性面（本组件自带，PLAN-038 render-switch 白名单登记）：
    :::details 容器变换 / katex+mermaid 模块级启用 / codeBlockProps
    (showHeader/showCopyButton/showExpandButton) / placeholder 机制 /
    lowlight 后处理(MutationObserver)。
  - musk 保留：PrismCodeBlock 经 markstream-vue setCustomComponents 的全局注册
    （code_block 高亮路径不变）。@autodown/vue 与本文件解析到同一 markstream-vue
    实例（依赖提升，无嵌套副本），注册对上游内部 MarkdownRender 同样生效。
  - 样式：上游增量样式经 inject_styles.ts 的 '@autodown/vue/style.css' 引入。

  此文件经 auto-man mount_platform_impls 升格复制为 gen 的 platform/markdown.vue。
-->
<script lang="ts">
import { StreamingRenderer as UpstreamStreamingRenderer } from '@autodown/vue'
import { setCustomComponents } from 'markstream-vue'
import PrismCodeBlock from './PrismCodeBlock.vue'

// code_block → prism 语法高亮（注册一次，全局 mapping；行为不变约束下的
// musk 专属路径——上游 lowlight 后处理不覆盖自定义组件渲染的块）。
setCustomComponents({ code_block: PrismCodeBlock })

export default UpstreamStreamingRenderer
</script>
