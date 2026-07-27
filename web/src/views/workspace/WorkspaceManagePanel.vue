<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { deleteWorkspace, updateWorkspace } from '@/api/workspace'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'

const { t } = useLocale()
const {
  workspaces,
  loadingWorkspaces,
  refreshWorkspaces,
  openCreateWorkspace,
} = useWorkspaceContext()

const editingId = ref<number | null>(null)
const deletingId = ref<number | null>(null)

async function handleRename(workspaceId: number, currentName: string) {
  try {
    const { value } = await ElMessageBox.prompt(
      t('workspace.renameWorkspacePrompt'),
      t('workspace.renameWorkspace'),
      {
        inputValue: currentName,
        inputPattern: /\S+/,
        inputErrorMessage: t('workspace.nameRequired'),
      },
    )
    if (!value?.trim()) return

    editingId.value = workspaceId
    await updateWorkspace(workspaceId, value.trim())
    ElMessage.success(t('workspace.updateWorkspaceSuccess'))
    await refreshWorkspaces()
  } catch (error) {
    if (error !== 'cancel' && error !== 'close') {
      ElMessage.error(error instanceof Error ? error.message : t('workspace.updateWorkspaceFailed'))
    }
  } finally {
    editingId.value = null
  }
}

async function handleDelete(workspaceId: number, name: string) {
  try {
    await ElMessageBox.confirm(
      t('workspace.deleteWorkspaceConfirm', { name }),
      t('workspace.deleteWorkspace'),
      { type: 'warning' },
    )
  } catch {
    return
  }

  deletingId.value = workspaceId
  try {
    await deleteWorkspace(workspaceId)
    ElMessage.success(t('workspace.deleteWorkspaceSuccess'))
    await refreshWorkspaces()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('workspace.deleteWorkspaceFailed'))
  } finally {
    deletingId.value = null
  }
}
</script>

<template>
  <div class="workspace-manage workspace-panel">
    <div class="workspace-manage__toolbar workspace-panel__toolbar">
      <div>
        <h2>{{ t('workspace.workspaceManage') }}</h2>
        <p>{{ t('workspace.workspaceManageDesc') }}</p>
      </div>
      <el-button type="primary" class="workspace-action-btn" @click="openCreateWorkspace">
        <span class="workspace-action-btn__plus">+</span>
        <span>{{ t('workspace.createWorkspace') }}</span>
      </el-button>
    </div>

    <el-table v-loading="loadingWorkspaces" :data="workspaces" class="workspace-data-table">
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="name" :label="t('workspace.columns.name')" min-width="160" />
      <el-table-column prop="owner_name" :label="t('workspace.columns.owner')" min-width="120" />
      <el-table-column prop="created_at" :label="t('workspace.columns.createdAt')" min-width="180" />
      <el-table-column :label="t('member.columns.actions')" width="160" fixed="right">
        <template #default="{ row }">
          <el-button
            type="primary"
            link
            :loading="editingId === row.id"
            @click="handleRename(row.id, row.name)"
          >
            {{ t('common.rename') }}
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
  </div>
</template>

<style scoped>
.workspace-manage__toolbar {
  padding: 10px 10px 10px 8px;
}
</style>
