import http, { getData } from '@/utils/request'
import type { MemberItem, WorkspaceRole } from '@/types/workspace'

export function fetchMembers(workspaceId: number) {
  return getData<MemberItem[]>(
    http.get('/v1/members', { params: { workspace_id: workspaceId } }),
  )
}

export function inviteMember(workspaceId: number, userId: number, role: WorkspaceRole) {
  return getData<null>(
    http.post('/v1/members', { workspace_id: workspaceId, user_id: userId, role }),
  )
}

export function updateMemberRole(workspaceId: number, memberId: number, role: WorkspaceRole) {
  return getData<null>(
    http.put('/v1/members', { workspace_id: workspaceId, member_id: memberId, role }),
  )
}

export function removeMember(workspaceId: number, memberId: number) {
  return getData<null>(
    http.delete('/v1/members', { params: { workspace_id: workspaceId, member_id: memberId } }),
  )
}

export interface InviteLinkInfo {
  workspace_id: number
  workspace_name: string
  invite_code: string
  invite_url: string
  site_url: string
}

export interface InvitePreviewInfo {
  workspace_id: number
  workspace_name: string
  invite_code: string
}

export interface AcceptInviteResult {
  workspace_id: number
  workspace_name: string
  already_member: boolean
}

export function fetchInviteLink(workspaceId: number) {
  return getData<InviteLinkInfo>(
    http.get('/v1/members/invite/link', { params: { workspace_id: workspaceId } }),
  )
}

export function refreshInviteLink(workspaceId: number) {
  return getData<InviteLinkInfo>(
    http.post('/v1/members/invite/link/refresh', { workspace_id: workspaceId }),
  )
}

export function fetchInvitePreview(inviteCode: string) {
  return getData<InvitePreviewInfo>(
    http.get('/v1/members/invite/preview', { params: { invite_code: inviteCode } }),
  )
}

export function acceptInvite(inviteCode: string) {
  return getData<AcceptInviteResult>(
    http.post('/v1/members/accept/invite', { invite_code: inviteCode }),
  )
}
