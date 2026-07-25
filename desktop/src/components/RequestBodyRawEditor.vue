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
          @dblclick.prevent="onDblClick"
          @blur="onBlur"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { diagnoseRawJson, highlightRawContent } from '../utils/raw-editor-highlight'
import {
  applyJsonEnter,
  findJsonTokenRangeAt,
  formatJsonLenient,
} from '../utils/raw-json-utils'

type LineRect = {
  top: number
  left: number
  width: number
  height: number
}

type RawContentType = 'JSON' | 'Text'

const props = withDefaults(
  defineProps<{
    modelValue: string
    readonly?: boolean
    placeholder?: string
    contentType?: RawContentType
  }>(),
  {
    readonly: false,
    placeholder: '',
    contentType: 'JSON',
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
let pendingTokenSelect: { start: number; end: number } | null = null
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
const SEL_RADIUS = 5
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
    cachedLineRects = []
    mirrorText = ''
    scheduleSelectionUpdate()
  })
})

const jsonError = computed(() =>
  props.contentType === 'JSON' ? diagnoseRawJson(localRaw.value) : null,
)
const errorLine = computed(() => jsonError.value?.line ?? null)

const lineNumbers = computed(() => {
  const count = Math.max(1, localRaw.value.split('\n').length)
  return Array.from({ length: count }, (_, index) => index + 1)
})

const highlightedContent = computed(() =>
  highlightRawContent(
    localRaw.value,
    props.contentType,
    'raw',
    jsonError.value?.index ?? null,
  ),
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
  collapseAndClearSelection()
}

function clearSelectionUi() {
  lastSelStart = -1
  lastSelEnd = -1
  cachedLineRects = []
  selectionPathD.value = ''
  selectionEndsWithNewline = false
  syncSpaceDots(0, 0)
  document.removeEventListener('mousedown', onDocMouseDownOutside, true)
}

function collapseAndClearSelection() {
  const textarea = textareaRef.value
  if (textarea && textarea.selectionStart !== textarea.selectionEnd) {
    const cursor = textarea.selectionEnd
    try {
      textarea.setSelectionRange(cursor, cursor)
    } catch {
      /* ignore */
    }
  }
  dragSelStart = dragSelEnd = selectAnchor
  clearSelectionUi()
}

function onDocMouseDownOutside(event: MouseEvent) {
  if (pointerDown) return
  const wrap = textareaRef.value?.closest('.interface-body-raw__code-wrap')
  const target = event.target as Node | null
  if (wrap && target && wrap.contains(target)) return
  collapseAndClearSelection()
  document.removeEventListener('mousedown', onDocMouseDownOutside, true)
}

function detachDocListeners() {
  document.removeEventListener('mousemove', onDocMouseMove, true)
  document.removeEventListener('mouseup', onDocMouseUp, true)
}

