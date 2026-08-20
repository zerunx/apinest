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

        <aside class="nav">
          <button
            type="button"
            class="nav-item"
            :class="{ active: section === 'envs' }"
            @click="section = 'envs'"
          >
            <span class="nav-icon" v-html="iconEnv" />
            <span>{{ t('env.manage') }}</span>
          </button>
          <button
            type="button"
            class="nav-item"
            :class="{ active: section === 'vars' }"
            @click="section = 'vars'"
          >
            <span class="nav-icon" v-html="iconVars" />
            <span>{{ t('env.variables') }}</span>
          </button>
        </aside>

        <section class="panel">
          <!-- Environments -->
          <div v-if="section === 'envs'" class="panel-inner">
            <div class="toolbar">
              <div>
                <h2>{{ t('env.manage') }}</h2>
                <p v-if="projectName" class="muted">
                  {{ t('env.manageDesc', { name: projectName }) }}
                </p>
              </div>
              <button class="btn btn-primary" type="button" @click="openEnvForm('create')">
                {{ t('env.create') }}
              </button>
            </div>

            <div class="table-wrap">
              <table class="data-table">
                <thead>
                  <tr>
                    <th>{{ t('env.colName') }}</th>
                    <th>{{ t('env.remark') }}</th>
                    <th style="width: 110px">{{ t('env.default') }}</th>
                    <th style="width: 140px">{{ t('common.actions') }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="row in environments" :key="row.id">
                    <td>{{ row.name }}</td>
                    <td class="muted">{{ row.remark || '—' }}</td>
                    <td>
                      <span v-if="row.is_default" class="default-yes">{{ t('env.yes') }}</span>
                      <button
                        v-else
                        type="button"
                        class="link-btn"
                        @click="setDefault(row)"
                      >
                        {{ t('env.setDefault') }}
                      </button>
                    </td>
                    <td class="actions-cell">
                      <button type="button" class="link-btn" @click="openEnvForm('edit', row)">
                        {{ t('common.edit') }}
                      </button>
                      <button
                        type="button"
                        class="link-btn link-danger"
                        @click="askDeleteEnv(row)"
                      >
                        {{ t('common.delete') }}
                      </button>
                    </td>
                  </tr>
                  <tr v-if="!environments.length">
                    <td colspan="4" class="empty muted">{{ t('env.none') }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <!-- Variables -->
          <div v-else class="panel-inner">
            <div class="toolbar">
              <div>
                <h2>{{ t('env.variables') }}</h2>
                <p v-if="projectName && currentEnvName" class="muted">
                  {{
                    t('env.variableManageDesc', {
                      project: projectName,
                      env: currentEnvName,
                    })
                  }}
                </p>
                <p v-else class="muted">{{ t('env.noEnvironmentForVariables') }}</p>
              </div>
              <button
                class="btn btn-primary"
                type="button"
                :disabled="!manageEnvId"
                @click="openVarForm('create')"
              >
                {{ t('env.createVariable') }}
              </button>
            </div>

            <div v-if="environments.length" class="env-select-row">
              <label>{{ t('env.label') }}</label>
              <MenuSelect
                v-model="manageEnvId"
                class="env-select"
                :options="envOptions"
                :placeholder="t('env.label')"
                compact
              />
            </div>

            <div class="table-wrap">
              <table class="data-table">
                <thead>
                  <tr>
                    <th>{{ t('env.variableKey') }}</th>
                    <th>{{ t('env.variableValue') }}</th>
                    <th>{{ t('env.variableDescription') }}</th>
                    <th style="width: 140px">{{ t('common.actions') }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="row in variables" :key="row.id">
                    <td>
                      <code class="var-key">{{ row.key }}</code>
                    </td>
                    <td class="cell-ellipsis" :title="row.value">{{ row.value || '—' }}</td>
                    <td class="muted cell-ellipsis" :title="row.description">
                      {{ row.description || '—' }}
                    </td>
                    <td class="actions-cell">
                      <button type="button" class="link-btn" @click="openVarForm('edit', row)">
                        {{ t('common.edit') }}
                      </button>
                      <button
                        type="button"
                        class="link-btn link-danger"
                        @click="askDeleteVar(row)"
                      >
                        {{ t('common.delete') }}
                      </button>
                    </td>
                  </tr>
                  <tr v-if="manageEnvId && !variables.length">
                    <td colspan="4" class="empty muted">{{ t('env.noVariables') }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </section>
      </div>
    </div>
  </Teleport>

  <!-- Env form -->
  <Teleport to="body">
    <div
      v-if="envFormOpen"
      class="overlay overlay-nested"
      @mousedown="onFormOverlayDown"
      @mouseup="onEnvFormOverlayUp"
    >
      <div
        class="form-dialog"
        role="dialog"
        aria-modal="true"
        @mousedown.stop
        @keydown.escape.prevent="envFormOpen = false"
      >
        <h3>{{ envFormMode === 'create' ? t('env.create') : t('env.rename') }}</h3>
        <label class="field">
          <span>{{ t('env.colName') }}</span>
          <input
            v-model="envForm.name"
            class="input"
            type="text"
            maxlength="100"
            :placeholder="t('env.namePlaceholder')"
            @keydown.enter.prevent="submitEnvForm"
          />
        </label>
        <label class="field">
          <span>{{ t('env.remark') }}</span>
          <textarea
            v-model="envForm.remark"
            class="input textarea"
            rows="3"
            maxlength="500"
            :placeholder="t('env.remarkPlaceholder')"
          />
        </label>
        <p v-if="formError" class="error">{{ formError }}</p>
        <div class="form-actions">
          <button class="btn" type="button" @click="envFormOpen = false">
            {{ t('common.cancel') }}
          </button>
          <button class="btn btn-primary" type="button" :disabled="saving" @click="submitEnvForm">
            {{ saving ? '…' : t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <!-- Variable form -->
  <Teleport to="body">
    <div
      v-if="varFormOpen"
      class="overlay overlay-nested"
      @mousedown="onFormOverlayDown"
      @mouseup="onVarFormOverlayUp"
    >
      <div
        class="form-dialog"
        role="dialog"
        aria-modal="true"
        @mousedown.stop
        @keydown.escape.prevent="varFormOpen = false"
      >
        <h3>
          {{ varFormMode === 'create' ? t('env.createVariable') : t('env.editVariable') }}
        </h3>
        <label class="field">
          <span>{{ t('env.variableKey') }}</span>
          <input
            v-model="varForm.key"
            class="input"
            type="text"
            maxlength="100"
            :placeholder="t('env.variableKeyPlaceholder')"
            @keydown.enter.prevent="submitVarForm"
          />
        </label>
        <label class="field">
          <span>{{ t('env.variableValue') }}</span>
          <textarea
            v-model="varForm.value"
            class="input textarea"
            rows="3"
            :placeholder="t('env.variableValuePlaceholder')"
          />
        </label>
        <label class="field">
          <span>{{ t('env.variableDescription') }}</span>
          <input
            v-model="varForm.description"
            class="input"
            type="text"
            maxlength="255"
            :placeholder="t('env.variableDescriptionPlaceholder')"
          />
        </label>
        <p v-if="formError" class="error">{{ formError }}</p>
        <div class="form-actions">
          <button class="btn" type="button" @click="varFormOpen = false">
            {{ t('common.cancel') }}
          </button>
          <button class="btn btn-primary" type="button" :disabled="saving" @click="submitVarForm">
            {{ saving ? '…' : t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <ConfirmDialog
    v-model:open="confirmOpen"
    :title="confirmTitle"
    :message="confirmMessage"
    @confirm="onConfirmDelete"
  />
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, toRef, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import { useOverlayDismiss } from '../composables/useOverlayDismiss'
import { useWorkspaceContext } from '../composables/useWorkspaceContext'
import type { EnvironmentRow, EnvironmentVariableRow } from '../types'
import ConfirmDialog from './ConfirmDialog.vue'
import MenuSelect from './MenuSelect.vue'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [boolean]
}>()

const { t } = useI18n()
const {
  environments,
  workspaceId,
  projectId,
  environmentId,
  projects,
  loadEnvs,
  selectEnvironment,
  bumpEnvVarsRevision,
} = useWorkspaceContext()

const openRef = toRef(props, 'open')
const section = ref<'envs' | 'vars'>('envs')
const variables = ref<EnvironmentVariableRow[]>([])
const manageEnvId = ref<number | null>(null)
const saving = ref(false)
const formError = ref('')

const envFormOpen = ref(false)
const envFormMode = ref<'create' | 'edit'>('create')
const editingEnvId = ref<number | null>(null)
const envForm = reactive({ name: '', remark: '' })

const varFormOpen = ref(false)
const varFormMode = ref<'create' | 'edit'>('create')
const editingVarId = ref<number | null>(null)
const varForm = reactive({ key: '', value: '', description: '' })

const confirmOpen = ref(false)
const confirmTitle = ref('')
const confirmMessage = ref('')
const pendingDelete = ref<
  | { kind: 'env'; row: EnvironmentRow }
  | { kind: 'var'; row: EnvironmentVariableRow }
  | null
>(null)

const iconEnv =
  `<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="3"/><path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1"/></svg>`
const iconVars =
  `<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M4 7h16M4 12h10M4 17h14"/><circle cx="18" cy="12" r="2"/></svg>`

const projectName = computed(
  () => projects.value.find((p) => p.id === projectId.value)?.name ?? '',
)

const currentEnvName = computed(
  () => environments.value.find((e) => e.id === manageEnvId.value)?.name ?? '',
)

const envOptions = computed(() =>
  environments.value.map((e) => ({ value: e.id, label: e.name })),
)

function close() {
  emit('update:open', false)
}

const { onOverlayMouseDown, onOverlayMouseUp } = useOverlayDismiss(openRef, close)

function onEsc(event: KeyboardEvent) {
  if (event.key !== 'Escape') return
  if (envFormOpen.value) {
    event.preventDefault()
    envFormOpen.value = false
    return
  }
  if (varFormOpen.value) {
    event.preventDefault()
    varFormOpen.value = false
    return
  }
  if (confirmOpen.value) {
    event.preventDefault()
    confirmOpen.value = false
    return
  }
  if (props.open) {
    event.preventDefault()
    close()
  }
}

onMounted(() => {
  window.addEventListener('keydown', onEsc)
})
onUnmounted(() => {
  window.removeEventListener('keydown', onEsc)
})

let formOverlayDown = false
function onFormOverlayDown() {
  formOverlayDown = true
}
function onEnvFormOverlayUp() {
  if (formOverlayDown) envFormOpen.value = false
  formOverlayDown = false
}
function onVarFormOverlayUp() {
  if (formOverlayDown) varFormOpen.value = false
  formOverlayDown = false
}

async function loadVariables() {
  if (!manageEnvId.value) {
    variables.value = []
    return
  }
  try {
    variables.value = await invoke('list_local_variables', {
      environmentId: manageEnvId.value,
    })
  } catch {
    variables.value = []
  }
}

watch(
  () => props.open,
  async (v) => {
    if (!v) return
    section.value = 'envs'
    formError.value = ''
    await loadEnvs()
    manageEnvId.value =
      environmentId.value ?? environments.value[0]?.id ?? null
    await loadVariables()
  },
)

watch(manageEnvId, () => {
  if (props.open && section.value === 'vars') void loadVariables()
})

watch(section, (s) => {
  if (s === 'vars') {
    if (manageEnvId.value == null) {
      manageEnvId.value = environmentId.value ?? environments.value[0]?.id ?? null
    }
    void loadVariables()
  }
})

function openEnvForm(mode: 'create' | 'edit', row?: EnvironmentRow) {
  envFormMode.value = mode
  formError.value = ''
  if (mode === 'edit' && row) {
    editingEnvId.value = row.id
    envForm.name = row.name
    envForm.remark = row.remark ?? ''
  } else {
    editingEnvId.value = null
    envForm.name = ''
    envForm.remark = ''
  }
  envFormOpen.value = true
}

function openVarForm(mode: 'create' | 'edit', row?: EnvironmentVariableRow) {
  varFormMode.value = mode
  formError.value = ''
  if (mode === 'edit' && row) {
    editingVarId.value = row.id
    varForm.key = row.key
    varForm.value = row.value
    varForm.description = row.description ?? ''
  } else {
    editingVarId.value = null
    varForm.key = ''
    varForm.value = ''
    varForm.description = ''
  }
  varFormOpen.value = true
}

async function submitEnvForm() {
  if (!workspaceId.value || !projectId.value) return
  const name = envForm.name.trim()
  if (!name) {
    formError.value = t('create.nameRequired')
    return
  }
  saving.value = true
  formError.value = ''
  try {
    if (envFormMode.value === 'create') {
      const created = await invoke<EnvironmentRow>('create_local_environment', {
        args: {
          workspace_id: workspaceId.value,
          project_id: projectId.value,
          name,
          remark: envForm.remark.trim(),
          is_default: environments.value.length === 0,
        },
      })
      await loadEnvs()
      if (environmentId.value == null) {
        await selectEnvironment(created.id)
      }
      manageEnvId.value = manageEnvId.value ?? created.id
    } else if (editingEnvId.value != null) {
      const current = environments.value.find((e) => e.id === editingEnvId.value)
      await invoke('update_local_environment', {
        args: {
          id: editingEnvId.value,
          workspace_id: workspaceId.value,
          project_id: projectId.value,
          name,
          remark: envForm.remark.trim(),
          is_default: current?.is_default ?? false,
        },
      })
      await loadEnvs()
    }
    envFormOpen.value = false
  } catch (e) {
    formError.value = String(e)
  } finally {
    saving.value = false
  }
}

async function setDefault(row: EnvironmentRow) {
  if (!workspaceId.value || !projectId.value) return
  try {
    await invoke('update_local_environment', {
      args: {
        id: row.id,
        workspace_id: workspaceId.value,
        project_id: projectId.value,
        name: row.name,
        remark: row.remark ?? '',
        is_default: true,
      },
    })
    await loadEnvs()
  } catch (e) {
    alert(String(e))
  }
}

function askDeleteEnv(row: EnvironmentRow) {
  pendingDelete.value = { kind: 'env', row }
  confirmTitle.value = t('env.delete')
  confirmMessage.value = t('env.deleteConfirm', { name: row.name })
  confirmOpen.value = true
}

function askDeleteVar(row: EnvironmentVariableRow) {
  pendingDelete.value = { kind: 'var', row }
  confirmTitle.value = t('common.delete')
  confirmMessage.value = t('env.variableDeleteConfirm', { name: row.key })
  confirmOpen.value = true
}

async function onConfirmDelete() {
  const pending = pendingDelete.value
  pendingDelete.value = null
  if (!pending) return
  try {
    if (pending.kind === 'env') {
      if (!workspaceId.value || !projectId.value) return
      await invoke('delete_local_environment', {
        workspaceId: workspaceId.value,
        projectId: projectId.value,
        environmentId: pending.row.id,
      })
      if (environmentId.value === pending.row.id) {
        await loadEnvs()
        const next = environments.value[0]?.id ?? null
        await selectEnvironment(next)
      } else {
        await loadEnvs()
      }
      if (manageEnvId.value === pending.row.id) {
        manageEnvId.value = environmentId.value ?? environments.value[0]?.id ?? null
      }
      await loadVariables()
    } else {
      await invoke('delete_local_variable', {
        environmentId: pending.row.environment_id,
        variableId: pending.row.id,
      })
      await loadVariables()
      bumpEnvVarsRevision()
    }
  } catch (e) {
    alert(String(e))
  }
}

async function submitVarForm() {
  if (!manageEnvId.value) return
  const key = varForm.key.trim()
  if (!key) {
    formError.value = t('env.variableKeyPlaceholder')
    return
  }
  saving.value = true
  formError.value = ''
  try {
    if (varFormMode.value === 'create') {
      await invoke('create_local_variable', {
        args: {
          environment_id: manageEnvId.value,
          key,
          value: varForm.value,
          description: varForm.description.trim(),
        },
      })
    } else if (editingVarId.value != null) {
      await invoke('update_local_variable', {
        args: {
          id: editingVarId.value,
          environment_id: manageEnvId.value,
          key,
          value: varForm.value,
          description: varForm.description.trim(),
        },
      })
    }
    varFormOpen.value = false
    await loadVariables()
    bumpEnvVarsRevision()
  } catch (e) {
    formError.value = String(e)
  } finally {
    saving.value = false
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

.overlay-nested {
  z-index: 1100;
}

.dialog {
  position: relative;
  width: min(920px, 96vw);
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
  line-height: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
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
  padding: 24px 52px 28px 28px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-height: 100%;
  box-sizing: border-box;
}

.toolbar {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.toolbar .btn {
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

.env-select-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.env-select-row label {
  color: var(--color-text-secondary);
  font-size: 13px;
}

.env-select {
  width: min(240px, 100%);
  min-width: 160px;
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

.default-yes {
  color: #49cc90;
  font-weight: 600;
}

.link-btn {
  border: 0;
  background: transparent;
  color: var(--color-primary);
  padding: 0;
  font-size: 13px;
  cursor: pointer;
}

.link-btn + .link-btn {
  margin-left: 10px;
}

.link-danger {
  color: var(--color-danger);
}

.actions-cell {
  white-space: nowrap;
}

.cell-ellipsis {
  max-width: 220px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.var-key {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 12px;
  color: var(--color-primary);
}

.form-dialog {
  width: min(420px, calc(100vw - 24px));
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  box-shadow: var(--shadow-dialog);
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-dialog h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
}

.textarea {
  resize: vertical;
  min-height: 72px;
  font-family: inherit;
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
  font-size: 13px;
}
</style>
