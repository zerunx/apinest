import { onActivated, onBeforeUnmount, onDeactivated, onMounted, ref } from 'vue'

/** Bind Ctrl/Cmd+W to close the active tab; prevents browser tab/window close. */
export function useCloseTabShortcut(
  closeTab: () => void,
  enabled?: () => boolean,
) {
  const listening = ref(false)

  function onKeydown(event: KeyboardEvent) {
    if (!(event.ctrlKey || event.metaKey)) return
    if (event.key.toLowerCase() !== 'w') return
    event.preventDefault()
    if (enabled && !enabled()) return
    closeTab()
  }

  function startListening() {
    if (listening.value) return
    window.addEventListener('keydown', onKeydown)
    listening.value = true
  }

  function stopListening() {
    if (!listening.value) return
    window.removeEventListener('keydown', onKeydown)
    listening.value = false
  }

  onMounted(startListening)
  onActivated(startListening)
  onDeactivated(stopListening)
  onBeforeUnmount(stopListening)
}
