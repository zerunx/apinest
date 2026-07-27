export type DialogAnchor = {
  x: number
  y: number
}

export function anchorFromEvent(event: MouseEvent, mode: 'click' | 'element' = 'element') {
  const el = event.currentTarget
  if (el instanceof HTMLElement) {
    const rect = el.getBoundingClientRect()
    return { x: rect.left, y: rect.bottom + 4 }
  }
  if (mode === 'click') {
    return { x: event.clientX, y: event.clientY + 4 }
  }
  return { x: event.clientX, y: event.clientY + 4 }
}

export function computeDialogPosition(
  anchor: DialogAnchor,
  width: number,
  height: number,
  padding = 12,
) {
  let left = anchor.x
  let top = anchor.y

  if (left + width > window.innerWidth - padding) {
    left = Math.max(padding, window.innerWidth - width - padding)
  }
  if (left < padding) left = padding

  if (top + height > window.innerHeight - padding) {
    top = Math.max(padding, anchor.y - height - 8)
  }
  if (top < padding) top = padding

  return { left, top }
}
