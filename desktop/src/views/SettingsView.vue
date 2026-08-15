<template>
  <div class="settings">
    <header class="bar">
      <button class="btn" type="button" @click="router.push({ name: 'main' })">
        {{ t('settings.back') }}
      </button>
      <h1>{{ t('settings.title') }}</h1>
    </header>

    <div class="content">
      <section class="card">
        <h2>{{ t('settings.language') }}</h2>
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
      </section>

      <section class="card sync-card">
        <div class="sync-main">
          <h2>{{ t('settings.sync') }}</h2>
          <p class="muted">{{ t('settings.server') }}: {{ session?.base_url || '—' }}</p>
          <p class="muted">{{ session?.account || '' }}</p>
          <p class="muted">{{ t('settings.syncHint') }}</p>

          <div class="row">
            <button class="btn" type="button" :disabled="busy" @click="onTest">
              {{ testing ? t('settings.testing') : t('settings.test') }}
            </button>
            <button class="btn btn-primary" type="button" :disabled="busy" @click="onSync">
              {{ syncing ? t('settings.syncing') : t('settings.syncNow') }}
            </button>
          </div>
        </div>
        <aside class="sync-aside">
          <div class="sync-meta">
            <span class="sync-meta__label">{{ t('settings.lastSync') }}</span>
            <span class="sync-meta__value">{{ lastSync ? formatChinaDateTime(lastSync) : '—' }}</span>
          </div>
          <h3>{{ t('settings.syncResult') }}</h3>
          <ul class="sync-stats">
            <li v-for="item in syncStatItems" :key="item.key">
              <span>{{ item.label }}</span>
              <strong :class="{ empty: item.empty }">{{ item.value }}</strong>
            </li>
          </ul>
        </aside>
      </section>

      <section class="card">
        <h2>{{ t('settings.clearCache') }}</h2>
        <p class="muted">{{ t('settings.clearHint') }}</p>
        <button class="btn btn-danger" type="button" :disabled="busy" @click="onClear">
          {{ t('settings.clearCache') }}
        </button>
      </section>

      <section class="card">
        <h2>{{ t('settings.dbPath') }}</h2>
        <code class="path">{{ dbPath }}</code>
      </section>
    </div>

    <ConfirmDialog
      v-model:open="clearConfirmOpen"
      :title="t('settings.clearCache')"
      :message="t('settings.clearHint')"
      :confirm-label="t('settings.clearCache')"
      variant="danger"
      @confirm="confirmClearCache"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import ConfirmDialog from '../components/ConfirmDialog.vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import type { SessionInfo, SyncResult } from '../types'
import { formatChinaDateTime } from '../utils/datetime'
import { useToast } from '../composables/useToast'

type SyncStats = Pick<
  SyncResult,
  'workspaces' | 'projects' | 'folders' | 'interfaces' | 'environments'
>

const { t, locale, setLocale } = useI18n()
const router = useRouter()

const session = ref<SessionInfo | null>(null)
const testing = ref(false)
const syncing = ref(false)
const lastSync = ref('')
const lastSyncStats = ref<SyncStats | null>(null)
const dbPath = ref('')
const clearConfirmOpen = ref(false)
const { showToast } = useToast()

const busy = computed(() => testing.value || syncing.value)

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

onMounted(async () => {
  session.value = await invoke('get_session')
  dbPath.value = await invoke('get_db_path')
  const saved = await invoke<string | null>('get_setting_value', { key: 'last_sync_at' })
  lastSync.value = saved || ''
  const savedStats = await invoke<string | null>('get_setting_value', { key: 'last_sync_stats' })
  lastSyncStats.value = parseSyncStats(savedStats)
})

async function onTest() {
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
.settings {
  min-height: 100%;
  background: var(--color-bg);
}

.bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--color-topbar);
  border-bottom: 1px solid var(--color-border);
}

h1 {
  margin: 0;
  font-size: 19px;
}

.content {
  max-width: 720px;
  margin: 0 auto;
  padding: 20px 16px 40px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.card h2 {
  margin: 0;
  font-size: 16px;
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
  padding: 16px;
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

.sync-aside h3 {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
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
  font-size: 12px;
  color: var(--color-text-secondary);
}

.sync-stats strong {
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

.sync-stats strong.empty {
  color: var(--color-text-secondary);
  background: color-mix(in srgb, var(--color-text-secondary) 10%, transparent);
  font-weight: 500;
}

.row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
}

.btn.active {
  border-color: var(--color-primary);
  background: var(--color-active);
}

.path {
  font-family: var(--font-mono);
  font-size: 13px;
  word-break: break-all;
}
</style>
