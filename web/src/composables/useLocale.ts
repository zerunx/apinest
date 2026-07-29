import { computed, ref } from 'vue'
import zhCN from '@/locales/zh-CN'
import enUS from '@/locales/en-US'
import zhCnEp from 'element-plus/es/locale/lang/zh-cn'
import enEp from 'element-plus/es/locale/lang/en'

export type LocaleCode = 'zh-CN' | 'en-US'

const LOCALE_KEY = 'apinest_locale'

const locale = ref<LocaleCode>(
  (localStorage.getItem(LOCALE_KEY) as LocaleCode) || 'zh-CN',
)

document.documentElement.lang = locale.value === 'zh-CN' ? 'zh-CN' : 'en'

const messageMap = {
  'zh-CN': zhCN,
  'en-US': enUS,
} as const

const elementLocaleMap = {
  'zh-CN': zhCnEp,
  'en-US': enEp,
} as const

function resolveMessage(obj: Record<string, unknown>, path: string): string {
  const value = path.split('.').reduce<unknown>((current, key) => {
    if (current && typeof current === 'object') {
      return (current as Record<string, unknown>)[key]
    }
    return undefined
  }, obj)

  return typeof value === 'string' ? value : path
}

export function useLocale() {
  const elementLocale = computed(() => elementLocaleMap[locale.value])

  function t(key: string, params?: Record<string, string>) {
    let text = resolveMessage(messageMap[locale.value] as Record<string, unknown>, key)

    if (params) {
      Object.entries(params).forEach(([name, value]) => {
        text = text.replace(`{${name}}`, value)
      })
    }

    return text
  }

  function setLocale(code: LocaleCode) {
    locale.value = code
    localStorage.setItem(LOCALE_KEY, code)
    document.documentElement.lang = code === 'zh-CN' ? 'zh-CN' : 'en'
  }

  return {
    locale,
    elementLocale,
    t,
    setLocale,
  }
}
