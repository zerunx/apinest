import { reactive } from 'vue'

type ToastType = 'info' | 'success' | 'warning' | 'error'

const state = reactive({
  visible: false,
  message: '',
  type: 'info' as ToastType,
})

let hideTimer: ReturnType<typeof setTimeout> | null = null

export function useToast() {
  function hideToast() {
    state.visible = false
    if (hideTimer) {
      clearTimeout(hideTimer)
      hideTimer = null
    }
  }

  function showToast(message: string, duration?: number, type: ToastType = 'info') {
    const text = message.trim()
    if (!text) return
    const ms =
      duration ??
      (type === 'error' ? 2800 : type === 'warning' ? 2200 : 1500)
    state.message = text
    state.type = type
    state.visible = true
    if (hideTimer) clearTimeout(hideTimer)
    hideTimer = setTimeout(() => {
      state.visible = false
      hideTimer = null
    }, ms)
  }

  return { toast: state, showToast, hideToast }
}
