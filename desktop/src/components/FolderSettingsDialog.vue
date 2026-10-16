<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="overlay"
      @mousedown="onOverlayMouseDown"
      @mouseup="onOverlayMouseUp"
    >
      <div class="dialog" role="dialog" aria-modal="true" @mousedown.stop>
        <button type="button" class="close-btn" :title="t('common.close')" @click="close">
          <span>×</span>
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
          <div v-if="active === 'general'" class="panel-inner">
            <h2>{{ t('settings.nav.general') }}</h2>
            <div class="block">
              <h3>{{ t('folderSettings.folderName') }}</h3>
              <p class="muted">{{ t('folderSettings.folderNameHint') }}</p>
              <div class="name-row">
                <input
                  v-model="folderName"
                  class="input name-input"
                  type="text"
                  maxlength="100"
                  :placeholder="t('folderSettings.folderNamePlaceholder')"
                  @keydown.enter.prevent="onSave"
                />
              </div>
            </div>
            <div class="block">
              <h3>{{ t('folderSettings.description') }}</h3>
              <p class="muted">{{ t('folderSettings.descriptionHint') }}</p>
              <div class="name-row">
                <textarea
                  v-model="folderDescription"
                  class="input desc-input"
                  rows="4"
                  maxlength="500"
                  :placeholder="t('folderSettings.descriptionPlaceholder')"
                />
              </div>
            </div>
            <div class="actions">
              <button class="btn" type="button" @click="close">{{ t('common.cancel') }}</button>
              <button
                class="btn btn-primary"
                type="button"
                :disabled="saving || !folderId"
                @click="onSave"
              >
                {{ saving ? '…' : t('common.save') }}
              </button>
            </div>
          </div>

          <div v-else-if="active === 'request'" class="panel-inner">
            <h2>{{ t('folderSettings.nav.request') }}</h2>
            <div class="block">
              <h3>{{ t('settings.requestOptions') }}</h3>
              <p class="muted">{{ t('folderSettings.timeoutHint') }}</p>
              <div class="timeout-row">
                <label for="folder-request-timeout">{{ t('settings.requestTimeout') }}</label>
                <input
                  id="folder-request-timeout"
                  v-model.number="timeoutSecs"
                  class="input timeout-input"
                  type="number"
                  min="0"
                  max="1800"
                  step="1"
                />
                <span class="muted">s</span>
              </div>
            </div>

            <div class="block">
              <h3>{{ t('settings.httpVersion') }}</h3>
              <p class="muted">{{ t('settings.httpVersionHint') }}</p>
              <div class="row">
                <button
                  class="btn"
                  type="button"
                  :class="{ active: httpVersion === 'auto' }"
                  @click="httpVersion = 'auto'"
                >
                  {{ t('settings.httpVersionAuto') }}
                </button>
                <button
                  class="btn"
                  type="button"
                  :class="{ active: httpVersion === 'http1.1' }"
                  @click="httpVersion = 'http1.1'"
                >
                  HTTP/1.1
                </button>
                <button
                  class="btn"
                  type="button"
                  :class="{ active: httpVersion === 'http2' }"
                  @click="httpVersion = 'http2'"
                >
                  HTTP/2
                </button>
              </div>
            </div>

            <div class="actions">
              <button class="btn" type="button" @click="close">{{ t('common.cancel') }}</button>
              <button
                class="btn btn-primary"
                type="button"
                :disabled="saving || !folderId"
                @click="onSave"
              >
                {{ saving ? '…' : t('common.save') }}
              </button>
            </div>
          </div>

          <div v-else-if="active === 'headers'" class="panel-inner">
            <h2>{{ t('request.headers') }}</h2>
            <p class="muted section-hint">{{ t('folderSettings.headersHint') }}</p>
            <div class="param-table">
              <div class="param-table__row param-table__row--head" :style="{ gridTemplateColumns: headerGrid }">
                <label class="param-table__check">
                  <input
                    type="checkbox"
                    :checked="headersSelectAll.checked"
                    :ref="(el) => setCheckboxIndeterminate(el, headersSelectAll.indeterminate)"
                    :title="t('common.selectAll')"
                    :aria-label="t('common.selectAll')"
                    @change="setAllHeadersEnabled(($event.target as HTMLInputElement).checked)"
                  />
                </label>
                <span class="param-table__head-cell">{{ t('common.key') }}</span>
                <span class="param-table__head-cell">{{ t('common.value') }}</span>
                <span class="param-table__head-action" />
              </div>
              <div
                v-for="(h, idx) in headers"
                :key="idx"
                class="param-table__row"
                :class="{ 'param-table__row--off': h.enabled === false }"
                :style="{ gridTemplateColumns: headerGrid }"
              >
                <label class="param-table__check">
                  <input
                    type="checkbox"
                    :checked="h.enabled !== false"
                    @change="toggleHeader(idx, ($event.target as HTMLInputElement).checked)"
                  />
                </label>
                <input
                  class="param-table__cell"
                  :value="h.key"
                  :placeholder="t('common.key')"
                  @input="patchHeader(idx, 'key', ($event.target as HTMLInputElement).value)"
                />
                <div class="param-table__value">
                  <div class="param-table__value-shell">
                    <textarea
                      class="param-table__cell param-table__cell--inner param-table__value-input"
                      rows="1"
                      :value="h.value"
                      :placeholder="t('common.value')"
                      @focus="onExpandableValueFocus"
                      @blur="onExpandableValueBlur"
                      @input="onHeaderValueInput(idx, $event)"
                    />
                  </div>
                </div>
                <button
                  type="button"
                  class="param-table__del"
                  :disabled="headers.length <= 1"
                  @click="removeHeader(idx)"
                >
                  ×
                </button>
              </div>
            </div>
            <div class="actions">
              <button class="btn" type="button" @click="close">{{ t('common.cancel') }}</button>
              <button
                class="btn btn-primary"
                type="button"
                :disabled="saving || !folderId"
                @click="onSave"
              >
                {{ saving ? '…' : t('common.save') }}
              </button>
            </div>
          </div>

          <div v-else class="panel-inner panel-auth">
            <h2>{{ t('request.authorization') }}</h2>
            <p class="muted section-hint">{{ t('folderSettings.authHint') }}</p>
            <div class="auth-pane">
              <div class="auth-pane__layout">
                <div class="auth-pane__left">
                  <span class="auth-pane__label">{{ t('request.authType') }}</span>
                  <MenuSelect
                    :model-value="authConfig.type"
                    compact
                    class="auth-pane__type"
                    :menu-width="200"
                    :options="authTypeOptions"
                    @update:model-value="onAuthTypeChange"
                  />
                  <p v-if="authConfig.type !== 'none'" class="auth-pane__hint muted">{{ authTypeHint }}</p>
                </div>
                <div class="auth-pane__divider" />
                <div class="auth-pane__right">
                  <template v-if="authConfig.type === 'none'">
                    <span class="auth-pane__label auth-pane__label--spacer" aria-hidden="true">&nbsp;</span>
                    <p class="auth-pane__hint auth-pane__hint--none muted">{{ t('request.authNoneHint') }}</p>
                  </template>
                  <template v-else>
                    <div class="auth-pane__fields">
                      <template v-for="field in authFields" :key="String(field.key)">
                        <label
                          v-if="field.kind === 'checkbox'"
                          class="auth-pane__check"
                        >
                          <input
                            :checked="authBool(field.key)"
                            type="checkbox"
                            @change="
                              setAuthBool(field.key, ($event.target as HTMLInputElement).checked)
                            "
                          />
                          <span>{{ t(field.labelKey) }}</span>
                        </label>
                        <template v-else>
                          <span class="auth-pane__label">{{ t(field.labelKey) }}</span>
                          <MenuSelect
                            v-if="field.kind === 'select'"
                            :model-value="authString(field.key)"
                            compact
                            class="auth-pane__input"
                            :menu-width="200"
                            :options="authFieldSelectOptions(field)"
                            @update:model-value="(v) => setAuthField(field.key, v)"
                          />
                          <textarea
                            v-else-if="field.kind === 'textarea'"
                            class="input auth-pane__textarea"
                            :rows="field.rows ?? 4"
                            :value="authString(field.key)"
                            :placeholder="field.placeholderKey ? t(field.placeholderKey) : ''"
                            @input="
                              setAuthField(field.key, ($event.target as HTMLTextAreaElement).value)
                            "
                          />
                          <input
                            v-else
                            class="input auth-pane__input"
                            :type="field.kind === 'password' ? 'password' : 'text'"
                            :value="authString(field.key)"
                            :placeholder="field.placeholderKey ? t(field.placeholderKey) : ''"
                            @input="
                              setAuthField(field.key, ($event.target as HTMLInputElement).value)
                            "
                          />
                        </template>
                      </template>
                    </div>
                  </template>
                </div>
              </div>
            </div>
            <div class="actions">
              <button class="btn" type="button" :disabled="saving" @click="close">
                {{ t('common.cancel') }}
              </button>
              <button
                class="btn btn-primary"
                type="button"
                :disabled="saving || !folderId"
                @click="onSave"
              >
                {{ saving ? '…' : t('common.save') }}
              </button>
            </div>
          </div>
        </section>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, reactive, ref, toRef, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import { useToast } from '../composables/useToast'
