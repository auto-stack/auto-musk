<!--
  StreamingRenderer.vue — 流式文档渲染器平台实现（PLAN-038 T12 → 0.2.0 收口）
  Plan 022 Phase 7.3 初版为 markstream-vue 骨架的本地移植；PLAN-038 收敛为
  @autodown/vue StreamingRenderer（同源超集）的再导出；vendor 0.2.0 起
  markstream-vue 彻底消灭（auto-down 008 解析器 Auto 化）：

  - 上游特性面（本组件自带，PLAN-038 render-switch 白名单登记）：
    :::details 容器变换 / katex+mermaid 模块级启用 / codeBlockProps / 
    placeholder 机制。
  - code_block 高亮：0.2.0 删 setCustomComponents 出口——切上游内置
    lowlight（hljs）路径经 enableHighlight() 模块级启用；musk 原
    PrismCodeBlock（prismjs）退役，视觉近似由 038 T15 矩阵背书
    （三引擎 token 级一致不可达，≤71%）。
  - 样式：上游样式经 inject_styles.ts 的 '@autodown/vue/style.css' 引入。

  此文件经 auto-man mount_platform_impls 升格复制为 gen 的 platform/markdown.vue。
-->
<script lang="ts">
import { StreamingRenderer as UpstreamStreamingRenderer, enableHighlight } from '@autodown/vue'

// code_block 高亮：上游内置 lowlight 路径（enableKatex/enableMermaid 的
// 同族开关；模块级一次性启用）。
enableHighlight()

export default UpstreamStreamingRenderer
</script>
