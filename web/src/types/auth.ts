export interface UserProfile {
  id: number
  name: string
  account: string
  email: string
  avatar: string
  is_admin: boolean
  status: number
}

export interface LoginPayload {
  account: string
  password: string
}

export interface RegisterPayload {
  name: string
  account: string
  email: string
  password: string
  confirm_password: string
}

export interface LoginResult {
  access_token: string
  refresh_token: string
  user: UserProfile
}

export interface SiteInfo {
  installed: boolean
  site_url: string
}
