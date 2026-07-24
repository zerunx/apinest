<template>
  <div class="interface-body-raw">
    <div class="interface-body-raw__editor">
      <div ref="gutterRef" class="interface-body-raw__gutter" aria-hidden="true">
        <div
          v-for="line in lineNumbers"
          :key="line"
          class="interface-body-raw__line-no"
          :class="{ 'interface-body-raw__line-no--error': line === errorLine }"
        >
          {{ line }}
        </div>
      </div>
      <div class="interface-body-raw__code-wrap interface-body-raw__code-wrap--json">
        <div class="interface-body-raw__selection-layer" aria-hidden="true">
          <svg v-if="selectionPathD" class="interface-body-raw__selection-svg">
            <path :d="selectionPathD" class="interface-body-raw__selection-path" />
          </svg>
        </div>
        <pre ref="highlightRef" class="interface-body-raw__highlight" aria-hidden="true"><code v-html="highlightedContent" /></pre>
        <textarea
          ref="textareaRef"
          v-model="localRaw"
          class="interface-body-raw__textarea"
          :readonly="readonly"
          :placeholder="placeholder"
          spellcheck="false"
          @scroll="onScroll"
          @keydown="onKeydown"
          @keyup="scheduleSelectionUpdate"
          @select="scheduleSelectionUpdate"
          @mousedown="onMouseDown"
          @blur="onBlur"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { diagnoseRawJson, highlightRawContent } from '../utils/raw-editor-highlight'
import { applyJsonEnter, formatJsonLenient } from '../utils/raw-json-utils'

type LineRect = {
  top: number
  left: number
  width: number
  height: number
}

const props = withDefaults(
  defineProps<{
    modelValue: string
    readonly?: boolean
    placeholder?: string
  }>(),
  {
    readonly: false,
    placeholder: '',
  },
)

const emit = defineEmits<{
  'update:modelValue': [string]
}>()

const gutterRef = ref<HTMLElement | null>(null)
const highlightRef = ref<HTMLElement | null>(null)
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const localRaw = ref(props.modelValue)
const selectionPathD = ref('')
let syncingFromParent = false
let mirrorEl: HTMLDivElement | null = null
let mirrorWidth = 0
let lineHeightPx = 20
let rafId = 0
let pointerDown = false
let pointerDragged = false
let selectAnchor = 0
let dragSelStart = 0
let dragSelEnd = 0
let downClientX = 0
let downClientY = 0
let pendingMoveX = 0
let pendingMoveY = 0
let moveRafId = 0
let lastSelStart = -1
let lastSelEnd = -1
let lastDotStart = -1
let lastDotEnd = -1
let cachedLineRects: LineRect[] = []
let mirrorText = ''
let spaceWidthPx = 8
let selectionEndsWithNewline = false
let cachedLineStarts: number[] = [0]
let cachedLineStartsFor = ''
const SEL_RADIUS = 4
const DRAG_THRESHOLD_PX = 3

watch(
  () => props.modelValue,
  (value) => {
    if (value === localRaw.value) return
    syncingFromParent = true
    localRaw.value = value
    syncingFromParent = false
  },
)

watch(localRaw, (value) => {
  if (syncingFromParent) return
  emit('update:modelValue', value)
  void nextTick(() => {
    lastSelStart = -1
    lastSelEnd = -1
    cachedContentRects = []
    mirrorText = ''
    scheduleSelectionUpdate()
  })
})

const jsonError = computed(() => diagnoseRawJson(localRaw.value))
const errorLine = computed(() => jsonError.value?.line ?? null)

const lineNumbers = computed(() => {
  const count = Math.max(1, localRaw.value.split('\n').length)
  return Array.from({ length: count }, (_, index) => index + 1)
})

const highlightedContent = computed(() =>
  highlightRawContent(localRaw.value, 'JSON', 'raw', jsonError.value?.index ?? null),
)

