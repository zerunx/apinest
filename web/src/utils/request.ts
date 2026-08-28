import axios, { type AxiosInstance, type AxiosResponse } from 'axios'
import type { ApiResponse } from '@/types/install'
import { clearAuth, getAccessToken } from '@/utils/auth-storage'

const http: AxiosInstance = axios.create({
  baseURL: '/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

http.interceptors.request.use((config) => {
  const token = getAccessToken()
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

const AUTH_PUBLIC_PREFIXES = ['/login', '/register', '/install', '/share', '/invite']

let redirectingToLogin = false

function redirectToLogin() {
  if (typeof window === 'undefined') return
  const { pathname, search, hash } = window.location
  if (AUTH_PUBLIC_PREFIXES.some((prefix) => pathname === prefix || pathname.startsWith(`${prefix}/`))) {
    return
  }
  if (redirectingToLogin) return
  redirectingToLogin = true

  const redirect = `${pathname}${search}${hash}`
  const loginUrl =
    redirect && redirect !== '/'
      ? `/login?redirect=${encodeURIComponent(redirect)}`
      : '/login'

  // Full navigation resets in-memory auth state and avoids circular imports with the router.
  window.location.assign(loginUrl)
}

http.interceptors.response.use(
  (response: AxiosResponse<ApiResponse>) => {
    const payload = response.data

    if (payload.code !== 0) {
      if (payload.code === 401) {
        clearAuth()
        redirectToLogin()
      }
      return Promise.reject(new Error(payload.message || '请求失败'))
    }

    return response
  },
  (error) => {
    if (error.response?.status === 401) {
      clearAuth()
      redirectToLogin()
    }

    const message =
      error.response?.data?.message ||
      error.message ||
      '网络请求失败，请检查后端服务是否已启动'

    return Promise.reject(new Error(message))
  },
)

export default http

export async function getData<T>(promise: Promise<AxiosResponse<ApiResponse<T>>>): Promise<T> {
  const response = await promise
  return response.data.data
}
