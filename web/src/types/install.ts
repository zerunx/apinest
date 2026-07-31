export interface ApiResponse<T = unknown> {
  code: number
  message: string
  data: T
}

export interface InstallStatus {
  installed: boolean
  site_url?: string
}

export interface DatabaseConfig {
  driver: 'postgres' | 'mysql'
  host: string
  port: number
  name: string
  user: string
  password: string
  ssl_mode: string
}

export interface AppDatabaseConfig {
  username: string
  password: string
}

export interface AdminConfig {
  username: string
  password: string
  confirm_password: string
}

export interface InstallPayload {
  site_url: string
  database: DatabaseConfig
  app_database: AppDatabaseConfig
  admin: AdminConfig
}

export interface TestDatabaseResult {
  ok: boolean
  message: string
}

export interface InstallResult {
  message?: string
  admin_username?: string
  admin_password?: string
  database_user?: string
  database_password?: string
}

export interface InstallCredentials {
  admin: {
    username: string
    password: string
  }
  database: {
    username: string
    password: string
  }
}