watch(highlightedContent, () => {
  lastDotStart = -1
  lastDotEnd = -1
  void nextTick(() => {
    const el = textareaRef.value
    if (!el || el.selectionStart === el.selectionEnd) return
    syncSpaceDots(el.selectionStart, el.selectionEnd)
  })
})

function syncScroll(scrollTop: number) {
  if (gutterRef.value) gutterRef.value.scrollTop = scrollTop
  if (highlightRef.value) highlightRef.value.scrollTop = scrollTop
}

function onScroll(event: Event) {
  const target = event.target as HTMLTextAreaElement
  syncScroll(target.scrollTop)
  paintSelectionPath(target)
}

function onBlur() {
  if (pointerDown) {
    textareaRef.value?.focus({ preventScroll: true })
    return
  }
  clearSelectionUi()
}

function clearSelectionUi() {
  lastSelStart = -1
  lastSelEnd = -1
  lastDotStart = -1
  lastDotEnd = -1
  cachedLineRects = []
  selectionPathD.value = ''
  selectionEndsWithNewline = false
  syncSpaceDots(0, 0)
}

function detachDocListeners() {
  document.removeEventListener('mousemove', onDocMouseMove, true)
  document.removeEventListener('mouseup', onDocMouseUp, true)
}

function setDocUserSelect(locked: boolean) {
  if (locked) {
    document.documentElement.style.userSelect = 'none'
    document.documentElement.style.setProperty('-webkit-user-select', 'none')
  } else {
    document.documentElement.style.userSelect = ''
    document.documentElement.style.removeProperty('-webkit-user-select')
  }
}

function ensureLineStarts(value: string): number[] {
  if (value === cachedLineStartsFor) return cachedLineStarts
  cachedLineStartsFor = value
  const starts = [0]
  for (let i = 0; i < value.length; i += 1) {
    if (value[i] === '\n') starts.push(i + 1)
  }
  cachedLineStarts = starts
  return starts
}

function applyPointerSelection(textarea: HTMLTextAreaElement, clientX: number, clientY: number) {
  const caret = caretIndexAtClientPoint(textarea, clientX, clientY)
  dragSelStart = Math.min(selectAnchor, caret)
  dragSelEnd = Math.max(selectAnchor, caret)
  if (document.activeElement !== textarea) {
    textarea.focus({ preventScroll: true })
  }
  if (textarea.selectionStart !== dragSelStart || textarea.selectionEnd !== dragSelEnd) {
    try {
      textarea.setSelectionRange(dragSelStart, dragSelEnd)
    } catch {
      /* overlay uses dragSel* */
    }
  }
}

function flushPendingMove() {
  moveRafId = 0
  if (!pointerDown) return
  const textarea = textareaRef.value
  if (!textarea) return

  const clientX = pendingMoveX
  const clientY = pendingMoveY
  applyPointerSelection(textarea, clientX, clientY)

  const rect = textarea.getBoundingClientRect()
  const edge = 24
  if (clientY < rect.top + edge) {
    textarea.scrollTop -= Math.max(4, (rect.top + edge - clientY) * 0.35)
    syncScroll(textarea.scrollTop)
  } else if (clientY > rect.bottom - edge) {
    textarea.scrollTop += Math.max(4, (clientY - (rect.bottom - edge)) * 0.35)
    syncScroll(textarea.scrollTop)
  }

  scheduleSelectionUpdate()
}

function onMouseDown(event: MouseEvent) {
  if (props.readonly || event.button !== 0) return
  const textarea = textareaRef.value
  if (!textarea) return

  event.preventDefault()

  pointerDown = true
  pointerDragged = false
  downClientX = event.clientX
  downClientY = event.clientY
  selectAnchor = caretIndexAtClientPoint(textarea, event.clientX, event.clientY)
  dragSelStart = selectAnchor
  dragSelEnd = selectAnchor

  setDocUserSelect(true)
  textarea.focus({ preventScroll: true })
  try {
    textarea.setSelectionRange(selectAnchor, selectAnchor)
  } catch {
    /* ignore */
  }
  lastSelStart = -1
  lastSelEnd = -1
  clearSelectionUi()

  document.addEventListener('mousemove', onDocMouseMove, true)
  document.addEventListener('mouseup', onDocMouseUp, true)
}

