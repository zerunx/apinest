import { nextTick, reactive, ref, watch, type Ref } from 'vue'
import { computeDialogPosition, type DialogAnchor } from './useDialogAnchor'

export function useAnchoredDialog(
  open: Ref<boolean>,
  anchor: Ref<DialogAnchor | null | undefined>,
) {
  const dialogRef = ref<HTMLElement | null>(null)
  const dialogStyle = reactive({
    top: '0px',
    left: '0px',
  })

  async function placeDialog() {
    await nextTick()
    const el = dialogRef.value
    if (!el) return
    const rect = el.getBoundingClientRect()
    const width = rect.width || el.offsetWidth || 360
    const height = rect.height || el.offsetHeight || 180

    if (anchor.value) {
      const pos = computeDialogPosition(anchor.value, width, height)
      dialogStyle.left = `${pos.left}px`
      dialogStyle.top = `${pos.top}px`
      return
    }

    dialogStyle.left = `${Math.max(12, (window.innerWidth - width) / 2)}px`
    dialogStyle.top = `${Math.max(12, (window.innerHeight - height) / 2)}px`
  }

  watch(open, (visible) => {
    if (visible) void placeDialog()
  })

  function onReposition() {
    if (open.value) void placeDialog()
  }

  return { dialogRef, dialogStyle, placeDialog, onReposition }
}
