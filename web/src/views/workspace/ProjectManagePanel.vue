<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { deleteProject, updateProject } from '@/api/project'
import MemberManagePanel from '@/views/workspace/MemberManagePanel.vue'
import WorkspaceManagePanel from '@/views/workspace/WorkspaceManagePanel.vue'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'

const { t } = useLocale()
const {
  activeModuleTab,
  projects,
  activeWorkspace,
  activeWorkspaceId,
  loadingProjects,
  refreshProjects,
  openCreateProject,
} = useWorkspaceContext()

const editingId = ref<number | null>(null)
const deletingId = ref<number | null>(null)

async function handleRename(projectId: number, currentName: string) {
  try {
    const { value } = await ElMessageBox.prompt(
      t('workspace.renameProjectPrompt'),
      t('workspace.renameProject'),
      {
        inputValue: currentName,
        inputPattern: /\S+/,
        inputErrorMessage: t('workspace.nameRequired'),
      },
    )
    if (!activeWorkspaceId.value || !value?.trim()) return

    editingId.value = projectId
    await updateProject(activeWorkspaceId.value, projectId, value.trim())
    ElMessage.success(t('workspace.updateProjectSuccess'))
    await refreshProjects()
  } catch (error) {
    if (error !== 'cancel' && error !== 'close') {
      ElMessage.error(error instanceof Error ? error.message : t('workspace.updateProjectFailed'))
    }
  } finally {
    editingId.value = null
  }
}

async function handleDelete(projectId: number, name: string) {
  if (!activeWorkspaceId.value) return

  try {
    await ElMessageBox.confirm(
      t('workspace.deleteProjectConfirm', { name }),
      t('workspace.deleteProject'),
      { type: 'warning' },
    )
  } catch {
    return
  }

  deletingId.value = projectId
  try {
    await deleteProject(activeWorkspaceId.value, projectId)
    ElMessage.success(t('workspace.deleteProjectSuccess'))
    await refreshProjects()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('workspace.deleteProjectFailed'))
  } finally {
    deletingId.value = null
  }
}
</script>

<template>
  <div class="space-manage workspace-panel">
    <WorkspaceManagePanel v-if="activeModuleTab?.kind === 'workspace-list'" />

    <MemberManagePanel v-else-if="activeModuleTab?.kind === 'member-list'" />

    <template v-else>
      <div class="space-manage__toolbar workspace-panel__toolbar">
        <div>
          <h2>{{ t('workspace.projectManage') }}</h2>
          <p v-if="activeWorkspace">
            {{ t('workspace.projectManageDesc', { name: activeWorkspace.name }) }}
          </p>
        </div>
        <el-button type="primary" class="workspace-action-btn" @click="openCreateProject">
          <span class="workspace-action-btn__plus">+</span>
          <span>{{ t('workspace.createProject') }}</span>
        </el-button>
      </div>

      <el-table v-loading="loadingProjects" :data="projects" class="workspace-data-table">
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="name" :label="t('workspace.columns.name')" min-width="160" />
        <el-table-column prop="creator_name" :label="t('workspace.columns.creator')" min-width="120" />
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
    </template>
  </div>
</template>

<style scoped>
.space-manage__toolbar {
  padding: 10px 10px 10px 8px;
}
</style>
