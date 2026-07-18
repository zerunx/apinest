<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
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
      <div>
        <h2>{{ t('member.title') }}</h2>
        <p v-if="activeWorkspace">
          {{ t('member.desc', { name: activeWorkspace.name }) }}
        </p>
      </div>
      <el-button type="primary" @click="inviteVisible = true">
        {{ t('member.invite') }}
      </el-button>
    </div>

    <el-table v-loading="loading" :data="members" class="workspace-data-table">
      <el-table-column prop="name" :label="t('member.columns.name')" min-width="120" />
      <el-table-column prop="account" :label="t('member.columns.account')" min-width="140" />
      <el-table-column :label="t('member.columns.role')" width="160">
        <template #default="{ row }">
          <el-select
            v-if="canManageMember(row)"
            :model-value="row.role"
            :loading="updatingId === row.id"
            style="width: 130px"
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
          <el-tag v-else type="warning" size="small">{{ roleLabel(row.role) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="created_at" :label="t('member.columns.joinedAt')" min-width="180" />
      <el-table-column :label="t('member.columns.actions')" width="100" fixed="right">
        <template #default="{ row }">
          <el-button
            v-if="canManageMember(row)"
            type="danger"
            link
            :loading="removingId === row.id"
            @click="handleRemove(row)"
          >
            {{ t('member.remove') }}
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="inviteVisible" :title="t('member.invite')" width="420px" destroy-on-close>
      <el-form label-position="top" @submit.prevent="handleInvite">
        <el-form-item :label="t('member.userId')" required>
          <el-input
            v-model="inviteForm.userId"
            :placeholder="t('member.userIdPlaceholder')"
            type="number"
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
.member-panel {
  padding: 10px 8px 20px;
}

.member-panel__toolbar {
  margin-bottom: 12px;
}
</style>
