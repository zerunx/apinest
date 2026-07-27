import http, { getData } from '@/utils/request'
import type { InterfaceDetail } from '@/types/workspace'

export interface ProjectShareItem {
  id: number
  project_id: number
  workspace_id: number
  name: string
  share_code: string
  share_url: string
  enabled: boolean
  has_password: boolean
  permission: number
  interface_ids?: number[]
  interface_count: number
  created_at: string
  updated_at: string
}

export interface SharePreview {
  share_code: string
  name: string
  project_name: string
  enabled: boolean
  has_password: boolean
  permission: number
}

export interface SharedInterfaceItem {
  id: number
  folder_id: number
  folder_name?: string
  name: string
  method: string
  url: string
  status: number
}

export interface SharedFolderItem {
  id: number
  parent_id: number
  name: string
}

export interface ShareTreeNode {
  id: number
  name: string
  type: 'folder' | 'api'
  method?: string
  url?: string
  status?: number
  children?: ShareTreeNode[]
}

export interface ShareContent {
  share_code: string
  name: string
  project_name: string
  permission: number
  interfaces: SharedInterfaceItem[]
  folders?: SharedFolderItem[]
  tree?: ShareTreeNode[]
}

export function fetchProjectShares(workspaceId: number, projectId: number) {
  return getData<ProjectShareItem[]>(
    http.get('/v1/project/shares', {
      params: { workspace_id: workspaceId, project_id: projectId },
    }),
  )
}

export function fetchProjectShareDetail(workspaceId: number, shareId: number) {
  return getData<ProjectShareItem>(
    http.get('/v1/project/shares/detail', {
      params: { workspace_id: workspaceId, share_id: shareId },
    }),
  )
}

export function createProjectShare(payload: {
  workspace_id: number
  project_id: number
  name: string
  enabled?: boolean
  password?: string
  permission?: number
  interface_ids: number[]
}) {
  return getData<ProjectShareItem>(http.post('/v1/project/shares', payload))
}

export function updateProjectShare(payload: {
  workspace_id: number
  share_id: number
  name: string
  enabled?: boolean
  password?: string | null
  permission?: number
  interface_ids: number[]
}) {
  return getData<ProjectShareItem>(http.put('/v1/project/shares', payload))
}

export function deleteProjectShare(workspaceId: number, shareId: number) {
  return getData<null>(
    http.delete('/v1/project/shares', {
      params: { workspace_id: workspaceId, share_id: shareId },
    }),
  )
}

export function fetchSharePreview(shareCode: string) {
  return getData<SharePreview>(
    http.get('/v1/share/preview', { params: { share_code: shareCode } }),
  )
}

export function fetchShareContent(shareCode: string, password = '') {
  return getData<ShareContent>(
    http.post('/v1/share/content', { share_code: shareCode, password }),
  )
}

export function fetchShareInterfaceDetail(
  shareCode: string,
  interfaceId: number,
  password = '',
) {
  return getData<InterfaceDetail>(
    http.post('/v1/share/interface', {
      share_code: shareCode,
      interface_id: interfaceId,
      password,
    }),
  )
}
