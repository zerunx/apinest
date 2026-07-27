import { onBeforeUnmount, ref, type Ref } from 'vue'

type Axis = 'x' | 'y'

type DragResizeOptions = {
  axis: Axis
  min: number
  max: number
  initial: number
  storageKey?: string
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

export function useDragResize(options: DragResizeOptions) {
  const size = ref(readStored(options.storageKey, options.initial))
  const dragging = ref(false)
  let startPos = 0
  let startSize = 0
  let maxSize = options.max

  function clamp(value: number) {
    return Math.min(maxSize, Math.max(options.min, value))
  }

  function onMove(e: MouseEvent) {
    const delta = options.axis === 'x' ? e.clientX - startPos : e.clientY - startPos
    const next = options.axis === 'x' ? startSize + delta : startSize - delta
    size.value = clamp(next)
  }

  function onUp() {
    dragging.value = false
    writeStored(options.storageKey, size.value)
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

  return { size, dragging, startDrag }
}

export function useContainerBoundedResize(
  containerRef: Ref<HTMLElement | null>,
  options: Omit<DragResizeOptions, 'max'> & { maxRatio?: number; min: number },
) {
  const { size, dragging, startDrag } = useDragResize({
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
    writeStored(options.storageKey, size.value)
  }

  function startBoundedDrag(e: MouseEvent) {
    clampToContainer()
    startDrag(e, getMax())
  }

  return { size, dragging, startDrag: startBoundedDrag, clampToContainer }
}
