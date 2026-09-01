import { EditorView, layer, type LayerMarker } from '@codemirror/view'

function layerOrigin(view: EditorView): { left: number; top: number } {
  const scrollRect = view.scrollDOM.getBoundingClientRect()
  return {
    left: scrollRect.left - view.scrollDOM.scrollLeft * view.scaleX,
    top: scrollRect.top - view.scrollDOM.scrollTop * view.scaleY,
  }
}

class SelectedSpaceMarker implements LayerMarker {
  constructor(
    readonly left: number,
    readonly top: number,
    readonly size: number,
  ) {}

  eq(other: LayerMarker): boolean {
    return (
      other instanceof SelectedSpaceMarker &&
      this.left === other.left &&
      this.top === other.top &&
      this.size === other.size
    )
  }

  draw(): HTMLElement {
    const elt = document.createElement('div')
    elt.className = 'cm-selectedSpaceDot'
    this.adjust(elt)
    return elt
  }

  update(elt: HTMLElement): boolean {
    if (!elt.classList.contains('cm-selectedSpaceDot')) return false
    this.adjust(elt)
    return true
  }

  private adjust(elt: HTMLElement) {
    elt.style.left = `${this.left}px`
    elt.style.top = `${this.top}px`
    elt.style.width = `${this.size}px`
    elt.style.height = `${this.size}px`
  }
}

function selectedSpaceMarkers(view: EditorView): LayerMarker[] {
  const origin = layerOrigin(view)
  const markers: LayerMarker[] = []
  const size = 2

  for (const range of view.state.selection.ranges) {
    if (range.empty) continue
    const from = Math.max(range.from, view.viewport.from)
    const to = Math.min(range.to, view.viewport.to)
    if (from >= to) continue
    const text = view.state.doc.sliceString(from, to)
    for (let i = 0; i < text.length; i += 1) {
      if (text[i] !== ' ') continue
      const pos = from + i
      const start = view.coordsAtPos(pos, 1)
      const end = view.coordsAtPos(pos + 1, -1)
      if (!start || !end) continue
      const width = Math.max(0, end.left - start.left)
      const height = Math.max(0, start.bottom - start.top)
      if (width < 0.5 || height < 0.5) continue
      const left = (start.left + end.left) / 2 - origin.left - size / 2
      const top = start.top + height / 2 - origin.top - size / 2
      markers.push(new SelectedSpaceMarker(left, top, size))
    }
  }
  return markers
}

/**
 * Draw selection space dots on a separate layer.
 * Avoid Decoration.mark on content — wrapping spaces in spans splits text runs and
 * makes adjacent CJK glyphs jitter while dragging a selection.
 */
export const selectedSpaceHighlighter = layer({
  above: true,
  class: 'cm-selectedSpaceLayer',
  markers: selectedSpaceMarkers,
  update(update) {
    return (
      update.selectionSet ||
      update.docChanged ||
      update.viewportChanged ||
      update.geometryChanged
    )
  },
})
