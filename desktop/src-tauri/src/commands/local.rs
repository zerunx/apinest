use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::commands::auth::current_server_id;
use crate::commands::catalog::{
    EnvironmentRow, EnvironmentVariableRow, InterfaceRow, ProjectRow, WorkspaceRow,
};
use crate::db::{ensure_catalog_server, next_local_id, now_iso, Db};
use crate::error::{AppError, AppResult};
use crate::grpc::store::{delete_grpc_rpcs, delete_grpc_rpcs_for_folder, replace_grpc_rpcs_from_detail};

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
    let method = args.method.trim().to_uppercase();
    let url = args.url.trim().to_string();
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
        replace_grpc_rpcs_from_detail(conn, server_id, id, &detail)?;
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
    let url = args.url.trim().to_string();
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
        replace_grpc_rpcs_from_detail(conn, server_id, args.id, &detail)?;
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

#[derive(Debug, Deserialize)]
pub struct ReorderInterfacesArgs {
    pub workspace_id: i64,
    pub project_id: i64,
    pub folder_id: i64,
    pub interface_ids: Vec<i64>,
}

#[tauri::command]
pub fn reorder_local_interfaces(
    db: State<'_, Db>,
    args: ReorderInterfacesArgs,
) -> AppResult<()> {
    if args.interface_ids.is_empty() {
        return Err(AppError::msg("接口列表不能为空"));
    }

    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id FROM interfaces
             WHERE server_id = ?1 AND workspace_id = ?2 AND project_id = ?3 AND folder_id = ?4
             ORDER BY sort_order, id",
        )?;
        let existing: Vec<i64> = stmt
            .query_map(
                params![
                    server_id,
                    args.workspace_id,
                    args.project_id,
                    args.folder_id
                ],
                |r| r.get(0),
            )?
            .collect::<Result<Vec<_>, _>>()?;

        if existing.len() != args.interface_ids.len() {
            return Err(AppError::msg("接口列表不完整"));
        }

        let existing_set: std::collections::HashSet<i64> = existing.into_iter().collect();
        let mut seen = std::collections::HashSet::new();
        for id in &args.interface_ids {
            if !existing_set.contains(id) {
                return Err(AppError::msg("接口不属于该文件夹"));
            }
            if !seen.insert(*id) {
                return Err(AppError::msg("接口列表存在重复"));
            }
        }

        let tx = conn.unchecked_transaction()?;
        let now = now_iso();
        for (idx, id) in args.interface_ids.iter().enumerate() {
            let changed = tx.execute(
                "UPDATE interfaces SET sort_order = ?1, synced_at = ?2
                 WHERE server_id = ?3 AND id = ?4 AND folder_id = ?5",
                params![idx as i64, now, server_id, id, args.folder_id],
            )?;
            if changed == 0 {
                return Err(AppError::msg("接口不存在"));
            }
        }
        tx.commit()?;
        Ok(())
    })
}

#[derive(Debug, Deserialize)]
pub struct MoveReorderInterfacesArgs {
    pub workspace_id: i64,
    pub project_id: i64,
    pub interface_id: i64,
    pub from_folder_id: i64,
    pub to_folder_id: i64,
    pub target_interface_ids: Vec<i64>,
    pub source_interface_ids: Vec<i64>,
}

