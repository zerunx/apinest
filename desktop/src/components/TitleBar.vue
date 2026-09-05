<template>
  <header
    class="titlebar"
    :class="{ 'titlebar--mac': isMac }"
    data-tauri-drag-region
    @dblclick="toggleMaximize"
  >
    <div class="titlebar-left" data-tauri-drag-region>
      <img class="app-logo" src="/nest.png" alt="" draggable="false" />

      <div class="ws-wrap no-drag" ref="wsWrapRef">
        <button type="button" class="ws-trigger" @click.stop="toggleWs">
          <span class="ws-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8">
              <circle cx="9" cy="8" r="3.2" />
              <circle cx="16.5" cy="9.5" r="2.6" />
              <path d="M3.5 19c.6-3.2 2.8-5 5.5-5s4.9 1.8 5.5 5" />
              <path d="M13.2 16.2c.7-1.7 2.2-2.7 4.1-2.7 1.5 0 2.8.6 3.6 1.7" />
            </svg>
          </span>
          <span class="ws-name">{{ currentWorkspace?.name || t('workspace.empty') }}</span>
          <span class="ws-chevron" :class="{ open: wsOpen }" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
              <path d="M6 9l6 6 6-6" />
            </svg>
          </span>
        </button>

        <div v-if="wsOpen" class="ws-menu" @click.stop>
          <input
            v-model="wsQuery"
            class="ws-search"
            type="search"
            :placeholder="t('workspace.search')"
            autofocus
          />
          <div class="ws-list">
            <div
              v-for="w in filteredWorkspaces"
              :key="w.id"
              class="ws-row"
              :class="{ active: w.id === workspaceId }"
            >
              <button type="button" class="ws-item ws-item-main" @click="onPickWorkspace(w.id)">
                <span class="ws-icon">
                  <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8">
                    <circle cx="9" cy="8" r="3.2" />
                    <circle cx="16.5" cy="9.5" r="2.6" />
                    <path d="M3.5 19c.6-3.2 2.8-5 5.5-5s4.9 1.8 5.5 5" />
                    <path d="M13.2 16.2c.7-1.7 2.2-2.7 4.1-2.7 1.5 0 2.8.6 3.6 1.7" />
                  </svg>
                </span>
                <span class="ws-item-name">{{ w.name }}</span>
              </button>
              <div class="ws-item-actions">
                <button
                  type="button"
                  class="ws-action"
                  :title="t('dict.title')"
                  @click.stop="openDictionary(w)"
                >
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                    <path d="M4 5.5A2.5 2.5 0 0 1 6.5 3H12v18H6.5A2.5 2.5 0 0 1 4 18.5v-13z" />
                    <path d="M20 5.5A2.5 2.5 0 0 0 17.5 3H12v18h5.5A2.5 2.5 0 0 0 20 18.5v-13z" />
                  </svg>
                </button>
                <button
                  type="button"
                  class="ws-action"
                  :title="t('common.edit')"
                  @click.stop="openEditWorkspace(w, $event)"
                >
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                    <path d="M12 20h9" />
                    <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4 12.5-12.5z" />
                  </svg>
                </button>
                <button
                  type="button"
                  class="ws-action danger"
                  :title="t('common.delete')"
                  @click.stop="openDeleteWorkspace(w, $event)"
                >
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                    <path d="M3 6h18" />
                    <path d="M8 6V4h8v2" />
                    <path d="M19 6l-1 14H6L5 6" />
                    <path d="M10 11v6M14 11v6" />
                  </svg>
                </button>
              </div>
            </div>
            <p v-if="!filteredWorkspaces.length" class="ws-empty muted">{{ t('workspace.none') }}</p>
          </div>
          <div class="ws-footer">
            <button type="button" class="ws-create" @click="openCreateWorkspace($event)">
              <span>{{ t('create.workspace') }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="titlebar-right no-drag">
      <div class="app-actions">
        <button
          type="button"
          class="action-btn"
          :title="t('nav.settings')"
          @click.stop="openSettings"
        >
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8">
            <circle cx="12" cy="12" r="3" />
            <path
              d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9c.3.6.9 1 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"
            />
          </svg>
        </button>

        <div class="avatar-wrap" ref="avatarWrapRef">
          <button
            v-if="loggedIn"
            type="button"
            class="avatar-btn"
            :title="accountLabel"
            @click.stop="toggleAvatar"
          >
            {{ avatarText }}
          </button>
          <button
            v-else
            type="button"
            class="avatar-btn avatar-btn-login"
            :title="t('nav.login')"
            @click.stop="goLogin"
          >
            {{ t('nav.loginShort') }}
          </button>
          <div v-if="avatarOpen && loggedIn" class="avatar-menu" @click.stop>
            <div class="avatar-meta">
              <strong>{{ accountLabel }}</strong>
              <span class="muted">{{ session?.base_url }}</span>
            </div>
            <button type="button" class="avatar-item" @click="onLogout">
              {{ t('nav.logout') }}
            </button>
          </div>
        </div>
      </div>

      <div v-if="!isMac" class="win-controls">
        <button type="button" class="win-btn" :title="t('window.minimize')" @click="minimize">
          <svg class="win-icon" viewBox="0 0 16 16" aria-hidden="true">
            <path d="M3.5 8h9" />
          </svg>
        </button>
        <button
          type="button"
          class="win-btn"
          :title="maximized ? t('window.restore') : t('window.maximize')"
          @click="toggleMaximize"
        >
          <svg v-if="!maximized" class="win-icon" viewBox="0 0 16 16" aria-hidden="true">
            <rect x="3.5" y="3.5" width="9" height="9" />
          </svg>
          <svg v-else class="win-icon" viewBox="0 0 16 16" aria-hidden="true">
            <path d="M5 6.5V3.5h7v7H9.5" />
            <rect x="3.5" y="5.5" width="7" height="7" />
          </svg>
        </button>
        <button type="button" class="win-btn win-btn-close" :title="t('window.close')" @click="close">
          <svg class="win-icon" viewBox="0 0 16 16" aria-hidden="true">
            <path d="M4.25 4.25l7.5 7.5M11.75 4.25l-7.5 7.5" />
          </svg>
        </button>
      </div>
    </div>
  </header>

  <SettingsDialog />
  <NameDialog
    v-model:open="wsDialogOpen"
    :title="wsDialogTitle"
    :label="t('workspace.label')"
    :initial="wsDialogInitial"
    :anchor="wsDialogAnchor"
    @confirm="confirmWorkspaceName"
  />
  <ConfirmDialog
    v-model:open="wsDeleteOpen"
    :title="t('workspace.delete')"
    :message="t('workspace.deleteConfirm', { name: wsDeleteTarget?.name || '' })"
    :anchor="wsDialogAnchor"
    @confirm="confirmDeleteWorkspace"
  />
  <DictionaryManageDialog
    v-model:open="dictOpen"
    :workspace-id="dictWorkspaceId"
    :workspace-name="dictWorkspaceName"
  />
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import { useAppUi } from '../composables/useAppUi'
import { useWorkspaceContext } from '../composables/useWorkspaceContext'
import { useToast } from '../composables/useToast'
import SettingsDialog from './SettingsDialog.vue'
import NameDialog from './NameDialog.vue'
import ConfirmDialog from './ConfirmDialog.vue'
import DictionaryManageDialog from './DictionaryManageDialog.vue'
import { anchorFromEvent, type DialogAnchor } from '../composables/useDialogAnchor'
import { bindOutsidePointerDismiss } from '../composables/useOverlayDismiss'
import { isMacOS } from '../utils/platform'
import type { WorkspaceRow } from '../types'

const { t } = useI18n()
const { showToast } = useToast()
const { session, refreshSession, openSettings, openLogin } = useAppUi()
const {
  workspaces,
  workspaceId,
  currentWorkspace,
  selectWorkspace,
  loadWorkspaces,
  refreshAfterSync,
} = useWorkspaceContext()

/** Native traffic lights on macOS; custom buttons only on Windows/Linux. */
const isMac = isMacOS()
const maximized = ref(false)
const wsOpen = ref(false)
const wsQuery = ref('')
const avatarOpen = ref(false)
const wsWrapRef = ref<HTMLElement | null>(null)
const avatarWrapRef = ref<HTMLElement | null>(null)
const wsDialogOpen = ref(false)
const wsDialogMode = ref<'create' | 'edit'>('create')
const wsDialogInitial = ref('')
const wsEditTarget = ref<WorkspaceRow | null>(null)
const wsDeleteOpen = ref(false)
const wsDeleteTarget = ref<WorkspaceRow | null>(null)
const wsDialogAnchor = ref<DialogAnchor | null>(null)
const dictOpen = ref(false)
const dictWorkspaceId = ref<number | null>(null)
const dictWorkspaceName = ref('')

const wsDialogTitle = computed(() =>
  wsDialogMode.value === 'edit' ? t('workspace.rename') : t('create.workspace'),
)

let unlistenResize: (() => void) | undefined
let unbindOutside: (() => void) | undefined

function getAppWindow() {
  try {
    return getCurrentWindow()
  } catch {
    return null
  }
}

const loggedIn = computed(() => !!session.value?.logged_in)
const accountLabel = computed(() => session.value?.account || 'User')
const avatarText = computed(() => accountLabel.value.slice(0, 1).toUpperCase())

const filteredWorkspaces = computed(() => {
  const q = wsQuery.value.trim().toLowerCase()
  if (!q) return workspaces.value
  return workspaces.value.filter((w) => w.name.toLowerCase().includes(q))
})

onMounted(async () => {
  await refreshSession()
  await loadWorkspaces()
  const appWindow = getAppWindow()
  if (appWindow) {
    try {
      maximized.value = await appWindow.isMaximized()
      unlistenResize = await appWindow.onResized(async () => {
        maximized.value = await appWindow.isMaximized()
      })
    } catch {
      // browser preview without Tauri window
    }
  }
  unbindOutside = bindOutsidePointerDismiss(
    () => wsOpen.value || avatarOpen.value,
    () => [wsWrapRef.value, avatarWrapRef.value],
    () => {
      wsOpen.value = false
      avatarOpen.value = false
    },
  )
})

onUnmounted(() => {
  unlistenResize?.()
  unbindOutside?.()
})

function toggleWs() {
  avatarOpen.value = false
  wsOpen.value = !wsOpen.value
  if (wsOpen.value) wsQuery.value = ''
}

function toggleAvatar() {
  wsOpen.value = false
  avatarOpen.value = !avatarOpen.value
}

async function onPickWorkspace(id: number) {
  await selectWorkspace(id)
  wsOpen.value = false
}

function openCreateWorkspace(event: MouseEvent) {
  wsOpen.value = false
  wsDialogMode.value = 'create'
  wsDialogInitial.value = ''
  wsEditTarget.value = null
  wsDialogAnchor.value = anchorFromEvent(event)
  wsDialogOpen.value = true
}

function openEditWorkspace(workspace: WorkspaceRow, event: MouseEvent) {
  wsOpen.value = false
  wsDialogMode.value = 'edit'
  wsDialogInitial.value = workspace.name
  wsEditTarget.value = workspace
  wsDialogAnchor.value = anchorFromEvent(event)
  wsDialogOpen.value = true
}

function openDeleteWorkspace(workspace: WorkspaceRow, event: MouseEvent) {
  wsOpen.value = false
  wsDeleteTarget.value = workspace
  wsDialogAnchor.value = anchorFromEvent(event)
  wsDeleteOpen.value = true
}

function openDictionary(workspace: WorkspaceRow) {
  wsOpen.value = false
  dictWorkspaceId.value = workspace.id
  dictWorkspaceName.value = workspace.name
  dictOpen.value = true
}

async function confirmWorkspaceName(name: string) {
  try {
    if (wsDialogMode.value === 'edit' && wsEditTarget.value) {
      await invoke('update_local_workspace', {
        args: { id: wsEditTarget.value.id, name },
      })
      wsDialogOpen.value = false
      await loadWorkspaces()
      showToast(t('workspace.updateSuccess'), 1500, 'success')
      return
    }
    const row = await invoke<{ id: number }>('create_local_workspace', { args: { name } })
    wsDialogOpen.value = false
    await loadWorkspaces()
    await selectWorkspace(row.id)
  } catch (e) {
    showToast(String(e), 2200, 'error')
  }
}

async function confirmDeleteWorkspace() {
  const target = wsDeleteTarget.value
  if (!target) return
  try {
    await invoke('delete_local_workspace', { workspaceId: target.id })
    wsDeleteOpen.value = false
    wsDeleteTarget.value = null
    await refreshAfterSync()
    showToast(t('workspace.deleteSuccess'), 1500, 'success')
  } catch (e) {
    showToast(String(e), 2200, 'error')
  }
}

function goLogin() {
  avatarOpen.value = false
  openLogin()
}

async function onLogout() {
  avatarOpen.value = false
  try {
    await invoke('logout')
  } catch {
    // ignore in browser preview
  }
  await refreshSession()
}

async function minimize() {
  await getAppWindow()?.minimize()
}

async function toggleMaximize() {
  const appWindow = getAppWindow()
  if (!appWindow) return
  await appWindow.toggleMaximize()
  maximized.value = await appWindow.isMaximized()
}

async function close() {
  await getAppWindow()?.close()
}
</script>

<style scoped>
.titlebar {
  height: 38px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-left: 16px;
  background: var(--color-titlebar);
  border-bottom: 1px solid var(--color-border);
  user-select: none;
  color: var(--color-text);
  -webkit-app-region: drag;
  app-region: drag;
}

/* Leave room for native macOS traffic lights (close / minimize / zoom). */
.titlebar--mac {
  padding-left: 78px;
  padding-right: 10px;
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 18px;
  min-width: 0;
  flex: 1;
  height: 100%;
  -webkit-app-region: drag;
  app-region: drag;
}

.titlebar .no-drag,
.titlebar button,
.titlebar input,
.titlebar .ws-menu,
.titlebar .avatar-menu,
.titlebar .app-actions,
.titlebar .win-controls {
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.app-logo {
  width: 18px;
  height: 18px;
  object-fit: contain;
  display: block;
  flex-shrink: 0;
}

.ws-wrap {
  position: relative;
  min-width: 0;
  margin-left: 4px;
}

.ws-trigger {
  max-width: 240px;
  display: inline-flex;
  align-items: center;
  gap: 7px;
  border: 0;
  background: transparent;
  color: inherit;
  padding: 4px 8px;
  border-radius: 6px;
}

.ws-trigger:hover {
  background: var(--color-hover);
}

.ws-icon {
  display: inline-flex;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.ws-name {
  font-size: 14px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ws-chevron {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  transition: transform 0.15s ease;
  flex-shrink: 0;
}

.ws-chevron svg {
  display: block;
}

.ws-chevron.open {
  transform: rotate(180deg);
}

.ws-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  z-index: 50;
  width: 300px;
  max-height: min(420px, calc(100vh - 60px));
  padding: 8px;
  border-radius: 10px;
  background: var(--color-menu);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-menu);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.ws-search {
  width: 100%;
  margin-bottom: 6px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 7px 10px;
  background: var(--color-surface);
  outline: none;
  flex-shrink: 0;
}

.ws-search:focus {
  border-color: var(--color-primary);
}

.ws-list {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.ws-row {
  display: flex;
  align-items: center;
  gap: 2px;
  border-radius: 6px;
}

.ws-row:hover,
.ws-row.active {
  background: var(--color-menu-hover);
}

.ws-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  border: 0;
  background: transparent;
  color: inherit;
  text-align: left;
  padding: 8px 10px;
  border-radius: 6px;
  font-size: 14px;
}

.ws-item-main {
  flex: 1;
  min-width: 0;
  padding-right: 4px;
}

.ws-row:hover .ws-item-main,
.ws-row.active .ws-item-main,
.ws-item-main:hover {
  background: transparent;
}

.ws-item-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ws-item-actions {
  display: none;
  align-items: center;
  gap: 1px;
  padding-right: 4px;
  flex-shrink: 0;
}

.ws-row:hover .ws-item-actions,
.ws-row.active .ws-item-actions {
  display: inline-flex;
}

.ws-action {
  width: 26px;
  height: 26px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--color-text-secondary);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.ws-action:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.ws-action.danger:hover {
  background: var(--color-danger-hover);
  color: var(--color-danger);
}

.ws-empty {
  margin: 4px 0;
  padding: 8px 10px;
  font-size: 13px;
}

.ws-footer {
  margin-top: 4px;
  padding-top: 8px;
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
}

.ws-create {
  width: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 38px;
  border: 1px dashed var(--color-border);
  border-radius: 8px;
  background: transparent;
  color: var(--color-primary-light);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.15s ease, border-color 0.15s ease;
}

.ws-create:hover {
  background: var(--color-active);
  border-color: var(--color-primary);
}

.titlebar-right {
  display: flex;
  align-items: stretch;
  height: 100%;
}

.app-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  padding-right: 22px;
  margin-right: 4px;
  border-right: 1px solid transparent;
}

.titlebar--mac .app-actions {
  padding-right: 4px;
  margin-right: 0;
}

.action-btn,
.avatar-btn {
  width: 30px;
  height: 30px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-secondary);
  display: grid;
  place-items: center;
  padding: 0;
}

.action-btn:hover,
.avatar-btn:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.avatar-wrap {
  position: relative;
}

.avatar-btn {
  border-radius: 50%;
  background: var(--color-active);
  color: var(--color-primary);
  font-size: 13px;
  font-weight: 700;
}

.avatar-btn-login {
  width: auto;
  min-width: 30px;
  padding: 0 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
}

.avatar-menu {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  z-index: 50;
  width: 220px;
  padding: 8px;
  border-radius: 10px;
  background: var(--color-menu);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-menu);
}