import { useOverlayDismiss } from '../composables/useOverlayDismiss'
import MenuSelect from './MenuSelect.vue'
import {
  AUTH_FIELDS,
  AUTH_TYPE_HINT_KEYS,
  AUTH_TYPE_LABEL_KEYS,
  AUTH_TYPE_PRIMARY,
  AUTH_TYPE_SECONDARY,
  authConfigFromPersisted,
  authConfigToPersisted,
  emptyAuthConfig,
  type AuthConfig,
  type AuthFieldDef,
  type AuthType,
} from '../utils/auth'
import type { HeaderPair } from '../types'

type FolderSettings = {
  id: number
  name: string
  description: string
  headers: HeaderPair[]
  authorization?: Record<string, unknown>
  request_timeout_secs: number
  timeout_override: boolean
  http_version: string
  global_timeout_secs: number
}

type MenuId = 'general' | 'request' | 'headers' | 'auth'

const props = defineProps<{
  open: boolean
  folderId: number | null
}>()

const emit = defineEmits<{
  'update:open': [boolean]
  saved: []
}>()

const { t } = useI18n()
const { showToast } = useToast()
const openRef = toRef(props, 'open')

const active = ref<MenuId>('general')
const folderName = ref('')
const folderDescription = ref('')
const headers = ref<HeaderPair[]>([emptyHeader()])
const authConfig = reactive<AuthConfig>(emptyAuthConfig())
const timeoutSecs = ref(60)
type HttpVersion = 'auto' | 'http1.1' | 'http2'
const httpVersion = ref<HttpVersion>('auto')
const saving = ref(false)
const headerGrid = '40px minmax(0, 1fr) minmax(0, 1.2fr) 40px'

