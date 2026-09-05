import { ref, watch } from 'vue'

export type ThemeMode = 'light' | 'dark'

const THEME_KEY = 'apinest_theme'
const theme = ref<ThemeMode>((localStorage.getItem(THEME_KEY) as ThemeMode) || 'dark')

function applyTheme(mode: ThemeMode) {
  const root = document.documentElement
  root.setAttribute('data-theme', mode)
  root.style.colorScheme = mode
  localStorage.setItem(THEME_KEY, mode)
}

applyTheme(theme.value)

watch(theme, (mode) => applyTheme(mode))

export function useTheme() {
  function toggleTheme() {
    theme.value = theme.value === 'light' ? 'dark' : 'light'
  }

  function setTheme(mode: ThemeMode) {
    theme.value = mode
  }

  return { theme, toggleTheme, setTheme }
}
