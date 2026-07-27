import { onMounted, onUnmounted, type Ref } from 'vue'

/**
 * Close overlay only when both pointerdown and pointerup land on the overlay itself.
 * Prevents closing when the user selects text inside and releases outside.
 */
export function useOverlayDismiss(open: Ref<boolean>, onDismiss: () => void) {
  let downOnOverlay = false

  function onOverlayMouseDown(event: MouseEvent) {
    downOnOverlay = event.target === event.currentTarget
  }

  function onOverlayMouseUp(event: MouseEvent) {
    if (downOnOverlay && event.target === event.currentTarget && open.value) {
      onDismiss()
    }
    downOnOverlay = false
  }

  function onWindowMouseUp() {
    // Selection drag ended elsewhere — don't keep a stale "down" intent.
    downOnOverlay = false
  }

  onMounted(() => {
    window.addEventListener('mouseup', onWindowMouseUp)
  })

  onUnmounted(() => {
    window.removeEventListener('mouseup', onWindowMouseUp)
  })

  return { onOverlayMouseDown, onOverlayMouseUp }
}

/**
 * Dismiss popovers/menus only when the gesture both starts and ends outside roots.
 */
export function bindOutsidePointerDismiss(
  isActive: () => boolean,
  getRoots: () => Array<HTMLElement | null | undefined>,
  onOutside: () => void,
) {
  let downOutside = false

  function contains(target: EventTarget | null) {
    if (!(target instanceof Node)) return false
    return getRoots().some((root) => !!root && root.contains(target))
  }

  function onPointerDown(event: PointerEvent) {
    if (!isActive()) return
    downOutside = !contains(event.target)
  }

  function onPointerUp(event: PointerEvent) {
    if (!isActive() || !downOutside) {
      downOutside = false
      return
    }
    const upOutside = !contains(event.target)
    downOutside = false
    if (upOutside) onOutside()
  }

  document.addEventListener('pointerdown', onPointerDown, true)
  document.addEventListener('pointerup', onPointerUp, true)

  return () => {
    document.removeEventListener('pointerdown', onPointerDown, true)
    document.removeEventListener('pointerup', onPointerUp, true)
  }
}
