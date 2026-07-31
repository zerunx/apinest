<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import { acceptInvite, fetchInvitePreview } from '@/api/member'
import { useAuth } from '@/composables/useAuth'
import { useLocale } from '@/composables/useLocale'
import { useWorkspaceContext } from '@/composables/useWorkspaceContext'
import { getAccessToken } from '@/utils/auth-storage'

const route = useRoute()
const router = useRouter()
const { t } = useLocale()
const { user, login, bootstrap } = useAuth()
const { selectWorkspace, bootstrap: bootstrapWorkspace } = useWorkspaceContext()

const inviteCode = computed(() => String(route.query.inviteCode || '').trim())

const loadingPreview = ref(true)
const joining = ref(false)
const loggingIn = ref(false)
const previewError = ref('')
const workspaceName = ref('')

const isLoggedIn = computed(() => Boolean(getAccessToken() && user.value))

const loginFormRef = ref<FormInstance>()
const loginForm = reactive({
  account: '',
  password: '',
})

const loginRules: FormRules = {
  account: [{ required: true, message: () => t('invite.accountRequired'), trigger: 'blur' }],
  password: [{ required: true, message: () => t('invite.passwordRequired'), trigger: 'blur' }],
}

async function loadPreview() {
  if (!inviteCode.value) {
    previewError.value = t('invite.invalidLink')
    loadingPreview.value = false
    return
  }

  loadingPreview.value = true
  previewError.value = ''
  try {
    const data = await fetchInvitePreview(inviteCode.value)
    workspaceName.value = data.workspace_name
  } catch (error) {
    previewError.value = error instanceof Error ? error.message : t('invite.invalidLink')
  } finally {
    loadingPreview.value = false
  }
}

async function finishJoin() {
  if (!inviteCode.value) return

  joining.value = true
  try {
    const result = await acceptInvite(inviteCode.value)
    await bootstrapWorkspace()
    selectWorkspace(result.workspace_id)
    ElMessage.success(
      result.already_member ? t('invite.alreadyMember', { name: result.workspace_name }) : t('invite.joinSuccess', { name: result.workspace_name }),
    )
    router.push('/home')
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('invite.joinFailed'))
  } finally {
    joining.value = false
  }
}

async function handleJoin() {
  if (!inviteCode.value || previewError.value) return
  await finishJoin()
}

async function handleLoginAndJoin() {
  if (!loginFormRef.value) return

  try {
    await loginFormRef.value.validate()
  } catch {
    return
  }

  loggingIn.value = true
  try {
    await login(loginForm)
    await bootstrap()
    await finishJoin()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : t('invite.loginFailed'))
  } finally {
    loggingIn.value = false
  }
}

function goRegister() {
  router.push({
    path: '/register',
    query: { redirect: route.fullPath },
  })
}

function goLogin() {
  router.push({
    path: '/login',
    query: { redirect: route.fullPath },
  })
}

onMounted(async () => {
  if (getAccessToken()) {
    await bootstrap()
  }
  await loadPreview()
})
</script>

<template>
  <div class="invite-accept-view">
    <div v-if="loadingPreview" class="invite-accept-view__loading">
      {{ t('invite.loading') }}
    </div>

    <template v-else-if="previewError">
      <h2>{{ t('invite.invalidTitle') }}</h2>
      <p class="invite-accept-view__desc">{{ previewError }}</p>
      <button type="button" class="invite-accept-view__btn" @click="router.push('/login')">
        {{ t('invite.backToLogin') }}
      </button>
    </template>

    <template v-else>
      <h2>{{ t('invite.title') }}</h2>
      <p class="invite-accept-view__desc">{{ t('invite.intro') }}</p>

      <div class="invite-accept-view__card">
        <span class="invite-accept-view__label">{{ t('invite.workspaceName') }}</span>
        <strong class="invite-accept-view__workspace">{{ workspaceName }}</strong>
        <span class="invite-accept-view__role">{{ t('invite.defaultRole') }}</span>
      </div>

      <template v-if="isLoggedIn">
        <button
          type="button"
          class="invite-accept-view__btn invite-accept-view__btn--primary"
          :disabled="joining"
          @click="handleJoin"
        >
          {{ joining ? t('invite.joining') : t('invite.joinNow') }}
        </button>
      </template>

      <template v-else>
        <p class="invite-accept-view__login-hint">{{ t('invite.loginHint') }}</p>

        <el-form
          ref="loginFormRef"
          :model="loginForm"
          :rules="loginRules"
          label-position="top"
          class="invite-accept-view__form"
          @submit.prevent
        >
          <el-form-item :label="t('invite.account')" prop="account">
            <el-input v-model="loginForm.account" :placeholder="t('invite.accountPlaceholder')" />
          </el-form-item>
          <el-form-item :label="t('invite.password')" prop="password">
            <el-input
              v-model="loginForm.password"
              type="password"
              show-password
              :placeholder="t('invite.passwordPlaceholder')"
              @keyup.enter="handleLoginAndJoin"
            />
          </el-form-item>
        </el-form>

        <button
          type="button"
          class="invite-accept-view__btn invite-accept-view__btn--primary"
          :disabled="loggingIn || joining"
          @click="handleLoginAndJoin"
        >
          {{ loggingIn || joining ? t('invite.joining') : t('invite.loginAndJoin') }}
        </button>

        <p class="invite-accept-view__footer">
          {{ t('invite.noAccount') }}
          <button type="button" class="invite-accept-view__link" @click="goRegister">
            {{ t('invite.registerNow') }}
          </button>
          <span class="invite-accept-view__sep">·</span>
          <button type="button" class="invite-accept-view__link" @click="goLogin">
            {{ t('invite.useOtherAccount') }}
          </button>
        </p>
      </template>
    </template>
  </div>
</template>

<style scoped>
.invite-accept-view h2 {
  margin: 0 0 8px;
  font-size: 28px;
  font-weight: 700;
  color: #0f172a;
}

.invite-accept-view__loading {
  color: #64748b;
  font-size: 14px;
}

.invite-accept-view__desc {
  margin: 0 0 20px;
  color: #64748b;
  font-size: 14px;
  line-height: 1.6;
}

.invite-accept-view__card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 20px;
  padding: 16px;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #f8fafc;
}

.invite-accept-view__label {
  font-size: 12px;
  color: #64748b;
}

.invite-accept-view__workspace {
  font-size: 20px;
  color: #0f172a;
}

.invite-accept-view__role {
  font-size: 13px;
  color: #475569;
}

.invite-accept-view__login-hint {
  margin: 0 0 16px;
  color: #64748b;
  font-size: 14px;
}

.invite-accept-view__form {
  margin-bottom: 12px;
}

.invite-accept-view__btn {
  width: 100%;
  height: 42px;
  border: 1px solid #cbd5e1;
  border-radius: 10px;
  background: #ffffff;
  color: #0f172a;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
}

.invite-accept-view__btn--primary {
  border-color: #0f172a;
  background: #0f172a;
  color: #ffffff;
}

.invite-accept-view__btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.invite-accept-view__footer {
  margin: 20px 0 0;
  text-align: center;
  color: #64748b;
  font-size: 14px;
}

.invite-accept-view__link {
  border: none;
  background: transparent;
  color: #2563eb;
  font-size: 14px;
  cursor: pointer;
  padding: 0;
}

.invite-accept-view__sep {
  margin: 0 6px;
  color: #cbd5e1;
}
</style>
