<script setup lang="ts">
import { ref } from 'vue'
import { Delete, Edit } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { deleteWorkspace, updateWorkspace } from '@/api/workspace'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'
import type { WorkspaceItem } from '@/types/workspace'

const { t } = useLocale()
const {
  workspaces,
  loadingWorkspaces,
  refreshWorkspaces,
  openCreateWorkspace,
} = useWorkspaceContext()

const editingId = ref<number | null>(null)
const deletingId = ref<number | null>(null)

/** Only workspace Owner can rename/delete (creator becomes Owner on create). */
function canManageWorkspace(row: WorkspaceItem) {
  return row.role === 1
}

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
    // Silent refresh: avoid table loading flash that looks like a full page reload.
    await refreshWorkspaces({ silent: true })
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
      <el-button type="primary" class="workspace-action-btn" @click="openCreateWorkspace">
        <span>{{ t('workspace.createWorkspace') }}</span>
      </el-button>
    </div>

    <el-table
      v-loading="loadingWorkspaces"
      :data="workspaces"
      class="workspace-data-table"
      :header-cell-style="{ borderBottom: 'none' }"
    >
      <el-table-column prop="id" label="ID" width="88" />
      <el-table-column prop="name" :label="t('workspace.columns.name')" min-width="180" show-overflow-tooltip />
      <el-table-column prop="owner_name" :label="t('workspace.columns.owner')" min-width="120" show-overflow-tooltip />
      <el-table-column prop="created_at" :label="t('workspace.columns.createdAt')" min-width="180" />
      <el-table-column
        :label="t('member.columns.actions')"
        width="108"
        class-name="workspace-table-actions-col"
        label-class-name="workspace-table-actions-col"
      >
        <template #default="{ row }">
          <div v-if="canManageWorkspace(row)" class="workspace-table-actions">
            <button
              type="button"
              class="workspace-table-action-btn"
              :title="t('common.rename')"
              :disabled="editingId === row.id"
              @click="handleRename(row.id, row.name)"
            >
              <el-icon :size="14"><Edit /></el-icon>
            </button>
            <button
              type="button"
              class="workspace-table-action-btn workspace-table-action-btn--danger"
              :title="t('common.delete')"
              :disabled="deletingId === row.id"
              @click="handleDelete(row.id, row.name)"
            >
              <el-icon :size="14"><Delete /></el-icon>
            </button>
          </div>
          <span v-else class="workspace-manage__muted">—</span>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<style scoped>
.workspace-manage__muted {
  color: var(--color-text-secondary);
  opacity: 0.55;
}
</style>
