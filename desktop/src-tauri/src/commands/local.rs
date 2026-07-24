use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::commands::auth::current_server_id;
use crate::commands::catalog::{InterfaceRow, ProjectRow, WorkspaceRow};
use crate::db::{ensure_catalog_server, next_local_id, now_iso, Db};
use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceArgs {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectArgs {
    pub workspace_id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFolderArgs {
    pub workspace_id: i64,
    pub project_id: i64,
    pub parent_id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateInterfaceArgs {
    pub workspace_id: i64,
    pub project_id: i64,
    pub folder_id: i64,
    pub name: String,
    pub method: String,
    pub url: String,
    pub status: Option<i64>,
    pub detail: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInterfaceArgs {
    pub id: i64,
    pub workspace_id: i64,
    pub project_id: i64,
    pub folder_id: i64,
    pub name: String,
    pub method: String,
    pub url: String,
    pub status: Option<i64>,
    pub detail: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct FolderOption {
    pub id: i64,
    pub name: String,
    pub parent_id: i64,
    pub label: String,
}

#[tauri::command]
pub fn create_local_workspace(db: State<'_, Db>, args: CreateWorkspaceArgs) -> AppResult<WorkspaceRow> {
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::msg("工作空间名称不能为空"));
    }
    db.with_conn(|conn| {
        let server_id = ensure_catalog_server(conn)?;
        let id = next_local_id(conn)?;
        let now = now_iso();
        conn.execute(
            "INSERT INTO workspaces(id, server_id, name, owner_id, role, synced_at, is_local)
             VALUES(?1, ?2, ?3, NULL, NULL, ?4, 1)",
            params![id, server_id, name, now],
        )?;
        Ok(WorkspaceRow {
            id,
            name,
            role: None,
        })
    })
}

#[tauri::command]
pub fn create_local_project(db: State<'_, Db>, args: CreateProjectArgs) -> AppResult<ProjectRow> {
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::msg("项目名称不能为空"));
    }
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let id = next_local_id(conn)?;
        let now = now_iso();
        conn.execute(
            "INSERT INTO projects(id, server_id, workspace_id, name, synced_at, is_local)
             VALUES(?1, ?2, ?3, ?4, ?5, 1)",
            params![id, server_id, args.workspace_id, name, now],
        )?;
        Ok(ProjectRow {
            id,
            workspace_id: args.workspace_id,
            name,
        })
    })
}

#[tauri::command]
pub fn create_local_folder(
    db: State<'_, Db>,
    args: CreateFolderArgs,
) -> AppResult<serde_json::Value> {
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::msg("文件夹名称不能为空"));
    }
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let id = next_local_id(conn)?;
        let now = now_iso();
        conn.execute(
            "INSERT INTO folders(id, server_id, workspace_id, project_id, parent_id, name, sort_order, synced_at, is_local)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, 1)",
            params![
                id,
                server_id,
                args.workspace_id,
                args.project_id,
                args.parent_id,
                name,
                now
            ],
        )?;
        Ok(serde_json::json!({
            "id": id,
            "name": name,
            "parent_id": args.parent_id,
        }))
    })
}

#[tauri::command]
pub fn create_local_interface(
    db: State<'_, Db>,
    args: CreateInterfaceArgs,
) -> AppResult<InterfaceRow> {
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::msg("接口名称不能为空"));
    }
    if args.folder_id == 0 {
        return Err(AppError::msg("请选择文件夹"));
    }
    let method = args.method.trim().to_uppercase();
    let url = if args.url.trim().is_empty() {
        "/".to_string()
    } else {
        args.url.trim().to_string()
    };
    let status = args.status.unwrap_or(1);
    let server_id = current_server_id(&db)?;
    let detail = args
        .detail
        .unwrap_or_else(|| serde_json::json!({}));
    let detail_str = serde_json::to_string(&detail)?;

    db.with_conn(|conn| {
        let id = next_local_id(conn)?;
        let now = now_iso();
        conn.execute(
            "INSERT INTO interfaces(id, server_id, workspace_id, project_id, folder_id, name, method, url, status, sort_order, synced_at, is_local)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, 1)",
            params![
                id,
                server_id,
                args.workspace_id,
                args.project_id,
                args.folder_id,
                name,
                method,
                url,
                status,
                now
            ],
        )?;
        conn.execute(
            "INSERT INTO interface_details(server_id, interface_id, payload_json, synced_at)
             VALUES(?1, ?2, ?3, ?4)
             ON CONFLICT(server_id, interface_id) DO UPDATE SET
               payload_json = excluded.payload_json,
               synced_at = excluded.synced_at",
            params![server_id, id, detail_str, now],
        )?;
        Ok(InterfaceRow {
            id,
            workspace_id: args.workspace_id,
            project_id: args.project_id,
            folder_id: args.folder_id,
            name,
            method,
            url,
            detail: Some(detail),
        })
    })
}

