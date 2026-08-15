import type { EditorView } from '@codemirror/view'
import { formatJsonLenient } from '../utils/raw-json-utils'

/** Pretty-print the document as JSON. Returns false when nothing changed. */
export function formatJsonDocument(view: EditorView): boolean {
  const current = view.state.doc.toString()
  const formatted = formatJsonLenient(current.trim())
  if (!formatted || formatted === current) return false
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: formatted },
    userEvent: 'format',
  })
  return true
}
