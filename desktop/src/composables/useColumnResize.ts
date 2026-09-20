import { onBeforeUnmount, ref, type Ref } from 'vue'

type WidthMap = Record<string, number>

export type ColumnFitLayout = 'params' | 'form-data'

function clampWidth(n: number, min: number, max?: number) {
  const capped = Math.max(min, n)
  return max == null ? capped : Math.min(max, capped)
}

function readStored(key: string, fallback: WidthMap, mins: WidthMap, maxs?: WidthMap) {
  try {
    const raw = localStorage.getItem(key)
    if (!raw) return { ...fallback }
    const parsed = JSON.parse(raw) as WidthMap
    const next = { ...fallback }
    for (const k of Object.keys(fallback)) {
      const n = Number(parsed[k])
      if (Number.isFinite(n)) {
        next[k] = clampWidth(n, mins[k] ?? 48, maxs?.[k])
      }
    }
    return next
  } catch {
    return { ...fallback }
  }
}

function writeStored(key: string, value: WidthMap) {
  try {
    localStorage.setItem(key, JSON.stringify(value))
  } catch {
    // ignore
  }
}

/**
 * Fit preferred column widths into `availablePx`.
 * Shrink order: description → value → name (key). Extra space goes to description.
 */
export function fitColumnWidths(
  preferred: WidthMap,
  availablePx: number,
  layout: ColumnFitLayout,
  mins: WidthMap,
): WidthMap {
  if (!(availablePx > 0)) return { ...preferred }

  const keyMin = mins.key ?? 48
  const valueMin = mins.value ?? 48
  // Description may fully collapse before value/name are touched.
  const descMin = 0
  const type = preferred.type ?? 90

  let key = preferred.key ?? keyMin
  let value = preferred.value ?? valueMin
  let desc = preferred.desc ?? 0

  const chrome = layout === 'form-data'
    ? 40 + type + 72 // checkbox + type + actions
    : 40 + 40 // checkbox + delete

  let total = chrome + key + value + desc

  if (total < availablePx) {
    desc += availablePx - total
  } else if (total > availablePx) {
    let overflow = total - availablePx

    const descTake = Math.min(overflow, Math.max(0, desc - descMin))
    desc -= descTake
    overflow -= descTake

    const valueTake = Math.min(overflow, Math.max(0, value - valueMin))
    value -= valueTake
    overflow -= valueTake

    const keyTake = Math.min(overflow, Math.max(0, key - keyMin))
    key -= keyTake
  }

  return {
    ...preferred,
    key,
    value,
    desc,
    ...(layout === 'form-data' ? { type } : {}),
  }
}

export function useColumnResize(
  storageKey: string,
  defaults: WidthMap,
  mins: WidthMap,
  maxs?: WidthMap,
) {
  const preferred = ref<WidthMap>(readStored(storageKey, defaults, mins, maxs))
  const widths = ref<WidthMap>({ ...preferred.value })
  let activeKey: string | null = null
  let startX = 0
  let startWidth = 0
  let lastAvailable = 0
  let lastLayout: ColumnFitLayout = 'params'

  function applyFit(availablePx = lastAvailable, layout = lastLayout) {
    if (availablePx > 0) {
      lastAvailable = availablePx
      lastLayout = layout
    }
    if (!(lastAvailable > 0)) {
      widths.value = { ...preferred.value }
      return
    }
    widths.value = fitColumnWidths(preferred.value, lastAvailable, lastLayout, mins)
  }

  function fitToWidth(availablePx: number, layout: ColumnFitLayout) {
    applyFit(availablePx, layout)
  }

  function onMove(e: MouseEvent) {
    if (!activeKey) return
    const min = mins[activeKey] ?? 48
    const max = maxs?.[activeKey]
    const delta = e.clientX - startX
    const next = clampWidth(startWidth + delta, min, max)
    // Drag updates the preferred width; display is re-fitted so the row still fits.
    preferred.value = {
      ...preferred.value,
      [activeKey]: next,
    }
    applyFit()
  }

  function onUp() {
    if (activeKey) writeStored(storageKey, preferred.value)
    activeKey = null
    document.body.style.removeProperty('cursor')
    document.body.style.removeProperty('user-select')
    window.removeEventListener('mousemove', onMove)
    window.removeEventListener('mouseup', onUp)
  }

  function start(column: string, e: MouseEvent) {
    e.preventDefault()
    activeKey = column
    startX = e.clientX
    // Use the visible width so dragging feels continuous even when columns are auto-fitted.
    startWidth = widths.value[column] ?? preferred.value[column] ?? defaults[column] ?? 80
    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
  }

  onBeforeUnmount(onUp)

  return {
    widths: widths as Ref<WidthMap>,
    preferred: preferred as Ref<WidthMap>,
    start,
    fitToWidth,
  }
}
