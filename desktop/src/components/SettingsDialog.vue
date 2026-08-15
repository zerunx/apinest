<template>
  <Teleport to="body">
    <div
      v-if="settingsOpen"
      class="overlay"
      @mousedown="onOverlayMouseDown"
      @mouseup="onOverlayMouseUp"
    >
      <div class="dialog" role="dialog" aria-modal="true">
        <button type="button" class="close-btn" :title="t('common.close')" @click="closeSettings">
          ×
        </button>

        <aside class="nav">
          <button
            v-for="item in menu"
            :key="item.id"
            type="button"
            class="nav-item"
            :class="{ active: active === item.id }"
            @click="active = item.id"
          >
            <span class="nav-icon" v-html="item.icon" />
            <span>{{ t(item.labelKey) }}</span>
          </button>
        </aside>

        <section class="panel">
          <!-- General -->
          <div v-if="active === 'general'" class="panel-inner">
            <h2>{{ t('settings.nav.general') }}</h2>
            <div class="block">
              <h3>{{ t('settings.language') }}</h3>
              <div class="row">
                <button
                  class="btn"
                  type="button"
                  :class="{ active: locale === 'zh-CN' }"
                  @click="setLocale('zh-CN')"
                >
                  中文
                </button>
                <button
                  class="btn"
                  type="button"
                  :class="{ active: locale === 'en-US' }"
                  @click="setLocale('en-US')"
                >
                  English
                </button>
              </div>
            </div>

            <div class="block">
              <h3>{{ t('settings.requestOptions') }}</h3>
              <p class="muted">{{ t('settings.requestTimeoutHint') }}</p>
              <div class="timeout-row">
                <label for="request-timeout">{{ t('settings.requestTimeout') }}</label>
                <input
                  id="request-timeout"
                  v-model.number="timeoutSecs"
                  class="input timeout-input"
                  type="number"
                  min="1"
                  max="600"
                  step="1"
                  @change="saveTimeout"
                />
                <span class="muted">s</span>
              </div>
            </div>
          </div>

          <!-- Themes -->
          <div v-else-if="active === 'themes'" class="panel-inner">
            <h2>{{ t('settings.nav.themes') }}</h2>
            <div class="block">
              <h3>{{ t('settings.theme') }}</h3>
              <div class="row">
                <button
                  class="btn"
                  type="button"
                  :class="{ active: theme === 'light' }"
                  @click="setTheme('light')"
                >
                  {{ t('settings.themeLight') }}
                </button>
                <button
                  class="btn"
                  type="button"
                  :class="{ active: theme === 'dark' }"
                  @click="setTheme('dark')"
                >
                  {{ t('settings.themeDark') }}
                </button>
              </div>
            </div>
          </div>

          <!-- Data -->
          <div v-else-if="active === 'data'" class="panel-inner panel-data">
            <h2>{{ t('settings.nav.data') }}</h2>

            <div class="data-card sync-card">
              <div class="sync-main">
                <h3>{{ t('settings.sync') }}</h3>
                <div class="data-meta">
                  <p class="muted">{{ t('settings.server') }}: {{ session?.base_url || '—' }}</p>
                  <p v-if="session?.account" class="muted">{{ session.account }}</p>
                  <p v-if="!loggedIn" class="muted">{{ t('settings.loginRequired') }}</p>
                  <p v-else class="muted">{{ t('settings.syncHint') }}</p>
                </div>
                <div class="row">
                  <button
                    class="btn"
                    type="button"
                    :disabled="busy || !loggedIn"
                    @click="onTest"
                  >
                    {{ testing ? t('settings.testing') : t('settings.test') }}
                  </button>
                  <button
                    class="btn btn-primary"
                    type="button"
                    :disabled="busy || !loggedIn"
                    @click="onSync"
                  >
                    {{ syncing ? t('settings.syncing') : t('settings.syncNow') }}
                  </button>
                  <button
                    v-if="!loggedIn"
                    class="btn"
                    type="button"
                    @click="goLoginFromSettings"
                  >
                    {{ t('nav.login') }}
                  </button>
                </div>
              </div>
              <aside class="sync-aside">
                <div class="sync-meta">
                  <span class="sync-meta__label">{{ t('settings.lastSync') }}</span>
                  <span class="sync-meta__value">{{ lastSync ? formatChinaDateTime(lastSync) : '—' }}</span>
                </div>
                <h4>{{ t('settings.syncResult') }}</h4>
                <ul class="sync-stats">
                  <li v-for="item in syncStatItems" :key="item.key">
                    <span class="sync-stat-label">{{ item.label }}</span>
                    <strong class="sync-stat-value" :class="{ empty: item.empty }">{{ item.value }}</strong>
                  </li>
                </ul>
              </aside>
            </div>

            <div class="data-card">
              <h3>{{ t('settings.clearCache') }}</h3>
              <p class="muted data-desc">{{ t('settings.clearHint') }}</p>
              <button class="btn btn-danger" type="button" :disabled="busy" @click="onClear">
                {{ t('settings.clearCache') }}
              </button>
            </div>

            <div class="data-card">
              <h3>{{ t('settings.dbPath') }}</h3>
              <code class="path">{{ dbPath }}</code>
            </div>
          </div>

          <!-- Shortcuts -->
          <div v-else-if="active === 'shortcuts'" class="panel-inner">
            <h2>{{ t('settings.nav.shortcuts') }}</h2>
            <p class="muted shortcuts-hint">{{ t('settings.shortcuts.hint') }}</p>
            <ul class="shortcut-list">
              <li v-for="item in shortcutItems" :key="item.id" class="shortcut-row">
                <span class="shortcut-desc">{{ t(item.labelKey) }}</span>
                <span class="shortcut-keys">
                  <template v-for="(key, idx) in item.keys" :key="`${item.id}-${idx}`">
                    <span v-if="idx > 0" class="shortcut-plus">+</span>
                    <kbd>{{ key }}</kbd>
                  </template>
                </span>
              </li>
            </ul>
          </div>

          <!-- Proxy (reserved) -->
          <div v-else-if="active === 'proxy'" class="panel-inner panel-placeholder">
            <h2>{{ t('settings.nav.proxy') }}</h2>
            <p class="muted">{{ t('settings.comingSoon') }}</p>
          </div>

          <!-- Update (reserved) -->
          <div v-else-if="active === 'update'" class="panel-inner panel-placeholder">
            <h2>{{ t('settings.nav.update') }}</h2>
            <p class="muted">{{ t('settings.comingSoon') }}</p>
          </div>

          <!-- About -->
          <div v-else-if="active === 'about'" class="panel-inner about">
            <img class="about-logo" src="/nest.png" alt="ApiNest" width="72" height="72" />
            <h2 class="about-title">{{ t('app.name') }}</h2>
            <dl class="about-meta">
              <div>
                <dt>{{ t('settings.about.version') }}</dt>
                <dd>{{ appVersion }}</dd>
              </div>
              <div>
                <dt>{{ t('settings.about.platform') }}</dt>
                <dd>{{ platformLabel }}</dd>
              </div>
              <div>
                <dt>{{ t('settings.about.arch') }}</dt>
                <dd>{{ archLabel }}</dd>
              </div>
            </dl>
          </div>
        </section>
      </div>
    </div>

    <ConfirmDialog
      v-model:open="clearConfirmOpen"
      :title="t('settings.clearCache')"
      :message="t('settings.clearHint')"
      :confirm-label="t('settings.clearCache')"
      variant="danger"
      @confirm="confirmClearCache"
    />
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import { useAppUi } from '../composables/useAppUi'
import { useTheme } from '../composables/useTheme'
import { useWorkspaceContext } from '../composables/useWorkspaceContext'
import { useOverlayDismiss } from '../composables/useOverlayDismiss'
import ConfirmDialog from './ConfirmDialog.vue'
import type { SyncResult } from '../types'
import { isMacOS } from '../utils/platform'
import { formatChinaDateTime } from '../utils/datetime'
import { useToast } from '../composables/useToast'

