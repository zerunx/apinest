import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { reactive } from 'vue'

import type { HeaderPair } from '../types'

export type WsMessageItem = {
  id: string
  direction: 'in' | 'out' | 'system'
  kind: string
  messageType: string
  data: string
  at: number
}

export type WsTabSession = {
  tabId: string
  sessionId: string | null
  connected: boolean
  connecting: boolean
  disconnecting: boolean
  sending: boolean
  messages: WsMessageItem[]
  responseHeaders: HeaderPair[]
  statusText: string
  connectError: boolean
}

type WsSessionEventPayload = {
  session_id: string
  kind: 'connected' | 'sent' | 'message' | 'error' | 'closed'
  direction: string
  data: string
  message_type: string
  at_ms: number
}

const byTabId = reactive(new Map<string, WsTabSession>())
let msgSeq = 0
let eventUnlisten: UnlistenFn | null = null
let eventListenPromise: Promise<void> | null = null

function emptySession(tabId: string): WsTabSession {
  return {
    tabId,
    sessionId: null,
    connected: false,
    connecting: false,
    disconnecting: false,
    sending: false,
    messages: [],
    responseHeaders: [],
    statusText: '',
    connectError: false,
  }
}

export function ensureWsTabSession(tabId: string): WsTabSession {
  let session = byTabId.get(tabId)
  if (!session) {
    session = reactive(emptySession(tabId))
    byTabId.set(tabId, session)
  }
  return session
}

export function getWsTabSession(tabId: string): WsTabSession | undefined {
  return byTabId.get(tabId)
}

function findTabIdBySessionId(sessionId: string): string | null {
  for (const session of byTabId.values()) {
    if (session.sessionId === sessionId) return session.tabId
  }
  return null
}

/** Connect result and Connected event can race; bind the in-flight tab. */
function findConnectingTabId(): string | null {
  for (const session of byTabId.values()) {
    if (session.connecting && !session.sessionId) return session.tabId
  }
  return null
}

function appendMessage(
  session: WsTabSession,
  partial: Omit<WsMessageItem, 'id' | 'at'> & { at?: number },
) {
  msgSeq += 1
  session.messages.push({
    id: `m-${msgSeq}`,
    at: partial.at ?? Date.now(),
    direction: partial.direction,
    kind: partial.kind,
    messageType: partial.messageType,
    data: partial.data,
  })
}

function resetSessionFlags(session: WsTabSession) {
  session.connected = false
  session.connecting = false
  session.disconnecting = false
  session.sending = false
  session.sessionId = null
}

function applyEvent(session: WsTabSession, payload: WsSessionEventPayload, closedLabel: string) {
  const direction =
    payload.direction === 'in' || payload.direction === 'out' || payload.direction === 'system'
      ? payload.direction
      : 'system'

  if (payload.kind === 'connected') {
    session.connected = true
    session.connecting = false
    session.connectError = false
    appendMessage(session, {
      direction: 'system',
      kind: payload.kind,
      messageType: payload.message_type || 'system',
      data: payload.data,
    })
    return
  }

  if (payload.kind === 'sent' || payload.kind === 'message') {
    appendMessage(session, {
      direction,
      kind: payload.kind,
      messageType: payload.message_type || 'text',
      data: payload.data,
    })
    return
  }

  if (payload.kind === 'error') {
    session.connectError = true
    appendMessage(session, {
      direction: 'system',
      kind: payload.kind,
      messageType: 'error',
      data: payload.data,
    })
    return
  }

  if (payload.kind === 'closed') {
    appendMessage(session, {
      direction: 'system',
      kind: payload.kind,
      messageType: payload.message_type || 'system',
      data: payload.data || closedLabel,
    })
    resetSessionFlags(session)
    if (!session.statusText.includes('101')) {
      session.statusText = payload.data || closedLabel
    }
  }
}

export async function ensureWsEventListener(closedLabel: string): Promise<void> {
  if (eventUnlisten) return
  if (eventListenPromise) {
    await eventListenPromise
    return
  }
  eventListenPromise = (async () => {
    eventUnlisten = await listen<WsSessionEventPayload>('ws-session-event', (event) => {
      const payload = event.payload
      if (!payload?.session_id) return
      let tabId = findTabIdBySessionId(payload.session_id)
      if (!tabId && payload.kind === 'connected') {
        tabId = findConnectingTabId()
        if (tabId) {
          const pending = byTabId.get(tabId)
          if (pending) pending.sessionId = payload.session_id
        }
      }
      if (!tabId) return
      const session = byTabId.get(tabId)
      if (!session) return
      applyEvent(session, payload, closedLabel)
    })
  })()
  try {
    await eventListenPromise
  } finally {
    eventListenPromise = null
  }
}

export function clearWsMessages(tabId: string) {
  const session = byTabId.get(tabId)
  if (session) session.messages = []
}

export function appendWsTabMessage(
  tabId: string,
  partial: Omit<WsMessageItem, 'id' | 'at'> & { at?: number },
) {
  appendMessage(ensureWsTabSession(tabId), partial)
}

export async function disconnectWsTab(tabId: string): Promise<void> {
  const session = byTabId.get(tabId)
  if (!session?.sessionId) {
    if (session) resetSessionFlags(session)
    return
  }
  const sessionId = session.sessionId
  session.disconnecting = true
  try {
    await invoke('ws_disconnect', { args: { session_id: sessionId } })
  } catch {
    // Closed event / timeout below clears UI.
  } finally {
    window.setTimeout(() => {
      if (session.sessionId === sessionId) resetSessionFlags(session)
    }, 800)
  }
}

/** Drop a tab's WS binding; disconnect if it still owns a live session. */
export async function disposeWsTab(tabId: string): Promise<void> {
  const session = byTabId.get(tabId)
  if (!session) return
  if (session.sessionId) {
    try {
      await invoke('ws_disconnect', { args: { session_id: session.sessionId } })
    } catch {
      // ignore
    }
  }
  byTabId.delete(tabId)
}

/** Disconnect every live session (workspace/project teardown). */
export async function disposeAllWsTabs(): Promise<void> {
  const tabIds = [...byTabId.keys()]
  await Promise.all(tabIds.map((id) => disposeWsTab(id)))
}
