use rusqlite::params;
use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::commands::auth::{current_auth, current_server_id};
use crate::db::{now_iso, get_setting, set_setting, Db};
use crate::error::{AppError, AppResult};
use crate::nest::NestClient;

#[derive(Debug, Serialize)]
pub struct WorkspaceRow {
    pub id: i64,
    pub name: String,
    pub role: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ProjectRow {
    pub id: i64,
    pub workspace_id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum TreeItem {
    #[serde(rename = "folder")]
    Folder {
        id: i64,
        name: String,
        children: Vec<TreeItem>,
    },
    #[serde(rename = "api")]
    Api {
        id: i64,
        name: String,
        method: String,
        url: String,
    },
}

#[derive(Debug, Serialize)]
pub struct InterfaceRow {
    pub id: i64,
    pub workspace_id: i64,
    pub project_id: i64,
    pub folder_id: i64,
    pub name: String,
    pub method: String,
    pub url: String,
    pub detail: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct EnvironmentRow {
    pub id: i64,
    pub name: String,
    pub is_default: bool,
}

#[tauri::command]
pub fn list_local_workspaces(db: State<'_, Db>) -> AppResult<Vec<WorkspaceRow>> {
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, name, role FROM workspaces WHERE server_id = ?1 ORDER BY name",
        )?;
        let rows = stmt
            .query_map(params![server_id], |r| {
                Ok(WorkspaceRow {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    role: r.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

#[tauri::command]
pub fn list_local_projects(db: State<'_, Db>, workspace_id: i64) -> AppResult<Vec<ProjectRow>> {
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, name FROM projects
             WHERE server_id = ?1 AND workspace_id = ?2 ORDER BY name",
        )?;
        let rows = stmt
            .query_map(params![server_id, workspace_id], |r| {
                Ok(ProjectRow {
                    id: r.get(0)?,
                    workspace_id: r.get(1)?,
                    name: r.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

#[tauri::command]
pub fn get_local_tree(
    db: State<'_, Db>,
    workspace_id: i64,
    project_id: i64,
    search: Option<String>,
) -> AppResult<Vec<TreeItem>> {
    let server_id = current_server_id(&db)?;
    let q = search.unwrap_or_default().trim().to_lowercase();

    db.with_conn(|conn| {
        let mut folder_stmt = conn.prepare(
            "SELECT id, parent_id, name FROM folders
             WHERE server_id = ?1 AND workspace_id = ?2 AND project_id = ?3
             ORDER BY sort_order, id",
        )?;
        let folders: Vec<(i64, i64, String)> = folder_stmt
            .query_map(params![server_id, workspace_id, project_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut api_stmt = conn.prepare(
            "SELECT id, folder_id, name, method, url FROM interfaces
             WHERE server_id = ?1 AND workspace_id = ?2 AND project_id = ?3
             ORDER BY sort_order, id",
        )?;
        let apis: Vec<(i64, i64, String, String, String)> = api_stmt
            .query_map(params![server_id, workspace_id, project_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(build_tree(&folders, &apis, 0, &q))
    })
}

fn build_tree(
    folders: &[(i64, i64, String)],
    apis: &[(i64, i64, String, String, String)],
    parent_id: i64,
    q: &str,
) -> Vec<TreeItem> {
    let mut items = Vec::new();

    for (id, pid, name) in folders {
        if *pid != parent_id {
            continue;
        }
        let children = build_tree(folders, apis, *id, q);
        let name_match = q.is_empty() || name.to_lowercase().contains(q);
        if name_match || !children.is_empty() {
            items.push(TreeItem::Folder {
                id: *id,
                name: name.clone(),
                children,
            });
        }
    }

    for (id, folder_id, name, method, url) in apis {
        if *folder_id != parent_id {
            continue;
        }
        let matched = q.is_empty()
            || name.to_lowercase().contains(q)
            || method.to_lowercase().contains(q)
            || url.to_lowercase().contains(q);
        if matched {
            items.push(TreeItem::Api {
                id: *id,
                name: name.clone(),
                method: method.clone(),
                url: url.clone(),
            });
        }
    }

    items
}

#[tauri::command]
pub fn get_interface(
    db: State<'_, Db>,
    workspace_id: i64,
    project_id: i64,
    interface_id: i64,
    refresh: bool,
) -> AppResult<InterfaceRow> {
    let server_id = current_server_id(&db)?;

    let local = db.with_conn(|conn| {
        conn.query_row(
            "SELECT id, workspace_id, project_id, folder_id, name, method, url FROM interfaces
             WHERE server_id = ?1 AND id = ?2",
            params![server_id, interface_id],
            |r| {
                Ok(InterfaceRow {
                    id: r.get(0)?,
                    workspace_id: r.get(1)?,
                    project_id: r.get(2)?,
                    folder_id: r.get(3)?,
                    name: r.get(4)?,
                    method: r.get(5)?,
                    url: r.get(6)?,
                    detail: None,
                })
            },
        )
        .map_err(AppError::from)
    });

    let mut row = match local {
        Ok(r) => r,
        Err(_) => InterfaceRow {
            id: interface_id,
            workspace_id,
            project_id,
            folder_id: 0,
            name: String::new(),
            method: "GET".into(),
            url: String::new(),
            detail: None,
        },
    };

    let cached_detail = db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT payload_json FROM interface_details WHERE server_id = ?1 AND interface_id = ?2",
        )?;
        let mut rows = stmt.query(params![server_id, interface_id])?;
        if let Some(r) = rows.next()? {
            let raw: String = r.get(0)?;
            Ok(Some(serde_json::from_str::<Value>(&raw)?))
        } else {
            Ok(None)
        }
    })?;

    if refresh || cached_detail.is_none() {
        let is_local = db.with_conn(|conn| {
            Ok(conn
                .query_row(
                    "SELECT IFNULL(is_local, 0) FROM interfaces WHERE server_id = ?1 AND id = ?2",
                    params![server_id, interface_id],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0)
                == 1)
        })?;

        if !is_local {
            if let Ok((base_url, token)) = current_auth(&db) {
                let client = NestClient::new(&base_url, Some(token))?;
                let detail = client.interface_detail(workspace_id, project_id, interface_id)?;
                let synced_at = now_iso();
                let payload = serde_json::to_string(&detail)?;

                if let Some(name) = detail.get("name").and_then(|v| v.as_str()) {
                    row.name = name.to_string();
                }
                if let Some(method) = detail.get("method").and_then(|v| v.as_str()) {
                    row.method = method.to_string();
                }
                if let Some(url) = detail.get("url").and_then(|v| v.as_str()) {
                    row.url = url.to_string();
                }
                if let Some(fid) = detail.get("folder_id").and_then(|v| v.as_i64()) {
                    row.folder_id = fid;
                }

                db.with_conn(|conn| {
                    conn.execute(
                        "INSERT INTO interfaces(
                           id, server_id, workspace_id, project_id, folder_id, name, method, url, status, sort_order, synced_at, is_local
                         ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,NULL,0,?9,0)
                         ON CONFLICT(server_id, id) DO UPDATE SET
                           name = excluded.name,
                           method = excluded.method,
                           url = excluded.url,
                           folder_id = excluded.folder_id,
                           synced_at = excluded.synced_at",
                        params![
                            interface_id,
                            server_id,
                            workspace_id,
                            project_id,
                            row.folder_id,
                            row.name,
                            row.method,
                            row.url,
                            synced_at
                        ],
                    )?;
                    conn.execute(
                        "INSERT INTO interface_details(server_id, interface_id, payload_json, synced_at)
                         VALUES(?1,?2,?3,?4)
                         ON CONFLICT(server_id, interface_id) DO UPDATE SET
                           payload_json = excluded.payload_json,
                           synced_at = excluded.synced_at",
                        params![server_id, interface_id, payload, synced_at],
                    )?;
                    Ok(())
                })?;
                row.detail = Some(detail);
                return Ok(row);
            }
        }
    }

    row.detail = cached_detail;
    Ok(row)
}

#[tauri::command]
pub fn list_local_environments(
    db: State<'_, Db>,
    workspace_id: i64,
    project_id: i64,
) -> AppResult<Vec<EnvironmentRow>> {
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, name, is_default FROM environments
             WHERE server_id = ?1 AND workspace_id = ?2 AND project_id = ?3
             ORDER BY is_default DESC, name",
        )?;
        let rows = stmt
            .query_map(params![server_id, workspace_id, project_id], |r| {
                Ok(EnvironmentRow {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    is_default: r.get::<_, i64>(2)? == 1,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

#[tauri::command]
pub fn get_env_vars(db: State<'_, Db>, environment_id: i64) -> AppResult<Vec<(String, String)>> {
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT key, value FROM environment_variables
             WHERE server_id = ?1 AND environment_id = ?2 AND enabled = 1",
        )?;
        let rows = stmt
            .query_map(params![server_id, environment_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

#[tauri::command]
pub fn get_active_context(db: State<'_, Db>) -> AppResult<(Option<i64>, Option<i64>, Option<i64>)> {
    db.with_conn(|conn| {
        let ws = get_setting(conn, "current_workspace_id")?
            .and_then(|s| s.parse().ok());
        let project = get_setting(conn, "current_project_id")?
            .and_then(|s| s.parse().ok());
        let env = get_setting(conn, "current_environment_id")?
            .and_then(|s| s.parse().ok());
        Ok((ws, project, env))
    })
}

#[tauri::command]
pub fn set_active_context(
    db: State<'_, Db>,
    workspace_id: Option<i64>,
    project_id: Option<i64>,
    environment_id: Option<i64>,
) -> AppResult<()> {
    db.with_conn(|conn| {
        if let Some(id) = workspace_id {
            set_setting(conn, "current_workspace_id", &id.to_string())?;
        }
        if let Some(id) = project_id {
            set_setting(conn, "current_project_id", &id.to_string())?;
        }
        if let Some(id) = environment_id {
            set_setting(conn, "current_environment_id", &id.to_string())?;
        }
        Ok(())
    })
}