type SyncStats = Pick<
  SyncResult,
  'workspaces' | 'projects' | 'folders' | 'interfaces' | 'environments'
>

type MenuId = 'general' | 'themes' | 'data' | 'shortcuts' | 'proxy' | 'update' | 'about'

const { t, locale, setLocale } = useI18n()
const { settingsOpen, closeSettings, session, refreshSession, openLogin } = useAppUi()
const { onOverlayMouseDown, onOverlayMouseUp } = useOverlayDismiss(settingsOpen, closeSettings)
const clearConfirmOpen = ref(false)
const { showToast } = useToast()
const { theme, setTheme } = useTheme()
const { refreshAfterSync } = useWorkspaceContext()

const active = ref<MenuId>('general')
const testing = ref(false)
const syncing = ref(false)
const lastSync = ref('')
const lastSyncStats = ref<SyncStats | null>(null)
const dbPath = ref('')
const timeoutSecs = ref(60)
const appVersion = '0.0.1'
const platformLabel = ref('—')
const archLabel = ref('—')
const busy = computed(() => testing.value || syncing.value)
const loggedIn = computed(() => !!session.value?.logged_in)
const modKey = computed(() => (isMacOS() ? '⌘' : 'Ctrl'))

const syncStatItems = computed(() => {
  const stats = lastSyncStats.value
  const item = (key: string, labelKey: string, n: number | undefined) => ({
    key,
    label: t(labelKey),
    value: stats == null ? '—' : String(n ?? 0),
    empty: stats == null,
  })
  return [
    item('workspaces', 'settings.syncStat.workspaces', stats?.workspaces),
    item('projects', 'settings.syncStat.projects', stats?.projects),
    item('folders', 'settings.syncStat.folders', stats?.folders),
    item('interfaces', 'settings.syncStat.interfaces', stats?.interfaces),
    item('environments', 'settings.syncStat.environments', stats?.environments),
  ]
})