const authTypeOptions = computed(() => {
  const primary = AUTH_TYPE_PRIMARY.map((value) => ({
    value,
    label: t(AUTH_TYPE_LABEL_KEYS[value]),
  }))
  const secondary = AUTH_TYPE_SECONDARY.map((value) => ({
    value,
    label: t(AUTH_TYPE_LABEL_KEYS[value]),
  }))
  return [...primary, { separator: true }, ...secondary]
})

const authFields = computed(() => AUTH_FIELDS[authConfig.type] ?? [])
const authTypeHint = computed(() => t(AUTH_TYPE_HINT_KEYS[authConfig.type]))

function onAuthTypeChange(value: string | number | null) {
  authConfig.type = String(value ?? 'none') as AuthType
}

function authFieldSelectOptions(field: AuthFieldDef) {
  return (field.options ?? []).map((o) => ({
    value: o.value,
    label: t(o.labelKey),
  }))
}

function authString(key: keyof AuthConfig): string {
  const v = authConfig[key]
  return v == null ? '' : String(v)
}

function authBool(key: keyof AuthConfig): boolean {
  return Boolean(authConfig[key])
}

function setAuthField(key: keyof AuthConfig, value: string | number | null) {
  ;(authConfig as Record<string, unknown>)[key as string] = value == null ? '' : String(value)
}

