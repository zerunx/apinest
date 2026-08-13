import type { Extension } from '@codemirror/state'
import { json } from '@codemirror/lang-json'
import { StreamLanguage } from '@codemirror/language'
import { shell } from '@codemirror/legacy-modes/mode/shell'

export type EditorLanguage = 'JSON' | 'Text' | 'CURL'

export function languageExtension(language: EditorLanguage): Extension {
  if (language === 'JSON') return json()
  if (language === 'CURL') return StreamLanguage.define(shell)
  return []
}
