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

http.interceptors.response.use(
  (response: AxiosResponse<ApiResponse>) => {
    const payload = response.data

    if (payload.code !== 0) {
      return Promise.reject(new Error(payload.message || '请求失败'))
    }

    return response
  },
  (error) => {
    if (error.response?.status === 401) {
      clearAuth()
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
