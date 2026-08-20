import {
  closeSearchPanel,
  findNext,
  findPrevious,
  getSearchQuery,
  search,
  searchKeymap,
  SearchQuery,
  setSearchQuery,
} from '@codemirror/search'
import { Prec, type Extension } from '@codemirror/state'
import { EditorView, keymap, type Panel, type ViewUpdate } from '@codemirror/view'
import { t } from '../i18n'

function matchStats(view: EditorView): { current: number; total: number } {
  const query = getSearchQuery(view.state)
  if (!query.valid || !query.search) return { current: 0, total: 0 }
  const cursor = query.getCursor(view.state)
  const sel = view.state.selection.main
  let total = 0
  let current = 0
  for (;;) {
    const step = cursor.next()
    if (step.done) break
    total++
    if (step.value.from === sel.from && step.value.to === sel.to) current = total
  }
  return { current, total }
}

function createFindPanel(view: EditorView): Panel {
  const wrap = document.createElement('div')
  wrap.className = 'cm-find-widget'
  wrap.setAttribute('role', 'search')

  const input = document.createElement('input')
  input.className = 'cm-find-widget__input'
  input.type = 'text'
  input.placeholder = t('editor.find')
  input.setAttribute('aria-label', t('editor.find'))
  input.autocomplete = 'off'
  input.spellcheck = false
  input.setAttribute('main-field', 'true')

  const count = document.createElement('span')
  count.className = 'cm-find-widget__count'
  count.setAttribute('aria-live', 'polite')

  function iconButton(name: string, title: string, svg: string) {
    const btn = document.createElement('button')
    btn.type = 'button'
    btn.name = name
    btn.className = `cm-find-widget__btn cm-find-widget__btn--${name}`
    btn.title = title
    btn.setAttribute('aria-label', title)
    btn.innerHTML = svg
    btn.addEventListener('mousedown', (e) => e.preventDefault())
    return btn
  }

  const prev = iconButton(
    'prev',
    t('editor.findPrev'),
    '<svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true"><path fill="currentColor" d="M8 5.2 3.6 9.6l.9.9L8 7l3.5 3.5.9-.9z"/></svg>',
  )
  const next = iconButton(
    'next',
    t('editor.findNext'),
    '<svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true"><path fill="currentColor" d="M8 10.8 3.6 6.4l.9-.9L8 9l3.5-3.5.9.9z"/></svg>',
  )
  const close = iconButton(
    'close',
    t('editor.findClose'),
    '<svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true"><path fill="currentColor" d="M4.2 3.5 3.5 4.2 7.3 8l-3.8 3.8.7.7L8 8.7l3.8 3.8.7-.7L8.7 8l3.8-3.8-.7-.7L8 7.3z"/></svg>',
  )

  wrap.append(input, count, prev, next, close)

  function commitQuery(selectFirst: boolean) {
    const nextQuery = new SearchQuery({
      search: input.value,
      caseSensitive: false,
      literal: true,
    })
    const effects = [setSearchQuery.of(nextQuery)]
    view.dispatch({ effects })
    if (selectFirst && nextQuery.valid && nextQuery.search) {
      const cursor = nextQuery.getCursor(view.state, 0)
      const first = cursor.next()
      if (!first.done) {
        view.dispatch({
          selection: { anchor: first.value.from, head: first.value.to },
          scrollIntoView: true,
        })
      }
    }
    renderCount()
  }

  function renderCount() {
    const { current, total } = matchStats(view)
    if (!input.value) {
      count.textContent = ''
      count.classList.remove('cm-find-widget__count--empty')
      return
    }
    if (total === 0) {
      count.textContent = t('editor.findNoResult')
      count.classList.add('cm-find-widget__count--empty')
      return
    }
    count.textContent = `${current || 1}/${total}`
    count.classList.remove('cm-find-widget__count--empty')
  }

  input.addEventListener('input', () => commitQuery(true))
  input.addEventListener('keydown', (event) => {
    if (event.key === 'Enter') {
      event.preventDefault()
      event.stopPropagation()
      if (event.shiftKey) findPrevious(view)
      else findNext(view)
      renderCount()
    } else if (event.key === 'Escape') {
      event.preventDefault()
      event.stopPropagation()
      closeSearchPanel(view)
      view.focus()
    }
  })
  prev.addEventListener('click', () => {
    findPrevious(view)
    renderCount()
  })
  next.addEventListener('click', () => {
    findNext(view)
    renderCount()
  })
  close.addEventListener('click', () => {
    closeSearchPanel(view)
    view.focus()
  })

  return {
    dom: wrap,
    top: true,
    mount() {
      const query = getSearchQuery(view.state)
      input.value = query.search
      input.focus()
      input.select()
      renderCount()
    },
    update(update: ViewUpdate) {
      if (update.docChanged || update.selectionSet) renderCount()
    },
  }
}

export function searchExtension(): Extension {
  return [
    search({ top: true, literal: true, createPanel: createFindPanel }),
    Prec.highest(keymap.of(searchKeymap)),
  ]
}
