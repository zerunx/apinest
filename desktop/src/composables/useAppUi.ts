import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SessionInfo } from '../types'

const session = ref<SessionInfo | null>(null)
const settingsOpen = ref(false)

async function refreshSession() {
  try {
    session.value = await invoke('get_session')
  } catch {
    session.value = null
  }
}

export function useAppUi() {
  return {
    session,
    settingsOpen,
    refreshSession,
    openSettings() {
      settingsOpen.value = true
    },
    closeSettings() {
      settingsOpen.value = false
    },
  }
}
