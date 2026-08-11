// raw_upload.ts — Wiki raw 资源上传逃生舱（对应 useWiki.ts 的 uploadRawFiles）
//
// Plan 022 遗留（raw DropZone 完整闭环）：AutoUI .at 无法表达 XHR FormData 上传
// （带进度回调），用 use { fn } 引入逃生舱。对齐原版 web/src/composables/useWiki.ts:160-195：
// multipart FormData + XMLHttpRequest + upload progress。
//
// 上传成功由调用方（WikiNav.vue emit 'uploaded' → wiki_view.at → store.LoadRawTree()）
// 刷新 raw tree，与逃生舱组件模式一致（见 forge_stream.ts）。

const RAW_BASE = '/api/forge/raw'

export interface RawUploadProgress {
  loaded: number
  total: number
  percent: number
}

/**
 * POST /api/forge/raw/{project}/upload?prefix= — multipart 上传文件。
 * @param workspace 工作区/项目名
 * @param files File[] 列表
 * @param prefix 目标子目录（可选）
 * @param onProgress 进度回调（percent 0-100）
 * @returns Promise<void>，成功 resolve，失败 reject(Error)
 */
export function uploadRawFiles(
  workspace: string,
  files: File[],
  prefix = '',
  onProgress?: (p: RawUploadProgress) => void,
): Promise<void> {
  const formData = new FormData()
  for (const file of files) {
    formData.append('files', file, file.name)
  }
  const query = prefix ? `?prefix=${encodeURIComponent(prefix)}` : ''
  const url = `${RAW_BASE}/${encodeURIComponent(workspace)}/upload${query}`

  return new Promise<void>((resolve, reject) => {
    const xhr = new XMLHttpRequest()
    xhr.upload.addEventListener('progress', (e) => {
      if (e.lengthComputable && onProgress) {
        onProgress({
          loaded: e.loaded,
          total: e.total,
          percent: Math.round((e.loaded / e.total) * 100),
        })
      }
    })
    xhr.addEventListener('load', () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        resolve()
      } else {
        reject(new Error(`Upload failed: ${xhr.status}`))
      }
    })
    xhr.addEventListener('error', () => reject(new Error('Upload failed')))
    xhr.open('POST', url)
    xhr.send(formData)
  })
}

/**
 * GET /api/forge/raw/{project}/file/{path} — raw 文件 URL（预览/下载链接）。
 * 对齐原版 useWiki.ts:156-158 rawFileUrl。
 */
export function rawFileUrl(workspace: string, path: string): string {
  return `${RAW_BASE}/${encodeURIComponent(workspace)}/file/${encodeURIComponent(path)}`
}

/**
 * GET /api/forge/raw/{project}/file/{path} — 读取文本 raw 文件内容。
 * 对齐原版 WikiView.vue:364-374 selectRawNode 对文本文件 authFetch 读内容。
 * 二进制文件请用 rawFileUrl 直接预览（img/pdf/download）。
 */
export async function loadRawFileText(workspace: string, path: string): Promise<string> {
  const resp = await fetch(rawFileUrl(workspace, path), {
    headers: { 'Content-Type': 'text/plain' },
  })
  if (!resp.ok) throw new Error(`Failed to load raw file: ${resp.status}`)
  return resp.text()
}

/**
 * 判断 raw 文件预览类型：image / pdf / text / other。
 * 对齐 RawPreview.vue 原 isImage/isPdf/isText 正则（.at 无正则能力，放逃生舱 fn，
 * component fn 经 use { fn } 引入——Plan 023 队列 A 模式）。
 */
export function rawFileKind(path: string): 'image' | 'pdf' | 'text' | 'other' {
  if (/\.(png|jpe?g|gif|svg|webp|bmp|ico)$/i.test(path)) return 'image'
  if (/\.pdf$/i.test(path)) return 'pdf'
  if (/\.(md|txt|csv|json|xml|yaml|yml|html|css|js|ts|rs|toml|sh|bat|py)$/i.test(path)) return 'text'
  return 'other'
}

/**
 * iframe 预览 HTML（.at 无 iframe 标签映射，Plan 023 队列 A1 用 v-html 兜底）。
 * fileUrl 已 encodeURIComponent，无引号注入风险。
 */
export function rawIframeHtml(fileUrl: string): string {
  return `<iframe src="${fileUrl}" class="raw-preview-pdf"></iframe>`
}

/**
 * 下载链接 HTML（<a download>）。.at 的 link 标签 → codegen 映射成 shadcn
 * router-link（不是原生 <a>），Plan 023 队列 A1 用 v-html 兜底。
 * path 是用户输入的文件名，需 HTML 转义。
 */
export function rawDownloadHtml(fileUrl: string, path: string): string {
  const esc = (s: string) =>
    s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
  return `<a href="${esc(fileUrl)}" download class="download-link">${esc(path)}</a>`
}
