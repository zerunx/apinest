use rusqlite::{params, Connection};
use serde::Serialize;
use tauri::State;

use crate::commands::auth::{current_auth, current_server_id};
use crate::db::{now_iso, Db};
use crate::error::AppResult;
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
