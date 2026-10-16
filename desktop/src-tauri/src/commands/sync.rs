use std::collections::{BTreeSet, HashMap, HashSet};

use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::commands::auth::{current_auth, current_server_id};
use crate::db::{now_iso, Db};
use crate::error::{AppError, AppResult};
use crate::nest::{parse_folder_id, NestClient, TreeNode};

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub workspaces: usize,
    pub projects: usize,
    pub folders: usize,
    pub interfaces: usize,
    pub environments: usize,
    pub synced_at: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SyncArgs {
    /// If set, only sync this workspace; otherwise sync all.
    pub workspace_id: Option<i64>,
}

#[tauri::command]
pub fn sync_now(db: State<'_, Db>, args: SyncArgs) -> AppResult<SyncResult> {
    let (base_url, token) = current_auth(&db)?;
    let server_id = current_server_id(&db)?;
    let client = NestClient::new(&base_url, Some(token))?;
    let synced_at = now_iso();

    let mut workspaces = client.list_workspaces()?;
    if let Some(wid) = args.workspace_id {
        workspaces.retain(|w| w.id == wid);
    }

    let mut project_count = 0usize;
    let mut folder_count = 0usize;
    let mut interface_count = 0usize;
    let mut env_count = 0usize;

    let workspace_ids: Vec<i64> = workspaces.iter().map(|w| w.id).collect();

    db.with_conn(|conn| {
        let tx = conn.unchecked_transaction()?;

        if args.workspace_id.is_none() {
            tx.execute(
                "DELETE FROM grpc_rpcs WHERE server_id = ?1 AND interface_id IN (
                   SELECT id FROM interfaces WHERE server_id = ?1 AND IFNULL(is_local, 0) = 0
                 )",
                params![server_id],
            )?;
            tx.execute(
                "DELETE FROM interface_details WHERE server_id = ?1 AND interface_id IN (
                   SELECT id FROM interfaces WHERE server_id = ?1 AND IFNULL(is_local, 0) = 0
                 )",
                params![server_id],
            )?;
            tx.execute(
                "DELETE FROM interfaces WHERE server_id = ?1 AND IFNULL(is_local, 0) = 0",
                params![server_id],
            )?;
            tx.execute(
                "DELETE FROM folders WHERE server_id = ?1 AND IFNULL(is_local, 0) = 0",
                params![server_id],
            )?;
            tx.execute(
                "DELETE FROM projects WHERE server_id = ?1 AND IFNULL(is_local, 0) = 0",
                params![server_id],
            )?;
            tx.execute(
                "DELETE FROM workspaces WHERE server_id = ?1 AND IFNULL(is_local, 0) = 0",
                params![server_id],
            )?;
            tx.execute(
                "DELETE FROM environment_variables
                 WHERE server_id = ?1 AND IFNULL(is_local, 0) = 0",
                params![server_id],
            )?;
            tx.execute(
                "DELETE FROM environments WHERE server_id = ?1 AND IFNULL(is_local, 0) = 0",
                params![server_id],
            )?;
        } else if let Some(wid) = args.workspace_id {
            clear_workspace_cache(&tx, server_id, wid)?;
        }

        for ws in &workspaces {
            tx.execute(
                "INSERT INTO workspaces(id, server_id, name, owner_id, role, synced_at)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(server_id, id) DO UPDATE SET
                   name = excluded.name,
                   owner_id = excluded.owner_id,
                   role = excluded.role,
                   synced_at = excluded.synced_at",
                params![ws.id, server_id, ws.name, ws.owner_id, ws.role, synced_at],
            )?;
        }
        tx.commit()?;
        Ok(())
    })?;

    for ws in &workspaces {
        let projects = client.list_projects(ws.id)?;
        project_count += projects.len();

        db.with_conn(|conn| {
            for p in &projects {
                conn.execute(
                    "INSERT INTO projects(id, server_id, workspace_id, name, synced_at)
                     VALUES(?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(server_id, id) DO UPDATE SET
                       workspace_id = excluded.workspace_id,
                       name = excluded.name,
                       synced_at = excluded.synced_at",
                    params![p.id, server_id, ws.id, p.name, synced_at],
                )?;
            }
            Ok(())
        })?;

        for p in &projects {
            let tree = client.folder_tree(ws.id, p.id)?;
            let interfaces = client.list_interfaces(ws.id, p.id)?;
            let envs = client.list_environments(ws.id, p.id)?;

            folder_count += count_folders(&tree);
            interface_count += interfaces.len();
            env_count += envs.len();

            db.with_conn(|conn| {
                walk_folders(conn, server_id, ws.id, p.id, 0, &tree, &synced_at, &mut 0)?;

                for (idx, item) in interfaces.iter().enumerate() {
                    conn.execute(
                        "INSERT INTO interfaces(
                           id, server_id, workspace_id, project_id, folder_id,
                           name, method, url, status, sort_order, synced_at
                         ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
                         ON CONFLICT(server_id, id) DO UPDATE SET
                           workspace_id = excluded.workspace_id,
                           project_id = excluded.project_id,
                           folder_id = excluded.folder_id,
                           name = excluded.name,
                           method = excluded.method,
                           url = excluded.url,
                           status = excluded.status,
                           sort_order = excluded.sort_order,
                           synced_at = excluded.synced_at",
                        params![
                            item.id,
                            server_id,
                            ws.id,
                            p.id,
                            item.folder_id,
                            item.name,
                            item.method,
                            item.url,
                            item.status.map(|s| s as i64),
                            item.sort_order.unwrap_or(idx as i64),
                            synced_at
                        ],
                    )?;
                }

                for env in &envs {
                    conn.execute(
                        "INSERT INTO environments(
                           id, server_id, workspace_id, project_id, name, remark, is_default, synced_at, is_local
                         ) VALUES(?1,?2,?3,?4,?5,'',?6,?7,0)
                         ON CONFLICT(server_id, id) DO UPDATE SET
                           workspace_id = excluded.workspace_id,
                           project_id = excluded.project_id,
                           name = excluded.name,
                           is_default = excluded.is_default,
                           synced_at = excluded.synced_at,
                           is_local = 0",
                        params![
                            env.id,
                            server_id,
                            ws.id,
                            p.id,
                            env.name,
                            if env.is_default { 1 } else { 0 },
                            synced_at
                        ],
                    )?;

                    let vars = client.list_variables(ws.id, p.id, env.id)?;
                    for v in vars {
                        conn.execute(
                            "INSERT INTO environment_variables(
                               id, server_id, environment_id, key, value, description, enabled, synced_at, is_local
                             ) VALUES(?1,?2,?3,?4,?5,'',1,?6,0)
                             ON CONFLICT(server_id, id) DO UPDATE SET
                               environment_id = excluded.environment_id,
                               key = excluded.key,
                               value = excluded.value,
                               synced_at = excluded.synced_at,
                               is_local = 0",
                            params![v.id, server_id, env.id, v.key, v.value, synced_at],
                        )?;
                    }
                }
                Ok(())
            })?;
        }
    }

    db.with_conn(|conn| {
        let scope = if args.workspace_id.is_some() {
            "workspace"
        } else {
            "all"
        };
        conn.execute(
            "DELETE FROM sync_state WHERE server_id = ?1 AND scope_type = ?2 AND (
               (?3 IS NULL AND workspace_id IS NULL) OR workspace_id = ?3
             )",
            params![server_id, scope, args.workspace_id],
        )?;
        conn.execute(
            "INSERT INTO sync_state(server_id, scope_type, workspace_id, project_id, last_synced_at, last_status, last_error)
             VALUES(?1, ?2, ?3, NULL, ?4, 'success', NULL)",
            params![server_id, scope, args.workspace_id, synced_at],
        )?;
        Ok(())
    })?;

    let _ = workspace_ids;

    Ok(SyncResult {
        workspaces: workspaces.len(),
        projects: project_count,
        folders: folder_count,
        interfaces: interface_count,
        environments: env_count,
        synced_at,
    })
}