const shortcutItems = computed(() => [
  { id: 'save', labelKey: 'settings.shortcuts.save', keys: [modKey.value, 'S'] },
  { id: 'closeTab', labelKey: 'settings.shortcuts.closeTab', keys: [modKey.value, 'W'] },
  { id: 'undo', labelKey: 'settings.shortcuts.undo', keys: [modKey.value, 'Z'] },
  { id: 'escape', labelKey: 'settings.shortcuts.escape', keys: ['Esc'] },
])

const menu: { id: MenuId; labelKey: string; icon: string }[] = [
  {
    id: 'general',
    labelKey: 'settings.nav.general',
    icon: `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9c.3.6.9 1 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"/></svg>`,
  },
  {
    id: 'themes',
    labelKey: 'settings.nav.themes',
    icon: `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="9"/><path d="M12 3a9 9 0 0 0 0 18z"/><path d="M12 7v2M12 15v2M8.5 9.5l1.4 1.4M14.1 13.1l1.4 1.4M7 12h2M15 12h2M8.5 14.5l1.4-1.4M14.1 10.9l1.4-1.4"/></svg>`,
  },
  {
    id: 'data',
    labelKey: 'settings.nav.data',
    icon: `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"><ellipse cx="12" cy="6" rx="7" ry="3"/><path d="M5 6v6c0 1.7 3.1 3 7 3s7-1.3 7-3V6"/><path d="M5 12v6c0 1.7 3.1 3 7 3s7-1.3 7-3v-6"/></svg>`,
  },
  {
    id: 'shortcuts',
    labelKey: 'settings.nav.shortcuts',
    icon: `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="6" width="18" height="12" rx="2"/><path d="M7 10h.01M11 10h6M7 14h.01M11 14h4"/></svg>`,
  },
  {
    id: 'proxy',
    labelKey: 'settings.nav.proxy',
    icon: `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M4 8h10M14 8a2 2 0 1 0 4 0 2 2 0 0 0-4 0zM20 16H10M10 16a2 2 0 1 1-4 0 2 2 0 0 1 4 0z"/></svg>`,
  },
  {
    id: 'update',
    labelKey: 'settings.nav.update',
    icon: `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 4v10"/><path d="M8 10l4 4 4-4"/><path d="M5 18h14"/></svg>`,
  },
  {
    id: 'about',
    labelKey: 'settings.nav.about',
    icon: `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="9"/><path d="M12 10v6M12 7h.01"/></svg>`,
  },
]

