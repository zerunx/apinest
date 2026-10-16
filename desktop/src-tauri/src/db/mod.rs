use rusqlite::{Connection, params};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::{AppError, AppResult};

pub struct Db {
    pub conn: Mutex<Connection>,
    pub path: PathBuf,
}

impl Db {
    pub fn open(app_data_dir: PathBuf) -> AppResult<Self> {
        fs::create_dir_all(&app_data_dir)?;
        let path = app_data_dir.join("apinest-client.db");
        let conn = Connection::open(&path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let schema = include_str!("schema.sql");
        conn.execute_batch(schema)?;
        migrate(&conn)?;
        seed_example_data(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path,
        })
    }

    pub fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let conn = self.conn.lock().map_err(|_| AppError::DbLock)?;
        f(&conn)
    }

    /// Hot-backup the live database to `dest` (overwrites if present).
    pub fn export_to(&self, dest: &PathBuf) -> AppResult<()> {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        if dest.exists() {
            fs::remove_file(dest)?;
        }
        self.with_conn(|conn| {
            let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
            // Prefer VACUUM INTO for a consistent snapshot while the DB is open.
            let dest_sql = dest.to_string_lossy().replace('\'', "''");
            match conn.execute(&format!("VACUUM INTO '{dest_sql}'"), []) {
                Ok(_) => Ok(()),
                Err(_) => {
                    // Fallback: checkpointed file copy.
                    fs::copy(&self.path, dest)?;
                    Ok(())
                }
            }
        })
    }

    /// Replace the live database with the SQLite file at `src`.
    pub fn import_from(&self, src: &PathBuf) -> AppResult<()> {
        if !src.is_file() {
            return Err(AppError::msg("所选文件不存在"));
        }
        // Validate source looks like a readable SQLite DB with our schema.
        {
            let probe = Connection::open(src)?;
            let tables: i64 = probe.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('workspaces','projects','interfaces','app_settings')",
                [],
                |r| r.get(0),
            )?;
            if tables < 4 {
                return Err(AppError::msg("不是有效的 ApiNest 数据库文件"));
            }
        }

        let dest = self.path.clone();
        let tmp = dest.with_extension("db.import-tmp");
        if tmp.exists() {
            fs::remove_file(&tmp)?;
        }
        fs::copy(src, &tmp)?;

        let mut guard = self.conn.lock().map_err(|_| AppError::DbLock)?;
        // Release the file lock (important on Windows) before replacing.
        let placeholder = Connection::open_in_memory()?;
        let old = std::mem::replace(&mut *guard, placeholder);
        drop(old);

        let wal = PathBuf::from(format!("{}-wal", dest.display()));
        let shm = PathBuf::from(format!("{}-shm", dest.display()));
        let _ = fs::remove_file(&wal);
        let _ = fs::remove_file(&shm);
        if dest.exists() {
            fs::remove_file(&dest)?;
        }
        if fs::rename(&tmp, &dest).is_err() {
            fs::copy(&tmp, &dest)?;
            let _ = fs::remove_file(&tmp);
        }

        let new_conn = Connection::open(&dest)?;
        new_conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        migrate(&new_conn)?;
        *guard = new_conn;
        Ok(())
    }
}

