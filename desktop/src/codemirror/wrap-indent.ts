import { RangeSetBuilder } from '@codemirror/state'
import {
  Decoration,
  EditorView,
  ViewPlugin,
  type DecorationSet,
  type ViewUpdate,
} from '@codemirror/view'

function leadingColumns(text: string, tabSize: number): number {
  let col = 0
  for (let i = 0; i < text.length; i += 1) {
    if (text[i] === ' ') col += 1
    else if (text[i] === '\t') col += tabSize - (col % tabSize)
    else break
  }
  return col
}

function wrapIndentDecorations(view: EditorView): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>()
  const tabSize = view.state.tabSize
  let lastFrom = -1

  for (const { from, to } of view.visibleRanges) {
    let pos = from
    while (pos <= to) {
      const line = view.state.doc.lineAt(pos)
      pos = line.to + 1
      if (line.from === lastFrom) continue
      lastFrom = line.from
      const cols = leadingColumns(line.text, tabSize)
      if (cols <= 0) continue
      builder.add(
        line.from,
        line.from,
        Decoration.line({
          class: 'cm-wrapIndent',
          attributes: { style: `--cm-wrap-indent: ${cols}ch` },
        }),
      )
    }
  }
  return builder.finish()
}

/** Wrapped continuations keep the line's leading indent. First line is unchanged. */
export const wrappedLineIndent = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet
    constructor(view: EditorView) {
      this.decorations = wrapIndentDecorations(view)
    }
    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = wrapIndentDecorations(update.view)
      }
    }
  },
  { decorations: (value) => value.decorations },
)
