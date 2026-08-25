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
  const fullHeight = block.height * view.scaleY
  const rowHeight = view.defaultLineHeight * view.scaleY
  if (fullHeight > rowHeight * 1.4) {
    return rect
  }
  return {
    ...rect,
    top: markerTop(view, block.top),
    height: fullHeight,
  }
}

/** Map a wrap-row rect onto the editor's line-height grid so wrapped and
 *  unwrapped rows share the same height. Do not stretch to the line block —
 *  that block can be a few pixels taller than N rows and shows up as extra
 *  height under the last wrap while dragging. */
function snapToVisualRow(rect: SelRect, lineTop: number, rowHeight: number): SelRect {
  const idx = Math.max(0, Math.round((rect.top + rect.height / 2 - lineTop) / rowHeight))
  return { ...rect, top: lineTop + idx * rowHeight, height: rowHeight }
}

function layerLeft(view: EditorView): number {
  const rect = view.scrollDOM.getBoundingClientRect()
  return rect.left - view.scrollDOM.scrollLeft * view.scaleX
}

function posLeft(view: EditorView, pos: number): number | null {
  const coords = view.coordsAtPos(pos, 1)
  if (!coords) return null
  return coords.left - layerLeft(view)
}

/** One rect per visual wrap row (same top). */
function mergeVisualRows(pieces: SelRect[]): SelRect[] {
  if (pieces.length <= 1) return pieces
  const rows: SelRect[] = []
  const sorted = [...pieces].sort((a, b) => a.top - b.top || a.left - b.left)
  for (const piece of sorted) {
    const row = rows[rows.length - 1]
    if (row && Math.abs(row.top - piece.top) <= ALIGN_EPS) {
      const right = Math.max(rectRight(row), rectRight(piece))
      row.left = Math.min(row.left, piece.left)
      row.width = right - row.left
      row.height = Math.max(row.height, piece.height)
    } else {
      rows.push({ ...piece })
    }
  }
  return rows
}

/**
 * Fill hanging wrap-row gaps so a partial drag through later lines keeps a
 * straight left edge. Do not stretch to the content right — that paints an
 * extra full-width slab on the following lines (图一).
 */
function completeLineSelection(
  view: EditorView,
  pieces: SelRect[],
  lineFrom: number,
  rangeStart: number,
): SelRect[] {
  if (!pieces.length) return pieces
  const rows = mergeVisualRows(pieces)
  const lineLeft = posLeft(view, lineFrom)
  if (lineLeft == null) return rows
  const fromLineStart = rangeStart === lineFrom
  for (let i = 0; i < rows.length; i += 1) {
    const row = rows[i]
    const fillLeft = fromLineStart || i > 0
    if (fillLeft && lineLeft < row.left - ALIGN_EPS) {
      const rowRight = rectRight(row)
      row.width = rowRight - lineLeft
      row.left = lineLeft
    }
  }
  return rows
}

function rangePieces(view: EditorView, from: number, to: number) {
  if (from >= to) return []
  return RectangleMarker.forRange(view, 'cm-roundedSelection', EditorSelection.range(from, to))
}

/** End of the visual wrap row that contains `pos` (exclusive). */
function wrapRowEnd(view: EditorView, pos: number, lineTo: number): number {
  const next = view.moveToLineBoundary(EditorSelection.cursor(pos, 1), true, true).head
  if (next > pos) return Math.min(lineTo, next)
  return Math.min(lineTo, pos + 1)
}

/**
 * Measure each wrap row on its own. `RectangleMarker.forRange` across wrap
 * points inserts a full-width "between" rect, which looks like an extra
 * background layer on the row below the caret.
 */
function wrappedRangePieces(view: EditorView, start: number, end: number, lineTo: number) {
  const pieces: ReturnType<typeof RectangleMarker.forRange> = []
  let pos = start
  let guard = 0
  while (pos < end && guard < 10000) {
    guard += 1
    const rowTo = wrapRowEnd(view, pos, lineTo)
    const sliceTo = Math.min(end, rowTo)
    if (sliceTo > pos) pieces.push(...rangePieces(view, pos, sliceTo))
    pos = rowTo > pos ? rowTo : pos + 1
  }
  return pieces
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
      const block = view.lineBlockAt(line.from)
      const lineTop = markerTop(view, block.top)
      const lineBottom = lineTop + block.height * view.scaleY
      const rowHeight = view.defaultLineHeight * view.scaleY
      const wrapped = lineBottom - lineTop > rowHeight * 1.4
      const pieces = wrapped
        ? wrappedRangePieces(view, start, end, line.to)
        : rangePieces(view, start, end)
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
        linePieces.push(
          wrapped ? snapToVisualRow(rect, lineTop, rowHeight) : withLineHeight(view, start, rect),
        )
      }
      if (includesNewline) {
        const last = linePieces[linePieces.length - 1]
        if (last) last.width += view.defaultCharacterWidth
        else {
          const extra = newlineAtLineEnd(view, line.from, line.to)
          if (extra) linePieces.push(extra)
        }
      }
      rects.push(...completeLineSelection(view, linePieces, line.from, start))
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

