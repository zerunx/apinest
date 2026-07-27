export type AppModule = 'api' | 'quick-test' | 'environment' | 'project'

export type HttpProtocol = 'HTTP' | 'HTTPS'

export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS'

export type WorkspaceRole = 1 | 2 | 3 | 4

export interface WorkspaceItem {
  id: number
  name: string
  owner_id: number
  owner_name?: string
  role: WorkspaceRole
  created_at: string
}

export interface ProjectItem {
  id: number
  workspace_id: number
  name: string
  created_by: number
  creator_name?: string
  created_at: string
}

export interface MemberItem {
  id: number
  user_id: number
  name: string
  account: string
  avatar: string
  role: WorkspaceRole
  created_at: string
}

export interface ApiTreeNode {
  id: string
  project_id: number
  name: string
  type: 'folder' | 'api'
  method?: HttpMethod
  children?: ApiTreeNode[]
}

export type InterfaceStatus = 1 | 2 | 3 | 4 | 5 | 6

export interface InterfaceParamItem {
  id?: number
  name: string
  type: string
  required: boolean
  description: string
  example: string
}

export interface InterfaceBodyField {
  id?: number
  parent_id: number
  name: string
  type: string
  required: boolean
  description: string
  example: string
  children?: InterfaceBodyField[]
}

export interface InterfaceRequestBody {
  format: string
  data_type: string
  raw?: string
  fields: InterfaceBodyField[]
}

export interface InterfaceResponseHeader {
  id?: number
  name: string
  type: string
  required: boolean
  description: string
  example: string
}

export interface InterfaceResponseField {
  id?: number
  parent_id: number
  name: string
  type: string
  required: boolean
  description: string
  mock: string
  example: string
  children?: InterfaceResponseField[]
}

export interface InterfaceResponseResult {
  id?: number
  name: string
  status_code: number
  format: string
  data_type: string
  fields: InterfaceResponseField[]
}

export interface InterfaceResponseExample {
  id?: number
  name: string
  status_code: number
  content_type: string
  raw: string
  format?: string
  data_type?: string
}

export interface InterfaceItem {
  id: number
  project_id: number
  folder_id: number
  name: string
  method: HttpMethod
  url: string
  status: InterfaceStatus
  folder_name?: string
  updated_by?: number
  updated_by_name?: string
  created_at: string
  updated_at?: string
}

export interface InterfaceDetail extends InterfaceItem {
  request_headers: InterfaceParamItem[]
  request_body: InterfaceRequestBody
  query_params: InterfaceParamItem[]
  response_headers: InterfaceResponseHeader[]
  response_results: InterfaceResponseResult[]
  response_examples: InterfaceResponseExample[]
}

export type WorkspaceTabKind =
  | 'folder'
  | 'api'
  | 'create-api'
  | 'workspace-list'
  | 'project-list'
  | 'member-list'
  | 'env-list'
  | 'env-variables'
  | 'quick-test'

export interface WorkspaceTab {
  id: string
  kind: WorkspaceTabKind
  module: AppModule
  label: string
  method?: HttpMethod
  folderId?: string
  apiId?: string
  environmentId?: number
  dirty?: boolean
  closable?: boolean
}

export interface EnvironmentVariableItem {
  id: number
  environment_id: number
  key: string
  value: string
  description: string
  created_at: string
  updated_at: string
}

export interface EnvironmentItem {
  id: number
  project_id: number
  name: string
  remark: string
  is_default: boolean
  created_at: string
}