function attachOutsideClearListener() {
  document.removeEventListener('mousedown', onDocMouseDownOutside, true)
  document.addEventListener('mousedown', onDocMouseDownOutside, true)
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

function applyRangeSelection(textarea: HTMLTextAreaElement, start: number, end: number) {
  dragSelStart = start
  dragSelEnd = end
  selectAnchor = start
  try {
    textarea.setSelectionRange(start, end)
  } catch {
    /* ignore */
  }
  lastSelStart = -1
  lastSelEnd = -1
  scheduleSelectionUpdate()
  if (start !== end) attachOutsideClearListener()
}

function selectTokenAtPoint(textarea: HTMLTextAreaElement, clientX: number, clientY: number) {
  const caret = caretIndexAtClientPoint(textarea, clientX, clientY)
  return findJsonTokenRangeAt(textarea.value, caret)
}

function onMouseDown(event: MouseEvent) {
  if (event.button !== 0) return
  const textarea = textareaRef.value
  if (!textarea) return

  event.preventDefault()

  pointerDown = true
  pointerDragged = false
  pendingTokenSelect = null
  downClientX = event.clientX
  downClientY = event.clientY

  if (event.detail >= 2) {
    pendingTokenSelect = selectTokenAtPoint(textarea, event.clientX, event.clientY)
  }

  selectAnchor = caretIndexAtClientPoint(textarea, event.clientX, event.clientY)
  dragSelStart = selectAnchor
  dragSelEnd = selectAnchor

  setDocUserSelect(true)
  textarea.focus({ preventScroll: true })

  if (pendingTokenSelect) {
    applyRangeSelection(textarea, pendingTokenSelect.start, pendingTokenSelect.end)
    selectAnchor = pendingTokenSelect.start
    dragSelStart = pendingTokenSelect.start
    dragSelEnd = pendingTokenSelect.end
  } else {
    try {
      textarea.setSelectionRange(selectAnchor, selectAnchor)
    } catch {
      /* ignore */
    }
    lastSelStart = -1
    lastSelEnd = -1
    clearSelectionUi()
  }

  document.addEventListener('mousemove', onDocMouseMove, true)
  document.addEventListener('mouseup', onDocMouseUp, true)
}

function onDblClick(event: MouseEvent) {
  const textarea = textareaRef.value
  if (!textarea) return
  event.preventDefault()
  const range = selectTokenAtPoint(textarea, event.clientX, event.clientY)
  if (!range) return
  applyRangeSelection(textarea, range.start, range.end)
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
    } else if (pendingTokenSelect) {
      applyRangeSelection(textarea, pendingTokenSelect.start, pendingTokenSelect.end)
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
  pendingTokenSelect = null
  setDocUserSelect(false)
  if (moveRafId) {
    cancelAnimationFrame(moveRafId)
    moveRafId = 0
  }
  detachDocListeners()
  scheduleSelectionUpdate()
  if (textarea && dragSelStart !== dragSelEnd) {
    attachOutsideClearListener()
  }
}

function scheduleSelectionUpdate() {
  if (rafId) return
  rafId = requestAnimationFrame(() => {
    rafId = 0
    updateSelection()
    const textarea = textareaRef.value
    if (
      textarea &&
      document.activeElement === textarea &&
      (pointerDown ? dragSelStart !== dragSelEnd : textarea.selectionStart !== textarea.selectionEnd)
    ) {
      attachOutsideClearListener()
    }
  })
}

function syncSpaceDots(start: number, end: number) {
  const root = highlightRef.value
  if (!root) {
    lastDotStart = start
    lastDotEnd = end
    return
  }

  // Clearing must force-remove classes; lastDot* may already be reset.
  if (start >= end) {
    const lit = root.querySelectorAll<HTMLElement>('.raw-hl-ws.raw-hl-space, .raw-hl-ws.raw-hl-tab')
    for (const el of lit) {
      el.classList.remove('raw-hl-space', 'raw-hl-tab')
    }
    lastDotStart = 0
    lastDotEnd = 0
    return
  }

  if (start === lastDotStart && end === lastDotEnd) return

  const nodes = root.querySelectorAll<HTMLElement>('.raw-hl-ws')
  const oldStart = lastDotStart
  const oldEnd = lastDotEnd
  lastDotStart = start
  lastDotEnd = end

  for (const el of nodes) {
    const i = Number(el.dataset.i)
    const was = oldStart < oldEnd && i >= oldStart && i < oldEnd
    const now = i >= start && i < end
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

  // Keep horizontal padding tight so selection doesn't bleed onto neighboring
  // chars (e.g. JSON quotes around a dblclicked key/value).
  const padX = 0
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

/** Postman-style stepped selection: outer corners rounded, step joints use concave fillets. */
function buildSelectionPathD(lines: LineRect[], scrollTop: number, scrollLeft: number): string {
  const n = lines.length
  if (n === 0) return ''

  const L = (i: number) => lines[i].left - scrollLeft
  const R = (i: number) => lines[i].left + lines[i].width - scrollLeft
  const T = (i: number) => lines[i].top - scrollTop
  const B = (i: number) => T(i) + lines[i].height
  const rad = (...sizes: number[]) => {
    const min = Math.min(...sizes)
    if (!(min > 0)) return 0
    return Math.min(SEL_RADIUS, min / 2)
  }

  const p: string[] = []
  const rTop = rad(R(0) - L(0), B(0) - T(0))

  // Top edge → top-right outer corner
  p.push(`M ${L(0) + rTop} ${T(0)}`)
  p.push(`H ${R(0) - rTop}`)
  if (rTop > 0) p.push(`Q ${R(0)} ${T(0)} ${R(0)} ${T(0) + rTop}`)
  else p.push(`L ${R(0)} ${T(0)}`)

  // Right edge (top → bottom)
  for (let i = 0; i < n; i += 1) {
    const Ri = R(i)
    const Bi = B(i)
    const wi = Ri - L(i)
    const hi = Bi - T(i)

    if (i === n - 1) {
      const r = rad(wi, hi)
      p.push(`V ${Bi - r}`)
      if (r > 0) p.push(`Q ${Ri} ${Bi} ${Ri - r} ${Bi}`)
      else p.push(`L ${Ri} ${Bi}`)
      break
    }

    const R2 = R(i + 1)
    const T2 = T(i + 1)
    const w2 = R2 - L(i + 1)
    const h2 = B(i + 1) - T2
    const gap = Math.abs(Ri - R2)

    if (gap < 1) continue

    if (Ri > R2) {
      // Next line shorter: outer corner then concave into next
      const rOuter = rad(wi, hi, gap)
      const rInner = rad(w2, h2, gap)
      p.push(`V ${Bi - rOuter}`)
      if (rOuter > 0) p.push(`Q ${Ri} ${Bi} ${Ri - rOuter} ${Bi}`)
      else p.push(`L ${Ri} ${Bi}`)
      p.push(`H ${R2 + rInner}`)
      if (rInner > 0) p.push(`Q ${R2} ${Bi} ${R2} ${T2 + rInner}`)
      else p.push(`L ${R2} ${T2}`)
    } else {
      // Next line longer: concave fillet then outer corner
      const rInner = rad(wi, hi, gap)
      const rOuter = rad(w2, h2, gap)
      p.push(`V ${Bi - rInner}`)
      if (rInner > 0) p.push(`Q ${Ri} ${Bi} ${Ri + rInner} ${Bi}`)
      else p.push(`L ${Ri} ${Bi}`)
      p.push(`H ${R2 - rOuter}`)
      if (rOuter > 0) p.push(`Q ${R2} ${Bi} ${R2} ${T2 + rOuter}`)
      else p.push(`L ${R2} ${T2}`)
    }
  }

  // Bottom edge → bottom-left outer corner
  const last = n - 1
  const rBot = rad(R(last) - L(last), B(last) - T(last))
  p.push(`H ${L(last) + rBot}`)
  if (rBot > 0) p.push(`Q ${L(last)} ${B(last)} ${L(last)} ${B(last) - rBot}`)
  else p.push(`L ${L(last)} ${B(last)}`)

  // Left edge (bottom → top)
  for (let i = n - 1; i >= 0; i -= 1) {
    const Li = L(i)
    const Ti = T(i)
    const wi = R(i) - Li
    const hi = B(i) - Ti

    if (i === 0) {
      const r = rad(wi, hi)
      p.push(`V ${Ti + r}`)
      if (r > 0) p.push(`Q ${Li} ${Ti} ${Li + r} ${Ti}`)
      else p.push(`L ${Li} ${Ti}`)
      break
    }

    const Lprev = L(i - 1)
    const Tprev = T(i - 1)
    const wPrev = R(i - 1) - Lprev
    const hPrev = B(i - 1) - Tprev
    const gap = Math.abs(Li - Lprev)

    if (gap < 1) continue

    if (Li > Lprev) {
      // Line above extends further left
      const rInner = rad(wi, hi, gap)
      const rOuter = rad(wPrev, hPrev, gap)
      p.push(`V ${Ti + rInner}`)
      if (rInner > 0) p.push(`Q ${Li} ${Ti} ${Li - rInner} ${Ti}`)
      else p.push(`L ${Li} ${Ti}`)
      p.push(`H ${Lprev + rOuter}`)
      if (rOuter > 0) p.push(`Q ${Lprev} ${Ti} ${Lprev} ${Ti - rOuter}`)
      else p.push(`L ${Lprev} ${Tprev}`)
    } else {
      // Line above is more indented
      const rOuter = rad(wi, hi, gap)
      const rInner = rad(wPrev, hPrev, gap)
      p.push(`V ${Ti + rOuter}`)
      if (rOuter > 0) p.push(`Q ${Li} ${Ti} ${Li + rOuter} ${Ti}`)
      else p.push(`L ${Li} ${Ti}`)
      p.push(`H ${Lprev - rInner}`)
      if (rInner > 0) p.push(`Q ${Lprev} ${Ti} ${Lprev} ${Ti - rInner}`)
      else p.push(`L ${Lprev} ${Tprev}`)
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

function setTextAndSelection(text: string, start: number, end: number) {
  localRaw.value = text
  emit('update:modelValue', text)
  void nextTick(() => {
    const el = textareaRef.value
    if (!el) return
    applyRangeSelection(el, start, end)
  })
}

function duplicateLines(direction: 'up' | 'down') {
  const el = textareaRef.value
  if (!el || props.readonly) return
  const value = el.value
  const selStart = el.selectionStart
  const selEnd = el.selectionEnd

  const start = value.lastIndexOf('\n', selStart - 1) + 1
  let end: number
  if (selEnd > selStart && value[selEnd - 1] === '\n') {
    end = selEnd
  } else {
    const nl = value.indexOf('\n', selEnd)
    end = nl === -1 ? value.length : nl + 1
  }

  const rawBlock = value.slice(start, end)
  const block = rawBlock.endsWith('\n') ? rawBlock : `${rawBlock}\n`

  if (direction === 'up') {
    const newValue = value.slice(0, start) + block + value.slice(start)
    setTextAndSelection(newValue, start, start + block.length - 1)
    return
  }

  if (!rawBlock.endsWith('\n') && end === value.length) {
    const newValue = `${value}\n${rawBlock}`
    const ns = value.length + 1
    setTextAndSelection(newValue, ns, ns + rawBlock.length)
    return
  }

  const newValue = value.slice(0, end) + block + value.slice(end)
  setTextAndSelection(newValue, end, end + block.length - 1)
}

/** With a selection, typing " or ' wraps the selection instead of replacing it. */
function tryWrapSelectionWithQuote(quote: string): boolean {
  const el = textareaRef.value
  if (!el || props.readonly) return false
  const start = pointerDown ? dragSelStart : el.selectionStart
  const end = pointerDown ? dragSelEnd : el.selectionEnd
  if (start === end) return false

  const value = el.value
  // Already has matching quotes outside (typical after dblclick on a string) →
  // expand selection to include those quotes ("wrap" the content with quotes).
  if (start > 0 && end < value.length && value[start - 1] === quote && value[end] === quote) {
    applyRangeSelection(el, start - 1, end + 1)
    return true
  }

  const next = value.slice(0, start) + quote + value.slice(start, end) + quote + value.slice(end)
  setTextAndSelection(next, start + 1, end + 1)
  return true
}

function onKeydown(event: KeyboardEvent) {
  if (
    event.altKey &&
    event.shiftKey &&
    !event.ctrlKey &&
    !event.metaKey &&
    !event.isComposing &&
    (event.key === 'ArrowUp' || event.key === 'ArrowDown')
  ) {
    if (!props.readonly) {
      event.preventDefault()
      duplicateLines(event.key === 'ArrowUp' ? 'up' : 'down')
    }
    return
  }

  if (props.readonly) {
    scheduleSelectionUpdate()
    return
  }

  if (
    (event.key === '"' || event.key === "'") &&
    !event.isComposing &&
    !event.ctrlKey &&
    !event.metaKey &&
    !event.altKey
  ) {
    if (tryWrapSelectionWithQuote(event.key)) {
      event.preventDefault()
      return
    }
  }

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
  document.removeEventListener('mousedown', onDocMouseDownOutside, true)
  mirrorEl?.remove()
  mirrorEl = null
})

defineExpose({ formatContent })
</script>
