import {
  nextTick,
  onActivated,
  onBeforeUnmount,
  onDeactivated,
  onMounted,
  ref,
  watch,
  type WatchSource,
} from 'vue'

/**
 * Tracks editor dirty state against a captured baseline.
 * Call `captureBaseline()` after load / save / initial settle.
 * Dirty updates are ignored while the host component is deactivated (keep-alive).
 */
export function useInterfaceEditorDirty(options: {
  getSnapshot: () => string
  setDirty: (dirty: boolean) => void
  enabled?: () => boolean
  watchSource: WatchSource | object
}) {
  const suppress = ref(true)
  const viewActive = ref(true)
  let baseline = ''

  function syncDirty() {
    if (!viewActive.value) return
    if (suppress.value) return
    if (options.enabled && !options.enabled()) return
    options.setDirty(options.getSnapshot() !== baseline)
  }

  function beginSuppress() {
    suppress.value = true
  }

  async function captureBaseline() {
    suppress.value = true
    await nextTick()
    await nextTick()
    baseline = options.getSnapshot()
    suppress.value = false
    if (viewActive.value) {
      options.setDirty(false)
    }
  }

  watch(options.watchSource, syncDirty, { deep: true })

  onMounted(() => {
    viewActive.value = true
  })
  onActivated(() => {
    viewActive.value = true
    syncDirty()
  })
  onDeactivated(() => {
    viewActive.value = false
  })
  onBeforeUnmount(() => {
    viewActive.value = false
  })

  return {
    beginSuppress,
    captureBaseline,
    syncDirty,
  }
}

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