function onDocMouseMove(event: MouseEvent) {
  if (!pointerDown) return
  pendingMoveX = event.clientX
  pendingMoveY = event.clientY

  if (
    !pointerDragged &&
    (Math.abs(event.clientX - downClientX) > DRAG_THRESHOLD_PX ||
      Math.abs(event.clientY - downClientY) > DRAG_THRESHOLD_PX)
  ) {
    pointerDragged = true
  }

  event.preventDefault()
  if (moveRafId) return
  moveRafId = requestAnimationFrame(flushPendingMove)
}

function onDocMouseUp(event: MouseEvent) {
  if (!pointerDown) return
  const textarea = textareaRef.value

  event.preventDefault()
  if (textarea) {
    if (pointerDragged) {
      applyPointerSelection(textarea, event.clientX, event.clientY)
    } else {
      dragSelStart = selectAnchor
      dragSelEnd = selectAnchor
      try {
        textarea.setSelectionRange(selectAnchor, selectAnchor)
      } catch {
        /* ignore */
      }
      clearSelectionUi()
    }
  }

  pointerDown = false
  pointerDragged = false
  setDocUserSelect(false)
  if (moveRafId) {
    cancelAnimationFrame(moveRafId)
    moveRafId = 0
  }
  detachDocListeners()
  scheduleSelectionUpdate()
}

function scheduleSelectionUpdate() {
  if (rafId) return
  rafId = requestAnimationFrame(() => {
    rafId = 0
    updateSelection()
  })
}

function syncSpaceDots(start: number, end: number) {
  if (start === lastDotStart && end === lastDotEnd) return
  const root = highlightRef.value
  if (!root) return

  const nodes = root.querySelectorAll<HTMLElement>('.raw-hl-ws')
  const oldStart = lastDotStart
  const oldEnd = lastDotEnd
  lastDotStart = start
  lastDotEnd = end

  for (const el of nodes) {
    const i = Number(el.dataset.i)
    const was = oldStart < oldEnd && i >= oldStart && i < oldEnd
    const now = start < end && i >= start && i < end
    if (was === now) continue
    const isTab = el.textContent === '\t'
    el.classList.toggle('raw-hl-space', now && !isTab)
    el.classList.toggle('raw-hl-tab', now && isTab)
  }
}

function ensureMirror(textarea: HTMLTextAreaElement) {
  if (!mirrorEl) {
    mirrorEl = document.createElement('div')
    mirrorEl.setAttribute('aria-hidden', 'true')
    mirrorEl.style.cssText =
      'position:fixed;visibility:hidden;pointer-events:none;left:-10000px;top:0;overflow:hidden;white-space:pre-wrap;'
    document.body.appendChild(mirrorEl)
  }

  const style = getComputedStyle(textarea)
  const width = textarea.offsetWidth
  const lh = parseFloat(style.lineHeight) || 20
  lineHeightPx = lh
  // Approximate monospace space width from font-size.
  spaceWidthPx = parseFloat(style.fontSize) || 13
  if (width !== mirrorWidth) {
    mirrorWidth = width
    mirrorEl.style.boxSizing = 'border-box'
    mirrorEl.style.width = `${width}px`
    mirrorEl.style.padding = style.padding
    mirrorEl.style.border = style.border
    mirrorEl.style.font = style.font
    mirrorEl.style.letterSpacing = style.letterSpacing
    mirrorEl.style.lineHeight = style.lineHeight
    mirrorEl.style.wordBreak = style.wordBreak
    mirrorEl.style.overflowWrap = style.overflowWrap
    mirrorEl.style.tabSize = style.tabSize
  }
  return mirrorEl
}