.avatar-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 10px 10px;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: 6px;
}

.avatar-meta strong {
  font-size: 14px;
}

.avatar-meta .muted {
  font-size: 12px;
  word-break: break-all;
}

.avatar-item {
  width: 100%;
  border: 0;
  background: transparent;
  text-align: left;
  padding: 8px 10px;
  border-radius: 6px;
  color: var(--color-danger);
  font-size: 14px;
}

.avatar-item:hover {
  background: var(--color-danger-hover);
}

.win-controls {
  display: flex;
  align-items: stretch;
  height: 100%;
  margin: 0;
  flex-shrink: 0;
  --win-icon-color: var(--color-text-secondary);
}

:root[data-theme='dark'] .win-controls {
  --win-icon-color: #c4c4c4;
}

.win-btn {
  width: 46px;
  height: 100%;
  border: 0;
  background: transparent;
  color: var(--win-icon-color);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  line-height: 0;
}

.win-btn:hover {
  background: var(--color-win-hover);
  color: var(--color-text);
}

:root[data-theme='dark'] .win-btn:hover {
  color: #ececec;
}

.win-btn-close:hover {
  background: #e81123;
  color: #fff;
}

.win-icon {
  width: 14px;
  height: 14px;
  display: block;
  flex-shrink: 0;
  overflow: visible;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.5;
  stroke-linecap: square;
  stroke-linejoin: miter;
  color: inherit;
}

/* Diagonals read thinner optically — match min/max weight. */
.win-btn-close .win-icon {
  stroke-width: 1.85;
}
</style>
