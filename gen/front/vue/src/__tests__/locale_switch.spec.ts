// PLAN-063 T13b/T22 (KD 061 D29): 语言切换机制回归门禁。
// 此前 settingsChangeLocale 在 setup 外调 useI18n() 失效——locale 永不
// 翻转(实机实证 html lang 变了而组件 t() 仍中文)。根修后走生成的
// i18n-instance 模块直写 i18n.global.locale;本 spec 在 node 层钉死机制。
import { describe, it, expect, beforeEach } from 'vitest'
import { i18n } from '../i18n-instance'
import { settingsInitLocale, settingsChangeLocale } from '../ext/src/front/composables/useT'

const KEY = 'musk-language'

// node 环境的实验 localStorage 缺 clear/setItem 全集——用最小桩顶替。
const store = new Map<string, string>()
;(globalThis as any).localStorage = {
  getItem: (k: string) => store.get(k) ?? null,
  setItem: (k: string, v: string) => void store.set(k, v),
  removeItem: (k: string) => void store.delete(k),
  clear: () => void store.clear(),
}

describe('locale switch mechanism (D29 gate)', () => {
  beforeEach(() => {
    store.clear()
    // 回到默认 zh
    i18n.global.locale.value = 'zh'
  })

  it('generated i18n-instance is importable and carries both locales', () => {
    expect(i18n.global.locale.value).toBe('zh')
    expect(Object.keys(i18n.global.messages.value).sort()).toEqual(['en', 'zh'])
  })

  it('settingsChangeLocale flips the GLOBAL locale (the D29 root fix)', () => {
    settingsChangeLocale('en')
    expect(i18n.global.locale.value).toBe('en')
    expect(localStorage.getItem(KEY)).toBe('en')
    settingsChangeLocale('zh')
    expect(i18n.global.locale.value).toBe('zh')
  })

  it('settingsInitLocale restores persisted locale on the global instance', () => {
    localStorage.setItem(KEY, 'en')
    expect(i18n.global.locale.value).toBe('zh')
    const restored = settingsInitLocale()
    expect(restored).toBe('en')
    expect(i18n.global.locale.value).toBe('en')
  })

  it('t() follows the flipped locale (bindings actually translate)', () => {
    expect(i18n.global.t('nav.chat')).toContain('聊天')
    settingsChangeLocale('en')
    expect(i18n.global.t('nav.chat')).toBe('Chat')
  })
})