function fillMirrorText(textarea: HTMLTextAreaElement) {
  const mirror = ensureMirror(textarea)
  if (mirrorText !== textarea.value) {
    mirrorText = textarea.value
    mirror.textContent = mirrorText
  }
  return mirror
}

function rectsAtIndex(mirror: HTMLDivElement, index: number): DOMRect[] {
  const node = mirror.firstChild
  if (!node || node.nodeType !== Node.TEXT_NODE) return []
  const text = node.textContent ?? ''
  if (!text.length) return []

  // Newline rects often sit at the start of the next line — use previous char instead.
  let i = Math.max(0, Math.min(index, text.length - 1))
  if (text[i] === '\n' && i > 0) i -= 1

  const range = document.createRange()
  range.setStart(node, i)
  range.setEnd(node, Math.min(i + 1, text.length))
  return [...range.getClientRects()]
}

/** Map any client point (even outside the raw box) to a caret index. */
function caretIndexAtClientPoint(textarea: HTMLTextAreaElement, clientX: number, clientY: number): number {
  const value = textarea.value
  if (!value) return 0

  const taRect = textarea.getBoundingClientRect()
  const style = getComputedStyle(textarea)
  const padTop = parseFloat(style.paddingTop) || 0
  const contentX = clientX - taRect.left + textarea.scrollLeft
  const contentY = clientY - taRect.top + textarea.scrollTop

  const mirror = fillMirrorText(textarea)
  const mirrorRect = mirror.getBoundingClientRect()
  const targetX = mirrorRect.left + contentX
  const targetY = mirrorRect.top + contentY

  if (contentY <= padTop) return 0
  if (contentY >= mirror.scrollHeight) return value.length

  const lineStarts = ensureLineStarts(value)

  let lineIdx = 0
  for (let li = 0; li < lineStarts.length; li += 1) {
    const ls = lineStarts[li]
    const nlOrEnd = li + 1 < lineStarts.length ? lineStarts[li + 1] - 1 : value.length
    const empty = ls >= nlOrEnd
    const probe = empty ? Math.max(0, ls - 1) : ls
    const rects = rectsAtIndex(mirror, Math.min(probe, value.length - 1))
    if (!rects.length) continue
    let top = rects[0].top
    if (empty && ls > 0) top = rects[0].top + lineHeightPx
    const bottom = top + lineHeightPx
    if (targetY >= top - 1) lineIdx = li
    if (targetY < bottom) {
      lineIdx = li
      break
    }
  }

  const ls = lineStarts[lineIdx]
  const nlOrEnd = lineIdx + 1 < lineStarts.length ? lineStarts[lineIdx + 1] - 1 : value.length
  if (ls >= nlOrEnd) return nlOrEnd

  const firstRects = rectsAtIndex(mirror, ls)
  if (firstRects.length && targetX <= firstRects[0].left + 1) return ls

  const lastRects = rectsAtIndex(mirror, nlOrEnd - 1)
  if (lastRects.length && targetX >= lastRects[0].right - 1) return nlOrEnd

  let lo = ls
  let hi = nlOrEnd
  while (lo < hi) {
    const mid = (lo + hi) >> 1
    const rects = rectsAtIndex(mirror, mid)
    if (!rects.length) {
      hi = mid
      continue
    }
    const r = rects[0]
    if (r.left + r.width / 2 < targetX) lo = mid + 1
    else hi = mid
  }
  return lo
}