function setAuthBool(key: keyof AuthConfig, value: boolean) {
  ;(authConfig as Record<string, unknown>)[key as string] = value
}

function resetAuthConfig() {
  Object.assign(authConfig, emptyAuthConfig())
}

function emptyHeader(): HeaderPair {
  return { key: '', value: '', enabled: true }
}

const headersSelectAll = computed(() => {
  const rows = headers.value
  if (!rows.length) return { checked: false, indeterminate: false }
  const enabled = rows.filter((h) => h.enabled !== false).length
  return {
    checked: enabled === rows.length,
    indeterminate: enabled > 0 && enabled < rows.length,
  }
})

function setCheckboxIndeterminate(el: unknown, indeterminate: boolean) {
  const input = el as HTMLInputElement | null
  if (!input) return
  void nextTick(() => {
    input.indeterminate = indeterminate
  })
}

function setAllHeadersEnabled(enabled: boolean) {
  headers.value = headers.value.map((h) => ({ ...h, enabled }))
}

function normalizeHttpVersion(raw: string): HttpVersion {
  if (raw === 'http2') return 'http2'
  if (raw === 'http1.1') return 'http1.1'
  return 'auto'
}

function normalizeHeaders(list: HeaderPair[] | undefined | null): HeaderPair[] {
  const rows: HeaderPair[] = (list ?? [])
    .map((h) => ({
      key: String(h.key ?? ''),
      value: String(h.value ?? ''),
      enabled: h.enabled !== false,
    }))
    .filter((h) => h.key.trim() || h.value.trim() || h.enabled)
  if (!rows.length) return [emptyHeader()]
  if (rows[rows.length - 1].key.trim() || rows[rows.length - 1].value.trim()) {
    rows.push(emptyHeader())
  }
  return rows
}

const menu: { id: MenuId; labelKey: string; icon: string }[] = [
  {
    id: 'general',
    labelKey: 'settings.nav.general',
    icon: `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9c.3.6.9 1 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"/></svg>`,
  },
  {
    id: 'request',
    labelKey: 'folderSettings.nav.request',
    icon: `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M4 7h16M4 12h10M4 17h13"/><circle cx="18" cy="12" r="2"/></svg>`,
  },
  {
    id: 'headers',
    labelKey: 'request.headers',
    icon: `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M4 7h16M4 12h16M4 17h10"/></svg>`,
  },
  {
    id: 'auth',
    labelKey: 'request.authorization',
    icon: `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/></svg>`,
  },
]

function close() {
  emit('update:open', false)
}

const { onOverlayMouseDown, onOverlayMouseUp } = useOverlayDismiss(openRef, close, {
  escape: true,
})

function clampTimeout(raw: number) {
  if (!Number.isFinite(raw) || raw < 0) return 0
  return Math.min(1800, Math.floor(raw))
}

function ensureTrailingEmpty() {
  const rows = headers.value
  if (!rows.length) {
    headers.value = [emptyHeader()]
    return
  }
  const last = rows[rows.length - 1]
  if (last.key.trim() || last.value.trim()) {
    headers.value = [...rows, emptyHeader()]
  }
}

function patchHeader(idx: number, field: 'key' | 'value', value: string) {
  headers.value = headers.value.map((h, i) => (i === idx ? { ...h, [field]: value } : h))
  ensureTrailingEmpty()
}

function toggleHeader(idx: number, enabled: boolean) {
  headers.value = headers.value.map((h, i) => (i === idx ? { ...h, enabled } : h))
}

function removeHeader(idx: number) {
  const next = headers.value.filter((_, i) => i !== idx)
  headers.value = next.length ? next : [emptyHeader()]
  ensureTrailingEmpty()
}

const VALUE_MAX_HEIGHT = 240

function valueShell(el: HTMLTextAreaElement): HTMLElement | null {
  return el.closest('.param-table__value-shell') as HTMLElement | null
}