/// Push local data to Nest with local-wins semantics:
/// - existing server ids (id > 0): PUT update; if missing on server, POST create then remap
/// - local-only ids (id <= 0): POST create then remap local rows to the new server ids
#[tauri::command]
pub fn sync_upload(db: State<'_, Db>, args: SyncArgs) -> AppResult<SyncResult> {
    let (base_url, token) = current_auth(&db)?;
    let server_id = current_server_id(&db)?;
    let client = NestClient::with_timeout(&base_url, Some(token), 180)?;
    let synced_at = now_iso();

    // Pull offline/local-catalog trees onto the logged-in server before upload.
    db.with_conn(|conn| adopt_orphan_local_data(conn, server_id))?;

    let workspaces: Vec<(i64, String, i64)> = db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, name, IFNULL(is_local, 0) FROM workspaces
             WHERE server_id = ?1 AND (?2 IS NULL OR id = ?2)
             ORDER BY id",
        )?;
        let rows = stmt
            .query_map(params![server_id, args.workspace_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })?;

    if workspaces.is_empty() {
        return Err(AppError::msg(
            "Nothing to upload. Create a local project or sync download first.",
        ));
    }

    let mut workspace_map: HashMap<i64, i64> = HashMap::new();
    let mut project_count = 0usize;
    let mut folder_count = 0usize;
    let mut interface_count = 0usize;
    let mut uploaded_workspaces = BTreeSet::new();

    for (local_ws_id, ws_name, is_local) in &workspaces {
        let _ = is_local;
        let remote_ws_id = ensure_workspace_on_server(
            &db,
            &client,
            server_id,
            *local_ws_id,
            ws_name,
            &synced_at,
            &mut workspace_map,
        )?;
        uploaded_workspaces.insert(remote_ws_id);

        // After workspace remap, projects already point at remote_ws_id.
        let projects: Vec<(i64, String)> = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name FROM projects
                 WHERE server_id = ?1 AND workspace_id = ?2
                 ORDER BY id",
            )?;
            let rows = stmt
                .query_map(params![server_id, remote_ws_id], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?;

        let mut project_map: HashMap<i64, i64> = HashMap::new();

        for (local_proj_id, proj_name) in &projects {
            let remote_proj_id = ensure_project_on_server(
                &db,
                &client,
                server_id,
                remote_ws_id,
                *local_proj_id,
                proj_name,
                &synced_at,
                &mut project_map,
            )?;
            project_count += 1;

            let (folders_n, apis_n) = upload_project_tree(
                &db,
                &client,
                server_id,
                remote_ws_id,
                remote_proj_id,
                &synced_at,
            )?;
            folder_count += folders_n;
            interface_count += apis_n;
        }
    }

    db.with_conn(|conn| {
        conn.execute(
            "DELETE FROM sync_state WHERE server_id = ?1 AND scope_type = 'upload' AND (
               (?2 IS NULL AND workspace_id IS NULL) OR workspace_id = ?2
             )",
            params![server_id, args.workspace_id],
        )?;
        conn.execute(
            "INSERT INTO sync_state(server_id, scope_type, workspace_id, project_id, last_synced_at, last_status, last_error)
             VALUES(?1, 'upload', ?2, NULL, ?3, 'success', NULL)",
            params![server_id, args.workspace_id, synced_at],
        )?;
        Ok(())
    })?;

    Ok(SyncResult {
        workspaces: uploaded_workspaces.len(),
        projects: project_count,
        folders: folder_count,
        interfaces: interface_count,
        environments: 0,
        synced_at,
    })
}

