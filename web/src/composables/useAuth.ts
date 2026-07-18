import { computed, ref } from 'vue'
import { fetchCurrentUser, login as loginApi, register as registerApi } from '@/api/auth'
import type { LoginPayload, RegisterPayload, UserProfile } from '@/types/auth'
import { clearAuth, getAccessToken, getRefreshToken, getStoredUser, saveAuth } from '@/utils/auth-storage'

const user = ref<UserProfile | null>(getStoredUser())
let bootstrapTask: Promise<void> | null = null

export function useAuth() {
  const isLoggedIn = computed(() => !!user.value)

  async function bootstrap() {
    const token = getAccessToken()
    if (!token) {
      user.value = null
      return
    }

    if (!bootstrapTask) {
      bootstrapTask = (async () => {
        try {
          const profile = await fetchCurrentUser()
          user.value = profile
          saveAuth(token, getRefreshToken(), profile)
        } catch {
          clearAuth()
          user.value = null
        } finally {
          bootstrapTask = null
        }
      })()
    }

    await bootstrapTask
  }

  async function login(payload: LoginPayload) {
    const result = await loginApi(payload)
    saveAuth(result.access_token, result.refresh_token, result.user)
    user.value = result.user
    return result
  }

  async function register(payload: RegisterPayload) {
    const result = await registerApi(payload)
    saveAuth(result.access_token, result.refresh_token, result.user)
    user.value = result.user
    return result
  }

  function logout() {
    clearAuth()
    user.value = null
    bootstrapTask = null
  }

  return {
    user,
    isLoggedIn,
    bootstrap,
    login,
    register,
    logout,
  }
}
