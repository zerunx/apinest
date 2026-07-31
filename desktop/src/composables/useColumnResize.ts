import { onBeforeUnmount, ref } from 'vue'

type WidthMap = Record<string, number>

function readStored(key: string, fallback: WidthMap) {
  try {
    const raw = localStorage.getItem(key)
    if (!raw) return { ...fallback }
    const parsed = JSON.parse(raw) as WidthMap
    const next = { ...fallback }
    for (const k of Object.keys(fallback)) {
      const n = Number(parsed[k])
      if (Number.isFinite(n)) next[k] = n
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

export function useColumnResize(
  storageKey: string,
  defaults: WidthMap,
  mins: WidthMap,
) {
  const widths = ref<WidthMap>(readStored(storageKey, defaults))
  let activeKey: string | null = null
  let startX = 0
  let startWidth = 0

  function onMove(e: MouseEvent) {
    if (!activeKey) return
    const min = mins[activeKey] ?? 48
    const delta = e.clientX - startX
    widths.value = {
      ...widths.value,
      [activeKey]: Math.max(min, startWidth + delta),
    }
  }

  function onUp() {
    if (activeKey) writeStored(storageKey, widths.value)
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
    startWidth = widths.value[column] ?? defaults[column] ?? 80
    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
  }

  onBeforeUnmount(onUp)

  return { widths, start }
}