pub fn now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub fn get_setting(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO app_settings(key, value) VALUES(?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> AppResult<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(1))?;
    for row in rows {
        if row? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn ensure_is_local_column(conn: &Connection, table: &str) -> AppResult<()> {
    if !column_exists(conn, table, "is_local")? {
        conn.execute(
            &format!("ALTER TABLE {table} ADD COLUMN is_local INTEGER NOT NULL DEFAULT 0"),
            [],
        )?;
    }
    Ok(())
}

fn ensure_text_column(conn: &Connection, table: &str, column: &str, default: &str) -> AppResult<()> {
    if !column_exists(conn, table, column)? {
        conn.execute(
            &format!(
                "ALTER TABLE {table} ADD COLUMN {column} TEXT NOT NULL DEFAULT '{default}'"
            ),
            [],
        )?;
    }
    Ok(())
}

fn ensure_nullable_integer_column(conn: &Connection, table: &str, column: &str) -> AppResult<()> {
    if !column_exists(conn, table, column)? {
        conn.execute(
            &format!("ALTER TABLE {table} ADD COLUMN {column} INTEGER"),
            [],
        )?;
    }
    Ok(())
}

fn migrate(conn: &Connection) -> AppResult<()> {
    for table in [
        "workspaces",
        "projects",
        "folders",
        "interfaces",
        "environments",
        "environment_variables",
    ] {
        ensure_is_local_column(conn, table)?;
    }
    ensure_text_column(conn, "environments", "remark", "")?;
    ensure_text_column(conn, "environment_variables", "description", "")?;
    ensure_nullable_integer_column(conn, "folders", "request_timeout_secs")?;
    ensure_text_column(conn, "folders", "http_version", "auto")?;
    ensure_text_column(conn, "folders", "description", "")?;
    ensure_text_column(conn, "folders", "headers_json", "[]")?;
    ensure_text_column(conn, "folders", "auth_json", "{}")?;
    // Column may have been created earlier with DEFAULT 'http1.1'; rewrite once to auto.
    migrate_folder_http_version_default_auto(conn)?;
    Ok(())
}

fn migrate_folder_http_version_default_auto(conn: &Connection) -> AppResult<()> {
    if get_setting(conn, "migrate_folder_http_auto_v1")?.as_deref() == Some("1") {
        return Ok(());
    }
    conn.execute(
        "UPDATE folders SET http_version = 'auto'
         WHERE http_version IS NULL OR http_version = '' OR http_version = 'http1.1'",
        [],
    )?;
    set_setting(conn, "migrate_folder_http_auto_v1", "1")?;
    Ok(())
}

/// First-run demo: workspace EXAMPLE_WORK / project EXAMPLE_WORK / interface EXAMPLE_API.
fn seed_example_data(conn: &Connection) -> AppResult<()> {
    if get_setting(conn, "seed_example_v1")?.as_deref() == Some("1") {
        return Ok(());
    }
    let ws_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM workspaces", [], |r| r.get(0))?;
    if ws_count > 0 {
        set_setting(conn, "seed_example_v1", "1")?;
        return Ok(());
    }

    let server_id = ensure_catalog_server(conn)?;
    let now = now_iso();
    let ws_id = next_local_id(conn)?;
    let project_id = next_local_id(conn)?;
    let interface_id = next_local_id(conn)?;

    conn.execute(
        "INSERT INTO workspaces(id, server_id, name, owner_id, role, synced_at, is_local)
         VALUES(?1, ?2, 'EXAMPLE_WORK', NULL, NULL, ?3, 1)",
        params![ws_id, server_id, now],
    )?;
    conn.execute(
        "INSERT INTO projects(id, server_id, workspace_id, name, synced_at, is_local)
         VALUES(?1, ?2, ?3, 'EXAMPLE_WORK', ?4, 1)",
        params![project_id, server_id, ws_id, now],
    )?;
    conn.execute(
        "INSERT INTO interfaces(
           id, server_id, workspace_id, project_id, folder_id,
           name, method, url, status, sort_order, synced_at, is_local
         ) VALUES(?1, ?2, ?3, ?4, 0, 'EXAMPLE_API', 'GET', 'https://httpbin.org/get', 1, 0, ?5, 1)",
        params![interface_id, server_id, ws_id, project_id, now],
    )?;

    let detail = r#"{"protocol":"HTTP","request_headers":[],"query_params":[],"request_body":{"format":"none","data_type":"Object","raw":"","fields":[]},"response_headers":[],"response_results":[],"response_examples":[]}"#;
    conn.execute(
        "INSERT INTO interface_details(server_id, interface_id, payload_json, synced_at)
         VALUES(?1, ?2, ?3, ?4)",
        params![server_id, interface_id, detail, now],
    )?;

    set_setting(conn, "seed_example_v1", "1")?;
    Ok(())
}

/// Ensure a catalog server exists for local-only writes (no Nest login required).
pub fn ensure_catalog_server(conn: &Connection) -> AppResult<i64> {
    if let Some(active) = get_setting(conn, "active_server_id")? {
        if let Ok(id) = active.parse::<i64>() {
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM servers WHERE id = ?1",
                    params![id],
                    |_| Ok(true),
                )
                .unwrap_or(false);
            if exists {
                return Ok(id);
            }
        }
    }

    let mut stmt = conn.prepare("SELECT id FROM servers ORDER BY id DESC LIMIT 1")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        set_setting(conn, "active_server_id", &id.to_string())?;
        return Ok(id);
    }

    let now = now_iso();
    conn.execute(
        "INSERT INTO servers(name, base_url, created_at, updated_at)
         VALUES('Local', 'local://desktop', ?1, ?1)",
        params![now],
    )?;
    let id = conn.last_insert_rowid();
    set_setting(conn, "active_server_id", &id.to_string())?;
    Ok(id)
}

/// Allocate a negative local id (avoids colliding with Nest server ids).
pub fn next_local_id(conn: &Connection) -> AppResult<i64> {
    let current = get_setting(conn, "local_id_seq")?
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let next = current - 1;
    set_setting(conn, "local_id_seq", &next.to_string())?;
    Ok(next)
}
