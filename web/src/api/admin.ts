import http, { getData } from '@/utils/request'
import type { AdminUserItem, AdminWorkspaceItem } from '@/types/admin'

export type UpdateAdminUserPayload = {
  name: string
  account: string
  email: string
  password?: string
  is_admin: boolean
  status: number
}

export function fetchAdminUsers() {
  return getData<AdminUserItem[]>(http.get('/v1/admin/users'))
}

export function updateAdminUser(id: number, payload: UpdateAdminUserPayload) {
  return getData<null>(http.put(`/v1/admin/users/${id}`, payload))
}

export function deleteAdminUser(id: number) {
  return getData<null>(http.delete(`/v1/admin/users/${id}`))
}

export function fetchAdminWorkspaces() {
  return getData<AdminWorkspaceItem[]>(http.get('/v1/admin/workspaces'))
}

export function transferWorkspace(id: number, ownerId: number) {
  return getData<null>(http.put(`/v1/admin/workspaces/${id}/transfer`, { owner_id: ownerId }))
}
