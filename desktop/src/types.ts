export type SessionInfo = {
  server_id: number
  base_url: string
  account: string | null
  user_id: number | null
  logged_in: boolean
}

export type WorkspaceRow = {
  id: number
  name: string
  role: number | null
}

export type ProjectRow = {
  id: number
  workspace_id: number
  name: string
}

export type TreeItem =
  | { type: 'folder'; id: number; name: string; children: TreeItem[] }
  | { type: 'api'; id: number; name: string; method: string; url: string }

export type EnvironmentRow = {
  id: number
  name: string
  remark?: string
  is_default: boolean
}

export type EnvironmentVariableRow = {
  id: number
  environment_id: number
  key: string
  value: string
  description: string
  enabled: boolean
}

export type HeaderPair = {
  key: string
  value: string
  enabled?: boolean
}

export type SendResponse = {
  status: number
  status_text: string
  headers: HeaderPair[]
  body: string
  duration_ms: number
  history_id: number
}

export type SyncResult = {
  workspaces: number
  projects: number
  folders: number
  interfaces: number
  environments: number
  synced_at: string
}

export type InterfaceRow = {
  id: number
  workspace_id: number
  project_id: number
  folder_id: number
  name: string
  method: string
  url: string
  detail: Record<string, unknown> | null
}
