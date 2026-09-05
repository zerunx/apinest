<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { Delete } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { fetchMembers, inviteMember, removeMember, updateMemberRole } from '@/api/member'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'
import type { MemberItem, WorkspaceRole } from '@/types/workspace'

const { t } = useLocale()
const { activeWorkspace, activeWorkspaceId } = useWorkspaceContext()

const members = ref<MemberItem[]>([])
const loading = ref(false)
const inviteVisible = ref(false)
const inviting = ref(false)
const updatingId = ref<number | null>(null)
const removingId = ref<number | null>(null)

const inviteForm = reactive({
  userId: '',
  role: 3 as WorkspaceRole,
})

const roleOptions = computed(() => [
  { value: 2, label: t('member.roles.admin') },
  { value: 3, label: t('member.roles.editor') },
  { value: 4, label: t('member.roles.viewer') },
])

const editableRoleOptions = roleOptions

function roleLabel(role: number) {
  const map: Record<number, string> = {
    1: t('member.roles.owner'),
    2: t('member.roles.admin'),
    3: t('member.roles.editor'),
    4: t('member.roles.viewer'),
  }
  return map[role] || String(role)
}

function canManageMember(member: MemberItem) {
  return member.role !== 1
}

async function loadMembers() {
  if (!activeWorkspaceId.value) {
    members.value = []
    return
  }

  loading.value = true
  try {
    members.value = await fetchMembers(activeWorkspaceId.value)
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('member.loadFailed'))
  } finally {
    loading.value = false
  }
}

async function handleInvite() {
  const userId = Number(inviteForm.userId)
  if (!activeWorkspaceId.value || !userId || userId < 1) {
    ElMessage.warning(t('member.userIdRequired'))
    return
  }

  inviting.value = true
  try {
    await inviteMember(activeWorkspaceId.value, userId, inviteForm.role)
    ElMessage.success(t('member.inviteSuccess'))
    inviteVisible.value = false
    inviteForm.userId = ''
    inviteForm.role = 3
    await loadMembers()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('member.inviteFailed'))
  } finally {
    inviting.value = false
  }
}

async function handleRoleChange(member: MemberItem, role: WorkspaceRole) {
  if (!activeWorkspaceId.value || member.role === role) return

  updatingId.value = member.id
  try {
    await updateMemberRole(activeWorkspaceId.value, member.id, role)
    ElMessage.success(t('member.updateSuccess'))
    await loadMembers()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('member.updateFailed'))
  } finally {
    updatingId.value = null
  }
}

async function handleRemove(member: MemberItem) {
  if (!activeWorkspaceId.value) return

  try {
    await ElMessageBox.confirm(
      t('member.removeConfirm', { name: member.name || member.account }),
      t('member.removeTitle'),
      { type: 'warning' },
    )
  } catch {
    return
  }

  removingId.value = member.id
  try {
    await removeMember(activeWorkspaceId.value, member.id)
    ElMessage.success(t('member.removeSuccess'))
    await loadMembers()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('member.removeFailed'))
  } finally {
    removingId.value = null
  }
}

watch(activeWorkspaceId, () => {
  loadMembers()
})

onMounted(() => {
  loadMembers()
})
</script>

<template>
  <div class="member-panel workspace-panel">
    <div class="member-panel__toolbar workspace-panel__toolbar">
      <el-button type="primary" class="workspace-action-btn" @click="inviteVisible = true">
        <span>{{ t('member.invite') }}</span>
      </el-button>
      <p v-if="activeWorkspace" class="workspace-panel__toolbar-desc">
        {{ t('member.desc', { name: activeWorkspace.name }) }}
      </p>
    </div>

    <el-table
      v-loading="loading"
      :data="members"
      class="workspace-data-table member-panel__table"
      :header-cell-style="{ borderBottom: 'none' }"
    >
      <el-table-column prop="name" :label="t('member.columns.name')" min-width="140" show-overflow-tooltip />
      <el-table-column prop="account" :label="t('member.columns.account')" min-width="160" show-overflow-tooltip />
      <el-table-column :label="t('member.columns.role')" min-width="140" width="168">
        <template #default="{ row }">
          <el-select
            v-if="canManageMember(row)"
            class="member-panel__role-select"
            :model-value="row.role"
            :loading="updatingId === row.id"
            popper-class="app-action-dropdown"
            @change="(value: WorkspaceRole) => handleRoleChange(row, value)"
          >
            <el-option
              v-for="item in editableRoleOptions"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>
          <span v-else class="member-panel__role-badge">{{ roleLabel(row.role) }}</span>
        </template>
      </el-table-column>
      <el-table-column prop="created_at" :label="t('member.columns.joinedAt')" min-width="180" />
      <el-table-column
        :label="t('member.columns.actions')"
        width="108"
        class-name="workspace-table-actions-col"
        label-class-name="workspace-table-actions-col"
      >
        <template #default="{ row }">
          <div v-if="canManageMember(row)" class="workspace-table-actions">
            <button
              type="button"
              class="workspace-table-action-btn workspace-table-action-btn--danger"
              :title="t('member.remove')"
              :disabled="removingId === row.id"
              @click="handleRemove(row)"
            >
              <el-icon :size="14"><Delete /></el-icon>
            </button>
          </div>
          <span v-else class="member-panel__muted">—</span>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="inviteVisible" :title="t('member.invite')" width="420px" destroy-on-close>
      <el-form label-position="top" @submit.prevent="handleInvite">
        <el-form-item :label="t('member.userId')" required>
          <el-input
            v-model="inviteForm.userId"
            :placeholder="t('member.userIdPlaceholder')"
            inputmode="numeric"
          />
        </el-form-item>
        <el-form-item :label="t('member.columns.role')" required>
          <el-select v-model="inviteForm.role" style="width: 100%" popper-class="app-action-dropdown">
            <el-option
              v-for="item in roleOptions"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="inviteVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="inviting" @click="handleInvite">
          {{ t('common.confirm') }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.member-panel__role-select {
  width: 100%;
}

.member-panel__role-badge {
  display: inline-flex;
  align-items: center;
  height: 24px;
  padding: 0 10px;
  border-radius: 6px;
  background: color-mix(in srgb, var(--color-primary) 12%, transparent);
  color: var(--color-primary);
  font-size: 12px;
  font-weight: 600;
}

.member-panel__muted {
  color: var(--color-text-secondary);
  opacity: 0.55;
}

.member-panel__table :deep(td .cell:has(.member-panel__role-select)) {
  overflow: visible;
}

.member-panel__table :deep(.member-panel__role-select .el-select__wrapper) {
  width: 100%;
}
</style>
