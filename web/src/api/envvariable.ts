import http, { getData } from '@/utils/request'
import type { EnvironmentVariableItem } from '@/types/workspace'

export function fetchEnvironmentVariables(
  workspaceId: number,
  projectId: number,
  environmentId: number,
) {
  return getData<EnvironmentVariableItem[]>(
    http.get('/v1/variables', {
      params: {
        workspace_id: workspaceId,
        project_id: projectId,
        environment_id: environmentId,
      },
    }),
  )
}

export function createEnvironmentVariable(
  workspaceId: number,
  projectId: number,
  environmentId: number,
  payload: { key: string; value?: string; description?: string },
) {
  return getData<null>(
    http.post('/v1/variables', {
      workspace_id: workspaceId,
      project_id: projectId,
      environment_id: environmentId,
      ...payload,
    }),
  )
}

export function updateEnvironmentVariable(
  workspaceId: number,
  projectId: number,
  environmentId: number,
  variableId: number,
  payload: { key: string; value?: string; description?: string },
) {
  return getData<null>(
    http.put('/v1/variables', {
      workspace_id: workspaceId,
      project_id: projectId,
      environment_id: environmentId,
      variable_id: variableId,
      ...payload,
    }),
  )
}

export function deleteEnvironmentVariable(
  workspaceId: number,
  projectId: number,
  environmentId: number,
  variableId: number,
) {
  return getData<null>(
    http.delete('/v1/variables', {
      params: {
        workspace_id: workspaceId,
        project_id: projectId,
        environment_id: environmentId,
        variable_id: variableId,
      },
    }),
  )
}
