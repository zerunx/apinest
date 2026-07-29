import http, { getData } from '@/utils/request'
import type { ApiTreeNode } from '@/types/workspace'

export function fetchFolderTree(workspaceId: number, projectId: number) {
  return getData<ApiTreeNode[]>(
    http.get('/v1/folders/tree', { params: { workspace_id: workspaceId, project_id: projectId } }),
  )
}

export function createFolder(
  workspaceId: number,
  projectId: number,
  name: string,
  parentId = 0,
) {
  return getData<{ id: number }>(
    http.post('/v1/folders', {
      workspace_id: workspaceId,
      project_id: projectId,
      parent_id: parentId,
      name,
    }),
  )
}

export function updateFolder(
  workspaceId: number,
  projectId: number,
  folderId: number,
  name: string,
) {
  return getData<null>(
    http.put('/v1/folders', {
      workspace_id: workspaceId,
      project_id: projectId,
      folder_id: folderId,
      name,
    }),
  )
}

export function deleteFolder(workspaceId: number, projectId: number, folderId: number) {
  return getData<null>(
    http.delete('/v1/folders', {
      params: { workspace_id: workspaceId, project_id: projectId, folder_id: folderId },
    }),
  )
}
