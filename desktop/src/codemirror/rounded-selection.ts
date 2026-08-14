import { EditorSelection } from '@codemirror/state'
import { EditorView, layer, RectangleMarker, type LayerMarker } from '@codemirror/view'

type SelRect = {
  left: number
  top: number
  width: number
  height: number
}

const ALIGN_EPS = 1

function readRadius(view: EditorView): number {
  const raw = getComputedStyle(view.dom).getPropertyValue('--raw-editor-selection-radius')
  const value = parseFloat(raw)
  return Number.isFinite(value) ? value : 4
}

function aligned(a: number, b: number): boolean {
  return Math.abs(a - b) < ALIGN_EPS
}

function markerTop(view: EditorView, documentY: number): number {
  const scrollRect = view.scrollDOM.getBoundingClientRect()
  const baseTop = scrollRect.top - view.scrollDOM.scrollTop * view.scaleY
  return view.documentTop + documentY * view.scaleY - baseTop
}

function withLineHeight(view: EditorView, pos: number, rect: SelRect): SelRect {
  const block = view.lineBlockAt(pos)
  return {
    ...rect,
    top: markerTop(view, block.top),
    height: block.height * view.scaleY,
  }
}

function emptyLineRect(view: EditorView, pos: number): SelRect | null {
  const pieces = RectangleMarker.forRange(
    view,
    'cm-roundedSelection',
    EditorSelection.cursor(pos),
  )
  const piece = pieces[0]
  if (!piece) return null
  return withLineHeight(view, pos, {
    left: piece.left,
    top: piece.top,
    width: Math.max(view.defaultCharacterWidth, 6),
    height: piece.height,
  })
}

function newlineAtLineEnd(view: EditorView, lineFrom: number, lineTo: number): SelRect | null {
  const pieces = RectangleMarker.forRange(
    view,
    'cm-roundedSelection',
    EditorSelection.cursor(lineTo, -1),
  )
  const piece = pieces[0]
  if (!piece) return emptyLineRect(view, lineFrom)
  return withLineHeight(view, lineFrom, {
    left: piece.left,
    top: piece.top,
    width: Math.max(view.defaultCharacterWidth, 6),
    height: piece.height,
  })
}

function lineRects(view: EditorView, from: number, to: number): SelRect[] {
  const rects: SelRect[] = []
  let pos = from
  while (pos < to) {
    const line = view.state.doc.lineAt(pos)
    const start = Math.max(line.from, from)
    const end = Math.min(line.to, to)
    const includesNewline = to > line.to

    if (start < end) {
      const pieces = RectangleMarker.forRange(
        view,
        'cm-roundedSelection',
        EditorSelection.range(start, end),
      )
      const block = view.lineBlockAt(line.from)
      const lineTop = markerTop(view, block.top)
      const lineBottom = lineTop + block.height * view.scaleY
      const linePieces: SelRect[] = []
      for (const piece of pieces) {
        const mid = piece.top + piece.height / 2
        if (mid < lineTop - 2 || mid > lineBottom + 2) continue
        const rect = {
          left: piece.left,
          top: piece.top,
          width: piece.width ?? view.defaultCharacterWidth,
          height: piece.height,
        }
        linePieces.push(pieces.length === 1 ? withLineHeight(view, start, rect) : rect)
      }
      if (includesNewline) {
        const last = linePieces[linePieces.length - 1]
        if (last) last.width += view.defaultCharacterWidth
        else {
          const extra = newlineAtLineEnd(view, line.from, line.to)
          if (extra) linePieces.push(extra)
        }
      }
      rects.push(...linePieces)
    } else if (line.from === line.to && includesNewline) {
      const empty = emptyLineRect(view, line.from)
      if (empty) rects.push(empty)
    }
    pos = line.to + 1
  }
  return rects
}

function rectRight(rect: SelRect): number {
  return rect.left + rect.width
}

function overlapWidth(a: SelRect, b: SelRect): number {
  return Math.min(rectRight(a), rectRight(b)) - Math.max(a.left, b.left)
}

function stronglyOverlaps(a: SelRect, b: SelRect, charWidth: number): boolean {
  return overlapWidth(a, b) >= charWidth - ALIGN_EPS
}

function radiiFor(rects: SelRect[], index: number, radius: number, charWidth: number): string {
  const cur = rects[index]
  const prev = rects[index - 1]
  const next = rects[index + 1]
  const right = rectRight(cur)
  let tl = radius
  let tr = radius
  let br = radius
  let bl = radius

  if (prev) {
    const prevRight = rectRight(prev)
    const leftDocked = aligned(prev.left, cur.left)
    const rightDocked = aligned(prevRight, right)
    const overlap = stronglyOverlaps(prev, cur, charWidth)
    if (leftDocked || (overlap && cur.left > prev.left)) tl = 0
    if (rightDocked || ((leftDocked || overlap) && prevRight > right)) tr = 0
  }
  if (next) {
    const nextRight = rectRight(next)
    const leftDocked = aligned(next.left, cur.left)
    const rightDocked = aligned(nextRight, right)
    const overlap = stronglyOverlaps(next, cur, charWidth)
    if (leftDocked || (overlap && cur.left > next.left)) bl = 0
    if (rightDocked || ((leftDocked || overlap) && nextRight > right)) br = 0
  }

  return `${tl}px ${tr}px ${br}px ${bl}px`
}

