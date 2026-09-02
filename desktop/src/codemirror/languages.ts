import type { Extension } from '@codemirror/state'
import { json, jsonLanguage } from '@codemirror/lang-json'
import { StreamLanguage } from '@codemirror/language'
import { shell } from '@codemirror/legacy-modes/mode/shell'
import { jsonCommentsExtension } from './json-comments'

export type EditorLanguage = 'JSON' | 'Text' | 'CURL'

function jsonLanguageExtension(): Extension {
  return [
    json(),
    jsonLanguage.data.of({
      commentTokens: { line: '//', block: { open: '/*', close: '*/' } },
    }),
    jsonCommentsExtension(),
  ]
}

export function languageExtension(language: EditorLanguage): Extension {
  if (language === 'JSON') return jsonLanguageExtension()
  if (language === 'CURL') return StreamLanguage.define(shell)
  return []
}
