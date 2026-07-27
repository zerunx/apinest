<template>
  <Teleport to="body">
    <div
      v-if="loginOpen"
      class="overlay"
      @mousedown="onOverlayMouseDown"
      @mouseup="onOverlayMouseUp"
    >
      <div
        class="dialog"
        role="dialog"
        aria-modal="true"
        :aria-label="t('login.title')"
      >
        <button
          type="button"
          class="close-btn"
          :title="t('common.close')"
          @click="closeLogin"
        >
          ×
        </button>

        <form class="login-form" @submit.prevent="onSubmit">
          <div class="login-brand">
            <img src="/nest.png" alt="" width="40" height="40" draggable="false" />
            <div>
              <h2>{{ t('login.title') }}</h2>
              <p class="muted brand-sub">{{ t('app.name') }}</p>
            </div>
          </div>
          <p class="muted hint">{{ t('login.hint') }}</p>

          <label class="field">
            <span>{{ t('login.server') }}</span>
            <input
              ref="firstInputRef"
              v-model="baseUrl"
              type="url"
              required
              placeholder="https://nest.example.com"
            />
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
          <button class="btn btn-primary submit-btn" type="submit" :disabled="loading || !inTauri">
            {{ loading ? '…' : t('login.submit') }}
          </button>
          <p v-if="!inTauri" class="muted browser-hint">{{ t('login.browserHint') }}</p>
        </form>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import { useAppUi } from '../composables/useAppUi'
import { useOverlayDismiss } from '../composables/useOverlayDismiss'
import type { SessionInfo } from '../types'

const { t } = useI18n()
const { loginOpen, closeLogin, refreshSession } = useAppUi()
const { onOverlayMouseDown, onOverlayMouseUp } = useOverlayDismiss(loginOpen, closeLogin)

const baseUrl = ref('http://127.0.0.1:3000')
const account = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')
const firstInputRef = ref<HTMLInputElement | null>(null)

const inTauri = computed(
  () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window,
)

async function loadSessionDefaults() {
  if (!inTauri.value) return
  try {
    const session = await invoke<SessionInfo>('get_session')
    if (session.base_url) baseUrl.value = session.base_url
    if (session.account) account.value = session.account
  } catch {
    // ignore
  }
}

watch(loginOpen, async (open) => {
  if (!open) return
  error.value = ''
  password.value = ''
  loading.value = false
  await loadSessionDefaults()
  await nextTick()
  firstInputRef.value?.focus()
})

function onEsc(event: KeyboardEvent) {
  if (!loginOpen.value) return
  if (event.key === 'Escape') {
    event.preventDefault()
    closeLogin()
  }
}

onMounted(() => {
  window.addEventListener('keydown', onEsc)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onEsc)
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
    closeLogin()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 1200;
  background: rgba(0, 0, 0, 0.45);
  display: grid;
  place-items: center;
  padding: 24px;
}

.dialog {
  position: relative;
  width: min(420px, 100%);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  box-shadow: var(--shadow-dialog);
  padding: 28px 28px 24px;
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
  cursor: pointer;
}

.close-btn:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.login-brand {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-right: 24px;
}

.login-brand img {
  border-radius: 8px;
}

.login-brand h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
}

.brand-sub {
  margin: 2px 0 0;
  font-size: 12px;
}

.hint {
  margin: 0;
  font-size: 13px;
  line-height: 1.45;
  white-space: pre-line;
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

.submit-btn {
  width: 100%;
  margin-top: 4px;
}
</style>
