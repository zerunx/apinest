import { indentLess } from '@codemirror/commands'
import { getIndentUnit, indentString } from '@codemirror/language'
import {
  countColumn,
  EditorSelection,
  type ChangeSpec,
  type EditorState,
  type SelectionRange,
  type StateCommand,
} from '@codemirror/state'
import type { KeyBinding } from '@codemirror/view'

function changeBySelectedLine(
  state: EditorState,
  f: (line: { from: number; text: string; number: number }, changes: ChangeSpec[], range: SelectionRange) => void,
) {
  let atLine = -1
  return state.changeByRange((range) => {
    const changes: ChangeSpec[] = []
    for (let pos = range.from; pos <= range.to; ) {
      const line = state.doc.lineAt(pos)
      if (line.number > atLine && (range.empty || range.to > line.from)) {
        f(line, changes, range)
        atLine = line.number
      }
      pos = line.to + 1
    }
    const changeSet = state.changes(changes)
    return {
      changes,
      range: EditorSelection.range(changeSet.mapPos(range.anchor, 1), changeSet.mapPos(range.head, 1)),
    }
  })
}

/**
 * Indent selected lines to the next indent-unit column (tab stop),
 * matching how indentLess snaps downward.
 * e.g. 2 spaces + Tab → 4 spaces (align), not 6.
 */
const indentMoreToTabStop: StateCommand = ({ state, dispatch }) => {
  if (state.readOnly) return false
  const unit = getIndentUnit(state)
  dispatch(
    state.update(
      changeBySelectedLine(state, (line, changes) => {
        const space = /^\s*/.exec(line.text)?.[0] ?? ''
        const col = countColumn(space, state.tabSize)
        const nextCol = Math.ceil((col + 1) / unit) * unit
        const insert = indentString(state, nextCol)
        let keep = 0
        while (keep < space.length && keep < insert.length && space.charCodeAt(keep) === insert.charCodeAt(keep)) {
          keep += 1
        }
        changes.push({
          from: line.from + keep,
          to: line.from + space.length,
          insert: insert.slice(keep),
        })
      }),
      { userEvent: 'input.indent' },
    ),
  )
  return true
}

/** Tab / Shift-Tab: indent to next/previous indent-unit tab stop. */
export const indentWithTabStop: KeyBinding = {
  key: 'Tab',
  run: indentMoreToTabStop,
  shift: indentLess,
}
