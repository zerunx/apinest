import http, { getData } from '@/utils/request'
import type { AdminUserItem, AdminWorkspaceItem } from '@/types/admin'

export function fetchAdminUsers() {
  return getData<AdminUserItem[]>(http.get('/v1/admin/users'))
}

export function fetchAdminWorkspaces() {
  return getData<AdminWorkspaceItem[]>(http.get('/v1/admin/workspaces'))
}

export function transferWorkspace(id: number, ownerId: number) {
  return getData<null>(http.put(`/v1/admin/workspaces/${id}/transfer`, { owner_id: ownerId }))
}