function valueMaxWidthPx(el: HTMLTextAreaElement): number {
  const cell = el.closest('.param-table__value') as HTMLElement | null
  const table = el.closest('.param-table') as HTMLElement | null
  if (!cell || !table) return Math.max(el.offsetWidth, 160)
  const cellRect = cell.getBoundingClientRect()
  const tableRect = table.getBoundingClientRect()
  return Math.max(
    Math.floor(tableRect.right - cellRect.left - 16),
    Math.floor(cellRect.width - 12),
    160,
  )
}

function resizeValueInput(el: HTMLTextAreaElement, expanded: boolean) {
  const shell = valueShell(el)
  if (!shell) return
  if (!expanded) {
    shell.style.height = ''
    shell.style.width = ''
    shell.style.maxWidth = ''
    el.style.height = ''
    el.style.overflowY = ''
    return
  }
  shell.style.maxWidth = `${valueMaxWidthPx(el)}px`
  el.style.overflowY = 'hidden'
  el.style.height = '0px'
  const needed = el.scrollHeight + 2
  const next = Math.min(Math.max(needed, 34), VALUE_MAX_HEIGHT)
  shell.style.height = `${next}px`
  el.style.height = '100%'
  el.style.overflowY = needed > VALUE_MAX_HEIGHT ? 'auto' : 'hidden'
}

function onExpandableValueFocus(event: FocusEvent) {
  const el = event.target as HTMLTextAreaElement
  requestAnimationFrame(() => {
    if (document.activeElement === el) resizeValueInput(el, true)
  })
}

function onExpandableValueBlur(event: FocusEvent) {
  const el = event.target as HTMLTextAreaElement
  resizeValueInput(el, false)
}

function onHeaderValueInput(idx: number, event: Event) {
  const el = event.target as HTMLTextAreaElement
  patchHeader(idx, 'value', el.value)
  if (document.activeElement === el) resizeValueInput(el, true)
}

watch(
  () => [props.open, props.folderId] as const,
  async ([open, folderId]) => {
    if (!open || folderId == null) return
    active.value = 'general'
    saving.value = false
    try {
      const settings = await invoke<FolderSettings>('get_folder_settings', { folderId })
      folderName.value = settings.name
      folderDescription.value = settings.description ?? ''
      headers.value = normalizeHeaders(settings.headers)
      Object.assign(authConfig, authConfigFromPersisted(settings.authorization))
      timeoutSecs.value = clampTimeout(settings.request_timeout_secs)
      httpVersion.value = normalizeHttpVersion(settings.http_version)
    } catch (e) {
      folderName.value = ''
      folderDescription.value = ''
      headers.value = [emptyHeader()]
      resetAuthConfig()
      timeoutSecs.value = 60
      httpVersion.value = 'auto'
      showToast(String(e), 1800, 'error')
      close()
    }
  },
)

