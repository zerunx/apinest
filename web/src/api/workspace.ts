import http, { getData } from '@/utils/request'
import type { WorkspaceItem } from '@/types/workspace'

export function fetchWorkspaces() {
  return getData<WorkspaceItem[]>(http.get('/v1/workspaces'))
}

export function createWorkspace(name: string) {
  return getData<null>(http.post('/v1/workspaces', { name }))
}

export function updateWorkspace(workspaceId: number, name: string) {
  return getData<null>(http.put('/v1/workspaces', { workspace_id: workspaceId, name }))
}

export function deleteWorkspace(workspaceId: number) {
  return getData<null>(http.delete('/v1/workspaces', { params: { workspace_id: workspaceId } }))
}