function rightFillets(rects: SelRect[], radius: number, charWidth: number): SelectionRectMarker[] {
  const fillets: SelectionRectMarker[] = []
  for (let i = 0; i < rects.length - 1; i += 1) {
    const a = rects[i]
    const b = rects[i + 1]
    const aRight = rectRight(a)
    const bRight = rectRight(b)
    const gap = Math.abs(aRight - bRight)
    if (gap <= ALIGN_EPS) continue
    if (!aligned(a.left, b.left) && !stronglyOverlaps(a, b, charWidth)) continue
    const size = Math.min(radius, gap, a.height, b.height)
    if (size <= 0) continue
    if (aRight > bRight) {
      fillets.push(
        new SelectionRectMarker(bRight, b.top, size, size, '0', 'circle(50% at 0 0)'),
      )
    } else {
      fillets.push(
        new SelectionRectMarker(aRight, b.top - size, size, size, '0', 'circle(50% at 0 100%)'),
      )
    }
  }
  return fillets
}

function leftFillets(rects: SelRect[], radius: number, charWidth: number): SelectionRectMarker[] {
  const fillets: SelectionRectMarker[] = []
  for (let i = 0; i < rects.length - 1; i += 1) {
    const a = rects[i]
    const b = rects[i + 1]
    const gap = Math.abs(a.left - b.left)
    if (gap <= ALIGN_EPS) continue
    if (!aligned(rectRight(a), rectRight(b)) && !stronglyOverlaps(a, b, charWidth)) continue
    const size = Math.min(radius, gap, a.height, b.height)
    if (size <= 0) continue
    if (a.left < b.left) {
      fillets.push(
        new SelectionRectMarker(b.left - size, b.top, size, size, '0', 'circle(50% at 100% 0)'),
      )
    } else {
      fillets.push(
        new SelectionRectMarker(a.left - size, b.top - size, size, size, '0', 'circle(50% at 100% 100%)'),
      )
    }
  }
  return fillets
}

class SelectionRectMarker implements LayerMarker {
  constructor(
    readonly left: number,
    readonly top: number,
    readonly width: number,
    readonly height: number,
    readonly radii: string,
    readonly clip?: string,
  ) {}

  eq(other: LayerMarker): boolean {
    return (
      other instanceof SelectionRectMarker &&
      this.left === other.left &&
      this.top === other.top &&
      this.width === other.width &&
      this.height === other.height &&
      this.radii === other.radii &&
      this.clip === other.clip
    )
  }

  draw(): HTMLElement {
    const elt = document.createElement('div')
    elt.className = 'cm-roundedSelection'
    this.adjust(elt)
    return elt
  }

  update(elt: HTMLElement): boolean {
    this.adjust(elt)
    return true
  }

  private adjust(elt: HTMLElement) {
    elt.style.left = `${this.left}px`
    elt.style.top = `${this.top}px`
    elt.style.width = `${this.width}px`
    elt.style.height = `${this.height}px`
    elt.style.borderRadius = this.radii
    elt.style.clipPath = this.clip ?? 'none'
  }
}

function lineSelectionMarkers(view: EditorView): LayerMarker[] {
  const radius = readRadius(view)
  const charWidth = view.defaultCharacterWidth
  const markers: LayerMarker[] = []
  for (const range of view.state.selection.ranges) {
    if (range.empty) continue
    const from = Math.max(range.from, view.viewport.from)
    const to = Math.min(range.to, view.viewport.to)
    if (from >= to) continue

    const rects = lineRects(view, from, to)
    for (let i = 0; i < rects.length; i += 1) {
      const rect = rects[i]
      markers.push(
        new SelectionRectMarker(
          rect.left,
          rect.top,
          rect.width,
          rect.height,
          radiiFor(rects, i, radius, charWidth),
        ),
      )
    }
    markers.push(...rightFillets(rects, radius, charWidth), ...leftFillets(rects, radius, charWidth))
  }
  return markers
}

/** Per-line rounded selection. Does not fill to the end of the line. */
export const roundedSelectionLayer = layer({
  above: false,
  class: 'cm-roundedSelectionLayer',
  markers: lineSelectionMarkers,
  update(update) {
    return (
      update.docChanged ||
      update.selectionSet ||
      update.viewportChanged ||
      update.geometryChanged
    )
  },
})
