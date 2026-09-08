<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Delete } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { fetchMembers } from '@/api/member'
import {
  addProjectMember,
  fetchProjectMembers,
  removeProjectMember,
  type ProjectMemberItem,
} from '@/api/project-member'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'
import type { MemberItem } from '@/types/workspace'

const { t } = useLocale()
const {
  activeWorkspaceId,
  projects,
  loadingProjects,
} = useWorkspaceContext()

const selectedProjectId = ref<number | null>(null)
const members = ref<ProjectMemberItem[]>([])
const workspaceMembers = ref<MemberItem[]>([])
const loading = ref(false)
const adding = ref(false)
const removingId = ref<number | null>(null)
const pickUserId = ref<number | null>(null)

const projectOptions = computed(() => projects.value)

const selectedProject = computed(() =>
  projects.value.find((p) => p.id === selectedProjectId.value) ?? null,
)

const availableUsers = computed(() => {
  const taken = new Set(members.value.map((m) => m.user_id))
  return workspaceMembers.value.filter((m) => !taken.has(m.user_id))
})

function canRemove(row: ProjectMemberItem) {
  if (selectedProject.value?.created_by === row.user_id) return false
  return true
}

async function loadWorkspaceMembers() {
  if (!activeWorkspaceId.value) {
    workspaceMembers.value = []
    return
  }
  try {
    workspaceMembers.value = await fetchMembers(activeWorkspaceId.value)
  } catch {
    workspaceMembers.value = []
  }
}

async function loadProjectMembers() {
  if (!activeWorkspaceId.value || !selectedProjectId.value) {
    members.value = []
    return
  }
  loading.value = true
  try {
    members.value = await fetchProjectMembers(activeWorkspaceId.value, selectedProjectId.value)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('workspace.projectMembersLoadFailed'))
  } finally {
    loading.value = false
  }
}

async function handleAdd() {
  if (!activeWorkspaceId.value) return
  if (!selectedProjectId.value) {
    ElMessage.warning(t('workspace.projectMembersSelectProject'))
    return
  }
  if (!pickUserId.value) {
    ElMessage.warning(t('workspace.projectMembersPickUser'))
    return
  }
  adding.value = true
  try {
    await addProjectMember(activeWorkspaceId.value, selectedProjectId.value, pickUserId.value)
    ElMessage.success(t('workspace.projectMembersAddSuccess'))
    pickUserId.value = null
    await loadProjectMembers()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('workspace.projectMembersAddFailed'))
  } finally {
    adding.value = false
  }
}

async function handleRemove(row: ProjectMemberItem) {
  if (!activeWorkspaceId.value || !selectedProjectId.value) return
  try {
    await ElMessageBox.confirm(
      t('member.removeConfirm', { name: row.name || row.account }),
      t('member.removeTitle'),
      { type: 'warning' },
    )
  } catch {
    return
  }
  removingId.value = row.user_id
  try {
    await removeProjectMember(activeWorkspaceId.value, selectedProjectId.value, row.user_id)
    ElMessage.success(t('workspace.projectMembersRemoveSuccess'))
    await loadProjectMembers()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('workspace.projectMembersRemoveFailed'))
  } finally {
    removingId.value = null
  }
}

watch(
  () => activeWorkspaceId.value,
  async () => {
    selectedProjectId.value = projects.value[0]?.id ?? null
    await loadWorkspaceMembers()
    await loadProjectMembers()
  },
  { immediate: true },
)

watch(selectedProjectId, () => {
  pickUserId.value = null
  loadProjectMembers()
})

watch(
  () => projects.value.map((p) => p.id).join(','),
  () => {
    if (!selectedProjectId.value && projects.value[0]) {
      selectedProjectId.value = projects.value[0].id
    }
  },
)
</script>

<template>
  <div class="project-members-panel workspace-panel">
    <div class="workspace-panel__toolbar">
      <el-button
        type="primary"
        class="workspace-action-btn"
        :loading="adding"
        @click="handleAdd"
      >
        <span>{{ t('workspace.projectMembersAdd') }}</span>
      </el-button>
    </div>

    <div class="project-members-panel__controls">
      <el-select
        v-model="selectedProjectId"
        :placeholder="t('workspace.projectMembersSelectProject')"
        style="width: 260px"
        popper-class="app-action-dropdown"
        :loading="loadingProjects"
      >
        <el-option
          v-for="item in projectOptions"
          :key="item.id"
          :label="item.name"
          :value="item.id"
        />
      </el-select>

      <el-select
        v-model="pickUserId"
        :placeholder="t('workspace.projectMembersPickUser')"
        style="width: 240px"
        clearable
        filterable
        popper-class="app-action-dropdown"
        :disabled="!selectedProjectId"
      >
        <el-option
          v-for="item in availableUsers"
          :key="item.user_id"
          :label="`${item.name || item.account} (${item.account})`"
          :value="item.user_id"
        />
      </el-select>
    </div>

    <el-table
      v-if="selectedProjectId"
      v-loading="loading"
      :data="members"
      class="workspace-data-table"
      :header-cell-style="{ borderBottom: 'none' }"
    >
      <el-table-column prop="name" :label="t('member.columns.name')" min-width="140" show-overflow-tooltip />
      <el-table-column prop="account" :label="t('member.columns.account')" min-width="160" show-overflow-tooltip />
      <el-table-column prop="created_at" :label="t('member.columns.joinedAt')" min-width="180" />
      <el-table-column
        :label="t('member.columns.actions')"
        width="108"
        class-name="workspace-table-actions-col"
        label-class-name="workspace-table-actions-col"
      >
        <template #default="{ row }">
          <div v-if="canRemove(row)" class="workspace-table-actions">
            <button
              type="button"
              class="workspace-table-action-btn workspace-table-action-btn--danger"
              :title="t('member.remove')"
              :disabled="removingId === row.user_id"
              @click="handleRemove(row)"
            >
              <el-icon :size="14"><Delete /></el-icon>
            </button>
          </div>
          <span v-else class="project-members-panel__muted">—</span>
        </template>
      </el-table-column>
    </el-table>
    <div v-else class="project-members-panel__empty">
      {{ t('workspace.projectMembersEmpty') }}
    </div>
  </div>
</template>

<style scoped>
.project-members-panel__controls {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: center;
  margin: 0 10px 12px 16px;
  padding: 12px 14px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-workspace-elevated);
}

.project-members-panel__empty {
  margin: 0 10px 16px 16px;
  padding: 28px 16px;
  border: 1px dashed var(--color-border);
  border-radius: 6px;
  color: var(--color-text-secondary);
  font-size: 13px;
  text-align: center;
  background: color-mix(in srgb, var(--color-workspace-elevated) 70%, transparent);
}

.project-members-panel__muted {
  color: var(--color-text-secondary);
  opacity: 0.55;
}
</style>
