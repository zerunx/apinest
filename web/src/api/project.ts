import http, { getData } from '@/utils/request'
import type { ProjectItem } from '@/types/workspace'

export function fetchProjects(workspaceId: number) {
  return getData<ProjectItem[]>(
    http.get('/v1/projects', { params: { workspace_id: workspaceId } }),
  )
}

export function createProject(workspaceId: number, name: string) {
  return getData<null>(http.post('/v1/projects', { workspace_id: workspaceId, name }))
}

export function updateProject(workspaceId: number, projectId: number, name: string) {
  return getData<null>(
    http.put('/v1/projects', { workspace_id: workspaceId, project_id: projectId, name }),
  )
}

export function deleteProject(workspaceId: number, projectId: number) {
  return getData<null>(
    http.delete('/v1/projects', { params: { workspace_id: workspaceId, project_id: projectId } }),
  )
}