#[tauri::command]
pub fn move_reorder_local_interfaces(
    db: State<'_, Db>,
    args: MoveReorderInterfacesArgs,
) -> AppResult<()> {
    if args.target_interface_ids.is_empty() {
        return Err(AppError::msg("目标接口列表不能为空"));
    }
    if !args.target_interface_ids.contains(&args.interface_id) {
        return Err(AppError::msg("目标列表缺少被移动的接口"));
    }

    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let tx = conn.unchecked_transaction()?;
        let now = now_iso();

        let changed = tx.execute(
            "UPDATE interfaces SET folder_id = ?1, synced_at = ?2
             WHERE server_id = ?3 AND workspace_id = ?4 AND project_id = ?5 AND id = ?6",
            params![
                args.to_folder_id,
                now,
                server_id,
                args.workspace_id,
                args.project_id,
                args.interface_id
            ],
        )?;
        if changed == 0 {
            return Err(AppError::msg("接口不存在"));
        }

        // Target folder: full ordered list after move.
        let mut target_stmt = tx.prepare(
            "SELECT id FROM interfaces
             WHERE server_id = ?1 AND workspace_id = ?2 AND project_id = ?3 AND folder_id = ?4",
        )?;
        let target_existing: std::collections::HashSet<i64> = target_stmt
            .query_map(
                params![
                    server_id,
                    args.workspace_id,
                    args.project_id,
                    args.to_folder_id
                ],
                |r| r.get(0),
            )?
            .collect::<Result<std::collections::HashSet<_>, _>>()?;
        drop(target_stmt);

        if target_existing.len() != args.target_interface_ids.len() {
            return Err(AppError::msg("目标接口列表不完整"));
        }
        let mut seen = std::collections::HashSet::new();
        for id in &args.target_interface_ids {
            if !target_existing.contains(id) {
                return Err(AppError::msg("目标接口列表包含非法接口"));
            }
            if !seen.insert(*id) {
                return Err(AppError::msg("目标接口列表存在重复"));
            }
        }
        for (idx, id) in args.target_interface_ids.iter().enumerate() {
            tx.execute(
                "UPDATE interfaces SET sort_order = ?1, synced_at = ?2
                 WHERE server_id = ?3 AND id = ?4 AND folder_id = ?5",
                params![idx as i64, now, server_id, id, args.to_folder_id],
            )?;
        }

        // Source folder: remaining ordered list when moving across folders.
        if args.from_folder_id != args.to_folder_id {
            let mut source_stmt = tx.prepare(
                "SELECT id FROM interfaces
                 WHERE server_id = ?1 AND workspace_id = ?2 AND project_id = ?3 AND folder_id = ?4",
            )?;
            let source_existing: std::collections::HashSet<i64> = source_stmt
                .query_map(
                    params![
                        server_id,
                        args.workspace_id,
                        args.project_id,
                        args.from_folder_id
                    ],
                    |r| r.get(0),
                )?
                .collect::<Result<std::collections::HashSet<_>, _>>()?;
            drop(source_stmt);

            if source_existing.len() != args.source_interface_ids.len() {
                return Err(AppError::msg("源接口列表不完整"));
            }
            let mut source_seen = std::collections::HashSet::new();
            for id in &args.source_interface_ids {
                if !source_existing.contains(id) || *id == args.interface_id {
                    return Err(AppError::msg("源接口列表非法"));
                }
                if !source_seen.insert(*id) {
                    return Err(AppError::msg("源接口列表存在重复"));
                }
            }
            for (idx, id) in args.source_interface_ids.iter().enumerate() {
                tx.execute(
                    "UPDATE interfaces SET sort_order = ?1, synced_at = ?2
                     WHERE server_id = ?3 AND id = ?4 AND folder_id = ?5",
                    params![idx as i64, now, server_id, id, args.from_folder_id],
                )?;
            }
        }

        tx.commit()?;
        Ok(())
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
            delete_grpc_rpcs_for_folder(conn, server_id, *id)?;
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
        delete_grpc_rpcs(conn, server_id, interface_id)?;
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

#[derive(Debug, Deserialize)]
pub struct CreateEnvironmentArgs {
    pub workspace_id: i64,
    pub project_id: i64,
    pub name: String,
    pub remark: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEnvironmentArgs {
    pub id: i64,
    pub workspace_id: i64,
    pub project_id: i64,
    pub name: String,
    pub remark: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVariableArgs {
    pub environment_id: i64,
    pub key: String,
    pub value: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVariableArgs {
    pub id: i64,
    pub environment_id: i64,
    pub key: String,
    pub value: Option<String>,
    pub description: Option<String>,
}

#[tauri::command]
pub fn create_local_environment(
    db: State<'_, Db>,
    args: CreateEnvironmentArgs,
) -> AppResult<EnvironmentRow> {
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::msg("环境名称不能为空"));
    }
    let remark = args.remark.unwrap_or_default().trim().to_string();
    let is_default = args.is_default.unwrap_or(false);
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let id = next_local_id(conn)?;
        let now = now_iso();
        if is_default {
            conn.execute(
                "UPDATE environments SET is_default = 0
                 WHERE server_id = ?1 AND workspace_id = ?2 AND project_id = ?3",
                params![server_id, args.workspace_id, args.project_id],
            )?;
        }
        conn.execute(
            "INSERT INTO environments(
               id, server_id, workspace_id, project_id, name, remark, is_default, synced_at, is_local
             ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,1)",
            params![
                id,
                server_id,
                args.workspace_id,
                args.project_id,
                name,
                remark,
                if is_default { 1 } else { 0 },
                now
            ],
        )?;
        Ok(EnvironmentRow {
            id,
            name,
            remark,
            is_default,
        })
    })
}

