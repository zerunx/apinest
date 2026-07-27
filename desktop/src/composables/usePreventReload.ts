import { onBeforeUnmount, onMounted } from 'vue'

/** Block browser/webview reload shortcuts (Ctrl/Cmd+R, F5). */
export function usePreventReload() {
  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'F5') {
      event.preventDefault()
      return
    }
    if (!(event.ctrlKey || event.metaKey)) return
    if (event.key.toLowerCase() === 'r') {
      event.preventDefault()
    }
  }

  onMounted(() => window.addEventListener('keydown', onKeydown))
  onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))
}
