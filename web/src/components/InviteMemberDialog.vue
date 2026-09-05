<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { RefreshRight } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import {
  fetchInviteLink,
  refreshInviteLink,
  type InviteLinkRole,
} from '@/api/member'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'

const { t } = useLocale()
const { activeWorkspace, activeWorkspaceId } = useWorkspaceContext()

const visible = defineModel<boolean>({ default: false })

const loading = ref(false)
const refreshing = ref(false)
const inviteUrl = ref('')
const expiresAt = ref('')
const inviteRole = ref<InviteLinkRole>(4)

const workspaceName = computed(() => activeWorkspace.value?.name || '')

const roleOptions = computed(() => [
  { value: 3 as InviteLinkRole, label: t('member.roles.editor') },
  { value: 4 as InviteLinkRole, label: t('member.roles.viewer') },
])

async function loadInviteLink() {
  if (!activeWorkspaceId.value) return

  loading.value = true
  try {
    const data = await fetchInviteLink(activeWorkspaceId.value, inviteRole.value)
    inviteUrl.value = data.invite_url
    expiresAt.value = data.expires_at
    inviteRole.value = data.role
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('member.inviteLinkFailed'))
    visible.value = false
  } finally {
    loading.value = false
  }
}

async function handleRefresh() {
  if (!activeWorkspaceId.value) return

  refreshing.value = true
  try {
    const data = await refreshInviteLink(activeWorkspaceId.value, inviteRole.value)
    inviteUrl.value = data.invite_url
    expiresAt.value = data.expires_at
    inviteRole.value = data.role
    ElMessage.success(t('member.inviteLinkRefreshed'))
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('member.inviteLinkRefreshFailed'))
  } finally {
    refreshing.value = false
  }
}

async function copyLink() {
  if (!inviteUrl.value) return
  try {
    await navigator.clipboard.writeText(inviteUrl.value)
    ElMessage.success(t('member.copyLinkSuccess'))
  } catch {
    ElMessage.error(t('member.copyFailed'))
  }
}

watch(visible, (open) => {
  if (open) {
    inviteUrl.value = ''
    expiresAt.value = ''
    inviteRole.value = 4
    loadInviteLink()
  }
})

function selectRole(role: InviteLinkRole) {
  if (inviteRole.value === role) return
  inviteRole.value = role
  if (visible.value) void loadInviteLink()
}
</script>

<template>
  <el-dialog
    v-model="visible"
    :title="t('member.addMember')"
    width="560px"
    top="12vh"
    class="invite-member-dialog"
    destroy-on-close
  >
    <div v-loading="loading" class="invite-member-dialog__body">
      <p>{{ t('member.inviteIntro') }}</p>
      <p>{{ t('member.inviteWorkspaceName', { name: workspaceName }) }}</p>

      <div class="invite-member-dialog__field">
        <span class="invite-member-dialog__label">{{ t('member.inviteRoleLabel') }}</span>
        <div class="invite-member-dialog__roles" role="group">
          <button
            v-for="item in roleOptions"
            :key="item.value"
            type="button"
            class="invite-member-dialog__role-btn"
            :class="{ 'invite-member-dialog__role-btn--active': inviteRole === item.value }"
            @click="selectRole(item.value)"
          >
            {{ item.label }}
          </button>
        </div>
      </div>

      <p>{{ t('member.inviteClickLink') }}</p>
      <div class="invite-member-dialog__link-row">
        <span class="invite-member-dialog__link">{{ inviteUrl || '—' }}</span>
        <button
          type="button"
          class="invite-member-dialog__refresh"
          :disabled="refreshing || loading"
          :title="t('member.refreshInviteLink')"
          @click="handleRefresh"
        >
          <el-icon :class="{ 'is-loading': refreshing }" :size="15">
            <RefreshRight />
          </el-icon>
        </button>
      </div>
      <p v-if="expiresAt" class="invite-member-dialog__meta">
        {{ t('member.inviteExpiresLabel') }}：{{ expiresAt }}
      </p>
    </div>

    <template #footer>
      <div class="invite-member-dialog__footer">
        <button
          type="button"
          class="invite-member-dialog__btn invite-member-dialog__btn--primary"
          @click="copyLink"
        >
          {{ t('member.copyLinkOnly') }}
        </button>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped>
