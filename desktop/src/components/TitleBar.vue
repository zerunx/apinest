<template>
  <header class="titlebar" data-tauri-drag-region @dblclick="toggleMaximize">
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
          <button
            v-for="w in filteredWorkspaces"
            :key="w.id"
            type="button"
            class="ws-item"
            :class="{ active: w.id === workspaceId }"
            @click="onPickWorkspace(w.id)"
          >
            <span class="ws-icon">
              <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8">
                <circle cx="9" cy="8" r="3.2" />
                <circle cx="16.5" cy="9.5" r="2.6" />
                <path d="M3.5 19c.6-3.2 2.8-5 5.5-5s4.9 1.8 5.5 5" />
                <path d="M13.2 16.2c.7-1.7 2.2-2.7 4.1-2.7 1.5 0 2.8.6 3.6 1.7" />
              </svg>
            </span>
            <span>{{ w.name }}</span>
          </button>
          <p v-if="!filteredWorkspaces.length" class="ws-empty muted">{{ t('workspace.none') }}</p>
          <button type="button" class="ws-item ws-create" @click="openCreateWorkspace($event)">
            + {{ t('workspace.label') }}
          </button>
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

      <div class="win-controls">
        <button type="button" class="win-btn" :title="t('window.minimize')" @click="minimize">
          <svg class="win-icon" viewBox="0 0 16 16" aria-hidden="true">
            <rect x="3" y="7" width="10" height="2" fill="currentColor" />
          </svg>
        </button>
        <button
          type="button"
          class="win-btn"
          :title="maximized ? t('window.restore') : t('window.maximize')"
          @click="toggleMaximize"
        >
          <svg v-if="!maximized" class="win-icon" viewBox="0 0 16 16" aria-hidden="true">
            <rect
              x="3.5"
              y="3.5"
              width="9"
              height="9"
              fill="none"
              stroke="currentColor"
              stroke-width="1.25"
            />
          </svg>
          <svg v-else class="win-icon" viewBox="0 0 16 16" aria-hidden="true">
            <path
              fill="none"
              stroke="currentColor"
              stroke-width="1.25"
              stroke-linejoin="miter"
              d="M3.5 9.5V3.5H9.5"
            />
            <rect
              x="5.5"
              y="5.5"
              width="7"
              height="7"
              fill="none"
              stroke="currentColor"
              stroke-width="1.25"
            />
          </svg>
        </button>
        <button type="button" class="win-btn win-btn-close" :title="t('window.close')" @click="close">
          <svg class="win-icon" viewBox="0 0 16 16" aria-hidden="true">
            <path
              fill="none"
              stroke="currentColor"
              stroke-width="1.25"
              stroke-linecap="square"
              d="M3.75 3.75 12.25 12.25M12.25 3.75 3.75 12.25"
            />
          </svg>
        </button>
      </div>
    </div>
  </header>

  <SettingsDialog />
  <NameDialog
    v-model:open="wsDialogOpen"
    :title="t('create.workspace')"
    :label="t('workspace.label')"
    :anchor="wsDialogAnchor"
    @confirm="confirmCreateWorkspace"
  />
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import { useAppUi } from '../composables/useAppUi'
import { useWorkspaceContext } from '../composables/useWorkspaceContext'
import SettingsDialog from './SettingsDialog.vue'
import NameDialog from './NameDialog.vue'
import { anchorFromEvent, type DialogAnchor } from '../composables/useDialogAnchor'

const { t } = useI18n()
const router = useRouter()
const { session, refreshSession, openSettings } = useAppUi()
const { workspaces, workspaceId, currentWorkspace, selectWorkspace, loadWorkspaces } =
  useWorkspaceContext()

const maximized = ref(false)
const wsOpen = ref(false)
const wsQuery = ref('')
const avatarOpen = ref(false)
const wsWrapRef = ref<HTMLElement | null>(null)
const avatarWrapRef = ref<HTMLElement | null>(null)
const wsDialogOpen = ref(false)
const wsDialogAnchor = ref<DialogAnchor | null>(null)

let unlistenResize: (() => void) | undefined

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
  document.addEventListener('click', onDocClick)
})

onUnmounted(() => {
  unlistenResize?.()
  document.removeEventListener('click', onDocClick)
})

function onDocClick(e: MouseEvent) {
  const target = e.target as Node
  if (wsWrapRef.value && !wsWrapRef.value.contains(target)) wsOpen.value = false
  if (avatarWrapRef.value && !avatarWrapRef.value.contains(target)) avatarOpen.value = false
}

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
  wsDialogAnchor.value = anchorFromEvent(event)
  wsDialogOpen.value = true
}

async function confirmCreateWorkspace(name: string) {
  try {
    const row = await invoke<{ id: number }>('create_local_workspace', { args: { name } })
    wsDialogOpen.value = false
    await loadWorkspaces()
    await selectWorkspace(row.id)
  } catch (e) {
    alert(String(e))
  }
}

async function goLogin() {
  avatarOpen.value = false
  router.push({ name: 'login' })
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
  padding-left: 10px;
  background: var(--color-titlebar);
  border-bottom: 1px solid var(--color-border);
  user-select: none;
  color: var(--color-text);
  -webkit-app-region: drag;
  app-region: drag;
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
  width: 260px;
  padding: 8px;
  border-radius: 10px;
  background: var(--color-menu);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-menu);
  overflow: hidden;
}

.ws-search {
  width: 100%;
  margin-bottom: 6px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 7px 10px;
  background: var(--color-surface);
  outline: none;
}

.ws-search:focus {
  border-color: var(--color-primary);
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

.ws-item:hover,
.ws-item.active {
  background: var(--color-menu-hover);
}

.ws-empty {
  margin: 8px 4px;
  font-size: 13px;
}

.ws-create {
  margin-top: 4px;
  border-top: 1px solid var(--color-border);
  border-radius: 6px;
  color: var(--color-primary);
  font-weight: 600;
}

.ws-create:hover {
  background: var(--color-menu-hover);
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
  align-items: center;
  height: 100%;
  margin-right: -4px;
  --win-icon-color: var(--color-text);
}

:root[data-theme='dark'] .win-controls {
  --win-icon-color: #ffffff;
}

.win-btn {
  width: 40px;
  height: 100%;
  border: 0;
  background: transparent;
  color: var(--win-icon-color);
  display: grid;
  place-items: center;
  padding: 0;
}

.win-btn .win-icon {
  color: inherit;
}

.win-btn:hover {
  background: var(--color-win-hover);
}

.win-btn-close:hover {
  background: #e81123;
  color: #fff;
}

.win-icon {
  width: 16px;
  height: 16px;
  display: block;
  flex-shrink: 0;
  shape-rendering: geometricPrecision;
}
</style>