#[tauri::command]
pub fn update_local_interface(
    db: State<'_, Db>,
    args: UpdateInterfaceArgs,
) -> AppResult<InterfaceRow> {
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::msg("接口名称不能为空"));
    }
    let method = args.method.trim().to_uppercase();
    let url = if args.url.trim().is_empty() {
        "/".to_string()
    } else {
        args.url.trim().to_string()
    };
    let status = args.status.unwrap_or(1);
    let server_id = current_server_id(&db)?;
    let detail = args
        .detail
        .unwrap_or_else(|| serde_json::json!({}));
    let detail_str = serde_json::to_string(&detail)?;

    db.with_conn(|conn| {
        let now = now_iso();
        let changed = conn.execute(
            "UPDATE interfaces SET
               folder_id = ?1, name = ?2, method = ?3, url = ?4, status = ?5, synced_at = ?6
             WHERE server_id = ?7 AND id = ?8",
            params![
                args.folder_id,
                name,
                method,
                url,
                status,
                now,
                server_id,
                args.id
            ],
        )?;
        if changed == 0 {
            return Err(AppError::msg("接口不存在"));
        }
        conn.execute(
            "INSERT INTO interface_details(server_id, interface_id, payload_json, synced_at)
             VALUES(?1, ?2, ?3, ?4)
             ON CONFLICT(server_id, interface_id) DO UPDATE SET
               payload_json = excluded.payload_json,
               synced_at = excluded.synced_at",
            params![server_id, args.id, detail_str, now],
        )?;
        Ok(InterfaceRow {
            id: args.id,
            workspace_id: args.workspace_id,
            project_id: args.project_id,
            folder_id: args.folder_id,
            name,
            method,
            url,
            detail: Some(detail),
        })
    })
}

#[tauri::command]
pub fn list_local_folders(
    db: State<'_, Db>,
    workspace_id: i64,
    project_id: i64,
) -> AppResult<Vec<FolderOption>> {
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, parent_id, name FROM folders
             WHERE server_id = ?1 AND workspace_id = ?2 AND project_id = ?3
             ORDER BY sort_order, id",
        )?;
        let rows: Vec<(i64, i64, String)> = stmt
            .query_map(params![server_id, workspace_id, project_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut options = Vec::new();
        fn walk(
            rows: &[(i64, i64, String)],
            parent: i64,
            prefix: &str,
            out: &mut Vec<FolderOption>,
        ) {
            for (id, pid, name) in rows {
                if *pid != parent {
                    continue;
                }
                let label = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{prefix} / {name}")
                };
                out.push(FolderOption {
                    id: *id,
                    name: name.clone(),
                    parent_id: *pid,
                    label: label.clone(),
                });
                walk(rows, *id, &label, out);
            }
        }
        walk(&rows, 0, "", &mut options);
        Ok(options)
    })
}

#[derive(Debug, Deserialize)]
pub struct RenameFolderArgs {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RenameInterfaceArgs {
    pub id: i64,
    pub name: String,
}

#[tauri::command]
pub fn rename_local_folder(db: State<'_, Db>, args: RenameFolderArgs) -> AppResult<()> {
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::msg("文件夹名称不能为空"));
    }
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let now = now_iso();
        let n = conn.execute(
            "UPDATE folders SET name = ?1, synced_at = ?2, is_local = 1
             WHERE server_id = ?3 AND id = ?4",
            params![name, now, server_id, args.id],
        )?;
        if n == 0 {
            return Err(AppError::msg("文件夹不存在"));
        }
        Ok(())
    })
}

#[tauri::command]
pub fn delete_local_folder(
    db: State<'_, Db>,
    workspace_id: i64,
    project_id: i64,
    folder_id: i64,
) -> AppResult<()> {
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, parent_id FROM folders
             WHERE server_id = ?1 AND workspace_id = ?2 AND project_id = ?3",
        )?;
        let all: Vec<(i64, i64)> = stmt
            .query_map(params![server_id, workspace_id, project_id], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        fn collect(all: &[(i64, i64)], parent: i64, out: &mut Vec<i64>) {
            for (id, pid) in all {
                if *pid == parent {
                    out.push(*id);
                    collect(all, *id, out);
                }
            }
        }
        let mut ids = vec![folder_id];
        collect(&all, folder_id, &mut ids);

        for id in &ids {
            conn.execute(
                "DELETE FROM interface_details WHERE server_id = ?1 AND interface_id IN (
                   SELECT id FROM interfaces WHERE server_id = ?1 AND folder_id = ?2
                 )",
                params![server_id, id],
            )?;
            conn.execute(
                "DELETE FROM interfaces WHERE server_id = ?1 AND folder_id = ?2",
                params![server_id, id],
            )?;
            conn.execute(
                "DELETE FROM folders WHERE server_id = ?1 AND id = ?2",
                params![server_id, id],
            )?;
        }
        Ok(())
    })
}

#[tauri::command]
pub fn rename_local_interface(db: State<'_, Db>, args: RenameInterfaceArgs) -> AppResult<()> {
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::msg("接口名称不能为空"));
    }
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let now = now_iso();
        let n = conn.execute(
            "UPDATE interfaces SET name = ?1, synced_at = ?2, is_local = 1
             WHERE server_id = ?3 AND id = ?4",
            params![name, now, server_id, args.id],
        )?;
        if n == 0 {
            return Err(AppError::msg("接口不存在"));
        }
        Ok(())
    })
}

#[tauri::command]
pub fn delete_local_interface(db: State<'_, Db>, interface_id: i64) -> AppResult<()> {
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        conn.execute(
            "DELETE FROM interface_details WHERE server_id = ?1 AND interface_id = ?2",
            params![server_id, interface_id],
        )?;
        let n = conn.execute(
            "DELETE FROM interfaces WHERE server_id = ?1 AND id = ?2",
            params![server_id, interface_id],
        )?;
        if n == 0 {
            return Err(AppError::msg("接口不存在"));
        }
        Ok(())
    })
}
