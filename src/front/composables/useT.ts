// useT.ts — vue-i18n 宿主库桥（D 组永久保留）
//
// Plan 407：t 是 useI18n() 解构返回值，非静态导出——本桥包装一层供
// .at 的 `composable: useT` / `fn: t` 声明消费。
// Plan 029 T21：settings_helpers.ts 的语言切换并入。
// PLAN-063 T13b (KD 061 D29): 切换/恢复改走生成的 i18n-instance 模块
// 直写 i18n.global——此前 useI18n() 在 setup 外调用失效(vue-i18n 组合式
// API 依赖注入上下文),locale 永不翻转;存储键统一 musk-language。

import { useI18n } from 'vue-i18n'
import { i18n } from '@/i18n-instance'

const LOCALE_KEY = 'musk-language'

/** 返回 i18n 的 t 翻译函数。在组件 setup 顶层调用。 */
export function useT() {
  const { t } = useI18n()
  return t
}

/** 初始化语言：恢复 localStorage 保存的语言，返回生效 locale（en/zh）。 */
export function settingsInitLocale(): string {
  const saved = localStorage.getItem(LOCALE_KEY)
  const current = i18n.global.locale.value
  if (saved && saved !== current) {
    i18n.global.locale.value = saved
    return saved
  }
  return current
}

/** 切换语言：直写全局实例 locale + localStorage 持久化。 */
export function settingsChangeLocale(l: string): void {
  i18n.global.locale.value = l
  localStorage.setItem(LOCALE_KEY, l)
}
