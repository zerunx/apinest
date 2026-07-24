import type { App } from 'vue'
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import zhCN from './locales/zh-CN'
import enUS from './locales/en-US'

export type Locale = 'zh-CN' | 'en-US'

const messages: Record<Locale, Record<string, string>> = {
  'zh-CN': zhCN,
  'en-US': enUS,
}

const locale = ref<Locale>('zh-CN')

export function t(key: string, params?: Record<string, string | number>) {
  let text = messages[locale.value][key] ?? messages['en-US'][key] ?? key
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replace(`{${k}}`, String(v))
    }
  }
  return text
}

export function useI18n() {
  return {
    locale: computed(() => locale.value),
    t,
    async setLocale(next: Locale) {
      locale.value = next
      document.documentElement.lang = next === 'zh-CN' ? 'zh' : 'en'
      await invoke('set_setting_value', { key: 'locale', value: next })
    },
  }
}

export async function setupI18n(app: App) {
  try {
    const saved = await invoke<string | null>('get_setting_value', { key: 'locale' })
    if (saved === 'zh-CN' || saved === 'en-US') {
      locale.value = saved
    }
  } catch {
    // browser-only vite preview without tauri
  }
  document.documentElement.lang = locale.value === 'zh-CN' ? 'zh' : 'en'
  app.config.globalProperties.$t = t
}
