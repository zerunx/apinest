<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="overlay"
      @mousedown="onOverlayMouseDown"
      @mouseup="onOverlayMouseUp"
    >
      <div class="dialog" role="dialog" aria-modal="true">
        <button type="button" class="close-btn" :title="t('common.close')" @click="close">
          <span>×</span>
        </button>

        <div class="panel">
          <div class="panel-inner">
            <div class="toolbar">
              <div>
                <h2>{{ t('dict.title') }}</h2>
                <p v-if="workspaceName" class="muted">
                  {{ t('dict.manageDesc', { name: workspaceName }) }}
                </p>
              </div>
            </div>

            <div class="tabs-row">
              <div class="tabs" role="tablist">
                <button
                  type="button"
                  class="tab"
                  role="tab"
                  :class="{ active: tab === 'custom' }"
                  :aria-selected="tab === 'custom'"
                  @click="tab = 'custom'"
                >
                  {{ t('dict.tabCustom') }}
                </button>
                <button
                  type="button"
                  class="tab"
                  role="tab"
                  :class="{ active: tab === 'system' }"
                  :aria-selected="tab === 'system'"
                  @click="tab = 'system'"
                >
                  {{ t('dict.tabSystem') }}
                </button>
              </div>
              <button
                v-if="tab === 'custom'"
                class="btn btn-primary"
                type="button"
                @click="openForm('create')"
              >
                {{ t('dict.create') }}
              </button>
              <button
                v-else-if="pendingImportCount > 0"
                class="btn btn-primary"
                type="button"
                :disabled="importingAll"
                @click="importAllSystem"
              >
                {{ importingAll ? '…' : t('dict.importAll') }}
              </button>
            </div>

            <div v-if="tab === 'custom'" class="table-wrap">
              <table class="data-table">
                <thead>
                  <tr>
                    <th>{{ t('dict.colName') }}</th>
                    <th style="width: 100px">{{ t('dict.colType') }}</th>
                    <th>{{ t('dict.colDescription') }}</th>
                    <th>{{ t('dict.colDefault') }}</th>
                    <th style="width: 88px">{{ t('common.actions') }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="row in customRows" :key="row.id">
                    <td>
                      <code class="name-code">{{ row.name }}</code>
                    </td>
                    <td>{{ typeLabel(row.field_type) }}</td>
                    <td class="muted cell-ellipsis" :title="row.description">
                      {{ row.description || '—' }}
                    </td>
                    <td class="cell-ellipsis" :title="row.default_value">
                      {{ row.default_value || '' }}
                    </td>
                    <td class="actions-cell">
                      <button
                        type="button"
                        class="icon-btn"
                        :title="t('common.edit')"
                        @click="openForm('edit', row)"
                      >
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                          <path d="M12 20h9" />
                          <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4 12.5-12.5z" />
                        </svg>
                      </button>
                      <button
                        type="button"
                        class="icon-btn danger"
                        :title="t('common.delete')"
                        @click="deleteRow(row)"
                      >
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                          <path d="M3 6h18" />
                          <path d="M8 6V4h8v2" />
                          <path d="M19 6l-1 14H6L5 6" />
                          <path d="M10 11v6M14 11v6" />
                        </svg>
                      </button>
                    </td>
                  </tr>
                  <tr v-if="!customRows.length">
                    <td colspan="5" class="empty muted">{{ t('dict.emptyCustom') }}</td>
                  </tr>
                </tbody>
              </table>
            </div>

            <div v-else class="table-wrap">
              <table class="data-table">
                <thead>
                  <tr>
                    <th>{{ t('dict.colName') }}</th>
                    <th style="width: 100px">{{ t('dict.colType') }}</th>
                    <th>{{ t('dict.colDescription') }}</th>
                    <th>{{ t('dict.colDefault') }}</th>
                    <th style="width: 72px">{{ t('common.actions') }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="row in systemRows" :key="row.key">
                    <td>
                      <code class="name-code">{{ row.name }}</code>
                    </td>
                    <td>{{ typeLabel(row.field_type) }}</td>
                    <td class="muted cell-ellipsis" :title="row.description">
                      {{ row.description || '—' }}
                    </td>
                    <td class="cell-ellipsis" :title="row.default_value">
                      {{ row.default_value || '' }}
                    </td>
                    <td class="actions-cell">
                      <span v-if="importedKeys.has(row.key)" class="imported" :title="t('dict.imported')">
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                          <path d="M5 12l5 5L20 7" />
                        </svg>
                      </span>
                      <button
                        v-else
                        type="button"
                        class="icon-btn"
                        :title="t('dict.import')"
                        :disabled="importingKey === row.key"
                        @click="importSystem(row)"
                      >
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                          <path d="M12 3v12" />
                          <path d="M7 10l5 5 5-5" />
                          <path d="M4 21h16" />
                        </svg>
                      </button>
                    </td>
                  </tr>
                  <tr v-if="!systemRows.length">
                    <td colspan="5" class="empty muted">{{ t('dict.emptySystem') }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>

  <Teleport to="body">
    <div
      v-if="formOpen"
      class="overlay overlay-nested"
      @mousedown="onFormOverlayDown"
      @mouseup="onFormOverlayUp"
    >
      <div
        class="form-dialog"
        role="dialog"
        aria-modal="true"
        @mousedown.stop
        @keydown.escape.prevent="formOpen = false"
      >
        <h3>{{ formMode === 'create' ? t('dict.create') : t('dict.edit') }}</h3>
        <label class="field">
          <span>{{ t('dict.colName') }}</span>
          <input
            v-model="form.name"
            class="input"
            type="text"
            maxlength="100"
            :placeholder="t('dict.namePlaceholder')"
            @keydown.enter.prevent="submitForm"
          />
        </label>
        <label class="field">
          <span>{{ t('dict.colType') }}</span>
          <MenuSelect
            v-model="form.field_type"
            class="type-select"
            :options="typeOptions"
            :placeholder="t('dict.colType')"
            compact
          />
        </label>
        <label class="field">
          <span>{{ t('dict.colDescription') }}</span>
          <input
            v-model="form.description"
            class="input"
            type="text"
            maxlength="255"
            :placeholder="t('dict.descriptionPlaceholder')"
          />
        </label>
        <label class="field">
          <span>{{ t('dict.colDefault') }}</span>
          <input
            v-model="form.default_value"
            class="input"
            type="text"
            maxlength="500"
            :placeholder="t('dict.defaultPlaceholder')"
            @keydown.enter.prevent="submitForm"
          />
        </label>
        <p v-if="formError" class="error">{{ formError }}</p>
        <div class="form-actions">
          <button class="btn" type="button" @click="formOpen = false">
            {{ t('common.cancel') }}
          </button>
          <button class="btn btn-primary" type="button" :disabled="saving" @click="submitForm">
            {{ saving ? '…' : t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, reactive, ref, toRef, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import { useOverlayDismiss } from '../composables/useOverlayDismiss'
import { useToast } from '../composables/useToast'
import type { DictionaryEntryRow, SystemDictionaryItem } from '../types'
import MenuSelect from './MenuSelect.vue'

const props = defineProps<{
  open: boolean
  workspaceId: number | null
  workspaceName?: string
}>()

const emit = defineEmits<{
  'update:open': [boolean]
}>()

const { t } = useI18n()
const { showToast } = useToast()

const tab = ref<'custom' | 'system'>('custom')
const customRows = ref<DictionaryEntryRow[]>([])
const systemRows = ref<SystemDictionaryItem[]>([])
const importingKey = ref('')
const importingAll = ref(false)
const formOpen = ref(false)
const formMode = ref<'create' | 'edit'>('create')
const formError = ref('')
const saving = ref(false)
const editId = ref<number | null>(null)
const form = reactive({
  name: '',
  field_type: 'string',
  description: '',
  default_value: '',
})

const typeOptions = computed(() => [
  { value: 'string', label: 'string' },
  { value: 'integer', label: 'integer' },
  { value: 'number', label: 'number' },
  { value: 'boolean', label: 'boolean' },
  { value: 'file', label: 'file' },
])

const importedKeys = computed(() => {
  const keys = new Set<string>()
  for (const row of customRows.value) {
    if (row.source_system_key) keys.add(row.source_system_key)
  }
  return keys
})

const pendingImportCount = computed(
  () => systemRows.value.filter((row) => !importedKeys.value.has(row.key)).length,
)

const openRef = toRef(props, 'open')

function close() {
  emit('update:open', false)
}

const { onOverlayMouseDown, onOverlayMouseUp } = useOverlayDismiss(openRef, close)

let formPointerDownOnOverlay = false
function onFormOverlayDown(e: MouseEvent) {
  formPointerDownOnOverlay = e.target === e.currentTarget
}
function onFormOverlayUp(e: MouseEvent) {
  if (formPointerDownOnOverlay && e.target === e.currentTarget) {
    formOpen.value = false
  }
  formPointerDownOnOverlay = false
}

function typeLabel(type: string) {
  return type || 'string'
}

async function loadCustom() {
  if (!props.workspaceId) {
    customRows.value = []
    return
  }
  try {
    customRows.value = await invoke('list_workspace_dictionaries', {
      workspaceId: props.workspaceId,
    })
  } catch {
    customRows.value = []
  }
}

async function loadSystem() {
  try {
    systemRows.value = await invoke('list_system_dictionaries')
  } catch {
    systemRows.value = []
  }
}

async function reloadAll() {
  await Promise.all([loadCustom(), loadSystem()])
}

watch(
  () => [props.open, props.workspaceId] as const,
  async ([open]) => {
    if (!open) return
    tab.value = 'custom'
    formOpen.value = false
    await reloadAll()
  },
)

function openForm(mode: 'create' | 'edit', row?: DictionaryEntryRow) {
  formMode.value = mode
  formError.value = ''
  saving.value = false
  if (mode === 'edit' && row) {
    editId.value = row.id
    form.name = row.name
    form.field_type = row.field_type || 'string'
    form.description = row.description || ''
    form.default_value = row.default_value || ''
  } else {
    editId.value = null
    form.name = ''
    form.field_type = 'string'
    form.description = ''
    form.default_value = ''
  }
  formOpen.value = true
}

async function submitForm() {
  if (!props.workspaceId) return
  const name = form.name.trim()
  if (!name) {
    formError.value = t('dict.nameRequired')
    return
  }
  saving.value = true
  formError.value = ''
  try {
    if (formMode.value === 'edit' && editId.value != null) {
      await invoke('update_workspace_dictionary', {
        args: {
          id: editId.value,
          workspace_id: props.workspaceId,
          name,
          field_type: form.field_type,
          description: form.description,
          default_value: form.default_value,
        },
      })
      showToast(t('dict.updateSuccess'), 1500, 'success')
    } else {
      await invoke('create_workspace_dictionary', {
        args: {
          workspace_id: props.workspaceId,
          name,
          field_type: form.field_type,
          description: form.description,
          default_value: form.default_value,
        },
      })
      showToast(t('dict.createSuccess'), 1500, 'success')
    }
    formOpen.value = false
    await loadCustom()
  } catch (e) {
    formError.value = String(e)
  } finally {
    saving.value = false
  }
}

async function deleteRow(row: DictionaryEntryRow) {
  if (!props.workspaceId) return
  try {
    await invoke('delete_workspace_dictionary', {
      workspaceId: props.workspaceId,
      entryId: row.id,
    })
    showToast(t('dict.deleteSuccess'), 1500, 'success')
    await loadCustom()
  } catch (e) {
    showToast(String(e), 2200, 'error')
  }
}

async function importSystem(row: SystemDictionaryItem) {
  if (!props.workspaceId) return
  importingKey.value = row.key
  try {
    await invoke('import_system_dictionary', {
      args: {
        workspace_id: props.workspaceId,
        system_key: row.key,
      },
    })
    showToast(t('dict.importSuccess'), 1500, 'success')
    await loadCustom()
  } catch (e) {
    showToast(String(e), 2200, 'error')
  } finally {
    importingKey.value = ''
  }
}

async function importAllSystem() {
  if (!props.workspaceId) return
  const pending = systemRows.value.filter((row) => !importedKeys.value.has(row.key))
  if (!pending.length) return
  importingAll.value = true
  let ok = 0
  let failed = 0
  try {
    for (const row of pending) {
      try {
        await invoke('import_system_dictionary', {
          args: {
            workspace_id: props.workspaceId,
            system_key: row.key,
          },
        })
        ok += 1
      } catch {
        failed += 1
      }
    }
    await loadCustom()
    if (failed === 0) {
      showToast(t('dict.importAllSuccess', { count: ok }), 1500, 'success')
    } else {
      showToast(t('dict.importAllPartial', { ok, failed }), 2200, 'warning')
    }
  } finally {
    importingAll.value = false
  }
}
</script>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 80;
  background: rgba(0, 0, 0, 0.35);
  display: grid;
  place-items: center;
  padding: 24px;
}

.overlay-nested {
  z-index: 90;
  background: rgba(0, 0, 0, 0.4);
}

.dialog {
  position: relative;
  width: min(920px, 100%);
  height: min(640px, calc(100vh - 48px));
  border-radius: 12px;
  background: var(--color-settings-panel);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-menu);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.close-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 2;
  width: 28px;
  height: 28px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 18px;
  line-height: 1;
  display: grid;
  place-items: center;
}

.close-btn:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.panel {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  font-size: 13px;
  display: flex;
  flex-direction: column;
}

.panel-inner {
  padding: 24px 52px 28px 28px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  box-sizing: border-box;
}

.toolbar {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  flex-shrink: 0;
}

.toolbar h2 {
  margin: 0 0 4px;
  font-size: 17px;
  font-weight: 650;
}

.toolbar .muted {
  margin: 0;
  font-size: 12px;
}

.tabs-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  min-height: 34px;
  flex-shrink: 0;
}

.tabs-row .btn {
  flex-shrink: 0;
  height: 32px;
  padding: 0 14px;
  font-size: 13px;
}

.tabs {
  display: inline-flex;
  gap: 2px;
  padding: 3px;
  border-radius: 8px;
  background: var(--color-hover);
  width: fit-content;
}

.tab {
  border: 0;
  background: transparent;
  color: var(--color-text-secondary);
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
}

.tab:hover {
  color: var(--color-text);
}

.tab.active {
  background: var(--color-surface);
  color: var(--color-text);
  font-weight: 600;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.06);
}

.table-wrap {
  flex: 1;
  min-height: 0;
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  background: var(--color-surface);
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.data-table th,
.data-table td {
  padding: 10px 12px;
  text-align: left;
  border-bottom: 1px solid var(--color-border);
  vertical-align: middle;
}

.data-table th {
  font-weight: 600;
  color: var(--color-text-secondary);
  background: var(--color-hover);
  position: sticky;
  top: 0;
  z-index: 1;
}

.data-table tr:last-child td {
  border-bottom: 0;
}

.empty {
  text-align: center;
  padding: 28px 12px !important;
}

.name-code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}

.cell-ellipsis {
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.actions-cell {
  white-space: nowrap;
  display: flex;
  align-items: center;
  gap: 2px;
}

.icon-btn {
  width: 28px;
  height: 28px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-secondary);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.icon-btn:hover:not(:disabled) {
  background: var(--color-hover);
  color: var(--color-text);
}

.icon-btn.danger:hover:not(:disabled) {
  background: var(--color-danger-hover);
  color: var(--color-danger);
}

.icon-btn:disabled {
  opacity: 0.55;
  cursor: default;
}

.imported {
  width: 28px;
  height: 28px;
  color: #49cc90;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.form-dialog {
  width: min(420px, 100%);
  border-radius: 12px;
  background: var(--color-menu);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-menu);
  padding: 20px 20px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-dialog h3 {
  margin: 0;
  font-size: 16px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
}

.field > span {
  color: var(--color-text-secondary);
}

.type-select {
  width: 100%;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}

.error {
  margin: 0;
  color: var(--color-danger);
  font-size: 12px;
}
</style>
