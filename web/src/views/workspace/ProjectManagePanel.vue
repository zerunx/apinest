<script setup lang="ts">
import { computed, ref } from 'vue'
import { Delete, Edit } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { deleteProject, updateProject } from '@/api/project'
import MemberManagePanel from '@/views/workspace/MemberManagePanel.vue'
import ProjectMemberManagePanel from '@/views/workspace/ProjectMemberManagePanel.vue'
import WorkspaceManagePanel from '@/views/workspace/WorkspaceManagePanel.vue'
import { useAuth } from '@/composables/useAuth'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'
import type { ProjectItem } from '@/types/workspace'

const { t } = useLocale()
const { user } = useAuth()
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

const isWorkspaceAdmin = computed(() => {
  const role = activeWorkspace.value?.role
  return role === 1 || role === 2
})

const canCreateProject = computed(() => {
  const role = activeWorkspace.value?.role
  return role === 1 || role === 2 || role === 3
})

function canManageProject(row: ProjectItem) {
  if (isWorkspaceAdmin.value) return true
  return row.created_by === user.value?.id
}

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

    <ProjectMemberManagePanel v-else-if="activeModuleTab?.kind === 'project-member-list'" />

    <template v-else>
      <div class="space-manage__toolbar workspace-panel__toolbar">
        <el-button
          v-if="canCreateProject"
          type="primary"
          class="workspace-action-btn"
          @click="openCreateProject"
        >
          <span>{{ t('workspace.createProject') }}</span>
        </el-button>
        <p v-if="activeWorkspace" class="workspace-panel__toolbar-desc">
          {{ t('workspace.projectManageDesc', { name: activeWorkspace.name }) }}
        </p>
      </div>

      <el-table
        v-loading="loadingProjects"
        :data="projects"
        class="workspace-data-table"
        :header-cell-style="{ borderBottom: 'none' }"
      >
        <el-table-column prop="id" label="ID" width="88" />
        <el-table-column prop="name" :label="t('workspace.columns.name')" min-width="180" show-overflow-tooltip />
        <el-table-column prop="creator_name" :label="t('workspace.columns.creator')" min-width="120" show-overflow-tooltip />
        <el-table-column prop="created_at" :label="t('workspace.columns.createdAt')" min-width="180" />
        <el-table-column
          :label="t('member.columns.actions')"
          width="108"
          class-name="workspace-table-actions-col"
          label-class-name="workspace-table-actions-col"
        >
          <template #default="{ row }">
            <div v-if="canManageProject(row)" class="workspace-table-actions">
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
            <span v-else class="space-manage__muted">—</span>
          </template>
        </el-table-column>
      </el-table>
    </template>
  </div>
</template>

<style scoped>
.space-manage__muted {
  color: var(--color-text-secondary);
  opacity: 0.55;
}
</style>
