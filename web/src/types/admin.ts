export interface AdminUserItem {
  id: number
  name: string
  account: string
  email: string
  avatar: string
  is_admin: boolean
  status: number
}

export interface AdminWorkspaceItem {
  id: number
  name: string
  owner_id: number
  owner_name: string
  created_at: string
}
