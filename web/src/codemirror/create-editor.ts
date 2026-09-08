import { Compartment, EditorSelection, EditorState, type Extension } from '@codemirror/state'
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
import { editorTheme } from './theme'
import { wrapSelectionQuotes } from './wrap-selection-quotes'

const INDENT = '    '

function editableExtension(readonly: boolean): Extension {
  return [EditorView.editable.of(!readonly), EditorState.readOnly.of(readonly)]
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

/**
 * Empty space below the last line is still part of the box, but CodeMirror's
 * precise hit-test misses it. Place / extend the selection ourselves so
 * click-drag and double-click-drag keep working.
 */
function clickAnywhereCursor(): Extension {
  return EditorView.domEventHandlers({
    mousedown(event, view) {
      if (event.button !== 0) return false
      const target = event.target as HTMLElement | null
      if (target?.closest('.cm-gutters')) return false
      const coords = { x: event.clientX, y: event.clientY }
      if (view.posAtCoords(coords) != null) return false
      const start = view.posAtCoords(coords, false)
      if (start == null) return false

      view.focus()

      let anchor = start
      if (event.detail >= 3) {
        const line = view.state.doc.lineAt(start)
        anchor = line.from
        view.dispatch({ selection: EditorSelection.range(line.from, line.to) })
      } else if (event.detail === 2) {
        const word = view.state.wordAt(start)
        if (word) {
          anchor = word.from
          view.dispatch({ selection: EditorSelection.range(word.from, word.to) })
        } else {
          view.dispatch({ selection: EditorSelection.cursor(start) })
        }
      } else if (event.shiftKey) {
        anchor = view.state.selection.main.anchor
        view.dispatch({ selection: EditorSelection.range(anchor, start) })
      } else {
        view.dispatch({ selection: EditorSelection.cursor(start) })
      }

      const onMove = (move: MouseEvent) => {
        const pos = view.posAtCoords({ x: move.clientX, y: move.clientY }, false)
        if (pos == null) return
        view.dispatch({
          selection: EditorSelection.range(anchor, pos),
          scrollIntoView: true,
        })
      }
      const onUp = () => {
        window.removeEventListener('mousemove', onMove)
        window.removeEventListener('mouseup', onUp)
      }
      window.addEventListener('mousemove', onMove)
      window.addEventListener('mouseup', onUp)
      return true
    },
  })
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
        indentUnit.of(INDENT),
        EditorState.tabSize.of(INDENT.length),
        indentOnInput(),
        bracketMatching(),
        codeFolding({ placeholderText: '…' }),
        language.of(languageExtension(currentLanguage)),
        lint.of(lintExtension(currentLanguage)),
        editable.of(editableExtension(options.readonly ?? false)),
        gutters.of(gutterExtension(showLineNumbers, currentLanguage)),
        wrapSelectionQuotes,
        keymap.of([...defaultKeymap, ...historyKeymap, ...foldKeymap, indentWithTab]),
        clickAnywhereCursor(),
        EditorView.contentAttributes.of({ spellcheck: 'false' }),
        EditorView.updateListener.of((update) => {
          if (syncing || !update.docChanged) return
          options.onChange?.(update.state.doc.toString())
        }),
        ...extras,
      ],
    }),
  })

  const resizeObserver = new ResizeObserver(() => {
    view.requestMeasure({
      read: () => 0,
      write() {},
    })
  })
  resizeObserver.observe(view.dom)

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
      resizeObserver.disconnect()
      view.destroy()
    },
  }
}