.invite-member-dialog__body {
  display: flex;
  flex-direction: column;
  gap: 1em;
}

.invite-member-dialog__body > p,
.invite-member-dialog__link-row,
.invite-member-dialog__field,
.invite-member-dialog__meta {
  margin: 0;
  font-size: 14px;
  line-height: 1.6;
  color: var(--color-text);
}

.invite-member-dialog__field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.invite-member-dialog__label {
  font-weight: 600;
}

.invite-member-dialog__roles {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.invite-member-dialog__role-btn {
  min-width: 88px;
  height: 32px;
  padding: 0 14px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-workspace-elevated);
  color: var(--color-text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    border-color 0.15s ease,
    color 0.15s ease;
}

.invite-member-dialog__role-btn:hover {
  border-color: color-mix(in srgb, var(--color-primary) 45%, var(--color-border));
  color: var(--color-text);
}

.invite-member-dialog__role-btn--active {
  border-color: var(--color-primary);
  background: color-mix(in srgb, var(--color-primary) 14%, transparent);
  color: var(--color-primary);
  font-weight: 600;
}

.invite-member-dialog__meta {
  color: var(--color-text-secondary);
  font-size: 13px;
}

.invite-member-dialog__link-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 22px;
}

.invite-member-dialog__link {
  flex: 1;
  min-width: 0;
  color: var(--color-text);
  word-break: break-all;
}

.invite-member-dialog__refresh {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--color-text);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  opacity: 0.75;
}

.invite-member-dialog__refresh:hover:not(:disabled) {
  background: var(--color-hover);
  opacity: 1;
}

.invite-member-dialog__refresh:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.invite-member-dialog__footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.invite-member-dialog__btn {
  min-width: 112px;
  height: 36px;
  padding: 0 16px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 13px;
  cursor: pointer;
}

.invite-member-dialog__btn:hover {
  background: var(--color-hover);
}

.invite-member-dialog__btn--primary {
  background: var(--color-text);
  border-color: var(--color-text);
  color: var(--color-bg);
}

.invite-member-dialog__btn--primary:hover {
  background: color-mix(in srgb, var(--color-text) 72%, #000);
  border-color: color-mix(in srgb, var(--color-text) 72%, #000);
}
</style>

<style>
.invite-member-dialog.el-dialog {
  --el-dialog-padding-primary: 0;
  margin-bottom: 0;
  padding: 0;
}

.invite-member-dialog .el-dialog__header {
  position: relative;
  display: flex;
  align-items: center;
  margin-right: 0;
  padding: 16px 52px 16px 24px;
  border-bottom: 1px solid var(--color-border);
}

.invite-member-dialog .el-dialog__title {
  font-size: 16px;
  font-weight: 600;
  line-height: 1.4;
  color: var(--color-text);
}

.invite-member-dialog .el-dialog__headerbtn {
  top: 50%;
  right: 16px;
  width: 28px;
  height: 28px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  transform: translateY(-50%);
}

.invite-member-dialog .el-dialog__headerbtn:hover {
  background-color: var(--color-workspace-menu-hover);
}

.invite-member-dialog .el-dialog__headerbtn .el-dialog__close {
  font-size: 18px;
  color: var(--color-text);
  opacity: 0.85;
}

.invite-member-dialog .el-dialog__headerbtn:hover .el-dialog__close {
  color: var(--color-text);
  opacity: 1;
}

.invite-member-dialog .el-dialog__body {
  padding: 28px 24px;
}

.invite-member-dialog .el-dialog__footer {
  padding: 16px 24px;
  border-top: 1px solid var(--color-border);
}
</style>
