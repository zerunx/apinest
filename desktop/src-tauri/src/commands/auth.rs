use rusqlite::params;
use serde::Serialize;
use tauri::State;

use crate::db::{now_iso, set_setting, Db};
use crate::error::{AppError, AppResult};
use crate::nest::NestClient;

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub server_id: i64,
    pub base_url: String,
    pub account: Option<String>,
    pub user_id: Option<i64>,
    pub logged_in: bool,
}

#[tauri::command]
pub fn get_session(db: State<'_, Db>) -> AppResult<SessionInfo> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT s.id, s.base_url, a.account, a.user_id, a.access_token
             FROM servers s
             LEFT JOIN auth_sessions a ON a.server_id = s.id
             ORDER BY s.id DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let token: Option<String> = row.get(4)?;
            Ok(SessionInfo {
                server_id: row.get(0)?,
                base_url: row.get(1)?,
                account: row.get(2)?,
                user_id: row.get(3)?,
                logged_in: token.map(|t| !t.is_empty()).unwrap_or(false),
            })
        } else {
            Ok(SessionInfo {
                server_id: 0,
                base_url: String::new(),
                account: None,
                user_id: None,
                logged_in: false,
            })
        }
    })
}

#[derive(Debug, serde::Deserialize)]
pub struct LoginArgs {
    pub base_url: String,
    pub account: String,
    pub password: String,
}

#[tauri::command]
pub fn login(db: State<'_, Db>, args: LoginArgs) -> AppResult<SessionInfo> {
    let base_url = args.base_url.trim().trim_end_matches('/').to_string();
    let client = NestClient::new(&base_url, None)?;
    let result = client.login(&args.account, &args.password)?;
    let now = now_iso();

    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO servers(name, base_url, created_at, updated_at)
             VALUES(?1, ?2, ?3, ?3)
             ON CONFLICT(base_url) DO UPDATE SET updated_at = excluded.updated_at, name = excluded.name",
            params!["ApiNest", base_url, now],
        )?;
        let server_id: i64 = conn.query_row(
            "SELECT id FROM servers WHERE base_url = ?1",
            params![base_url],
            |r| r.get(0),
        )?;
        conn.execute(
            "INSERT INTO auth_sessions(server_id, user_id, account, access_token, refresh_token, updated_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(server_id) DO UPDATE SET
               user_id = excluded.user_id,
               account = excluded.account,
               access_token = excluded.access_token,
               refresh_token = excluded.refresh_token,
               updated_at = excluded.updated_at",
            params![
                server_id,
                result.user.id,
                result.user.account,
                result.access_token,
                result.refresh_token,
                now
            ],
        )?;
        set_setting(conn, "active_server_id", &server_id.to_string())?;
        Ok(SessionInfo {
            server_id,
            base_url,
            account: Some(result.user.account),
            user_id: Some(result.user.id),
            logged_in: true,
        })
    })
}

#[tauri::command]
pub fn logout(db: State<'_, Db>) -> AppResult<()> {
    db.with_conn(|conn| {
        conn.execute("DELETE FROM auth_sessions", [])?;
        Ok(())
    })
}

#[tauri::command]
pub fn test_connection(db: State<'_, Db>) -> AppResult<usize> {
    let (base_url, token) = current_auth(&db)?;
    let client = NestClient::new(&base_url, Some(token))?;
    let list = client.list_workspaces()?;
    Ok(list.len())
}

pub fn current_auth(db: &Db) -> AppResult<(String, String)> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT s.base_url, a.access_token
             FROM servers s
             INNER JOIN auth_sessions a ON a.server_id = s.id
             ORDER BY s.id DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let base_url: String = row.get(0)?;
            let token: String = row.get(1)?;
            if token.is_empty() {
                return Err(AppError::NotLoggedIn);
            }
            Ok((base_url, token))
        } else {
            Err(AppError::NotLoggedIn)
        }
    })
}

pub fn current_server_id(db: &Db) -> AppResult<i64> {
    db.with_conn(|conn| crate::db::ensure_catalog_server(conn))
}
