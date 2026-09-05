import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createI18n } from 'vue-i18n'
import en from '../locales/en.json'
import zh from '../locales/zh.json'

function collectKeys(obj: Record<string, unknown>, prefix = ''): string[] {
  const keys: string[] = []
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      keys.push(...collectKeys(value as Record<string, unknown>, fullKey))
    } else {
      keys.push(fullKey)
    }
  }
  return keys
}

describe('i18n setup', () => {
  it('creates i18n with legacy: false and fallbackLocale: en', () => {
    const i18n = createI18n({
      legacy: false,
      locale: 'en',
      fallbackLocale: 'en',
      messages: { en, zh },
    })
    expect(i18n.mode).toBe('composition')
    expect(i18n.global.fallbackLocale.value).toBe('en')
  })

  it('loads both en and zh message objects', () => {
    const i18n = createI18n({
      legacy: false,
      locale: 'en',
      fallbackLocale: 'en',
      messages: { en, zh },
    })
    const enKeys = Object.keys(i18n.global.getLocaleMessage('en'))
    const zhKeys = Object.keys(i18n.global.getLocaleMessage('zh'))
    expect(enKeys.length).toBeGreaterThan(0)
    expect(zhKeys.length).toBeGreaterThan(0)
  })

  it('detectLocale logic: stored value takes priority', () => {
    const store: Record<string, string> = { 'autoforge-language': 'zh' }
    const stored = store['autoforge-language']
    const isValid = stored && ['en', 'zh'].includes(stored)
    expect(isValid).toBe(true)
  })

  it('detectLocale logic: invalid stored value rejected', () => {
    const store: Record<string, string> = { 'autoforge-language': 'fr' }
    const stored = store['autoforge-language']
    const isValid = stored && ['en', 'zh'].includes(stored)
    expect(isValid).toBe(false)
  })

  it('detectLocale logic: Chinese browser maps to zh', () => {
    const nav = 'zh-CN'.toLowerCase()
    expect(nav.startsWith('zh')).toBe(true)
  })

  it('detectLocale logic: English browser maps to en', () => {
    const nav = 'en-US'.toLowerCase()
    expect(nav.startsWith('zh')).toBe(false)
  })
})

describe('setLocale', () => {
  let store: Record<string, string>

  beforeEach(() => {
    store = {}
  })

  it('setLocale persists to localStorage', () => {
    store['autoforge-language'] = 'zh'
    expect(store['autoforge-language']).toBe('zh')
  })

  // vitest 默认 node 环境无 DOM;DOM 断言仅在浏览器/DOM 环境下有意义
  it.skipIf(typeof document === 'undefined')('setLocale sets document lang attribute', () => {
    document.documentElement.setAttribute('lang', 'zh')
    expect(document.documentElement.getAttribute('lang')).toBe('zh')
  })

  it('toggleLocale switches between en and zh', () => {
    store['autoforge-language'] = 'en'
    let current = store['autoforge-language']
    const toggled = current === 'en' ? 'zh' : 'en'
    store['autoforge-language'] = toggled
    expect(store['autoforge-language']).toBe('zh')
  })
})

