import { onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

function hasOpenDialog(): boolean {
  // Modal overlays (settings, name/confirm dialogs, env/dict manage, login, import…)
  if (document.querySelector('body > .overlay')) return true
  // CodeMirror find panel — Esc closes find first
  if (document.querySelector('.cm-find-widget')) return true
  return false
}

/** Esc minimizes the window when no modal dialog / find panel is open. */
export function useEscapeMinimize() {
  function onKeydown(event: KeyboardEvent) {
    if (event.key !== 'Escape') return
    if (event.defaultPrevented) return
    if (hasOpenDialog()) return
    event.preventDefault()
    try {
      void getCurrentWindow().minimize()
    } catch {
      // browser preview without Tauri
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', onKeydown)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', onKeydown)
  })
}
