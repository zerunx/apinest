import { RangeSetBuilder } from '@codemirror/state'
import {
  Decoration,
  EditorView,
  ViewPlugin,
  type DecorationSet,
  type ViewUpdate,
} from '@codemirror/view'

const selectedSpaceMark = Decoration.mark({ class: 'cm-selected-space' })

function selectedSpaces(view: EditorView): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>()
  for (const range of view.state.selection.ranges) {
    if (range.empty) continue
    const from = Math.max(range.from, view.viewport.from)
    const to = Math.min(range.to, view.viewport.to)
    if (from >= to) continue
    const text = view.state.doc.sliceString(from, to)
    for (let i = 0; i < text.length; i += 1) {
      if (text[i] !== ' ') continue
      const pos = from + i
      builder.add(pos, pos + 1, selectedSpaceMark)
    }
  }
  return builder.finish()
}

/** Show a small dot on spaces that are inside the current selection. */
export const selectedSpaceHighlighter = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet
    constructor(view: EditorView) {
      this.decorations = selectedSpaces(view)
    }
    update(update: ViewUpdate) {
      if (update.selectionSet || update.docChanged || update.viewportChanged) {
        this.decorations = selectedSpaces(update.view)
      }
    }
  },
  { decorations: (value) => value.decorations },
)