fn is_missing_on_server(err: &AppError) -> bool {
    let msg = err.to_string().to_lowercase();
    msg.contains("not found")
        || msg.contains("no permission")
        || msg.contains("permission denied")
}

/// Move offline catalog (`local://desktop`) trees onto the active Nest server so upload can see them.
pub fn adopt_orphan_local_data(conn: &Connection, active_server_id: i64) -> AppResult<()> {
    let orphan_servers: Vec<i64> = {
        let mut stmt = conn.prepare(
            "SELECT id FROM servers
             WHERE id != ?1
               AND (base_url = 'local://desktop' OR base_url LIKE 'local://%')",
        )?;
        let rows = stmt
            .query_map(params![active_server_id], |r| r.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    for old_sid in orphan_servers {
        move_server_tree(conn, old_sid, active_server_id)?;
    }
    Ok(())
}

fn move_server_tree(conn: &Connection, from_sid: i64, to_sid: i64) -> AppResult<()> {
    if from_sid == to_sid {
        return Ok(());
    }

    // Reassign conflicting primary keys to fresh local ids before flipping server_id.
    rekey_conflicts(conn, from_sid, to_sid, "workspaces")?;
    rekey_conflicts(conn, from_sid, to_sid, "projects")?;
    rekey_conflicts(conn, from_sid, to_sid, "folders")?;
    rekey_conflicts(conn, from_sid, to_sid, "interfaces")?;
    rekey_conflicts(conn, from_sid, to_sid, "environments")?;
    rekey_conflicts(conn, from_sid, to_sid, "environment_variables")?;

    // Child tables keyed by server_id + foreign id (not always same PK shape).
    conn.execute(
        "UPDATE interface_details SET server_id = ?1 WHERE server_id = ?2",
        params![to_sid, from_sid],
    )?;
    conn.execute(
        "UPDATE grpc_rpcs SET server_id = ?1 WHERE server_id = ?2",
        params![to_sid, from_sid],
    )?;

    for table in [
        "environment_variables",
        "environments",
        "interfaces",
        "folders",
        "projects",
        "workspaces",
    ] {
        conn.execute(
            &format!("UPDATE {table} SET server_id = ?1 WHERE server_id = ?2"),
            params![to_sid, from_sid],
        )?;
    }
    Ok(())
}

fn rekey_conflicts(conn: &Connection, from_sid: i64, to_sid: i64, table: &str) -> AppResult<()> {
    let conflicting: Vec<i64> = {
        let sql = format!(
            "SELECT a.id FROM {table} a
             INNER JOIN {table} b ON b.server_id = ?1 AND b.id = a.id
             WHERE a.server_id = ?2"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map(params![to_sid, from_sid], |r| r.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    for old_id in conflicting {
        let new_id = crate::db::next_local_id(conn)?;
        match table {
            "workspaces" => remap_workspace_id(conn, from_sid, old_id, new_id, &now_iso())?,
            "projects" => {
                let ws: i64 = conn.query_row(
                    "SELECT workspace_id FROM projects WHERE server_id = ?1 AND id = ?2",
                    params![from_sid, old_id],
                    |r| r.get(0),
                )?;
                remap_project_id(conn, from_sid, old_id, new_id, ws, &now_iso())?;
            }
            "folders" => {
                let (ws, proj, parent): (i64, i64, i64) = conn.query_row(
                    "SELECT workspace_id, project_id, parent_id FROM folders
                     WHERE server_id = ?1 AND id = ?2",
                    params![from_sid, old_id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )?;
                remap_folder_id(conn, from_sid, old_id, new_id, ws, proj, parent, &now_iso())?;
            }
            "interfaces" => {
                let (ws, proj, folder): (i64, i64, i64) = conn.query_row(
                    "SELECT workspace_id, project_id, folder_id FROM interfaces
                     WHERE server_id = ?1 AND id = ?2",
                    params![from_sid, old_id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )?;
                remap_interface_id(conn, from_sid, old_id, new_id, ws, proj, folder, &now_iso())?;
            }
            "environments" => {
                conn.execute(
                    "UPDATE environment_variables SET environment_id = ?1
                     WHERE server_id = ?2 AND environment_id = ?3",
                    params![new_id, from_sid, old_id],
                )?;
                conn.execute(
                    "UPDATE environments SET id = ?1 WHERE server_id = ?2 AND id = ?3",
                    params![new_id, from_sid, old_id],
                )?;
            }
            "environment_variables" => {
                conn.execute(
                    "UPDATE environment_variables SET id = ?1 WHERE server_id = ?2 AND id = ?3",
                    params![new_id, from_sid, old_id],
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn ensure_workspace_on_server(
    db: &Db,
    client: &NestClient,
    server_id: i64,
    local_id: i64,
    name: &str,
    synced_at: &str,
    workspace_map: &mut HashMap<i64, i64>,
) -> AppResult<i64> {
    if let Some(mapped) = workspace_map.get(&local_id) {
        return Ok(*mapped);
    }

    // Positive ids: try update first; if missing/no access, create.
    // Negative / pure-local ids: always create.
    if local_id > 0 {
        match client.update_workspace(local_id, name) {
            Ok(()) => {
                db.with_conn(|conn| {
                    conn.execute(
                        "UPDATE workspaces SET is_local = 0, synced_at = ?1
                         WHERE server_id = ?2 AND id = ?3",
                        params![synced_at, server_id, local_id],
                    )?;
                    Ok(())
                })?;
                workspace_map.insert(local_id, local_id);
                return Ok(local_id);
            }
            Err(e) if is_missing_on_server(&e) => {}
            Err(e) => return Err(e),
        }
    }

    let new_id = client.create_workspace(name)?;
    if local_id != new_id {
        db.with_conn(|conn| remap_workspace_id(conn, server_id, local_id, new_id, synced_at))?;
    } else {
        db.with_conn(|conn| {
            conn.execute(
                "UPDATE workspaces SET is_local = 0, synced_at = ?1
                 WHERE server_id = ?2 AND id = ?3",
                params![synced_at, server_id, new_id],
            )?;
            Ok(())
        })?;
    }
    workspace_map.insert(local_id, new_id);
    Ok(new_id)
}

fn ensure_project_on_server(
    db: &Db,
    client: &NestClient,
    server_id: i64,
    workspace_id: i64,
    local_id: i64,
    name: &str,
    synced_at: &str,
    project_map: &mut HashMap<i64, i64>,
) -> AppResult<i64> {
    if let Some(mapped) = project_map.get(&local_id) {
        return Ok(*mapped);
    }

    if local_id > 0 {
        match client.update_project(workspace_id, local_id, name) {
            Ok(()) => {
                db.with_conn(|conn| {
                    conn.execute(
                        "UPDATE projects SET is_local = 0, synced_at = ?1, workspace_id = ?2
                         WHERE server_id = ?3 AND id = ?4",
                        params![synced_at, workspace_id, server_id, local_id],
                    )?;
                    Ok(())
                })?;
                project_map.insert(local_id, local_id);
                return Ok(local_id);
            }
            Err(e) if is_missing_on_server(&e) => {}
            Err(e) => return Err(e),
        }
    }

    let new_id = client.create_project(workspace_id, name)?;
    if local_id != new_id {
        db.with_conn(|conn| {
            remap_project_id(conn, server_id, local_id, new_id, workspace_id, synced_at)
        })?;
    } else {
        db.with_conn(|conn| {
            conn.execute(
                "UPDATE projects SET is_local = 0, synced_at = ?1, workspace_id = ?2
                 WHERE server_id = ?3 AND id = ?4",
                params![synced_at, workspace_id, server_id, new_id],
            )?;
            Ok(())
        })?;
    }
    project_map.insert(local_id, new_id);
    Ok(new_id)
}

fn upload_project_tree(
    db: &Db,
    client: &NestClient,
    server_id: i64,
    workspace_id: i64,
    project_id: i64,
    synced_at: &str,
) -> AppResult<(usize, usize)> {
    let folders: Vec<(i64, i64, String)> = db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, parent_id, name FROM folders
             WHERE server_id = ?1 AND project_id = ?2
             ORDER BY sort_order, id",
        )?;
        let rows = stmt
            .query_map(params![server_id, project_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })?;

    let mut folder_map: HashMap<i64, i64> = HashMap::new();
    let ordered = topological_folders(&folders);

    for (local_fid, parent_id, name) in ordered {
        let remote_parent = if parent_id <= 0 {
            0
        } else {
            *folder_map.get(&parent_id).unwrap_or(&parent_id)
        };

        if local_fid > 0 {
            match client.update_folder(workspace_id, project_id, local_fid, &name) {
                Ok(()) => {
                    folder_map.insert(local_fid, local_fid);
                    db.with_conn(|conn| {
                        conn.execute(
                            "UPDATE folders SET is_local = 0, synced_at = ?1,
                               workspace_id = ?2, project_id = ?3
                             WHERE server_id = ?4 AND id = ?5",
                            params![synced_at, workspace_id, project_id, server_id, local_fid],
                        )?;
                        Ok(())
                    })?;
                    continue;
                }
                Err(e) if is_missing_on_server(&e) => {}
                Err(e) => return Err(e),
            }
        }

        let new_id = client.create_folder(workspace_id, project_id, remote_parent, &name)?;
        if local_fid != new_id {
            db.with_conn(|conn| {
                remap_folder_id(
                    conn,
                    server_id,
                    local_fid,
                    new_id,
                    workspace_id,
                    project_id,
                    remote_parent,
                    synced_at,
                )
            })?;
        } else {
            db.with_conn(|conn| {
                conn.execute(
                    "UPDATE folders SET is_local = 0, synced_at = ?1,
                       workspace_id = ?2, project_id = ?3, parent_id = ?4
                     WHERE server_id = ?5 AND id = ?6",
                    params![
                        synced_at,
                        workspace_id,
                        project_id,
                        remote_parent,
                        server_id,
                        new_id
                    ],
                )?;
                Ok(())
            })?;
        }
        folder_map.insert(local_fid, new_id);
    }

    // Default folder for interfaces sitting at project root (folder_id = 0).
    let mut default_folder_id: Option<i64> = None;

    let interfaces: Vec<(i64, i64, String, String, String, i64, Option<String>)> =
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT i.id, i.folder_id, i.name, i.method, i.url, IFNULL(i.status, 1), d.payload_json
                 FROM interfaces i
                 LEFT JOIN interface_details d
                   ON d.server_id = i.server_id AND d.interface_id = i.id
                 WHERE i.server_id = ?1 AND i.project_id = ?2
                 ORDER BY i.sort_order, i.id",
            )?;
            let rows = stmt
                .query_map(params![server_id, project_id], |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?;

    let interface_total = interfaces.len();

    for (local_iid, folder_id, name, method, url, status, payload) in interfaces {
        let mut remote_folder = if folder_id <= 0 {
            0
        } else {
            *folder_map.get(&folder_id).unwrap_or(&folder_id)
        };
        if remote_folder <= 0 {
            if default_folder_id.is_none() {
                let id = client.create_folder(workspace_id, project_id, 0, "Ungrouped")?;
                db.with_conn(|conn| {
                    // Keep a local mirror for the default folder if missing.
                    let exists: i64 = conn.query_row(
                        "SELECT COUNT(*) FROM folders WHERE server_id = ?1 AND id = ?2",
                        params![server_id, id],
                        |r| r.get(0),
                    )?;
                    if exists == 0 {
                        conn.execute(
                            "INSERT INTO folders(
                               id, server_id, workspace_id, project_id, parent_id, name,
                               sort_order, synced_at, is_local
                             ) VALUES(?1,?2,?3,?4,0,'Ungrouped',0,?5,0)",
                            params![id, server_id, workspace_id, project_id, synced_at],
                        )?;
                    }
                    Ok(())
                })?;
                default_folder_id = Some(id);
            }
            remote_folder = default_folder_id.unwrap();
        }

        let detail: Value = payload
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_else(|| json!({}));
        let body = build_interface_payload(
            workspace_id,
            project_id,
            remote_folder,
            None,
            &name,
            &method,
            &url,
            status as u8,
            &detail,
        );

        if local_iid > 0 {
            let mut update_body = body.clone();
            if let Some(obj) = update_body.as_object_mut() {
                obj.insert("interface_id".into(), json!(local_iid));
                obj.insert("folder_id".into(), json!(remote_folder));
            }
            match client.update_interface(&update_body) {
                Ok(()) => {
                    db.with_conn(|conn| {
                        conn.execute(
                            "UPDATE interfaces SET is_local = 0, synced_at = ?1,
                               workspace_id = ?2, project_id = ?3, folder_id = ?4
                             WHERE server_id = ?5 AND id = ?6",
                            params![
                                synced_at,
                                workspace_id,
                                project_id,
                                remote_folder,
                                server_id,
                                local_iid
                            ],
                        )?;
                        Ok(())
                    })?;
                    continue;
                }
                Err(e) if is_missing_on_server(&e) => {}
                Err(e) => return Err(e),
            }
        }

        let create_body = build_interface_payload(
            workspace_id,
            project_id,
            remote_folder,
            None,
            &name,
            &method,
            &url,
            status as u8,
            &detail,
        );
        let new_id = client.create_interface(&create_body)?;
        if local_iid != new_id {
            db.with_conn(|conn| {
                remap_interface_id(
                    conn,
                    server_id,
                    local_iid,
                    new_id,
                    workspace_id,
                    project_id,
                    remote_folder,
                    synced_at,
                )
            })?;
        } else {
            db.with_conn(|conn| {
                conn.execute(
                    "UPDATE interfaces SET is_local = 0, synced_at = ?1,
                       workspace_id = ?2, project_id = ?3, folder_id = ?4
                     WHERE server_id = ?5 AND id = ?6",
                    params![
                        synced_at,
                        workspace_id,
                        project_id,
                        remote_folder,
                        server_id,
                        new_id
                    ],
                )?;
                Ok(())
            })?;
        }
    }

    Ok((folder_map.len(), interface_total))
}

fn topological_folders(folders: &[(i64, i64, String)]) -> Vec<(i64, i64, String)> {
    let ids: HashSet<i64> = folders.iter().map(|(id, _, _)| *id).collect();
    let mut remaining: HashMap<i64, (i64, String)> = folders
        .iter()
        .map(|(id, pid, name)| (*id, (*pid, name.clone())))
        .collect();
    let mut done: HashSet<i64> = HashSet::new();
    let mut out = Vec::with_capacity(folders.len());

    while !remaining.is_empty() {
        let mut ready: Vec<i64> = remaining
            .iter()
            .filter(|(_, (pid, _))| *pid <= 0 || !ids.contains(pid) || done.contains(pid))
            .map(|(id, _)| *id)
            .collect();
        if ready.is_empty() {
            // Break cycles by taking the smallest remaining id.
            ready.push(*remaining.keys().min().unwrap());
        }
        ready.sort();
        for id in ready {
            if let Some((pid, name)) = remaining.remove(&id) {
                out.push((id, pid, name));
                done.insert(id);
            }
        }
    }
    out
}

fn clip(s: &str, max: usize) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i >= max {
            break;
        }
        out.push(ch);
    }
    out
}

fn clip_json_str(v: &mut Value, key: &str, max: usize) {
    if let Some(obj) = v.as_object_mut() {
        if let Some(Value::String(s)) = obj.get(key) {
            if s.chars().count() > max {
                let clipped = clip(s, max);
                obj.insert(key.to_string(), Value::String(clipped));
            }
        }
    }
}

/// Truncate nested param/field strings to Nest MaxLen limits so create/update won't fail.
fn sanitize_param_item(item: &mut Value) {
    clip_json_str(item, "name", 100);
    clip_json_str(item, "type", 50);
    clip_json_str(item, "description", 1024);
    clip_json_str(item, "example", 1024);
    clip_json_str(item, "mock", 1024);
    if let Some(children) = item.get_mut("children").and_then(|c| c.as_array_mut()) {
        for child in children {
            sanitize_param_item(child);
        }
    }
}

fn sanitize_param_list(list: &mut Value) {
    if let Some(arr) = list.as_array_mut() {
        for item in arr {
            sanitize_param_item(item);
        }
    }
}

fn sanitize_request_body(body: &mut Value) {
    clip_json_str(body, "format", 20);
    clip_json_str(body, "data_type", 20);
    if let Some(fields) = body.get_mut("fields") {
        sanitize_param_list(fields);
    }
}

fn sanitize_response_results(list: &mut Value) {
    if let Some(arr) = list.as_array_mut() {
        for item in arr {
            clip_json_str(item, "name", 100);
            clip_json_str(item, "format", 20);
            clip_json_str(item, "data_type", 20);
            if let Some(fields) = item.get_mut("fields") {
                sanitize_param_list(fields);
            }
        }
    }
}

fn sanitize_response_examples(list: &mut Value) {
    if let Some(arr) = list.as_array_mut() {
        for item in arr {
            clip_json_str(item, "name", 100);
            clip_json_str(item, "content_type", 100);
        }
    }
}

fn build_interface_payload(
    workspace_id: i64,
    project_id: i64,
    folder_id: i64,
    interface_id: Option<i64>,
    name: &str,
    method: &str,
    url: &str,
    status: u8,
    detail: &Value,
) -> Value {
    let empty_arr = json!([]);
    let default_body = json!({
        "format": "json",
        "data_type": "Object",
        "raw": "",
        "fields": []
    });
    let protocol = detail
        .get("protocol")
        .and_then(|v| v.as_str())
        .unwrap_or("HTTP");

    let mut request_headers = detail
        .get("request_headers")
        .cloned()
        .unwrap_or_else(|| empty_arr.clone());
    let mut query_params = detail
        .get("query_params")
        .cloned()
        .unwrap_or_else(|| empty_arr.clone());
    let mut request_body = detail
        .get("request_body")
        .cloned()
        .unwrap_or(default_body);
    let mut response_headers = detail
        .get("response_headers")
        .cloned()
        .unwrap_or_else(|| empty_arr.clone());
    let mut response_results = detail
        .get("response_results")
        .cloned()
        .unwrap_or_else(|| empty_arr.clone());
    let mut response_examples = detail
        .get("response_examples")
        .cloned()
        .unwrap_or(empty_arr);

    sanitize_param_list(&mut request_headers);
    sanitize_param_list(&mut query_params);
    sanitize_request_body(&mut request_body);
    sanitize_param_list(&mut response_headers);
    sanitize_response_results(&mut response_results);
    sanitize_response_examples(&mut response_examples);

    let mut body = json!({
        "workspace_id": workspace_id,
        "project_id": project_id,
        "folder_id": folder_id,
        "name": clip(name, 100),
        "method": clip(method, 10),
        "protocol": clip(protocol, 10),
        "url": clip(url, 1024),
        "status": status,
        "request_headers": request_headers,
        "query_params": query_params,
        "request_body": request_body,
        "response_headers": response_headers,
        "response_results": response_results,
        "response_examples": response_examples,
    });
    if let Some(iid) = interface_id {
        if let Some(obj) = body.as_object_mut() {
            obj.insert("interface_id".into(), json!(iid));
        }
    }
    body
}

fn remap_workspace_id(
    conn: &Connection,
    server_id: i64,
    old_id: i64,
    new_id: i64,
    synced_at: &str,
) -> AppResult<()> {
    if old_id == new_id {
        return Ok(());
    }
    let conflict: i64 = conn.query_row(
        "SELECT COUNT(*) FROM workspaces WHERE server_id = ?1 AND id = ?2",
        params![server_id, new_id],
        |r| r.get(0),
    )?;
    if conflict > 0 {
        return Err(AppError::msg(format!(
            "Workspace id conflict {old_id} → {new_id}; sync download first"
        )));
    }
    conn.execute(
        "UPDATE projects SET workspace_id = ?1 WHERE server_id = ?2 AND workspace_id = ?3",
        params![new_id, server_id, old_id],
    )?;
    conn.execute(
        "UPDATE folders SET workspace_id = ?1 WHERE server_id = ?2 AND workspace_id = ?3",
        params![new_id, server_id, old_id],
    )?;
    conn.execute(
        "UPDATE interfaces SET workspace_id = ?1 WHERE server_id = ?2 AND workspace_id = ?3",
        params![new_id, server_id, old_id],
    )?;
    conn.execute(
        "UPDATE environments SET workspace_id = ?1 WHERE server_id = ?2 AND workspace_id = ?3",
        params![new_id, server_id, old_id],
    )?;
    conn.execute(
        "UPDATE workspaces SET id = ?1, is_local = 0, synced_at = ?2
         WHERE server_id = ?3 AND id = ?4",
        params![new_id, synced_at, server_id, old_id],
    )?;
    Ok(())
}

fn remap_project_id(
    conn: &Connection,
    server_id: i64,
    old_id: i64,
    new_id: i64,
    workspace_id: i64,
    synced_at: &str,
) -> AppResult<()> {
    if old_id == new_id {
        return Ok(());
    }
    let conflict: i64 = conn.query_row(
        "SELECT COUNT(*) FROM projects WHERE server_id = ?1 AND id = ?2",
        params![server_id, new_id],
        |r| r.get(0),
    )?;
    if conflict > 0 {
        return Err(AppError::msg(format!(
            "Project id conflict {old_id} → {new_id}; sync download first"
        )));
    }
    conn.execute(
        "UPDATE folders SET project_id = ?1, workspace_id = ?2
         WHERE server_id = ?3 AND project_id = ?4",
        params![new_id, workspace_id, server_id, old_id],
    )?;
    conn.execute(
        "UPDATE interfaces SET project_id = ?1, workspace_id = ?2
         WHERE server_id = ?3 AND project_id = ?4",
        params![new_id, workspace_id, server_id, old_id],
    )?;
    conn.execute(
        "UPDATE environments SET project_id = ?1, workspace_id = ?2
         WHERE server_id = ?3 AND project_id = ?4",
        params![new_id, workspace_id, server_id, old_id],
    )?;
    conn.execute(
        "UPDATE projects SET id = ?1, workspace_id = ?2, is_local = 0, synced_at = ?3
         WHERE server_id = ?4 AND id = ?5",
        params![new_id, workspace_id, synced_at, server_id, old_id],
    )?;
    Ok(())
}

fn remap_folder_id(
    conn: &Connection,
    server_id: i64,
    old_id: i64,
    new_id: i64,
    workspace_id: i64,
    project_id: i64,
    parent_id: i64,
    synced_at: &str,
) -> AppResult<()> {
    if old_id == new_id {
        return Ok(());
    }
    let conflict: i64 = conn.query_row(
        "SELECT COUNT(*) FROM folders WHERE server_id = ?1 AND id = ?2",
        params![server_id, new_id],
        |r| r.get(0),
    )?;
    if conflict > 0 {
        return Err(AppError::msg(format!(
            "Folder id conflict {old_id} → {new_id}; sync download first"
        )));
    }
    conn.execute(
        "UPDATE folders SET parent_id = ?1 WHERE server_id = ?2 AND parent_id = ?3",
        params![new_id, server_id, old_id],
    )?;
    conn.execute(
        "UPDATE interfaces SET folder_id = ?1 WHERE server_id = ?2 AND folder_id = ?3",
        params![new_id, server_id, old_id],
    )?;
    conn.execute(
        "UPDATE folders SET id = ?1, workspace_id = ?2, project_id = ?3, parent_id = ?4,
           is_local = 0, synced_at = ?5
         WHERE server_id = ?6 AND id = ?7",
        params![
            new_id,
            workspace_id,
            project_id,
            parent_id,
            synced_at,
            server_id,
            old_id
        ],
    )?;
    Ok(())
}

fn remap_interface_id(
    conn: &Connection,
    server_id: i64,
    old_id: i64,
    new_id: i64,
    workspace_id: i64,
    project_id: i64,
    folder_id: i64,
    synced_at: &str,
) -> AppResult<()> {
    if old_id == new_id {
        return Ok(());
    }
    let conflict: i64 = conn.query_row(
        "SELECT COUNT(*) FROM interfaces WHERE server_id = ?1 AND id = ?2",
        params![server_id, new_id],
        |r| r.get(0),
    )?;
    if conflict > 0 {
        return Err(AppError::msg(format!(
            "Interface id conflict {old_id} → {new_id}; sync download first"
        )));
    }
    conn.execute(
        "UPDATE interface_details SET interface_id = ?1
         WHERE server_id = ?2 AND interface_id = ?3",
        params![new_id, server_id, old_id],
    )?;
    conn.execute(
        "UPDATE grpc_rpcs SET interface_id = ?1
         WHERE server_id = ?2 AND interface_id = ?3",
        params![new_id, server_id, old_id],
    )?;
    conn.execute(
        "UPDATE interfaces SET id = ?1, workspace_id = ?2, project_id = ?3, folder_id = ?4,
           is_local = 0, synced_at = ?5
         WHERE server_id = ?6 AND id = ?7",
        params![
            new_id,
            workspace_id,
            project_id,
            folder_id,
            synced_at,
            server_id,
            old_id
        ],
    )?;
    Ok(())
}

fn clear_workspace_cache(conn: &Connection, server_id: i64, workspace_id: i64) -> AppResult<()> {
    conn.execute(
        "DELETE FROM environment_variables
         WHERE server_id = ?1 AND IFNULL(is_local, 0) = 0 AND environment_id IN (
           SELECT id FROM environments WHERE server_id = ?1 AND workspace_id = ?2
         )",
        params![server_id, workspace_id],
    )?;
    conn.execute(
        "DELETE FROM environments
         WHERE server_id = ?1 AND workspace_id = ?2 AND IFNULL(is_local, 0) = 0",
        params![server_id, workspace_id],
    )?;
    conn.execute(
        "DELETE FROM grpc_rpcs WHERE server_id = ?1 AND interface_id IN (
           SELECT id FROM interfaces
           WHERE server_id = ?1 AND workspace_id = ?2 AND IFNULL(is_local, 0) = 0
         )",
        params![server_id, workspace_id],
    )?;
    conn.execute(
        "DELETE FROM interface_details WHERE server_id = ?1 AND interface_id IN (
           SELECT id FROM interfaces
           WHERE server_id = ?1 AND workspace_id = ?2 AND IFNULL(is_local, 0) = 0
         )",
        params![server_id, workspace_id],
    )?;
    conn.execute(
        "DELETE FROM interfaces WHERE server_id = ?1 AND workspace_id = ?2 AND IFNULL(is_local, 0) = 0",
        params![server_id, workspace_id],
    )?;
    conn.execute(
        "DELETE FROM folders WHERE server_id = ?1 AND workspace_id = ?2 AND IFNULL(is_local, 0) = 0",
        params![server_id, workspace_id],
    )?;
    conn.execute(
        "DELETE FROM projects WHERE server_id = ?1 AND workspace_id = ?2 AND IFNULL(is_local, 0) = 0",
        params![server_id, workspace_id],
    )?;
    conn.execute(
        "DELETE FROM workspaces WHERE server_id = ?1 AND id = ?2 AND IFNULL(is_local, 0) = 0",
        params![server_id, workspace_id],
    )?;
    Ok(())
}

fn count_folders(nodes: &[TreeNode]) -> usize {
    let mut n = 0;
    for node in nodes {
        if node.node_type == "folder" {
            n += 1;
            if let Some(children) = &node.children {
                n += count_folders(children);
            }
        }
    }
    n
}

fn walk_folders(
    conn: &Connection,
    server_id: i64,
    workspace_id: i64,
    project_id: i64,
    parent_id: i64,
    nodes: &[TreeNode],
    synced_at: &str,
    sort_counter: &mut i64,
) -> AppResult<()> {
    for node in nodes {
        if node.node_type != "folder" {
            continue;
        }
        let Some(folder_id) = parse_folder_id(&node.id) else {
            continue;
        };
        let order = *sort_counter;
        *sort_counter += 1;
        conn.execute(
            "INSERT INTO folders(
               id, server_id, workspace_id, project_id, parent_id, name, sort_order, synced_at
             ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(server_id, id) DO UPDATE SET
               workspace_id = excluded.workspace_id,
               project_id = excluded.project_id,
               parent_id = excluded.parent_id,
               name = excluded.name,
               sort_order = excluded.sort_order,
               synced_at = excluded.synced_at",
            params![
                folder_id,
                server_id,
                workspace_id,
                project_id,
                parent_id,
                node.name,
                order,
                synced_at
            ],
        )?;
        if let Some(children) = &node.children {
            walk_folders(
                conn,
                server_id,
                workspace_id,
                project_id,
                folder_id,
                children,
                synced_at,
                sort_counter,
            )?;
        }
    }
    Ok(())
}

/// Clear locally synced mirror data only — does NOT delete server data.
#[tauri::command]
pub fn clear_local_cache(db: State<'_, Db>) -> AppResult<()> {
    let server_id = current_server_id(&db).unwrap_or(0);
    db.with_conn(|conn| {
        if server_id > 0 {
            conn.execute("DELETE FROM environment_variables WHERE server_id = ?1", params![server_id])?;
            conn.execute("DELETE FROM environments WHERE server_id = ?1", params![server_id])?;
            conn.execute("DELETE FROM grpc_rpcs WHERE server_id = ?1", params![server_id])?;
            conn.execute("DELETE FROM interface_details WHERE server_id = ?1", params![server_id])?;
            conn.execute("DELETE FROM interfaces WHERE server_id = ?1", params![server_id])?;
            conn.execute("DELETE FROM folders WHERE server_id = ?1", params![server_id])?;
            conn.execute("DELETE FROM projects WHERE server_id = ?1", params![server_id])?;
            conn.execute("DELETE FROM workspaces WHERE server_id = ?1", params![server_id])?;
            conn.execute("DELETE FROM sync_state WHERE server_id = ?1", params![server_id])?;
        } else {
            conn.execute_batch(
                "DELETE FROM environment_variables;
                 DELETE FROM environments;
                 DELETE FROM grpc_rpcs;
                 DELETE FROM interface_details;
                 DELETE FROM interfaces;
                 DELETE FROM folders;
                 DELETE FROM projects;
                 DELETE FROM workspaces;
                 DELETE FROM sync_state;",
            )?;
        }
        Ok(())
    })
}
