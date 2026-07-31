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
/** Bumps on every locale change so UI that captured strings can refresh. */
const localeVersion = ref(0)

function translate(key: string, params?: Record<string, string | number>) {
  // Explicit dependency for Vue render tracking
  const loc = locale.value
  void localeVersion.value
  let text = messages[loc][key] ?? messages['en-US'][key] ?? key
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replace(`{${k}}`, String(v))
    }
  }
  return text
}

/** @deprecated Prefer useI18n().t inside components for reactivity. */
export const t = translate

export function useI18n() {
  return {
    locale: computed(() => locale.value),
    localeVersion: computed(() => localeVersion.value),
    t: translate,
    async setLocale(next: Locale) {
      if (locale.value !== next) {
        locale.value = next
        localeVersion.value += 1
      }
      document.documentElement.lang = next === 'zh-CN' ? 'zh' : 'en'
      try {
        await invoke('set_setting_value', { key: 'locale', value: next })
      } catch {
        // browser preview without tauri
      }
    },
  }
}

export async function setupI18n(app: App) {
  try {
    const saved = await invoke<string | null>('get_setting_value', { key: 'locale' })
    if (saved === 'zh-CN' || saved === 'en-US') {
      locale.value = saved
      localeVersion.value += 1
    }
  } catch {
    // browser-only vite preview without tauri
  }
  document.documentElement.lang = locale.value === 'zh-CN' ? 'zh' : 'en'
  app.config.globalProperties.$t = translate
}
