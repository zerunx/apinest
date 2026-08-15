import { linter, type Diagnostic } from '@codemirror/lint'
import type { EditorView } from '@codemirror/view'
import type { Extension } from '@codemirror/state'
import { findJsonSyntaxError } from '../utils/raw-json-utils'
import type { EditorLanguage } from './languages'

function diagnosticRange(text: string, index: number): { from: number; to: number } {
  if (!text.length) return { from: 0, to: 0 }
  let from = Math.min(Math.max(0, index), text.length - 1)
  let to = from + 1
  if (/\s/.test(text[from] ?? '')) return { from, to }
  while (to < text.length && /[^\s,{}[\]:]/.test(text[to] ?? '')) to += 1
  return { from, to }
}

function jsonLint(view: EditorView): Diagnostic[] {
  const text = view.state.doc.toString()
  const err = findJsonSyntaxError(text)
  if (!err) return []
  const { from, to } = diagnosticRange(text, err.index)
  return [
    {
      from,
      to,
      severity: 'error',
      message: '',
    },
  ]
}

export function lintExtension(language: EditorLanguage): Extension {
  if (language !== 'JSON') return []
  return linter(jsonLint, { delay: 200, tooltipFilter: () => [] })
}
