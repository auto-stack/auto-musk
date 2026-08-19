<!--
  PrismCodeBlock.vue — code_block 语法高亮渲染（逃生舱）
  覆盖 markstream-vue 默认的 PreCodeNode（无高亮；Monaco/Shiki 是未安装的可选
  依赖）。用项目已装的 prismjs 做同步高亮——聊天体量的代码块足够快。
  经 StreamingRenderer 顶部 setCustomComponents({ code_block: ... }) 注册。
-->
<template>
  <div class="prism-code-wrap">
    <div v-if="langLabel" class="prism-code-lang">{{ langLabel }}</div>
    <pre class="prism-code" :class="'language-' + lang"><code v-html="html"></code></pre>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import Prism from 'prismjs'
// 默认 bundle 已含 markup/css/clike/javascript；按依赖顺序补常用语言
import 'prismjs/components/prism-rust'
import 'prismjs/components/prism-typescript'
import 'prismjs/components/prism-javascript'
import 'prismjs/components/prism-json'
import 'prismjs/components/prism-bash'
import 'prismjs/components/prism-python'
import 'prismjs/components/prism-markdown'
import 'prismjs/components/prism-yaml'
import 'prismjs/components/prism-toml'
import 'prismjs/components/prism-sql'
import 'prismjs/components/prism-java'
import 'prismjs/components/prism-cpp'
import 'prismjs/components/prism-go'

const props = defineProps<{
  node: { type: 'code_block'; language: string; code: string; raw?: string }
  final?: boolean
}>()

/** 语言别名归一 → prism grammar 键；无 grammar 时退回纯文本转义。 */
const ALIAS: Record<string, string> = {
  rs: 'rust', ts: 'typescript', js: 'javascript', mjs: 'javascript',
  cjs: 'javascript', sh: 'bash', shell: 'bash', zsh: 'bash', console: 'bash',
  yml: 'yaml', py: 'python', md: 'markdown', 'c++': 'cpp', golang: 'go',
}

const lang = computed(() => {
  const raw = (props.node.language || '').toLowerCase()
  return ALIAS[raw] ?? raw
})

const langLabel = computed(() => (props.node.language || '').trim())

const html = computed(() => {
  const code = props.node.code ?? ''
  const grammar = (Prism.languages as Record<string, any>)[lang.value]
  if (grammar) {
    try {
      return Prism.highlight(code, grammar, lang.value)
    } catch {
      /* fall through to escaped plain text */
    }
  }
  return code.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
})
</script>

<style>
.prism-code-wrap {
  margin: var(--ms-flow-codeblock-y) 0;
  border: 1px solid var(--code-border);
  border-radius: 8px;
  overflow: hidden;
  background: var(--code-bg);
}
.prism-code-lang {
  font-size: 0.7rem;
  color: var(--code-line-number);
  padding: 0.3rem 0.75rem 0;
  text-transform: lowercase;
  user-select: none;
}
.prism-code {
  margin: 0;
  padding: 0.6rem 0.8rem;
  overflow-x: auto;
  font-size: 0.82rem;
  line-height: 1.55;
  font-family: var(--ms-font-mono, ui-monospace, monospace);
  color: var(--code-fg);
  background: transparent;
  tab-size: 4;
}
.prism-code-wrap:has(.prism-code-lang) .prism-code { padding-top: 0.25rem; }
/* prism 默认主题的 pre 背景交由 wrap 控制 */
.prism-code code[class*='language-'], .prism-code pre[class*='language-'] {
  background: transparent; text-shadow: none;
}
</style>
