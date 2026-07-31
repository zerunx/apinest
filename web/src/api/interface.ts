import http, { getData } from '@/utils/request'
import type { InterfaceDetail, InterfaceItem } from '@/types/workspace'

export function fetchInterfaces(workspaceId: number, projectId: number) {
  return getData<InterfaceItem[]>(
    http.get('/v1/interfaces', { params: { workspace_id: workspaceId, project_id: projectId } }),
  )
}

export function fetchInterfaceDetail(
  workspaceId: number,
  projectId: number,
  interfaceId: number,
) {
  return getData<InterfaceDetail>(
    http.get('/v1/interfaces/detail', {
      params: {
        workspace_id: workspaceId,
        project_id: projectId,
        interface_id: interfaceId,
      },
    }),
  )
}

export type InterfaceWritePayload = {
  folder_id?: number
  name: string
  method: string
  url?: string
  status?: number
  request_headers?: InterfaceDetail['request_headers']
  request_body?: InterfaceDetail['request_body']
  query_params?: InterfaceDetail['query_params']
  response_headers?: InterfaceDetail['response_headers']
  response_results?: InterfaceDetail['response_results']
  response_examples?: InterfaceDetail['response_examples']
}

export function createInterface(
  workspaceId: number,
  projectId: number,
  payload: InterfaceWritePayload & { folder_id: number },
) {
  return getData<{ id: number }>(
    http.post('/v1/interfaces', {
      workspace_id: workspaceId,
      project_id: projectId,
      ...payload,
    }),
  )
}

export function updateInterface(
  workspaceId: number,
  projectId: number,
  interfaceId: number,
  payload: InterfaceWritePayload,
) {
  return getData<null>(
    http.put('/v1/interfaces', {
      workspace_id: workspaceId,
      project_id: projectId,
      interface_id: interfaceId,
      ...payload,
    }),
  )
}

export function deleteInterface(workspaceId: number, projectId: number, interfaceId: number) {
  return getData<null>(
    http.delete('/v1/interfaces', {
      params: {
        workspace_id: workspaceId,
        project_id: projectId,
        interface_id: interfaceId,
      },
    }),
  )
}

export function reorderInterfaces(
  workspaceId: number,
  projectId: number,
  folderId: number,
  interfaceIds: number[],
) {
  return getData<null>(
    http.put('/v1/interfaces/reorder', {
      workspace_id: workspaceId,
      project_id: projectId,
      folder_id: folderId,
      interface_ids: interfaceIds,
    }),
  )
}
