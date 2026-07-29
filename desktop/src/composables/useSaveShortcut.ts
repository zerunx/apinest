import { onActivated, onBeforeUnmount, onDeactivated, onMounted, ref } from 'vue'

/** Bind Ctrl/Cmd+S to save; prevents browser "Save Page". */
export function useSaveShortcut(
  save: () => void | boolean | Promise<void | boolean>,
  enabled?: () => boolean,
) {
  const listening = ref(false)

  function onKeydown(event: KeyboardEvent) {
    if (!(event.ctrlKey || event.metaKey)) return
    if (event.key.toLowerCase() !== 's') return
    event.preventDefault()
    if (enabled && !enabled()) return
    void save()
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
