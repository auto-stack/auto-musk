// settings_helpers.ts — SettingsMenu 宿主 API 逃生舱（仅剩 i18n/window 部分）
//
// Plan 029 T15：forge 模式读写已迁 settings_helpers.at（单一真源）。
// 本文件仅保留 useI18n().locale 赋值与 window.open（宿主库/DOM API 边界），
// 待 Phase C（T21）评估 dom.* 后并入或永久登记（D 组）。

import { useI18n } from 'vue-i18n'

/**
 * 初始化语言：恢复 localStorage 保存的语言（对齐原 onMounted 恢复逻辑）。
 * 返回生效的 locale（en/zh）。
 */
export function settingsInitLocale(): string {
  const saved = localStorage.getItem('musk-language')
  const current = useI18n().locale.value
  if (saved && saved !== current) {
    useI18n().locale.value = saved
    return saved
  }
  return current
}

/** 切换语言：设置 vue-i18n locale + localStorage 持久化。 */
export function settingsChangeLocale(l: string): void {
  useI18n().locale.value = l
  localStorage.setItem('musk-language', l)
}

/**
 * 打开 AutoOS 设置深链（POST /api/settings-link + window.open）。
 * 返回错误消息（成功返回 ''）——.at 侧直接赋给 autoOsError model。
 */
export async function settingsOpenAutoOs(): Promise<string> {
  try {
    const resp = await fetch('/api/settings-link', { method: 'POST' })
    const data = await resp.json()
    if (data.status === 'running' && data.url) {
      window.open(data.url + '/#ai-musk', '_blank')
      return ''
    }
    return data.error || 'Service not available'
  } catch (e: any) {
    return e.message || 'Could not reach settings service'
  }
}
