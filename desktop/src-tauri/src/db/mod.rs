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
        Ok(Self {
            conn: Mutex::new(conn),
            path,
        })
    }

    pub fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let conn = self.conn.lock().map_err(|_| AppError::DbLock)?;
        f(&conn)
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
