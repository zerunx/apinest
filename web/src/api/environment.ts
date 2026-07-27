import http, { getData } from '@/utils/request'
import type { EnvironmentItem } from '@/types/workspace'

export function fetchEnvironments(workspaceId: number, projectId: number) {
  return getData<EnvironmentItem[]>(
    http.get('/v1/environments', { params: { workspace_id: workspaceId, project_id: projectId } }),
  )
}

export function createEnvironment(
  workspaceId: number,
  projectId: number,
  payload: { name: string; remark?: string; is_default?: boolean },
) {
  return getData<null>(
    http.post('/v1/environments', {
      workspace_id: workspaceId,
      project_id: projectId,
      ...payload,
    }),
  )
}

export function updateEnvironment(
  workspaceId: number,
  projectId: number,
  environmentId: number,
  payload: { name: string; remark?: string; is_default?: boolean },
) {
  return getData<null>(
    http.put('/v1/environments', {
      workspace_id: workspaceId,
      project_id: projectId,
      environment_id: environmentId,
      ...payload,
    }),
  )
}

export function deleteEnvironment(workspaceId: number, projectId: number, environmentId: number) {
  return getData<null>(
    http.delete('/v1/environments', {
      params: {
        workspace_id: workspaceId,
        project_id: projectId,
        environment_id: environmentId,
      },
    }),
  )
}
