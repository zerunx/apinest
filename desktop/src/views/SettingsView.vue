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

      <section class="card">
        <h2>{{ t('settings.sync') }}</h2>
        <p class="muted">{{ t('settings.server') }}: {{ session?.base_url || '—' }}</p>
        <p class="muted">{{ session?.account || '' }}</p>

        <div class="row">
          <label class="radio">
            <input v-model="syncScope" type="radio" value="current" />
            {{ t('settings.syncCurrent') }}
          </label>
          <label class="radio">
            <input v-model="syncScope" type="radio" value="all" />
            {{ t('settings.syncAll') }}
          </label>
        </div>

        <div class="row">
          <button class="btn" type="button" :disabled="busy" @click="onTest">
            {{ testing ? t('settings.testing') : t('settings.test') }}
          </button>
          <button class="btn btn-primary" type="button" :disabled="busy" @click="onSync">
            {{ syncing ? t('settings.syncing') : t('settings.syncNow') }}
          </button>
        </div>

        <p v-if="message" class="msg">{{ message }}</p>
        <p v-if="lastSync" class="muted">{{ t('settings.lastSync') }}: {{ lastSync }}</p>
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
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import type { SessionInfo, SyncResult } from '../types'

const { t, locale, setLocale } = useI18n()
const router = useRouter()

const session = ref<SessionInfo | null>(null)
const syncScope = ref<'current' | 'all'>('current')
const testing = ref(false)
const syncing = ref(false)
const message = ref('')
const lastSync = ref('')
const dbPath = ref('')
const currentWorkspaceId = ref<number | null>(null)

const busy = computed(() => testing.value || syncing.value)

onMounted(async () => {
  session.value = await invoke('get_session')
  dbPath.value = await invoke('get_db_path')
  const ctx = await invoke<[number | null, number | null, number | null]>('get_active_context')
  currentWorkspaceId.value = ctx[0]
  const saved = await invoke<string | null>('get_setting_value', { key: 'last_sync_at' })
  if (saved) lastSync.value = saved
})

async function onTest() {
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
  syncing.value = true
  message.value = ''
  try {
    const workspaceId =
      syncScope.value === 'current' ? currentWorkspaceId.value : null
    const result = await invoke<SyncResult>('sync_now', {
      args: { workspace_id: workspaceId },
    })
    lastSync.value = result.synced_at
    await invoke('set_setting_value', { key: 'last_sync_at', value: result.synced_at })
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
    message.value = 'local cache cleared'
    lastSync.value = ''
  } catch (e) {
    message.value = String(e)
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

.row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
}

.radio {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 15px;
}

.btn.active {
  border-color: var(--color-primary);
  background: var(--color-active);
}

.msg {
  margin: 0;
  font-size: 14px;
  white-space: pre-wrap;
}

.path {
  font-family: var(--font-mono);
  font-size: 13px;
  word-break: break-all;
}
</style>
