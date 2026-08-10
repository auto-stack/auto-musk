// useT.ts — vue-i18n 的 t 函数 composable 包装（Plan 407）
//
// 背景：vue-i18n 的 t 是 useI18n() 的解构返回值，不是直接导出的函数。
// AutoUI 的 composable 机制生成 `const t = useT()`（非解构），
// 所以这里包装一层：useT() 内部调 useI18n() 并返回 t 函数，
// 让 .at 里 `fn: t from "src/front/composables/useT.ts"` 生成的
// `import { t } from '...'` + 在模板里 {{ t('key') }} 能正确工作。
//
// 注意：t 必须在 setup 上下文调用（useI18n 要求），所以 useT 也只能在
// 组件 setup 顶层调用——这和 composable 约定一致。

import { useI18n } from 'vue-i18n'

/** 返回 i18n 的 t 翻译函数。在组件 setup 顶层调用。 */
export function useT() {
  const { t } = useI18n()
  return t
}

// 也默认导出 t 的工厂，供 fn import 模式使用。
// AutoUI `fn: t from "..."` 生成 `import { t } from '...'`，
// 但 t 是 useI18n 的返回值（运行时绑定），无法静态导出。
// 所以这里导出一个占位——实际使用需走 composable 声明。
export default useT
