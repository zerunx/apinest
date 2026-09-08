import { reactive } from 'vue'

type ToastType = 'info' | 'success' | 'warning' | 'error'

const state = reactive({
  visible: false,
  message: '',
  type: 'info' as ToastType,
})

let hideTimer: ReturnType<typeof setTimeout> | null = null
let remainingMs = 0
let hideStartedAt = 0
let paused = false

function clearHideTimer() {
  if (hideTimer) {
    clearTimeout(hideTimer)
    hideTimer = null
  }
}

function scheduleHide(ms: number) {
  clearHideTimer()
  remainingMs = Math.max(0, ms)
  if (remainingMs <= 0) {
    state.visible = false
    return
  }
  hideStartedAt = Date.now()
  hideTimer = setTimeout(() => {
    state.visible = false
    hideTimer = null
    remainingMs = 0
  }, remainingMs)
}

export function useToast() {
  function hideToast() {
    state.visible = false
    paused = false
    remainingMs = 0
    clearHideTimer()
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
    paused = false
    scheduleHide(ms)
  }

  function pauseToast() {
    if (!state.visible || paused) return
    paused = true
    if (hideTimer) {
      remainingMs = Math.max(0, remainingMs - (Date.now() - hideStartedAt))
      clearHideTimer()
    }
  }

  function resumeToast() {
    if (!state.visible || !paused) return
    paused = false
    // Give a short buffer after leaving so copy (Ctrl+C) can finish.
    scheduleHide(Math.max(remainingMs, 1200))
  }

  return { toast: state, showToast, hideToast, pauseToast, resumeToast }
}
