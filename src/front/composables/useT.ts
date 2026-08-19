// useT.ts — vue-i18n 宿主库桥（D 组永久保留）
//
// Plan 407：t 是 useI18n() 解构返回值，非静态导出——本桥包装一层供
// .at 的 `composable: useT` / `fn: t` 声明消费。
// Plan 029 T21：settings_helpers.ts 的语言切换并入（useI18n().locale.value
// 赋值是宿主库 ref 写，.at 无法表达——待澄清#2 的既定结论）。

import { useI18n } from 'vue-i18n'

/** 返回 i18n 的 t 翻译函数。在组件 setup 顶层调用。 */
export function useT() {
  const { t } = useI18n()
  return t
}

/** 初始化语言：恢复 localStorage 保存的语言，返回生效 locale（en/zh）。 */
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