function mergeLineRects(rects: DOMRect[]): LineRect[] {
  const items = rects
    .map((r) => ({ top: r.top, left: r.left, right: r.right, bottom: r.bottom }))
    .sort((a, b) => a.top - b.top || a.left - b.left)

  const merged: { top: number; left: number; right: number; bottom: number }[] = []
  for (const item of items) {
    const last = merged[merged.length - 1]
    if (last && Math.abs(last.top - item.top) < 2) {
      last.left = Math.min(last.left, item.left)
      last.right = Math.max(last.right, item.right)
      last.bottom = Math.max(last.bottom, item.bottom)
    } else {
      merged.push({ ...item })
    }
  }

  const padX = 2
  const lh = lineHeightPx
  const spaceW = spaceWidthPx * 0.6
  const lines = merged
    .map((m) => {
      const rawH = Math.max(1, m.bottom - m.top)
      return {
        top: m.top - (lh - rawH) / 2,
        left: m.left - padX,
        width: Math.max(0, m.right - m.left) + padX * 2,
        height: lh,
      }
    })
    .sort((a, b) => a.top - b.top)

  const linked: boolean[] = lines.map(() => false)
  for (let i = 1; i < lines.length; i += 1) {
    const prev = lines[i - 1]
    const expected = prev.top + lh
    if (Math.abs(lines[i].top - expected) < lh * 0.75) {
      lines[i].top = expected
      lines[i].height = lh
      prev.height = lh
      linked[i] = true
    }
  }

  for (let i = 0; i < lines.length; i += 1) {
    const hasFollowingLine = i + 1 < lines.length && linked[i + 1]
    if (hasFollowingLine || (i === lines.length - 1 && selectionEndsWithNewline)) {
      lines[i].width += spaceW
    }
  }

  return lines
}

/** Postman-style stepped selection with outward fillets on the right edge. */
function buildSelectionPathD(lines: LineRect[], scrollTop: number, scrollLeft: number): string {
  const n = lines.length
  if (n === 0) return ''
  const r = SEL_RADIUS

  const L = (i: number) => lines[i].left - scrollLeft
  const R = (i: number) => lines[i].left + lines[i].width - scrollLeft
  const T = (i: number) => lines[i].top - scrollTop
  const B = (i: number) => T(i) + lh

  const p: string[] = []

  p.push(`M ${L(0) + r} ${T(0)}`)
  p.push(`H ${R(0) - r}`)
  p.push(`Q ${R(0)} ${T(0)} ${R(0)} ${T(0) + r}`)

  for (let i = 0; i < n; i += 1) {
    const Ri = R(i)
    const Bi = B(i)
    const isLast = i === n - 1

    if (isLast) {
      p.push(`V ${Bi - r}`)
      p.push(`Q ${Ri} ${Bi} ${Ri - r} ${Bi}`)
      break
    }

    const R2 = R(i + 1)
    const T2 = T(i + 1)

    if (Ri > R2 + 1) {
      p.push(`V ${Bi - r}`)
      p.push(`Q ${Ri} ${Bi} ${Ri - r} ${Bi}`)
      p.push(`H ${R2 + r}`)
      p.push(`Q ${R2 + r} ${Bi} ${R2} ${T2 + r}`)
    } else if (R2 > Ri + 1) {
      p.push(`V ${Bi - r}`)
      p.push(`Q ${Ri} ${Bi} ${Ri - r} ${Bi}`)
      p.push(`Q ${R2} ${Bi} ${R2} ${T2 + r}`)
    } else {
      p.push(`V ${T2 + r}`)
    }
  }

  const last = n - 1
  p.push(`H ${L(last) + r}`)
  p.push(`Q ${L(last)} ${B(last)} ${L(last)} ${B(last) - r}`)

  for (let i = n - 1; i >= 0; i -= 1) {
    const Li = L(i)
    const Ti = T(i)
    if (i === 0) {
      p.push(`V ${Ti + r}`)
      p.push(`Q ${Li} ${Ti} ${Li + r} ${Ti}`)
      break
    }
    const Lprev = L(i - 1)
    const Tprev = T(i - 1)

    if (Math.abs(Li - Lprev) < 2) {
      p.push(`V ${Tprev + r}`)
      continue
    }
    if (Li > Lprev) {
      p.push(`V ${Ti + r}`)
      p.push(`Q ${Li} ${Ti} ${Li + r} ${Ti}`)
      p.push(`H ${Lprev + r}`)
      p.push(`Q ${Lprev} ${Ti} ${Lprev} ${Ti + r}`)
    } else {
      p.push(`V ${Tprev + r}`)
      p.push(`Q ${Lprev} ${Ti} ${Lprev + r} ${Ti}`)
    }
  }

  p.push('Z')
  return p.join(' ')
}