async function onSave() {
  if (props.folderId == null) return
  const name = folderName.value.trim()
  if (!name) {
    showToast(t('folderSettings.folderNameRequired'), 1500, 'warning')
    active.value = 'general'
    return
  }
  saving.value = true
  try {
    const secs = clampTimeout(Number(timeoutSecs.value))
    timeoutSecs.value = secs
    await invoke('update_folder_settings', {
      args: {
        id: props.folderId,
        name,
        description: folderDescription.value.trim(),
        headers: headers.value.filter((h) => h.key.trim()),
        authorization: authConfigToPersisted(authConfig),
        request_timeout_secs: secs,
        http_version: httpVersion.value,
      },
    })
    folderName.value = name
    showToast(t('folderSettings.saved'), 1400, 'success')
    emit('saved')
    close()
  } catch (e) {
    showToast(String(e), 1800, 'error')
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 5000;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
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
  top: 8px;
  right: 10px;
  z-index: 2;
  width: 28px;
  height: 28px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 19px;
  line-height: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  cursor: pointer;
}

.close-btn:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.close-btn span {
  display: block;
  transform: translateY(-1px);
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
  cursor: pointer;
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

.panel-auth {
  display: flex;
  flex-direction: column;
  min-height: 100%;
  box-sizing: border-box;
}

.auth-pane {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  margin-bottom: 8px;
}

.auth-pane__layout {
  flex: 1;
  display: flex;
  align-items: stretch;
  min-height: 0;
}

.auth-pane__left {
  width: 240px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-right: 16px;
}

.auth-pane__type {
  width: 200px;
  max-width: 100%;
  flex-shrink: 0;
}

.auth-pane__type :deep(.menu-select) {
  width: 100%;
}

.auth-pane__divider {
  width: 1px;
  flex-shrink: 0;
  align-self: stretch;
  margin: 10px 0;
  background: var(--color-border);
}

.auth-pane__right {
  flex: 1;
  min-width: 0;
  max-width: 480px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-left: 16px;
  overflow: auto;
}

.auth-pane__fields {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-bottom: 12px;
}

.auth-pane__label {
  font-size: 13px;
  font-weight: 600;
  line-height: 20px;
  color: var(--color-text);
}

.auth-pane__label--spacer {
  visibility: hidden;
  user-select: none;
}

.auth-pane__hint {
  margin: 2px 0 0;
  font-size: 12px;
  line-height: 1.55;
}

.auth-pane__hint--none {
  margin: 0;
  line-height: 34px;
}

.auth-pane__input {
  width: 100%;
  max-width: 480px;
  box-sizing: border-box;
}

.auth-pane__textarea {
  width: 100%;
  max-width: 480px;
  min-height: 72px;
  resize: vertical;
  font-family: var(--font-mono, ui-monospace, SFMono-Regular, Menlo, Consolas, monospace);
  font-size: 12px;
  line-height: 1.45;
  padding: 8px 10px;
  box-sizing: border-box;
}

.auth-pane__check {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
  font-size: 13px;
  color: var(--color-text);
  cursor: pointer;
  user-select: none;
}

.section-hint {
  margin: -8px 0 14px;
}

.block {
  margin-bottom: 18px;
  padding: 14px 16px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  background: var(--color-surface);
}

.block h3 {
  margin: 0 0 8px;
  font-size: 14px;
}

.block .muted {
  margin: 0;
}

.name-row {
  margin-top: 10px;
}

.name-input,
.desc-input {
  width: 100%;
  max-width: 420px;
  padding: 8px 10px;
  box-sizing: border-box;
}

.desc-input {
  resize: vertical;
  min-height: 88px;
  line-height: 1.45;
  font: inherit;
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

.row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}

.row .btn.active {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: #fff;
}

.param-table {
  --param-table-head-height: 40px;
  border: 1px solid var(--color-interface-bar-border);
  border-radius: var(--raw-editor-selection-radius, 6px);
  overflow: hidden;
  min-height: 0;
  max-width: 100%;
  min-width: 0;
  background: var(--color-workspace-content);
  margin-bottom: 18px;
}

.param-table__row {
  position: relative;
  z-index: 1;
  display: grid;
  align-items: center;
  min-height: 42px;
  border-bottom: 1px solid var(--color-interface-bar-border);
  background: var(--color-workspace-content);
  transition: background-color 0.12s ease;
}

.param-table__row:last-child {
  border-bottom: 0;
}

.param-table__row:not(.param-table__row--head):hover {
  background: color-mix(in srgb, var(--color-text-secondary) 28%, var(--color-workspace-content));
}

.param-table__row--head {
  height: var(--param-table-head-height);
  min-height: var(--param-table-head-height);
  background: var(--color-workspace-content);
  pointer-events: none;
  user-select: none;
}

.param-table__row--head .param-table__check {
  pointer-events: auto;
  margin: 0;
  height: 100%;
  align-self: stretch;
}

.param-table__head-cell,
.param-table__head-action {
  display: flex;
  align-items: center;
  min-width: 0;
  padding: 0 14px;
  font-size: 12px;
  font-weight: 500;
  letter-spacing: 0.01em;
  color: var(--color-text-secondary);
  background: transparent;
  overflow: hidden;
}

.param-table__head-action {
  padding: 0;
}

.param-table__row--off {
  opacity: 0.55;
}

.param-table__check {
  display: flex;
  align-items: center;
  justify-content: center;
  align-self: center;
  height: 34px;
  margin: 3px 0;
  background: transparent;
  cursor: pointer;
}

.param-table__check input {
  appearance: none;
  -webkit-appearance: none;
  width: 15px;
  height: 15px;
  margin: 0;
  cursor: pointer;
  border: 1px solid color-mix(in srgb, var(--color-text-secondary) 65%, transparent);
  border-radius: var(--raw-editor-selection-radius, 4px);
  background-color: transparent;
  background-repeat: no-repeat;
  background-position: center;
  background-size: 11px 11px;
  box-sizing: border-box;
  transition:
    background-color 0.12s ease,
    border-color 0.12s ease;
}

.param-table__check input:checked {
  background-color: var(--color-primary);
  border-color: var(--color-primary);
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='none'%3E%3Cpath d='M3.2 8.2L6.4 11.4L12.8 4.4' stroke='%23fff' stroke-width='2.4' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
}

.param-table__check input:indeterminate {
  background-color: var(--color-primary);
  border-color: var(--color-primary);
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='none'%3E%3Cpath d='M3.5 8H12.5' stroke='%23fff' stroke-width='2.4' stroke-linecap='round'/%3E%3C/svg%3E");
}

.param-table__check input:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--color-primary) 40%, transparent);
  outline-offset: 1px;
}

