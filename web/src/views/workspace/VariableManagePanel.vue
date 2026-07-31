<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  createEnvironmentVariable,
  deleteEnvironmentVariable,
  updateEnvironmentVariable,
} from '@/api/envvariable'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'
import type { EnvironmentVariableItem } from '@/types/workspace'

const { t } = useLocale()
const {
  activeTab,
  activeProject,
  activeWorkspaceId,
  activeProjectId,
  activeEnvironmentId,
  environmentVariables,
  loadingEnvironmentVariables,
  loadEnvironmentVariables,
} = useWorkspaceContext()

const editingId = ref<number | null>(null)
const deletingId = ref<number | null>(null)
const submitting = ref(false)
const dialogVisible = ref(false)
const dialogMode = ref<'create' | 'edit'>('create')
const editingVariableId = ref<number | null>(null)

const form = reactive({
  key: '',
  value: '',
  description: '',
})

const currentEnvironmentId = computed(() => activeTab.value?.environmentId ?? activeEnvironmentId.value)

const currentEnvironmentName = computed(() => activeTab.value?.label ?? '')

watch(
  () => currentEnvironmentId.value,
  (environmentId) => {
    if (!environmentId) return
    loadEnvironmentVariables()
  },
  { immediate: true },
)

watch(dialogVisible, (visible) => {
  if (!visible) {
    form.key = ''
    form.value = ''
    form.description = ''
    editingVariableId.value = null
  }
})

function openCreateDialog() {
  dialogMode.value = 'create'
  form.key = ''
  form.value = ''
  form.description = ''
  dialogVisible.value = true
}

function openEditDialog(row: EnvironmentVariableItem) {
  dialogMode.value = 'edit'
  editingVariableId.value = row.id
  form.key = row.key
  form.value = row.value
  form.description = row.description ?? ''
  dialogVisible.value = true
}

async function handleSubmit() {
  if (!activeWorkspaceId.value || !activeProjectId.value || !currentEnvironmentId.value) return

  const key = form.key.trim()
  if (!key) {
    ElMessage.warning(t('environment.variableKeyPlaceholder'))
    return
  }

  submitting.value = true
  try {
    if (dialogMode.value === 'create') {
      await createEnvironmentVariable(
        activeWorkspaceId.value,
        activeProjectId.value,
        currentEnvironmentId.value,
        {
          key,
          value: form.value,
          description: form.description.trim(),
        },
      )
      ElMessage.success(t('environment.variableCreateSuccess'))
    } else if (editingVariableId.value) {
      editingId.value = editingVariableId.value
      await updateEnvironmentVariable(
        activeWorkspaceId.value,
        activeProjectId.value,
        currentEnvironmentId.value,
        editingVariableId.value,
        {
          key,
          value: form.value,
          description: form.description.trim(),
        },
      )
      ElMessage.success(t('environment.variableUpdateSuccess'))
    }
    dialogVisible.value = false
    await loadEnvironmentVariables()
  } catch (error) {
    ElMessage.error(
      error instanceof Error
        ? error.message
        : dialogMode.value === 'create'
          ? t('environment.variableCreateFailed')
          : t('environment.variableUpdateFailed'),
    )
  } finally {
    submitting.value = false
    editingId.value = null
  }
}

async function handleDelete(variableId: number, key: string) {
  if (!activeWorkspaceId.value || !activeProjectId.value || !currentEnvironmentId.value) return

  try {
    await ElMessageBox.confirm(
      t('environment.variableDeleteConfirm', { name: key }),
      t('common.delete'),
      { type: 'warning' },
    )
  } catch {
    return
  }

  deletingId.value = variableId
  try {
    await deleteEnvironmentVariable(
      activeWorkspaceId.value,
      activeProjectId.value,
      currentEnvironmentId.value,
      variableId,
    )
    ElMessage.success(t('environment.variableDeleteSuccess'))
    await loadEnvironmentVariables()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('environment.variableDeleteFailed'))
  } finally {
    deletingId.value = null
  }
}
</script>

<template>
  <div class="variable-manage workspace-panel">
    <div class="variable-manage__toolbar workspace-panel__toolbar">
      <div>
        <h2>{{ t('environment.variables') }}</h2>
        <p v-if="activeProject && currentEnvironmentName">
          {{ t('environment.variableManageDesc', { project: activeProject.name, env: currentEnvironmentName }) }}
        </p>
      </div>
      <el-button
        type="primary"
        class="workspace-action-btn"
        :disabled="!currentEnvironmentId"
        @click="openCreateDialog"
      >
        <span class="workspace-action-btn__plus">+</span>
        <span>{{ t('environment.createVariable') }}</span>
      </el-button>
    </div>

    <el-table
      v-loading="loadingEnvironmentVariables"
      :data="environmentVariables"
      class="workspace-data-table"
    >
      <el-table-column prop="key" :label="t('environment.variableKey')" min-width="160" />
      <el-table-column prop="value" :label="t('environment.variableValue')" min-width="200" show-overflow-tooltip />
      <el-table-column prop="description" :label="t('environment.variableDescription')" min-width="160" show-overflow-tooltip />
      <el-table-column prop="updated_at" :label="t('workspace.columns.updatedAt')" min-width="180" />
      <el-table-column :label="t('member.columns.actions')" width="160" fixed="right">
        <template #default="{ row }">
          <el-button
            type="primary"
            link
            :loading="editingId === row.id"
            @click="openEditDialog(row)"
          >
            {{ t('common.edit') }}
          </el-button>
          <el-button
            type="danger"
            link
            :loading="deletingId === row.id"
            @click="handleDelete(row.id, row.key)"
          >
            {{ t('common.delete') }}
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog
      v-model="dialogVisible"
      :title="dialogMode === 'create' ? t('environment.createVariable') : t('environment.editVariable')"
      width="460px"
      destroy-on-close
    >
      <el-form label-position="top" @submit.prevent="handleSubmit">
        <el-form-item :label="t('environment.variableKey')" required>
          <el-input
            v-model="form.key"
            :placeholder="t('environment.variableKeyPlaceholder')"
            maxlength="100"
            show-word-limit
          />
        </el-form-item>
        <el-form-item :label="t('environment.variableValue')">
          <el-input
            v-model="form.value"
            :placeholder="t('environment.variableValuePlaceholder')"
            type="textarea"
            :rows="3"
          />
        </el-form-item>
        <el-form-item :label="t('environment.variableDescription')">
          <el-input
            v-model="form.description"
            :placeholder="t('environment.variableDescriptionPlaceholder')"
            maxlength="255"
            show-word-limit
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="submitting" @click="handleSubmit">
          {{ t('common.confirm') }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.variable-manage__toolbar {
  padding: 10px 10px 10px 8px;
}

.variable-manage :deep(.el-dialog .el-input__inner),
.variable-manage :deep(.el-dialog .el-textarea__inner) {
  color: var(--color-text) !important;
  -webkit-text-fill-color: var(--color-text);
  caret-color: var(--color-text);
  font-size: 14px;
}

.variable-manage :deep(.el-dialog .el-input__inner::placeholder),
.variable-manage :deep(.el-dialog .el-textarea__inner::placeholder) {
  color: var(--color-text-secondary) !important;
  -webkit-text-fill-color: var(--color-text-secondary);
}

.variable-manage :deep(.el-dialog .el-form-item__label) {
  color: var(--color-text);
  font-weight: 500;
}
</style>
