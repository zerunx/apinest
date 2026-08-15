import type { Extension } from '@codemirror/state'
import { html } from '@codemirror/lang-html'
import { json } from '@codemirror/lang-json'
import { xml } from '@codemirror/lang-xml'

export type EditorLanguage = 'JSON' | 'Text' | 'XML' | 'HTML'

export function languageExtension(language: EditorLanguage): Extension {
  if (language === 'JSON') return json()
  if (language === 'XML') return xml()
  if (language === 'HTML') return html()
  return []
}

export function toEditorLanguage(value?: string): EditorLanguage {
  const normalized = (value ?? '').trim()
  if (normalized === 'XML' || normalized.toLowerCase().includes('xml')) return 'XML'
  if (normalized === 'HTML' || normalized.toLowerCase().includes('html')) return 'HTML'
  if (normalized === 'Text' || normalized.toLowerCase().includes('plain')) return 'Text'
  if (normalized === 'JSON' || normalized === 'Object' || normalized === 'Array') return 'JSON'
  if (normalized.toLowerCase().includes('json')) return 'JSON'
  return 'JSON'
}
