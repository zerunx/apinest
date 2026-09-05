import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export type ThemeMode = 'light' | 'dark'

const theme = ref<ThemeMode>('light')
let initialized = false

function applyTheme(mode: ThemeMode) {
  const root = document.documentElement
  root.setAttribute('data-theme', mode)
  root.style.colorScheme = mode
}

export async function initTheme() {
  if (initialized) return
  initialized = true
  try {
    const saved = await invoke<string | null>('get_setting_value', { key: 'theme' })
    if (saved === 'light' || saved === 'dark') {
      theme.value = saved
    }
  } catch {
    // ignore when not in tauri
  }
  applyTheme(theme.value)
}

watch(theme, async (mode) => {
  applyTheme(mode)
  try {
    await invoke('set_setting_value', { key: 'theme', value: mode })
  } catch {
    // ignore
  }
})

export function useTheme() {
  function setTheme(mode: ThemeMode) {
    theme.value = mode
  }

  function toggleTheme() {
    theme.value = theme.value === 'light' ? 'dark' : 'light'
  }

  return { theme, setTheme, toggleTheme }
}
