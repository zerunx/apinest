import { onBeforeUnmount, ref, type Ref } from 'vue'

type Axis = 'x' | 'y'

type StorageKey = string | (() => string | undefined) | undefined

type DragResizeOptions = {
  axis: Axis
  min: number
  max: number
  initial: number
  storageKey?: StorageKey
  /** Used when the dynamic key has no value yet (e.g. migrate from a global key). */
  fallbackStorageKey?: string
  /** When true, dragging positive direction shrinks `size` (e.g. right-side panel). */
  invert?: boolean
}

function resolveStorageKey(key: StorageKey) {
  return typeof key === 'function' ? key() : key
}

function readStored(key: string | undefined, fallback: number) {
  if (!key) return fallback
  try {
    const raw = localStorage.getItem(key)
    const n = raw != null ? Number(raw) : NaN
    return Number.isFinite(n) ? n : fallback
  } catch {
    return fallback
  }
}

function writeStored(key: string | undefined, value: number) {
  if (!key) return
  try {
    localStorage.setItem(key, String(Math.round(value)))
  } catch {
    // ignore quota / private mode
  }
}

function readStoredWithFallback(
  key: StorageKey,
  fallbackStorageKey: string | undefined,
  initial: number,
) {
  const primary = resolveStorageKey(key)
  if (primary) {
    try {
      const raw = localStorage.getItem(primary)
      if (raw != null) {
        const n = Number(raw)
        if (Number.isFinite(n)) return n
      }
    } catch {
      /* ignore */
    }
  }
  return readStored(fallbackStorageKey, initial)
}

export function useDragResize(options: DragResizeOptions) {
  const size = ref(
    readStoredWithFallback(options.storageKey, options.fallbackStorageKey, options.initial),
  )
  const dragging = ref(false)
  let startPos = 0
  let startSize = 0
  let maxSize = options.max

  function clamp(value: number) {
    return Math.min(maxSize, Math.max(options.min, value))
  }

  function currentKey() {
    return resolveStorageKey(options.storageKey)
  }

  function reloadFromStorage() {
    size.value = readStoredWithFallback(
      options.storageKey,
      options.fallbackStorageKey,
      options.initial,
    )
  }

  function onMove(e: MouseEvent) {
    const delta = options.axis === 'x' ? e.clientX - startPos : e.clientY - startPos
    const signed = options.invert ? -delta : delta
    const next = options.axis === 'x' ? startSize + signed : startSize - delta
    size.value = clamp(next)
  }

  function onUp() {
    dragging.value = false
    writeStored(currentKey(), size.value)
    window.removeEventListener('mousemove', onMove)
    window.removeEventListener('mouseup', onUp)
    document.body.style.removeProperty('cursor')
    document.body.style.removeProperty('user-select')
  }

  function startDrag(e: MouseEvent, dynamicMax?: number) {
    e.preventDefault()
    dragging.value = true
    maxSize = dynamicMax ?? options.max
    startPos = options.axis === 'x' ? e.clientX : e.clientY
    startSize = size.value
    document.body.style.cursor = options.axis === 'x' ? 'col-resize' : 'row-resize'
    document.body.style.userSelect = 'none'
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
  }

  onBeforeUnmount(onUp)

  return { size, dragging, startDrag, reloadFromStorage }
}

export function useContainerBoundedResize(
  containerRef: Ref<HTMLElement | null>,
  options: Omit<DragResizeOptions, 'max'> & { maxRatio?: number; min: number },
) {
  const { size, dragging, startDrag, reloadFromStorage } = useDragResize({
    ...options,
    max: options.initial * 4,
  })

  function getMax() {
    const el = containerRef.value
    if (!el) return options.initial * 4
    const bound =
      options.axis === 'x' ? el.getBoundingClientRect().width : el.getBoundingClientRect().height
    return Math.round(bound * (options.maxRatio ?? 0.75))
  }

  function clampToContainer() {
    size.value = Math.min(getMax(), Math.max(options.min, size.value))
    writeStored(resolveStorageKey(options.storageKey), size.value)
  }

  function startBoundedDrag(e: MouseEvent) {
    clampToContainer()
    startDrag(e, getMax())
  }

  function reloadAndClamp() {
    reloadFromStorage()
    clampToContainer()
  }

  return {
    size,
    dragging,
    startDrag: startBoundedDrag,
    clampToContainer,
    reloadFromStorage: reloadAndClamp,
  }
}