function paintSelectionPath(textarea: HTMLTextAreaElement) {
  if (!cachedLineRects.length) {
    selectionPathD.value = ''
    return
  }
  selectionPathD.value = buildSelectionPathD(
    cachedLineRects,
    textarea.scrollTop,
    textarea.scrollLeft,
  )
}

function updateSelection() {
  const textarea = textareaRef.value
  if (!textarea) {
    clearSelectionUi()
    return
  }
  if (document.activeElement !== textarea && !pointerDown) {
    clearSelectionUi()
    return
  }

  const start = pointerDown ? dragSelStart : textarea.selectionStart
  const end = pointerDown ? dragSelEnd : textarea.selectionEnd

  if (start === end) {
    lastSelStart = start
    lastSelEnd = end
    cachedLineRects = []
    selectionPathD.value = ''
    selectionEndsWithNewline = false
    syncSpaceDots(0, 0)
    return
  }

  const rangeChanged = start !== lastSelStart || end !== lastSelEnd
  lastSelStart = start
  lastSelEnd = end
  selectionEndsWithNewline = textarea.value.slice(start, end).endsWith('\n')
  syncSpaceDots(start, end)

  if (rangeChanged || cachedLineRects.length === 0) {
    const mirror = fillMirrorText(textarea)
    const before = valueSliceForMeasure(textarea.value, 0, start)
    const selected = textarea.value.slice(start, end).replace(/\n+$/, '')
    if (!selected) {
      cachedLineRects = []
    } else {
      mirror.replaceChildren()
      const mark = document.createElement('span')
      mark.textContent = selected
      mirror.append(document.createTextNode(before), mark)
      mirrorText = ''

      const range = document.createRange()
      range.selectNodeContents(mark)
      const minW = Math.max(3, lineHeightPx * 0.2)
      const rects = [...range.getClientRects()].filter((r) => r.width >= minW && r.height > 0)
      const mirrorRect = mirror.getBoundingClientRect()
      const merged = mergeLineRects(rects)

      cachedLineRects = merged.map((r) => ({
        top: r.top - mirrorRect.top,
        left: r.left - mirrorRect.left,
        width: r.width,
        height: r.height,
      }))
    }
  }

  paintSelectionPath(textarea)
}

function valueSliceForMeasure(value: string, from: number, to: number) {
  return value.slice(from, to)
}

function setTextAndCursor(text: string, cursor: number) {
  localRaw.value = text
  emit('update:modelValue', text)
  void nextTick(() => {
    const el = textareaRef.value
    if (!el) return
    el.selectionStart = cursor
    el.selectionEnd = cursor
    clearSelectionUi()
  })
}

function onKeydown(event: KeyboardEvent) {
  if (props.readonly) return

  if (event.key === 'Enter' && !event.isComposing && !event.ctrlKey && !event.metaKey && !event.altKey) {
    const el = textareaRef.value
    if (!el) return
    const result = applyJsonEnter(el.value, el.selectionStart, el.selectionEnd)
    if (result) {
      event.preventDefault()
      setTextAndCursor(result.text, result.cursor)
      return
    }
  }

  scheduleSelectionUpdate()
}

function formatContent() {
  const text = (textareaRef.value?.value ?? localRaw.value).trim()
  if (!text) return false
  const formatted = formatJsonLenient(text)
  if (!formatted) return false
  localRaw.value = formatted
  emit('update:modelValue', formatted)
  return true
}

onBeforeUnmount(() => {
  if (rafId) cancelAnimationFrame(rafId)
  if (moveRafId) cancelAnimationFrame(moveRafId)
  pointerDown = false
  pointerDragged = false
  setDocUserSelect(false)
  detachDocListeners()
  mirrorEl?.remove()
  mirrorEl = null
})

defineExpose({ formatContent })
</script>