#[tauri::command]
pub fn update_local_environment(
    db: State<'_, Db>,
    args: UpdateEnvironmentArgs,
) -> AppResult<EnvironmentRow> {
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::msg("环境名称不能为空"));
    }
    let remark = args.remark.unwrap_or_default().trim().to_string();
    let is_default = args.is_default.unwrap_or(false);
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let now = now_iso();
        if is_default {
            conn.execute(
                "UPDATE environments SET is_default = 0
                 WHERE server_id = ?1 AND workspace_id = ?2 AND project_id = ?3 AND id != ?4",
                params![server_id, args.workspace_id, args.project_id, args.id],
            )?;
        }
        let n = conn.execute(
            "UPDATE environments SET
               name = ?1, remark = ?2, is_default = ?3, synced_at = ?4, is_local = 1
             WHERE server_id = ?5 AND id = ?6",
            params![
                name,
                remark,
                if is_default { 1 } else { 0 },
                now,
                server_id,
                args.id
            ],
        )?;
        if n == 0 {
            return Err(AppError::msg("环境不存在"));
        }
        Ok(EnvironmentRow {
            id: args.id,
            name,
            remark,
            is_default,
        })
    })
}

#[tauri::command]
pub fn delete_local_environment(
    db: State<'_, Db>,
    workspace_id: i64,
    project_id: i64,
    environment_id: i64,
) -> AppResult<()> {
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM environments
                 WHERE server_id = ?1 AND workspace_id = ?2 AND project_id = ?3 AND id = ?4",
                params![server_id, workspace_id, project_id, environment_id],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if !exists {
            return Err(AppError::msg("环境不存在"));
        }
        conn.execute(
            "DELETE FROM environment_variables WHERE server_id = ?1 AND environment_id = ?2",
            params![server_id, environment_id],
        )?;
        conn.execute(
            "DELETE FROM environments WHERE server_id = ?1 AND id = ?2",
            params![server_id, environment_id],
        )?;
        Ok(())
    })
}

#[tauri::command]
pub fn create_local_variable(
    db: State<'_, Db>,
    args: CreateVariableArgs,
) -> AppResult<EnvironmentVariableRow> {
    let key = args.key.trim().to_string();
    if key.is_empty() {
        return Err(AppError::msg("变量名不能为空"));
    }
    let value = args.value.unwrap_or_default();
    let description = args.description.unwrap_or_default().trim().to_string();
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM environments WHERE server_id = ?1 AND id = ?2",
                params![server_id, args.environment_id],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if !exists {
            return Err(AppError::msg("环境不存在"));
        }
        let dup: bool = conn
            .query_row(
                "SELECT 1 FROM environment_variables
                 WHERE server_id = ?1 AND environment_id = ?2 AND key = ?3",
                params![server_id, args.environment_id, key],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if dup {
            return Err(AppError::msg("变量名已存在"));
        }
        let id = next_local_id(conn)?;
        let now = now_iso();
        conn.execute(
            "INSERT INTO environment_variables(
               id, server_id, environment_id, key, value, description, enabled, synced_at, is_local
             ) VALUES(?1,?2,?3,?4,?5,?6,1,?7,1)",
            params![id, server_id, args.environment_id, key, value, description, now],
        )?;
        Ok(EnvironmentVariableRow {
            id,
            environment_id: args.environment_id,
            key,
            value,
            description,
            enabled: true,
        })
    })
}

#[tauri::command]
pub fn update_local_variable(
    db: State<'_, Db>,
    args: UpdateVariableArgs,
) -> AppResult<EnvironmentVariableRow> {
    let key = args.key.trim().to_string();
    if key.is_empty() {
        return Err(AppError::msg("变量名不能为空"));
    }
    let value = args.value.unwrap_or_default();
    let description = args.description.unwrap_or_default().trim().to_string();
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let dup: bool = conn
            .query_row(
                "SELECT 1 FROM environment_variables
                 WHERE server_id = ?1 AND environment_id = ?2 AND key = ?3 AND id != ?4",
                params![server_id, args.environment_id, key, args.id],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if dup {
            return Err(AppError::msg("变量名已存在"));
        }
        let now = now_iso();
        let n = conn.execute(
            "UPDATE environment_variables SET
               key = ?1, value = ?2, description = ?3, synced_at = ?4, is_local = 1
             WHERE server_id = ?5 AND id = ?6 AND environment_id = ?7",
            params![
                key,
                value,
                description,
                now,
                server_id,
                args.id,
                args.environment_id
            ],
        )?;
        if n == 0 {
            return Err(AppError::msg("变量不存在"));
        }
        Ok(EnvironmentVariableRow {
            id: args.id,
            environment_id: args.environment_id,
            key,
            value,
            description,
            enabled: true,
        })
    })
}

#[tauri::command]
pub fn delete_local_variable(
    db: State<'_, Db>,
    environment_id: i64,
    variable_id: i64,
) -> AppResult<()> {
    let server_id = current_server_id(&db)?;
    db.with_conn(|conn| {
        let n = conn.execute(
            "DELETE FROM environment_variables
             WHERE server_id = ?1 AND environment_id = ?2 AND id = ?3",
            params![server_id, environment_id, variable_id],
        )?;
        if n == 0 {
            return Err(AppError::msg("变量不存在"));
        }
        Ok(())
    })
}
