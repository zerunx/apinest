import { Compartment, EditorState, type Extension } from '@codemirror/state'
import { EditorView, keymap, lineNumbers, placeholder as placeholderExt, drawSelection } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands'
import {
  bracketMatching,
  codeFolding,
  foldGutter,
  foldKeymap,
  indentOnInput,
  indentUnit,
} from '@codemirror/language'
import { languageExtension, type EditorLanguage } from './languages'
import { formatJsonDocument } from './format-json'
import { selectedSpaceHighlighter } from './selected-space'
import { roundedSelectionLayer } from './rounded-selection'
import { lintExtension } from './json-lint'
import { searchExtension } from './search-panel'
import { wrappedLineIndent } from './wrap-indent'
import { editorTheme } from './theme'
import { wrapSelectionQuotes } from './wrap-selection-quotes'

const INDENT = '    '

function editableExtension(readonly: boolean): Extension {
  // Keep editable so focus, caret, and shortcuts (e.g. Ctrl+A) still work;
  // readOnly alone blocks content changes.
  return [EditorView.editable.of(true), EditorState.readOnly.of(readonly)]
}

function foldMarker(open: boolean) {
  const marker = document.createElement('span')
  marker.className = open ? 'cm-fold-chevron' : 'cm-fold-chevron cm-fold-chevron--closed'
  marker.setAttribute('aria-hidden', 'true')
  return marker
}

function gutterExtension(showLineNumbers: boolean, language: EditorLanguage): Extension {
  if (!showLineNumbers) return []
  const gutters: Extension[] = [lineNumbers()]
  if (language === 'JSON') {
    gutters.push(foldGutter({ markerDOM: foldMarker }))
  }
  return gutters
}

export type CreateCodeMirrorOptions = {
  parent: HTMLElement
  doc?: string
  language?: EditorLanguage
  readonly?: boolean
  lineNumbers?: boolean
  placeholder?: string
  dark?: boolean
  extensions?: Extension[]
  onChange?: (value: string) => void
}

export type CodeMirrorHandle = {
  view: EditorView
  getDoc: () => string
  setDoc: (text: string) => void
  setLanguage: (language: EditorLanguage) => void
  setReadonly: (readonly: boolean) => void
  setLineNumbers: (show: boolean) => void
  setDark: (dark: boolean) => void
  formatJson: () => boolean
  destroy: () => void
}

/** Vanilla CodeMirror 6 editor. Add themes / extra extensions at the call site later. */
export function createCodeMirror(options: CreateCodeMirrorOptions): CodeMirrorHandle {
  const language = new Compartment()
  const editable = new Compartment()
  const gutters = new Compartment()
  const lint = new Compartment()
  const theme = new Compartment()
  let currentLanguage: EditorLanguage = options.language ?? 'JSON'
  let showLineNumbers = options.lineNumbers !== false
  let syncing = false

  const extras: Extension[] = []
  if (options.placeholder) extras.push(placeholderExt(options.placeholder))
  if (options.extensions?.length) extras.push(options.extensions)

  const view = new EditorView({
    parent: options.parent,
    state: EditorState.create({
      doc: options.doc ?? '',
      extensions: [
        theme.of(editorTheme(options.dark ?? false)),
        selectedSpaceHighlighter,
        roundedSelectionLayer,
        history(),
        drawSelection({ cursorBlinkRate: 1200 }),
        EditorView.lineWrapping,
        wrappedLineIndent,
        indentUnit.of(INDENT),
        EditorState.tabSize.of(INDENT.length),
        indentOnInput(),
        bracketMatching(),
        codeFolding({ placeholderText: '…' }),
        language.of(languageExtension(currentLanguage)),
        lint.of(lintExtension(currentLanguage)),
        editable.of(editableExtension(options.readonly ?? false)),
        gutters.of(gutterExtension(showLineNumbers, currentLanguage)),
        keymap.of([wrapSelectionQuotes, ...defaultKeymap, ...historyKeymap, ...foldKeymap, indentWithTab]),
        searchExtension(),
        EditorView.contentAttributes.of({ spellcheck: 'false' }),
        EditorView.updateListener.of((update) => {
          if (syncing || !update.docChanged) return
          options.onChange?.(update.state.doc.toString())
        }),
        ...extras,
      ],
    }),
  })

  return {
    view,
    getDoc() {
      return view.state.doc.toString()
    },
    setDoc(text) {
      if (view.state.doc.toString() === text) return
      syncing = true
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: text },
      })
      syncing = false
    },
    setLanguage(next) {
      currentLanguage = next
      view.dispatch({
        effects: [
          language.reconfigure(languageExtension(next)),
          lint.reconfigure(lintExtension(next)),
          gutters.reconfigure(gutterExtension(showLineNumbers, next)),
        ],
      })
    },
    setReadonly(readonly) {
      view.dispatch({
        effects: editable.reconfigure(editableExtension(readonly)),
      })
    },
    setLineNumbers(show) {
      showLineNumbers = show
      view.dispatch({
        effects: gutters.reconfigure(gutterExtension(show, currentLanguage)),
      })
    },
    setDark(dark) {
      view.dispatch({
        effects: theme.reconfigure(editorTheme(dark)),
      })
    },
    formatJson() {
      return formatJsonDocument(view)
    },
    destroy() {
      view.destroy()
    },
  }
}
