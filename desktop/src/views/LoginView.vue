<template>
  <div class="login-page">
    <form class="login-card" @submit.prevent="onSubmit">
      <div class="login-brand">
        <img src="/nest.png" alt="ApiNest" width="40" height="40" />
        <h1>{{ t('app.name') }}</h1>
      </div>
      <p class="muted">{{ t('login.hint') }}</p>

      <label class="field">
        <span>{{ t('login.server') }}</span>
        <input v-model="baseUrl" type="url" required placeholder="https://nest.example.com" />
      </label>
      <label class="field">
        <span>{{ t('login.account') }}</span>
        <input v-model="account" required autocomplete="username" />
      </label>
      <label class="field">
        <span>{{ t('login.password') }}</span>
        <input v-model="password" type="password" required autocomplete="current-password" />
      </label>

      <p v-if="error" class="error">{{ error }}</p>
      <button class="btn btn-primary" type="submit" :disabled="loading || !inTauri">
        {{ loading ? '…' : t('login.submit') }}
      </button>
      <button class="btn skip-btn" type="button" @click="enterApp">
        {{ t('login.skip') }}
      </button>
      <p v-if="!inTauri" class="muted browser-hint">{{ t('login.browserHint') }}</p>
    </form>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import { useAppUi } from '../composables/useAppUi'
import type { SessionInfo } from '../types'

const { t } = useI18n()
const router = useRouter()
const { refreshSession } = useAppUi()

const baseUrl = ref('http://127.0.0.1:3000')
const account = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')

const inTauri = computed(
  () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window,
)

function enterApp() {
  router.replace({ name: 'main' })
}

onMounted(async () => {
  if (!inTauri.value) return
  try {
    const session = await invoke<SessionInfo>('get_session')
    if (session.base_url) baseUrl.value = session.base_url
    if (session.logged_in) router.replace({ name: 'main' })
  } catch {
    // ignore
  }
})

async function onSubmit() {
  if (!inTauri.value) {
    error.value = t('login.browserHint')
    return
  }
  loading.value = true
  error.value = ''
  try {
    await invoke('login', {
      args: {
        base_url: baseUrl.value.trim(),
        account: account.value.trim(),
        password: password.value,
      },
    })
    await refreshSession()
    await router.replace({ name: 'main' })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-page {
  height: 100%;
  display: grid;
  place-items: center;
  padding: 24px;
  background: var(--color-content);
}

.login-card {
  width: min(420px, 100%);
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 28px;
  border-radius: 12px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-dialog);
}

.login-brand {
  display: flex;
  align-items: center;
  gap: 12px;
}

.login-brand img {
  border-radius: 8px;
}

.login-brand h1 {
  margin: 0;
  font-size: 23px;
  font-weight: 700;
}

.skip-btn {
  width: 100%;
}

.browser-hint {
  margin: 0;
  font-size: 12px;
  line-height: 1.4;
}

.error {
  margin: 0;
  color: var(--color-danger);
  font-size: 13px;
}

.btn-primary {
  width: 100%;
}
</style>
