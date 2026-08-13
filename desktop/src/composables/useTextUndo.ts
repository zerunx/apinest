export type TextUndoSnapshot = {
  text: string
  start: number
  end: number
}

type Options = {
  maxDepth?: number
  /** Typing changes coalesce into one undo step after this idle ms. */
  coalesceMs?: number
}

/**
 * Undo/redo stack for text fields where Vue v-model or programmatic
 * updates break the browser's native undo history.
 */
export function createTextUndo(options: Options = {}) {
  const maxDepth = options.maxDepth ?? 100
  const coalesceMs = options.coalesceMs ?? 400

  let stack: TextUndoSnapshot[] = []
  let index = -1
  let applying = false
  let coalesceTimer: ReturnType<typeof setTimeout> | null = null

  function clone(snap: TextUndoSnapshot): TextUndoSnapshot {
    return { text: snap.text, start: snap.start, end: snap.end }
  }

  function clearCoalesce() {
    if (coalesceTimer != null) {
      clearTimeout(coalesceTimer)
      coalesceTimer = null
    }
  }

  function pushImmediate(snap: TextUndoSnapshot) {
    const top = stack[index]
    if (top && top.text === snap.text) {
      stack[index] = clone(snap)
      return
    }
    stack = stack.slice(0, index + 1)
    stack.push(clone(snap))
    if (stack.length > maxDepth) {
      stack.shift()
    }
    index = stack.length - 1
  }

  /** Replace history with a single baseline (load / external sync). */
  function reset(snap: TextUndoSnapshot) {
    clearCoalesce()
    stack = [clone(snap)]
    index = 0
  }

  /**
   * Ensure `snap` (usually current value before an edit) is on the stack,
   * then clear redo. Use before discrete edits (Tab, Enter, Format, paste).
   */
  function checkpoint(snap: TextUndoSnapshot) {
    if (applying) return
    clearCoalesce()
    pushImmediate(snap)
  }

  /**
   * After a discrete edit, record the new state immediately and drop redo.
   */
  function commit(snap: TextUndoSnapshot) {
    if (applying) return
    clearCoalesce()
    pushImmediate(snap)
  }

  /**
   * Record typing changes with debounce so a burst of keystrokes is one step.
   * Call with the latest snapshot after each input.
   */
  function noteInput(snap: TextUndoSnapshot) {
    if (applying) return
    clearCoalesce()
    coalesceTimer = setTimeout(() => {
      coalesceTimer = null
      pushImmediate(snap)
    }, coalesceMs)
  }

  function flush(snap?: TextUndoSnapshot) {
    if (coalesceTimer == null) return
    clearCoalesce()
    if (snap) pushImmediate(snap)
  }

  function canUndo() {
    return index > 0
  }

  function canRedo() {
    return index >= 0 && index < stack.length - 1
  }

  function undo(current: TextUndoSnapshot): TextUndoSnapshot | null {
    flush(current)
    if (index <= 0) return null
    const top = stack[index]
    if (!top || top.text !== current.text) {
      pushImmediate(current)
    }
    if (index <= 0) return null
    applying = true
    index -= 1
    const snap = clone(stack[index])
    applying = false
    return snap
  }

  function redo(current: TextUndoSnapshot): TextUndoSnapshot | null {
    flush(current)
    if (index >= stack.length - 1) return null
    applying = true
    index += 1
    const snap = clone(stack[index])
    applying = false
    return snap
  }

  function isApplying() {
    return applying
  }

  function dispose() {
    clearCoalesce()
    stack = []
    index = -1
  }

  return {
    reset,
    checkpoint,
    commit,
    noteInput,
    flush,
    undo,
    redo,
    canUndo,
    canRedo,
    isApplying,
    dispose,
  }
}

export type TextUndoApi = ReturnType<typeof createTextUndo>

/** True for Ctrl/Cmd+Z (undo) or Ctrl/Cmd+Shift+Z / Ctrl+Y (redo). */
export function matchUndoRedoKey(
  event: KeyboardEvent,
): 'undo' | 'redo' | null {
  if (!(event.ctrlKey || event.metaKey) || event.altKey || event.isComposing) return null
  const key = event.key.toLowerCase()
  if (key === 'y' && !event.shiftKey && !event.metaKey) return 'redo'
  if (key === 'z') return event.shiftKey ? 'redo' : 'undo'
  return null
}
