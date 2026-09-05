import http, { getData } from '@/utils/request'

export interface ProjectMemberItem {
  id: number
  user_id: number
  name: string
  account: string
  avatar: string
  created_at: string
}

export function fetchProjectMembers(workspaceId: number, projectId: number) {
  return getData<ProjectMemberItem[]>(
    http.get('/v1/project-members', {
      params: { workspace_id: workspaceId, project_id: projectId },
    }),
  )
}

export function addProjectMember(workspaceId: number, projectId: number, userId: number) {
  return getData<null>(
    http.post('/v1/project-members', {
      workspace_id: workspaceId,
      project_id: projectId,
      user_id: userId,
    }),
  )
}

export function removeProjectMember(workspaceId: number, projectId: number, userId: number) {
  return getData<null>(
    http.delete('/v1/project-members', {
      params: { workspace_id: workspaceId, project_id: projectId, user_id: userId },
    }),
  )
}
