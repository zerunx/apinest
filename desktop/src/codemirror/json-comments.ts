import { Decoration, EditorView, ViewPlugin, type DecorationSet, type ViewUpdate } from '@codemirror/view'
import { Prec, RangeSetBuilder, type Extension } from '@codemirror/state'
import { readJsonStringEnd } from '../utils/raw-json-utils'

const commentMark = Decoration.mark({ class: 'cm-jsonComment' })
const commentLine = Decoration.line({ class: 'cm-jsonCommentLine' })

function buildCommentDecorations(docText: string): DecorationSet {
  type Item = { from: number; to?: number; deco: Decoration }
  const items: Item[] = []

  let i = 0
  let lineStart = 0
  while (i <= docText.length) {
    const atEnd = i >= docText.length
    const char = atEnd ? '\n' : docText[i]

    if (char === '\n' || atEnd) {
      const line = docText.slice(lineStart, i)
      const trimmed = line.match(/^[ \t]*/)
      const contentAt = lineStart + (trimmed?.[0].length ?? 0)
      if (docText.startsWith('//', contentAt) && contentAt < i) {
        items.push({ from: lineStart, deco: commentLine })
      }
      if (atEnd) break
      i += 1
      lineStart = i
      continue
    }

    if (char === '"') {
      i = readJsonStringEnd(docText, i)
      continue
    }
    if (char === '/' && docText[i + 1] === '/') {
      const from = i
      i += 2
      while (i < docText.length && docText[i] !== '\n' && docText[i] !== '\r') i += 1
      if (i > from) items.push({ from, to: i, deco: commentMark })
      continue
    }
    if (char === '/' && docText[i + 1] === '*') {
      const from = i
      i += 2
      while (i + 1 < docText.length && !(docText[i] === '*' && docText[i + 1] === '/')) i += 1
      if (i + 1 < docText.length) i += 2
      else i = docText.length
      if (i > from) items.push({ from, to: i, deco: commentMark })
      continue
    }
    i += 1
  }

  items.sort((a, b) => a.from - b.from || (a.to ?? a.from) - (b.to ?? b.from))
  const builder = new RangeSetBuilder<Decoration>()
  for (const item of items) {
    if (item.to == null) builder.add(item.from, item.from, item.deco)
    else builder.add(item.from, item.to, item.deco)
  }
  return builder.finish()
}

const jsonCommentHighlighter = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet

    constructor(view: EditorView) {
      this.decorations = buildCommentDecorations(view.state.doc.toString())
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = buildCommentDecorations(update.state.doc.toString())
      }
    }
  },
  { decorations: (v) => v.decorations },
)

/** Line/block comment highlighting for JSON (overrides syntax token colors). */
export function jsonCommentsExtension(): Extension {
  return Prec.highest([
    jsonCommentHighlighter,
    EditorView.theme({
      '.cm-jsonComment, .cm-jsonComment span': {
        color: 'var(--raw-hl-comment) !important',
        fontStyle: 'italic',
      },
      '.cm-line.cm-jsonCommentLine, .cm-line.cm-jsonCommentLine span': {
        color: 'var(--raw-hl-comment) !important',
        fontStyle: 'italic',
      },
    }),
  ])
}
