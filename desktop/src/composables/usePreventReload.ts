import { onBeforeUnmount, onMounted } from 'vue'

/** Block WebView shortcuts we handle ourselves or do not want (reload, in-page find). */
export function usePreventReload() {
  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'F5') {
      event.preventDefault()
      return
    }
    if (event.key === 'F3') {
      event.preventDefault()
      return
    }
    if (!(event.ctrlKey || event.metaKey)) return
    const key = event.key.toLowerCase()
    if (key === 'r' || key === 'f' || key === 'g') {
      event.preventDefault()
    }
  }

  onMounted(() => window.addEventListener('keydown', onKeydown, true))
  onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown, true))
}