watch(settingsOpen, async (open) => {
  if (!open) return
  active.value = 'general'
  await refreshSession()
  try {
    dbPath.value = await invoke('get_db_path')
    const saved = await invoke<string | null>('get_setting_value', { key: 'last_sync_at' })
    lastSync.value = saved || ''
    const savedStats = await invoke<string | null>('get_setting_value', { key: 'last_sync_stats' })
    lastSyncStats.value = parseSyncStats(savedStats)
    const timeoutSaved = await invoke<string | null>('get_setting_value', {
      key: 'request_timeout_secs',
    })
    const parsed = timeoutSaved ? Number(timeoutSaved) : 60
    timeoutSecs.value =
      Number.isFinite(parsed) && parsed > 0 ? Math.min(600, Math.floor(parsed)) : 60
  } catch {
    dbPath.value = '—'
  }
  try {
    platformLabel.value = navigator.platform || '—'
    archLabel.value =
      navigator.userAgent.includes('x64') || navigator.userAgent.includes('Win64')
        ? 'x64'
        : navigator.userAgent.includes('arm') || navigator.userAgent.includes('aarch64')
          ? 'arm64'
          : '—'
  } catch {
    platformLabel.value = '—'
    archLabel.value = '—'
  }
})

function goLoginFromSettings() {
  closeSettings()
  openLogin()
}

async function saveTimeout() {
  let secs = Number(timeoutSecs.value)
  if (!Number.isFinite(secs) || secs < 1) secs = 60
  if (secs > 600) secs = 600
  timeoutSecs.value = Math.floor(secs)
  try {
    await invoke('set_setting_value', {
      key: 'request_timeout_secs',
      value: String(timeoutSecs.value),
    })
  } catch {
    // browser preview
  }
}

function parseSyncStats(raw: string | null): SyncStats | null {
  if (!raw) return null
  try {
    const parsed = JSON.parse(raw) as Partial<SyncStats>
    if (typeof parsed.workspaces !== 'number') return null
    return {
      workspaces: parsed.workspaces,
      projects: parsed.projects ?? 0,
      folders: parsed.folders ?? 0,
      interfaces: parsed.interfaces ?? 0,
      environments: parsed.environments ?? 0,
    }
  } catch {
    return null
  }
}

async function onTest() {
  if (!loggedIn.value) return
  testing.value = true
  try {
    const count = await invoke<number>('test_connection')
    showToast(
      `${t('settings.testOk')} · ${t('settings.testOkHint', { count })}`,
      2200,
      'success',
    )
  } catch (e) {
    showToast(String(e), undefined, 'error')
  } finally {
    testing.value = false
  }
}

async function onSync() {
  if (!loggedIn.value) return
  syncing.value = true
  try {
    const result = await invoke<SyncResult>('sync_now', {
      args: { workspace_id: null },
    })
    lastSync.value = result.synced_at
    lastSyncStats.value = {
      workspaces: result.workspaces,
      projects: result.projects,
      folders: result.folders,
      interfaces: result.interfaces,
      environments: result.environments,
    }
    await invoke('set_setting_value', { key: 'last_sync_at', value: result.synced_at })
    await invoke('set_setting_value', {
      key: 'last_sync_stats',
      value: JSON.stringify(lastSyncStats.value),
    })
    await refreshAfterSync()
  } catch (e) {
    showToast(String(e), undefined, 'error')
  } finally {
    syncing.value = false
  }
}

function onClear() {
  clearConfirmOpen.value = true
}

async function confirmClearCache() {
  try {
    await invoke('clear_local_cache')
    await refreshAfterSync()
    lastSync.value = ''
    lastSyncStats.value = null
    await invoke('set_setting_value', { key: 'last_sync_at', value: '' })
    await invoke('set_setting_value', { key: 'last_sync_stats', value: '' })
  } catch (e) {
    showToast(String(e), undefined, 'error')
  }
}
</script>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.5);
  display: grid;
  place-items: center;
  padding: 24px;
}

.dialog {
  position: relative;
  width: min(860px, 96vw);
  height: min(620px, 86vh);
  display: grid;
  grid-template-columns: 200px 1fr;
  background: var(--color-settings-panel);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  box-shadow: var(--shadow-dialog);
  overflow: hidden;
}

.close-btn {
  position: absolute;
  top: 10px;
  right: 10px;
  z-index: 2;
  width: 28px;
  height: 28px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 19px;
  line-height: 1;
}

.close-btn:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.nav {
  background: var(--color-settings-nav);
  border-right: 1px solid var(--color-border);
  padding: 16px 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: auto;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  border: 0;
  background: transparent;
  color: var(--color-text-secondary);
  text-align: left;
  padding: 8px 10px;
  border-radius: 8px;
  font-size: 13px;
}