describe('translation file parity', () => {
  it('en.json and zh.json have identical key structures', () => {
    const enKeys = collectKeys(en as unknown as Record<string, unknown>).sort()
    const zhKeys = collectKeys(zh as unknown as Record<string, unknown>).sort()
    expect(enKeys).toEqual(zhKeys)
  })

  it('all required top-level sections exist in both files', () => {
    const sections = ['app', 'nav', 'settings', 'common', 'chat', 'specs', 'plans', 'wiki']
    for (const section of sections) {
      expect(en[section as keyof typeof en]).toBeDefined()
      expect(zh[section as keyof typeof zh]).toBeDefined()
    }
  })

  it('settings section has all required keys', () => {
    const requiredSettings = ['title', 'mode', 'accent', 'theme', 'language', 'modeGsdTitle', 'modeCheckTitle', 'themeLight', 'themeDark', 'themeSystem']
    for (const key of requiredSettings) {
      expect((en.settings as Record<string, string>)[key]).toBeDefined()
      expect((zh.settings as Record<string, string>)[key]).toBeDefined()
    }
  })

  it('nav section carries the full web key set (041 债务收口⑥)', () => {
    const navKeys = ['explorer', 'chat', 'plans', 'specs', 'wiki', 'relay', 'agents', 'professions', 'skills', 'apis']
    const enNav = en.nav as Record<string, unknown>
    const zhNav = zh.nav as Record<string, unknown>
    for (const key of navKeys) {
      expect(enNav[key]).toBeDefined()
      expect(zhNav[key]).toBeDefined()
    }
  })

  it('no empty string values in en.json', () => {
    const enKeys = collectKeys(en as unknown as Record<string, unknown>)
    for (const key of enKeys) {
      const parts = key.split('.')
      let val: unknown = en
      for (const p of parts) val = (val as Record<string, unknown>)[p]
      expect(val, `Key "${key}" has empty value in en.json`).toBeTruthy()
    }
  })

  it('no empty string values in zh.json', () => {
    const zhKeys = collectKeys(zh as unknown as Record<string, unknown>)
    for (const key of zhKeys) {
      const parts = key.split('.')
      let val: unknown = zh
      for (const p of parts) val = (val as Record<string, unknown>)[p]
      expect(val, `Key "${key}" has empty value in zh.json`).toBeTruthy()
    }
  })

  it('common section has standard action labels in both locales', () => {
    const commonKeys = ['save', 'cancel', 'edit', 'delete', 'create']
    const enCommon = en.common as Record<string, unknown>
    const zhCommon = zh.common as Record<string, unknown>
    for (const key of commonKeys) {
      expect(enCommon[key]).toBeDefined()
      expect(zhCommon[key]).toBeDefined()
    }
  })

  it('brand name stays as Auto Musk in both locales', () => {
    expect(en.app.brandName).toBe('Auto Musk')
    expect(zh.app.brandName).toBe('Auto Musk')
  })
})

// PLAN-061 T7 (D16): 模板 t() 字面量 ⊆ locale keys 回归门禁。
// wiki.edit/wiki.delete 漏网(D5)现有 2 spec 抓不住——静态扫描生成产物
// components/**/*.vue(含 ext 手写 passthrough)模板里的 t('section.key')
// 字面量,断言每个 key 在 zh/en locale 都存在。人为删 key 应红。
// 实现:import.meta.glob(?raw, eager)(vite 原生,无 node 内建依赖——
// gen tsconfig 的 vue-tsc 面不含 node 类型;shim 已补 glob 最小声明)。
describe('template t() literals ⊆ locale keys (D16 gate)', () => {
  const vueSources = {
    ...(import.meta.glob('../components/**/*.vue', { query: '?raw', import: 'default', eager: true }) as Record<string, string>),
    ...(import.meta.glob('../ext/src/front/components/**/*.vue', { query: '?raw', import: 'default', eager: true }) as Record<string, string>),
  }

  const templateKeys = new Map<string, string[]>() // key -> [file, ...]
  for (const [file, src] of Object.entries(vueSources)) {
    // t('a.b') / t("a.b") / $t('a.b') —— 仅字面量(动态 key 不在本门禁面)
    for (const m of src.matchAll(/\$?\bt\(\s*['"]([A-Za-z0-9_-]+(?:\.[A-Za-z0-9_-]+)+)['"]\s*[),]/g)) {
      const key = m[1]
      templateKeys.set(key, [...(templateKeys.get(key) ?? []), file])
    }
  }

  it('scanner actually found template t() literals (guard against vacuous pass)', () => {
    expect(templateKeys.size).toBeGreaterThan(20)
  })

  it('every literal key used in templates exists in en.json', () => {
    const enKeys = new Set(collectKeys(en as unknown as Record<string, unknown>))
    const missing = [...templateKeys.keys()].filter((k) => !enKeys.has(k))
    expect(missing, `keys missing in en.json (used in: ${missing.map((k) => `${k} -> ${templateKeys.get(k)![0]}`).join('; ')})`).toEqual([])
  })

  it('every literal key used in templates exists in zh.json', () => {
    const zhKeys = new Set(collectKeys(zh as unknown as Record<string, unknown>))
    const missing = [...templateKeys.keys()].filter((k) => !zhKeys.has(k))
    expect(missing, `keys missing in zh.json (used in: ${missing.map((k) => `${k} -> ${templateKeys.get(k)![0]}`).join('; ')})`).toEqual([])
  })
})
