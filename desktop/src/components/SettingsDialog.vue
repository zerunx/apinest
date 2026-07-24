<template>
  <Teleport to="body">
    <div v-if="settingsOpen" class="overlay" @click.self="closeSettings">
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

            <div class="data-card">
              <h3>{{ t('settings.sync') }}</h3>
              <div class="data-meta">
                <p class="muted">{{ t('settings.server') }}: {{ session?.base_url || '—' }}</p>
                <p v-if="session?.account" class="muted">{{ session.account }}</p>
                <p v-if="!loggedIn" class="muted">{{ t('settings.loginRequired') }}</p>
              </div>
              <div class="row">
                <label class="radio">
                  <input v-model="syncScope" type="radio" value="current" :disabled="!loggedIn" />
                  {{ t('settings.syncCurrent') }}
                </label>
                <label class="radio">
                  <input v-model="syncScope" type="radio" value="all" :disabled="!loggedIn" />
                  {{ t('settings.syncAll') }}
                </label>
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
              <p v-if="message" class="msg">{{ message }}</p>
              <p v-if="lastSync" class="muted last-sync">{{ t('settings.lastSync') }}: {{ lastSync }}</p>
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
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import { useAppUi } from '../composables/useAppUi'
import { useTheme } from '../composables/useTheme'
import { useWorkspaceContext } from '../composables/useWorkspaceContext'
import type { SyncResult } from '../types'

type MenuId = 'general' | 'themes' | 'data' | 'proxy' | 'update' | 'about'

const { t, locale, setLocale } = useI18n()
const router = useRouter()
const { settingsOpen, closeSettings, session, refreshSession } = useAppUi()
const { theme, setTheme } = useTheme()
const { workspaceId, refreshAfterSync } = useWorkspaceContext()

const active = ref<MenuId>('general')
const syncScope = ref<'current' | 'all'>('current')
const testing = ref(false)
const syncing = ref(false)
const message = ref('')
const lastSync = ref('')
const dbPath = ref('')
const timeoutSecs = ref(60)
const appVersion = '0.0.1'
const platformLabel = ref('—')
const archLabel = ref('—')
const busy = computed(() => testing.value || syncing.value)
const loggedIn = computed(() => !!session.value?.logged_in)

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
  message.value = ''
  active.value = 'general'
  await refreshSession()
  try {
    dbPath.value = await invoke('get_db_path')
    const saved = await invoke<string | null>('get_setting_value', { key: 'last_sync_at' })
    if (saved) lastSync.value = saved
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
  router.push({ name: 'login' })
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

async function onTest() {
  if (!loggedIn.value) return
  testing.value = true
  message.value = ''
  try {
    message.value = await invoke<string>('test_connection')
  } catch (e) {
    message.value = String(e)
  } finally {
    testing.value = false
  }
}

async function onSync() {
  if (!loggedIn.value) return
  syncing.value = true
  message.value = ''
  try {
    const wid = syncScope.value === 'current' ? workspaceId.value : null
    const result = await invoke<SyncResult>('sync_now', {
      args: { workspace_id: wid },
    })
    lastSync.value = result.synced_at
    await invoke('set_setting_value', { key: 'last_sync_at', value: result.synced_at })
    await refreshAfterSync()
    message.value = `OK · ws ${result.workspaces} / proj ${result.projects} / folder ${result.folders} / api ${result.interfaces} / env ${result.environments}`
  } catch (e) {
    message.value = String(e)
  } finally {
    syncing.value = false
  }
}

async function onClear() {
  if (!confirm(t('settings.clearHint'))) return
  message.value = ''
  try {
    await invoke('clear_local_cache')
    await refreshAfterSync()
    message.value = 'local cache cleared'
    lastSync.value = ''
  } catch (e) {
    message.value = String(e)
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

.last-sync {
  margin: 0;
}

.row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin-top: 6px;
}

.radio {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}

.btn.active {
  border-color: var(--color-primary);
  background: var(--color-active);
}

.msg {
  margin: 8px 0 0;
  font-size: 13px;
  white-space: pre-wrap;
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
</style>
