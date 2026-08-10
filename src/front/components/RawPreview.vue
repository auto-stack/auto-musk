<!--
  RawPreview.vue — raw 文件预览（逃生舱组件，Plan 022 遗留）
  对齐原版 WikiView.vue:142-155 的 viewing-raw 分支：
  img（图片）/ iframe（pdf）/ MarkdownRender（文本）/ 下载链接（其他）。
  文本内容读取走逃生舱 raw_upload.ts（异步 fetch），对齐原版 selectRawNode。
-->
<template>
  <div class="raw-preview">
    <img v-if="isImage" :src="fileUrl" class="raw-preview-img" />
    <iframe v-else-if="isPdf" :src="fileUrl" class="raw-preview-pdf" />
    <div v-else-if="isText" class="raw-preview-text">
      <MarkdownRender :content="textContent" />
    </div>
    <div v-else class="raw-download">
      <FileIcon :size="24" />
      <a :href="fileUrl" download class="download-link">{{ path }}</a>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { MarkdownRender } from 'markstream-vue'
import { rawFileUrl, loadRawFileText } from '../raw_upload'

const props = defineProps<{
  workspace: string
  path: string
}>()

const textContent = ref('')

const fileUrl = computed(() => rawFileUrl(props.workspace, props.path))
const isImage = computed(() => /\.(png|jpe?g|gif|svg|webp|bmp|ico)$/i.test(props.path))
const isPdf = computed(() => /\.pdf$/i.test(props.path))
const isText = computed(() =>
  /\.(md|txt|csv|json|xml|yaml|yml|html|css|js|ts|rs|toml|sh|bat|py)$/i.test(props.path),
)

async function load() {
  if (!isText.value) return
  try {
    textContent.value = await loadRawFileText(props.workspace, props.path)
  } catch {
    textContent.value = ''
  }
}

onMounted(load)
watch(() => props.path, load)
</script>

<style scoped>
.raw-preview { padding: 1rem; }
.raw-preview-img { max-width: 100%; max-height: 60vh; border-radius: 6px; }
.raw-preview-pdf { width: 100%; height: 70vh; border: none; border-radius: 6px; }
.raw-preview-text { font-size: 0.875rem; }
.raw-download {
  display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
  padding: 2rem; color: hsl(var(--muted-foreground));
}
.download-link { color: hsl(var(--primary)); text-decoration: underline; }
</style>