.nav-item:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.nav-item.active {
  background: var(--color-hover);
  color: var(--color-text);
  font-weight: 600;
}

.nav-icon {
  display: inline-flex;
  width: 15px;
  height: 15px;
  flex-shrink: 0;
}

.nav-icon :deep(svg) {
  display: block;
  width: 15px;
  height: 15px;
}

.panel {
  min-width: 0;
  overflow: auto;
  background: var(--color-settings-panel);
  font-size: 13px;
}

.panel-inner {
  padding: 24px 28px 28px;
}

.panel-inner h2 {
  margin: 0 0 16px;
  font-size: 17px;
  font-weight: 650;
}

.panel-placeholder {
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-height: 100%;
}

.block {
  margin-bottom: 18px;
  padding: 14px 16px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  background: var(--color-surface);
}

.block:last-child {
  margin-bottom: 0;
}

.block h3 {
  margin: 0 0 8px;
  font-size: 14px;
}

.timeout-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 10px;
  font-size: 13px;
}

.timeout-input {
  width: 88px;
  padding: 6px 8px;
}

.panel-data {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.panel-data > h2 {
  margin-bottom: 4px;
}

.data-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.data-card h3 {
  margin: 0;
  font-size: 14px;
}

.sync-card {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 220px;
  gap: 0;
  padding: 0;
  overflow: hidden;
}

.sync-main {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
  padding: 16px 18px;
}

.sync-aside {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px 16px;
  border-left: 1px solid var(--color-border);
  background: color-mix(in srgb, var(--color-hover) 55%, var(--color-surface));
}

.sync-meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sync-meta__label {
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--color-text-secondary);
}

.sync-meta__value {
  font-size: 12px;
  line-height: 1.4;
  font-variant-numeric: tabular-nums;
  color: var(--color-text);
}

.sync-aside h4 {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--color-text-secondary);
}

.sync-stats {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.sync-stats li {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.sync-stat-label {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.sync-stat-value {
  min-width: 28px;
  padding: 1px 7px;
  border-radius: 999px;
  text-align: center;
  font-size: 12px;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
  color: var(--color-primary);
  background: color-mix(in srgb, var(--color-primary) 12%, transparent);
}

.sync-stat-value.empty {
  color: var(--color-text-secondary);
  background: color-mix(in srgb, var(--color-text-secondary) 10%, transparent);
  font-weight: 500;
}

.data-meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.data-meta .muted,
.data-desc {
  margin: 0;
  line-height: 1.5;
}

.data-card .row {
  margin-top: 0;
}

.row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin-top: 6px;
}

.btn.active {
  border-color: var(--color-primary);
  background: var(--color-active);
}

.path {
  font-family: var(--font-mono);
  font-size: 12px;
  word-break: break-all;
}

.about {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 100%;
  text-align: center;
  gap: 8px;
}

.about-logo {
  display: block;
  object-fit: contain;
  margin-bottom: 4px;
  width: 64px;
  height: 64px;
}

.about-title {
  margin: 0 !important;
  font-size: 19px !important;
}

.about-meta {
  margin: 10px 0 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 260px;
}

.about-meta div {
  display: grid;
  grid-template-columns: 1fr 1.2fr;
  gap: 12px;
  text-align: left;
  font-size: 13px;
}

.about-meta dt {
  color: var(--color-text-secondary);
}

.about-meta dd {
  margin: 0;
}

.shortcuts-hint {
  margin: -6px 0 14px;
  line-height: 1.5;
}

.shortcut-list {
  list-style: none;
  margin: 0;
  padding: 0;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  background: var(--color-surface);
  overflow: hidden;
}

.shortcut-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 14px;
  border-bottom: 1px solid var(--color-border);
}

.shortcut-row:last-child {
  border-bottom: 0;
}

.shortcut-desc {
  font-size: 13px;
  color: var(--color-text);
}

.shortcut-keys {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.shortcut-plus {
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1;
  opacity: 0.7;
}

.shortcut-keys kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 22px;
  height: 24px;
  padding: 0 7px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-bg);
  color: var(--color-text-secondary);
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1;
  box-shadow: 0 1px 0 color-mix(in srgb, var(--color-border) 80%, transparent);
}
</style>