function filletSize(_radius: number, gap: number, a: SelRect, b: SelRect): number {
  const size = 2
  if (gap < size || a.height < size || b.height < size) return 0
  return size
}

/** Which quadrant of a circle to show. Inner join sits at the opposite corner. */
type FilletCorner = 'br' | 'tr' | 'bl' | 'tl'

const FILLET_SOLID = 'var(--raw-editor-selection)'
const FILLET_SOFT = 'color-mix(in srgb, var(--raw-editor-selection) 40%, transparent)'

/** 2x2 cells in row-major order: TL, TR, BL, BR. 1=transparent, 2=soft, 3=selection. */
function filletCells(corner: FilletCorner): [string, string, string, string] {
  switch (corner) {
    case 'br':
      return [FILLET_SOLID, FILLET_SOFT, FILLET_SOFT, 'transparent']
    case 'tr':
      return [FILLET_SOFT, 'transparent', FILLET_SOLID, FILLET_SOFT]
    case 'bl':
      return [FILLET_SOFT, FILLET_SOLID, 'transparent', FILLET_SOFT]
    case 'tl':
      return ['transparent', FILLET_SOFT, FILLET_SOFT, FILLET_SOLID]
  }
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
    if (leftDocked || ((rightDocked || overlap) && prev.left < cur.left)) tl = 0
    if (rightDocked || ((leftDocked || overlap) && prevRight > right)) tr = 0
  }
  if (next) {
    const nextRight = rectRight(next)
    const leftDocked = aligned(next.left, cur.left)
    const rightDocked = aligned(nextRight, right)
    const overlap = stronglyOverlaps(next, cur, charWidth)
    if (leftDocked || ((rightDocked || overlap) && next.left < cur.left)) bl = 0
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
    const size = filletSize(radius, gap, a, b)
    if (size <= 0) continue
    if (aRight > bRight) {
      fillets.push(new SelectionRectMarker(bRight, b.top, size, size, '0', 'br'))
    } else {
      fillets.push(new SelectionRectMarker(aRight, b.top - size, size, size, '0', 'tr'))
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
    const size = filletSize(radius, gap, a, b)
    if (size <= 0) continue
    if (a.left < b.left) {
      fillets.push(new SelectionRectMarker(b.left - size, b.top, size, size, '0', 'bl'))
    } else {
      fillets.push(new SelectionRectMarker(a.left - size, b.top - size, size, size, '0', 'tl'))
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
    readonly fillet?: FilletCorner,
  ) {}

  eq(other: LayerMarker): boolean {
    return (
      other instanceof SelectionRectMarker &&
      this.left === other.left &&
      this.top === other.top &&
      this.width === other.width &&
      this.height === other.height &&
      this.radii === other.radii &&
      this.fillet === other.fillet
    )
  }

  draw(): HTMLElement {
    const elt = document.createElement('div')
    if (this.fillet) {
      elt.className = 'cm-roundedSelectionFillet'
      for (let i = 0; i < 4; i += 1) elt.appendChild(document.createElement('div'))
    } else {
      elt.className = 'cm-roundedSelection'
    }
    this.adjust(elt)
    return elt
  }

  update(elt: HTMLElement): boolean {
    const isFillet = Boolean(this.fillet)
    if (isFillet !== elt.classList.contains('cm-roundedSelectionFillet')) return false
    if (isFillet && elt.childElementCount !== 4) return false
    this.adjust(elt)
    return true
  }

  private adjust(elt: HTMLElement) {
    elt.style.left = `${this.left}px`
    elt.style.top = `${this.top}px`
    elt.style.width = `${this.width}px`
    elt.style.height = `${this.height}px`
    elt.style.borderRadius = this.radii
    elt.style.clipPath = 'none'
    elt.style.removeProperty('background-image')
    elt.style.removeProperty('transform')
    elt.style.removeProperty('transform-origin')
    if (this.fillet) {
      elt.style.display = 'grid'
      elt.style.gridTemplateColumns = '1px 1px'
      elt.style.gridTemplateRows = '1px 1px'
      elt.style.overflow = 'hidden'
      elt.style.backgroundColor = 'transparent'
      const colors = filletCells(this.fillet)
      for (let i = 0; i < 4; i += 1) {
        const cell = elt.children[i] as HTMLElement | undefined
        if (cell) cell.style.backgroundColor = colors[i]
      }
    } else {
      elt.style.removeProperty('display')
      elt.style.removeProperty('grid-template-columns')
      elt.style.removeProperty('grid-template-rows')
      elt.style.overflow = 'visible'
      elt.style.removeProperty('background-color')
    }
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
