<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  createEnvironment,
  deleteEnvironment,
  updateEnvironment,
} from '@/api/environment'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'
import type { EnvironmentItem } from '@/types/workspace'

const { t } = useLocale()
const {
  activeProject,
  activeWorkspaceId,
  activeProjectId,
  environments,
  loadingEnvironments,
  refreshEnvironments,
} = useWorkspaceContext()

const editingId = ref<number | null>(null)
const deletingId = ref<number | null>(null)
const submitting = ref(false)
const dialogVisible = ref(false)
const dialogMode = ref<'create' | 'edit'>('create')
const editingEnvironmentId = ref<number | null>(null)

const form = reactive({
  name: '',
  remark: '',
})

watch(dialogVisible, (visible) => {
  if (!visible) {
    form.name = ''
    form.remark = ''
    editingEnvironmentId.value = null
  }
})

function openCreateDialog() {
  dialogMode.value = 'create'
  form.name = ''
  form.remark = ''
  dialogVisible.value = true
}

function openEditDialog(row: EnvironmentItem) {
  dialogMode.value = 'edit'
  editingEnvironmentId.value = row.id
  form.name = row.name
  form.remark = row.remark ?? ''
  dialogVisible.value = true
}

async function handleSubmit() {
  if (!activeWorkspaceId.value || !activeProjectId.value) return

  const name = form.name.trim()
  if (!name) {
    ElMessage.warning(t('workspace.nameRequired'))
    return
  }

  submitting.value = true
  try {
    if (dialogMode.value === 'create') {
      await createEnvironment(activeWorkspaceId.value, activeProjectId.value, {
        name,
        remark: form.remark.trim(),
      })
      ElMessage.success(t('environment.createSuccess'))
    } else if (editingEnvironmentId.value) {
      editingId.value = editingEnvironmentId.value
      const current = environments.value.find((item) => item.id === editingEnvironmentId.value)
      await updateEnvironment(
        activeWorkspaceId.value,
        activeProjectId.value,
        editingEnvironmentId.value,
        {
          name,
          remark: form.remark.trim(),
          is_default: current?.is_default ?? false,
        },
      )
      ElMessage.success(t('environment.updateSuccess'))
    }
    dialogVisible.value = false
    await refreshEnvironments()
  } catch (error) {
    ElMessage.error(
      error instanceof Error
        ? error.message
        : dialogMode.value === 'create'
          ? t('environment.createFailed')
          : t('environment.updateFailed'),
    )
  } finally {
    submitting.value = false
    editingId.value = null
  }
}

async function handleDelete(environmentId: number, name: string) {
  if (!activeWorkspaceId.value || !activeProjectId.value) return

  try {
    await ElMessageBox.confirm(
      t('environment.deleteConfirm', { name }),
      t('environment.delete'),
      { type: 'warning' },
    )
  } catch {
    return
  }

  deletingId.value = environmentId
  try {
    await deleteEnvironment(activeWorkspaceId.value, activeProjectId.value, environmentId)
    ElMessage.success(t('environment.deleteSuccess'))
    await refreshEnvironments()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('environment.deleteFailed'))
  } finally {
    deletingId.value = null
  }
}

async function handleSetDefault(row: EnvironmentItem) {
  if (!activeWorkspaceId.value || !activeProjectId.value) return

  try {
    await updateEnvironment(activeWorkspaceId.value, activeProjectId.value, row.id, {
      name: row.name,
      remark: row.remark ?? '',
      is_default: true,
    })
    ElMessage.success(t('environment.setDefaultSuccess'))
    await refreshEnvironments()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('environment.updateFailed'))
  }
}

watch(activeProjectId, () => {
  refreshEnvironments()
})
</script>

<template>
  <div class="environment-manage workspace-panel">
    <div class="environment-manage__toolbar workspace-panel__toolbar">
      <div>
        <h2>{{ t('workspace.modules.environment') }}</h2>
        <p v-if="activeProject">
          {{ t('environment.manageDesc', { name: activeProject.name }) }}
        </p>
      </div>
      <el-button type="primary" class="workspace-action-btn" @click="openCreateDialog">
        <span>{{ t('environment.create') }}</span>
      </el-button>
    </div>

    <el-table v-loading="loadingEnvironments" :data="environments" class="workspace-data-table">
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="name" :label="t('workspace.columns.name')" min-width="140" />
      <el-table-column prop="remark" :label="t('environment.remark')" min-width="200" show-overflow-tooltip />
      <el-table-column :label="t('environment.default')" min-width="110" width="120">
        <template #default="{ row }">
          <span v-if="row.is_default" class="environment-manage__default">{{ t('environment.yes') }}</span>
          <el-button
            v-else
            type="primary"
            link
            @click="handleSetDefault(row)"
          >
            {{ t('environment.setDefault') }}
          </el-button>
        </template>
      </el-table-column>
      <el-table-column prop="created_at" :label="t('workspace.columns.createdAt')" min-width="180" />
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
            @click="handleDelete(row.id, row.name)"
          >
            {{ t('common.delete') }}
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog
      v-model="dialogVisible"
      :title="dialogMode === 'create' ? t('environment.create') : t('environment.rename')"
      width="420px"
      destroy-on-close
    >
      <el-form label-position="top" @submit.prevent="handleSubmit">
        <el-form-item :label="t('workspace.columns.name')" required>
          <el-input
            v-model="form.name"
            :placeholder="t('environment.namePlaceholder')"
            maxlength="100"
            show-word-limit
          />
        </el-form-item>
        <el-form-item :label="t('environment.remark')">
          <el-input
            v-model="form.remark"
            :placeholder="t('environment.remarkPlaceholder')"
            maxlength="500"
            show-word-limit
            type="textarea"
            :rows="3"
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
.environment-manage__toolbar {
  padding: 10px 10px 10px 8px;
}

.environment-manage__default {
  color: #49cc90;
  font-size: 14px;
}

.environment-manage :deep(.el-dialog .el-input__inner),
.environment-manage :deep(.el-dialog .el-textarea__inner) {
  color: var(--color-text) !important;
  -webkit-text-fill-color: var(--color-text);
  caret-color: var(--color-text);
  font-size: 14px;
}

.environment-manage :deep(.el-dialog .el-input__inner::placeholder),
.environment-manage :deep(.el-dialog .el-textarea__inner::placeholder) {
  color: var(--color-text-secondary) !important;
  -webkit-text-fill-color: var(--color-text-secondary);
}

.environment-manage :deep(.el-dialog .el-form-item__label) {
  color: var(--color-text);
  font-weight: 500;
}
</style>
