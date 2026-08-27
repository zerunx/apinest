import { EditorSelection, type Extension } from '@codemirror/state'
import { EditorView, keymap } from '@codemirror/view'

function wrapSelectionWith(quote: string, view: EditorView): boolean {
  if (view.state.readOnly) return false
  const { state } = view
  const ranges = state.selection.ranges
  if (ranges.every((range) => range.empty)) return false

  const changes = ranges
    .filter((range) => !range.empty)
    .map((range) => ({
      from: range.from,
      to: range.to,
      insert: `${quote}${state.doc.sliceString(range.from, range.to)}${quote}`,
    }))
    .sort((a, b) => b.from - a.from)

  const newRanges = ranges.map((range) =>
    range.empty ? range : EditorSelection.range(range.from + 1, range.to + 1),
  )

  view.dispatch({
    changes,
    selection: EditorSelection.create(newRanges, state.selection.mainIndex),
  })
  return true
}

/** Wrap non-empty selections with quotes when typing " or '. */
export const wrapSelectionQuotes: Extension = keymap.of([
  { key: '"', run: (view) => wrapSelectionWith('"', view) },
  { key: "'", run: (view) => wrapSelectionWith("'", view) },
])
