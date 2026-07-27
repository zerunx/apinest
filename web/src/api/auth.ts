import http, { getData } from '@/utils/request'
import type { LoginPayload, LoginResult, RegisterPayload, SiteInfo, UserProfile } from '@/types/auth'

export function fetchSiteInfo() {
  return getData<SiteInfo>(http.get('/v1/auth/site'))
}

export function login(payload: LoginPayload) {
  return getData<LoginResult>(http.post('/v1/auth/login', payload))
}

export function register(payload: RegisterPayload) {
  return getData<LoginResult>(http.post('/v1/auth/register', payload))
}

export function fetchCurrentUser() {
  return getData<UserProfile>(http.get('/v1/auth/me'))
}