.param-table__cell {
  min-width: 0;
  width: calc(100% - 12px);
  height: 34px;
  min-height: 34px;
  margin: 3px 6px;
  border: 1px solid transparent;
  border-radius: 4px;
  background: transparent;
  color: var(--color-interface-field-text);
  padding: 0 10px;
  font: inherit;
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
  transition:
    background-color 0.12s ease,
    border-color 0.12s ease;
}

.param-table__row:not(.param-table__row--head):hover .param-table__cell {
  background: var(--color-workspace-inset);
}

.param-table__cell:focus {
  background: var(--color-workspace-inset);
  border-color: var(--color-primary);
}

.param-table__value {
  position: relative;
  min-width: 0;
  height: 100%;
  min-height: 42px;
  display: flex;
  align-items: flex-start;
}

.param-table:has(.param-table__value-shell:focus-within) {
  overflow: visible;
}

.param-table__row:has(.param-table__value-shell:focus-within) {
  z-index: 40;
}

.param-table__cell--inner {
  flex: 1;
  min-width: 0;
  width: calc(100% - 12px);
}

.param-table__value-shell {
  flex: 1;
  min-width: 0;
  width: calc(100% - 12px);
  height: 34px;
  margin: 3px 6px;
  display: flex;
  align-items: stretch;
  box-sizing: border-box;
  border: 1px solid transparent;
  border-radius: 4px;
}

.param-table__value-shell:focus-within {
  position: absolute;
  left: 6px;
  top: 3px;
  z-index: 41;
  width: calc(100% - 12px);
  height: auto;
  min-height: 34px;
  max-height: 240px;
  margin: 0;
  border-color: var(--color-primary);
  background: var(--color-workspace-content);
  box-shadow: 0 4px 16px color-mix(in srgb, #000 28%, transparent);
  overflow: hidden;
  resize: both;
}

.param-table__value-input {
  resize: none;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  border: none;
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 6px 10px;
  line-height: 20px;
  min-height: 0;
  box-sizing: border-box;
}

.param-table__value-shell:focus-within .param-table__value-input {
  border: none;
  background: transparent;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  text-overflow: clip;
  padding: 6px 10px;
  line-height: 20px;
  overflow-x: hidden;
  overflow-y: hidden;
}

.param-table__del {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  align-self: center;
  width: 28px;
  height: 28px;
  margin: 0;
  padding: 0 0 1px;
  border: 0;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  border-radius: var(--raw-editor-selection-radius, 4px);
  opacity: 0;
  pointer-events: none;
  position: relative;
  z-index: 2;
  transition:
    opacity 0.12s ease,
    color 0.12s ease,
    background-color 0.12s ease;
}

.param-table__row:not(.param-table__row--head):hover .param-table__del {
  opacity: 1;
  pointer-events: auto;
}

.param-table__del:hover {
  color: var(--color-text);
  background: color-mix(in srgb, var(--color-text-secondary) 18%, transparent);
}

.param-table__del:disabled {
  opacity: 0;
  pointer-events: none;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}
</style>
