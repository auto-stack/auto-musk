// settings_helpers.ts — SettingsMenu 数据/宿主 API 逻辑逃生舱（Plan 023 队列 A4）
//
// 对标 src/front/components/SettingsMenu.vue（逃生舱，已删除）。
// .at 无法表达：useI18n().locale.value 赋值（composable ref 写）、fetch 链式
// （fetch → json → 字段分支）、window.open、forge_mode API 调用。放逃生舱 fn。
//
// forge_mode_get/set 来自生成的 @/lib/api（src/back/api.at codegen）。

import { useI18n } from 'vue-i18n'
import { forge_mode_get, forge_mode_set } from '@/lib/api'

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
 * 加载后端持久化的 forge 模式（gsd/check，对齐原 loadForgeMode）。
 * 后端不可达或响应非法时回退默认 'gsd'（不抛异常）。
 */
export async function settingsLoadForgeMode(): Promise<string> {
  try {
    const resp = await forge_mode_get()
    if (resp && (resp.mode === 'gsd' || resp.mode === 'check')) {
      return resp.mode
    }
  } catch {
    // 后端不可达时保留默认 gsd
  }
  return 'gsd'
}

/** 持久化 forge 模式（写失败静默，保持本地显示——对齐原 setForgeMode）。 */
export async function settingsSetForgeMode(val: string): Promise<void> {
  try {
    await forge_mode_set(val)
  } catch {
    // 写失败静默回退（保持本地显示，下次打开重新读取后端值）
  }
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
