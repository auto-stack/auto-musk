<!--
  PrismCodeBlock.vue — code_block 语法高亮渲染（逃生舱）
  覆盖 markstream-vue 默认的 PreCodeNode（无高亮；Monaco/Shiki 是未安装的可选
  依赖）。用项目已装的 prismjs 做同步高亮——聊天体量的代码块足够快。
  经 StreamingRenderer 顶部 setCustomComponents({ code_block: ... }) 注册。

  工具栏对齐 markstream-vue 默认 CodeBlockNode 的形态：左语言名、右操作区
  （复制 + 折叠/展开，lucide 图标）。
-->
<template>
  <div class="prism-code-wrap">
    <div class="prism-code-header">
      <span v-if="langLabel" class="prism-code-lang">{{ langLabel }}</span>
      <span v-else class="prism-code-lang prism-code-lang-empty">text</span>
      <div class="prism-code-actions">
        <button
          v-if="node.code"
          class="prism-code-btn"
          :title="copied ? '已复制' : '复制代码'"
          @click="copyCode"
        >
          <Check v-if="copied" :size="13" />
          <Copy v-else :size="13" />
        </button>
        <button
          class="prism-code-btn"
          :title="collapsed ? '展开代码' : '折叠代码'"
          @click="collapsed = !collapsed"
        >
          <ChevronDown v-if="!collapsed" :size="14" />
          <ChevronUp v-else :size="14" />
        </button>
      </div>
    </div>
    <pre v-show="!collapsed" class="prism-code" :class="'language-' + lang"><code v-html="html"></code></pre>
    <div v-if="collapsed" class="prism-code-collapsed">已折叠 · {{ lineCount }} 行</div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { Check, ChevronDown, ChevronUp, Copy } from 'lucide-vue-next'
import Prism from 'prismjs'
// 默认 bundle 已含 markup/css/clike/javascript；按依赖顺序补常用语言。
// prism-c 必须先于 prism-cpp（cpp = extend('c')）；依赖顺序此前由旧模块图的
// 加载次序隐式满足，平台挂载（platform/）后图序改变而暴露，故显式声明。
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
import 'prismjs/components/prism-c'
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

const lineCount = computed(() => (props.node.code || '').split('\n').length)

const collapsed = ref(false)
const copied = ref(false)
let copyTimer: ReturnType<typeof setTimeout> | undefined

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

function copyCode(): void {
  const text = props.node.code ?? ''
  const done = (): void => {
    copied.value = true
    if (copyTimer) clearTimeout(copyTimer)
    copyTimer = setTimeout(() => { copied.value = false }, 1500)
  }
  if (navigator.clipboard?.writeText) {
    navigator.clipboard.writeText(text).then(done, done)
  } else {
    const ta = document.createElement('textarea')
    ta.value = text
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
    done()
  }
}
</script>

<style>
/* 整卡统一 prism-tomorrow 深色底（与 token 配色同源），避免 wrap 用浅色
   --code-bg 而代码区深色造成的"底部半条白边"。 */
.prism-code-wrap {
  margin: 0.4rem 0;
  border: 1px solid #3d3d3d;
  border-radius: 8px;
  overflow: hidden;
  background: #2d2d2d;
}
.prism-code-header {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.25rem 0.4rem 0.25rem 0.75rem;
  border-bottom: 1px solid #3d3d3d;
}
.prism-code-lang {
  flex: 1;
  min-width: 0;
  font-size: 0.7rem;
  color: #b3b3b3;
  text-transform: lowercase;
  user-select: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.prism-code-lang-empty { opacity: 0.55; }
.prism-code-actions { display: flex; align-items: center; gap: 0.15rem; flex-shrink: 0; }
.prism-code-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: #b3b3b3;
  cursor: pointer;
}
.prism-code-btn:hover { background: #3d3d3d; color: #fff; }
/* 关键修复：pre 自身带 .prism-code 类。原覆盖写成了后代选择器
   （.prism-code pre[...]，匹配的是 pre 内嵌套的 pre，从未命中），
   prism-tomorrow 的 pre[class*=language-] { margin:1em; padding:1em;
   background:#2d2d2d } 全部泄漏——wrap 浅色底 + pre 深色底叠出底部白条。
   用 .prism-code-wrap pre.prism-code（0,2,1）稳定压过（0,1,1）。 */
.prism-code-wrap pre.prism-code {
  margin: 0;
  padding: 0.55rem 0.8rem;
  overflow-x: auto;
  font-size: 0.82rem;
  line-height: 1.55;
  font-family: var(--ms-font-mono, ui-monospace, monospace);
  color: #ccc;
  background: transparent;
  tab-size: 4;
}
.prism-code-wrap pre.prism-code code {
  display: block;
  background: transparent;
}
.prism-code-collapsed {
  padding: 0.35rem 0.8rem;
  font-size: 0.75rem;
  color: #808080;
  user-select: none;
}
</style>
