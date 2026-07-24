-- ApiNest desktop local schema (disk SQLite)
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS servers (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  base_url TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS auth_sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  server_id INTEGER NOT NULL UNIQUE REFERENCES servers(id) ON DELETE CASCADE,
  user_id INTEGER,
  account TEXT,
  access_token TEXT NOT NULL,
  refresh_token TEXT,
  expires_at TEXT,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sync_state (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  server_id INTEGER NOT NULL,
  scope_type TEXT NOT NULL,
  workspace_id INTEGER,
  project_id INTEGER,
  last_synced_at TEXT,
  last_status TEXT NOT NULL DEFAULT 'idle',
  last_error TEXT,
  UNIQUE(server_id, scope_type, workspace_id, project_id)
);

CREATE TABLE IF NOT EXISTS workspaces (
  id INTEGER NOT NULL,
  server_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  owner_id INTEGER,
  role INTEGER,
  synced_at TEXT NOT NULL,
  is_local INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (server_id, id)
);

CREATE TABLE IF NOT EXISTS projects (
  id INTEGER NOT NULL,
  server_id INTEGER NOT NULL,
  workspace_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  synced_at TEXT NOT NULL,
  is_local INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (server_id, id)
);

CREATE TABLE IF NOT EXISTS folders (
  id INTEGER NOT NULL,
  server_id INTEGER NOT NULL,
  workspace_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  parent_id INTEGER NOT NULL DEFAULT 0,
  name TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  synced_at TEXT NOT NULL,
  is_local INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (server_id, id)
);

CREATE TABLE IF NOT EXISTS interfaces (
  id INTEGER NOT NULL,
  server_id INTEGER NOT NULL,
  workspace_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  folder_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  method TEXT NOT NULL,
  url TEXT NOT NULL DEFAULT '',
  status INTEGER,
  sort_order INTEGER NOT NULL DEFAULT 0,
  synced_at TEXT NOT NULL,
  is_local INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (server_id, id)
);

CREATE TABLE IF NOT EXISTS interface_details (
  server_id INTEGER NOT NULL,
  interface_id INTEGER NOT NULL,
  payload_json TEXT NOT NULL,
  synced_at TEXT NOT NULL,
  PRIMARY KEY (server_id, interface_id)
);

CREATE TABLE IF NOT EXISTS environments (
  id INTEGER NOT NULL,
  server_id INTEGER NOT NULL,
  workspace_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  is_default INTEGER NOT NULL DEFAULT 0,
  synced_at TEXT NOT NULL,
  PRIMARY KEY (server_id, id)
);

CREATE TABLE IF NOT EXISTS environment_variables (
  id INTEGER NOT NULL,
  server_id INTEGER NOT NULL,
  environment_id INTEGER NOT NULL,
  key TEXT NOT NULL,
  value TEXT NOT NULL DEFAULT '',
  enabled INTEGER NOT NULL DEFAULT 1,
  synced_at TEXT NOT NULL,
  PRIMARY KEY (server_id, id)
);

CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS request_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  interface_id INTEGER,
  method TEXT NOT NULL,
  url TEXT NOT NULL,
  request_json TEXT,
  status_code INTEGER,
  response_json TEXT,
  duration_ms INTEGER,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS local_drafts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  interface_id INTEGER,
  payload_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_folders_project ON folders(server_id, project_id, parent_id);
CREATE INDEX IF NOT EXISTS idx_interfaces_folder ON interfaces(server_id, project_id, folder_id);
CREATE INDEX IF NOT EXISTS idx_projects_workspace ON projects(server_id, workspace_id);
CREATE INDEX IF NOT EXISTS idx_env_vars ON environment_variables(server_id, environment_id);
