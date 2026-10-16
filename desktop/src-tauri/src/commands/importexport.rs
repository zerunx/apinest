use rusqlite::{params, Connection};
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::commands::auth::{current_auth, current_server_id};
use crate::db::{next_local_id, now_iso, Db};
use crate::error::{AppError, AppResult};
use crate::grpc::store::{attach_grpc_rpcs, replace_grpc_rpcs_from_detail};
use crate::nest::{ImportApiResult, NestClient};

#[derive(Debug, Deserialize)]
pub struct ExportApisArgs {
    pub workspace_id: i64,
    pub project_id: i64,
    pub folder_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ImportApisArgs {
    pub workspace_id: i64,
    pub project_id: i64,
    pub source: String,
    pub document: Value,
}

#[derive(Debug, Deserialize)]
pub struct SaveJsonFileArgs {
    pub default_name: String,
    pub document: Value,
}

#[tauri::command]
pub fn export_apinest(db: State<'_, Db>, args: ExportApisArgs) -> AppResult<Value> {
    let server_id = current_server_id(&db)?;
    let is_local = is_local_project(&db, server_id, args.project_id)?;
    if !is_local {
        if let Ok((base_url, token)) = current_auth(&db) {
            let client = NestClient::with_timeout(&base_url, Some(token), 60)?;
            return client.export_apinest(args.workspace_id, args.project_id, args.folder_id);
        }
    }
    db.with_conn(|conn| export_local(conn, server_id, args.workspace_id, args.project_id, args.folder_id))
}

#[tauri::command]
pub async fn save_json_file(app: tauri::AppHandle, args: SaveJsonFileArgs) -> AppResult<bool> {
    let mut name = args.default_name.trim().to_string();
    if name.is_empty() {
        name = "apinest.json".into();
    }
    if !name.to_ascii_lowercase().ends_with(".json") {
        name.push_str(".json");
    }

    // Use the async dialog API instead of blocking_save_file().
    // The blocking variant freezes the UI while the native save dialog is open.
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_file_name(&name)
        .add_filter("JSON", &["json"])
        .save_file(move |file| {
            let _ = tx.send(file);
        });

    let Some(file) = rx.await.unwrap_or(None) else {
        return Ok(false);
    };

    let path = file
        .into_path()
        .map_err(|e| AppError::msg(format!("invalid path: {e}")))?;
    let content = serde_json::to_vec_pretty(&args.document)?;
    tauri::async_runtime::spawn_blocking(move || std::fs::write(&path, content))
        .await
        .map_err(|e| AppError::msg(format!("write task failed: {e}")))?
        .map_err(|e| AppError::msg(format!("failed to write file: {e}")))?;
    Ok(true)
}

#[tauri::command]
pub fn import_apinest(db: State<'_, Db>, args: ImportApisArgs) -> AppResult<ImportApiResult> {
    let source = args.source.trim().to_lowercase();
    match source.as_str() {
        "apinest" | "postman" | "openapi" | "apipost" | "eolink" | "apifox" | "yapi" => {}
        _ => return Err(AppError::msg("该导入格式暂未支持")),
    }
    let server_id = current_server_id(&db)?;
    let is_local = is_local_project(&db, server_id, args.project_id)?;
    if !is_local {
        if let Ok((base_url, token)) = current_auth(&db) {
            let client = NestClient::with_timeout(&base_url, Some(token), 180)?;
            return client.import_apinest(
                args.workspace_id,
                args.project_id,
                &source,
                &args.document,
            );
        }
    }

    let document = match source.as_str() {
        "postman" => super::postman::convert_postman_collection(&args.document)?,
        "openapi" => super::openapi::convert_openapi_document(&args.document)?,
        "apipost" => super::apipost::convert_apipost_document(&args.document)?,
        "eolink" => super::eolink::convert_eolink_document(&args.document)?,
        "apifox" => super::apifox::convert_apifox_document(&args.document)?,
        "yapi" => super::yapi::convert_yapi_document(&args.document)?,
        _ => args.document.clone(),
    };
    db.with_conn(|conn| {
        import_local(
            conn,
            server_id,
            args.workspace_id,
            args.project_id,
            &document,
        )
    })
}

fn is_local_project(db: &Db, server_id: i64, project_id: i64) -> AppResult<bool> {
    db.with_conn(|conn| {
        let flag: i64 = conn
            .query_row(
                "SELECT IFNULL(is_local, 0) FROM projects WHERE server_id = ?1 AND id = ?2",
                params![server_id, project_id],
                |r| r.get(0),
            )
            .unwrap_or(1);
        Ok(flag == 1)
    })
}

pub(crate) fn export_local(
    conn: &Connection,
    server_id: i64,
    workspace_id: i64,
    project_id: i64,
    folder_id: Option<i64>,
) -> AppResult<Value> {
    let project_name: String = conn
        .query_row(
            "SELECT name FROM projects WHERE server_id = ?1 AND id = ?2",
            params![server_id, project_id],
            |r| r.get(0),
        )
        .map_err(|_| AppError::msg("项目不存在"))?;

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
        "SELECT i.id, i.folder_id, i.name, i.method, i.url, IFNULL(i.status, 1), d.payload_json
         FROM interfaces i
         LEFT JOIN interface_details d
           ON d.server_id = i.server_id AND d.interface_id = i.id
         WHERE i.server_id = ?1 AND i.workspace_id = ?2 AND i.project_id = ?3
         ORDER BY i.sort_order, i.id",
    )?;
    let apis: Vec<(i64, i64, String, String, String, i64, Option<String>)> = api_stmt
        .query_map(params![server_id, workspace_id, project_id], |r| {
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

    let folder_id = folder_id.filter(|id| *id > 0);
    let (scope, tree) = if let Some(root_id) = folder_id {
        let name = folders
            .iter()
            .find(|(id, _, _)| *id == root_id)
            .map(|(_, _, n)| n.clone())
            .ok_or_else(|| AppError::msg("文件夹不存在"))?;
        let allowed = descendant_ids(&folders, root_id);
        let tree_folders: Vec<_> = folders
            .iter()
            .filter(|(id, _, _)| allowed.contains(id))
            .cloned()
            .collect();
        let tree_apis: Vec<_> = apis
            .iter()
            .filter(|(_, fid, _, _, _, _, _)| allowed.contains(fid))
            .cloned()
            .collect();
        let children = build_local_tree(conn, server_id, &tree_folders, &tree_apis, root_id);
        (
            "folder",
            json!([{
                "type": "folder",
                "name": name,
                "children": children,
            }]),
        )
    } else {
        (
            "project",
            Value::Array(build_local_tree(conn, server_id, &folders, &apis, 0)),
        )
    };

    Ok(json!({
        "format": "apinest",
        "version": 1,
        "exported_at": now_iso(),
        "scope": scope,
        "project": { "name": project_name },
        "tree": tree,
    }))
}

fn descendant_ids(folders: &[(i64, i64, String)], root_id: i64) -> Vec<i64> {
    let mut ids = vec![root_id];
    fn walk(folders: &[(i64, i64, String)], parent: i64, ids: &mut Vec<i64>) {
        for (id, pid, _) in folders {
            if *pid == parent {
                ids.push(*id);
                walk(folders, *id, ids);
            }
        }
    }
    walk(folders, root_id, &mut ids);
    ids
}

fn build_local_tree(
    conn: &Connection,
    server_id: i64,
    folders: &[(i64, i64, String)],
    apis: &[(i64, i64, String, String, String, i64, Option<String>)],
    parent_id: i64,
) -> Vec<Value> {
    let mut nodes = Vec::new();
    for (id, pid, name) in folders {
        if *pid != parent_id {
            continue;
        }
        nodes.push(json!({
            "type": "folder",
            "name": name,
            "children": build_local_tree(conn, server_id, folders, apis, *id),
        }));
    }
    for (id, fid, name, method, url, status, payload) in apis {
        if *fid != parent_id {
            continue;
        }
        nodes.push(local_api_node(
            conn, server_id, *id, name, method, url, *status, payload.as_deref(),
        ));
    }
    nodes
}

fn local_api_node(
    conn: &Connection,
    server_id: i64,
    interface_id: i64,
    name: &str,
    method: &str,
    url: &str,
    status: i64,
    payload: Option<&str>,
) -> Value {
    let mut detail = payload
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
        .unwrap_or_else(|| json!({}));
    attach_grpc_rpcs(conn, server_id, interface_id, &mut detail);
    let grpc = detail.get("grpc").cloned();
    let mut node = json!({
        "type": "api",
        "name": detail.get("name").and_then(|v| v.as_str()).unwrap_or(name),
        "method": detail.get("method").and_then(|v| v.as_str()).unwrap_or(method),
        "url": detail.get("url").and_then(|v| v.as_str()).unwrap_or(url),
        "status": detail.get("status").and_then(|v| v.as_u64()).unwrap_or(status as u64),
        "request_headers": detail.get("request_headers").cloned().unwrap_or_else(|| json!([])),
        "request_body": detail.get("request_body").cloned().unwrap_or_else(|| json!({
            "format": "json",
            "data_type": "Object",
            "raw": "",
            "fields": []
        })),
        "query_params": detail.get("query_params").cloned().unwrap_or_else(|| json!([])),
        "response_headers": detail.get("response_headers").cloned().unwrap_or_else(|| json!([])),
        "response_results": detail.get("response_results").cloned().unwrap_or_else(|| json!([])),
        "response_examples": detail.get("response_examples").cloned().unwrap_or_else(|| json!([])),
    });
    if let Some(grpc) = grpc {
        if let Some(obj) = node.as_object_mut() {
            obj.insert("grpc".into(), grpc);
        }
    }
    node
}

fn import_local(
    conn: &Connection,
    server_id: i64,
    workspace_id: i64,
    project_id: i64,
    document: &Value,
) -> AppResult<ImportApiResult> {
    let format = document
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if !format.eq_ignore_ascii_case("apinest") {
        return Err(AppError::msg("不是 ApiNest 导出文件"));
    }
    let version = document.get("version").and_then(|v| v.as_i64()).unwrap_or(0);
    if version != 1 {
        return Err(AppError::msg("不支持的导出版本"));
    }
    let tree = document
        .get("tree")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let tx = conn.unchecked_transaction()?;
    let mut result = ImportApiResult { folders: 0, apis: 0 };
    let mut folder_sort = next_folder_sort(&tx, server_id, project_id, 0)?;
    import_nodes(
        &tx,
        server_id,
        workspace_id,
        project_id,
        0,
        &tree,
        &mut folder_sort,
        &mut result,
    )?;
    tx.commit()?;
    Ok(result)
}

fn import_nodes(
    conn: &Connection,
    server_id: i64,
    workspace_id: i64,
    project_id: i64,
    parent_id: i64,
    nodes: &[Value],
    folder_sort: &mut i64,
    result: &mut ImportApiResult,
) -> AppResult<()> {
    let mut api_sort = next_api_sort(conn, server_id, project_id, parent_id)?;
    let mut root_apis = Vec::new();

    for node in nodes {
        let kind = node
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if kind == "folder" {
            let raw_name = node.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let name = unique_folder_name(conn, server_id, project_id, parent_id, raw_name)?;
            let id = next_local_id(conn)?;
            let now = now_iso();
            conn.execute(
                "INSERT INTO folders(id, server_id, workspace_id, project_id, parent_id, name, sort_order, synced_at, is_local)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1)",
                params![
                    id,
                    server_id,
                    workspace_id,
                    project_id,
                    parent_id,
                    name,
                    *folder_sort,
                    now
                ],
            )?;
            *folder_sort += 1;
            result.folders += 1;
            let children = node
                .get("children")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let mut child_sort = 0i64;
            import_nodes(
                conn,
                server_id,
                workspace_id,
                project_id,
                id,
                &children,
                &mut child_sort,
                result,
            )?;
        } else if kind == "api" {
            if parent_id == 0 {
                root_apis.push(node.clone());
            } else {
                insert_local_api(
                    conn,
                    server_id,
                    workspace_id,
                    project_id,
                    parent_id,
                    api_sort,
                    node,
                )?;
                api_sort += 1;
                result.apis += 1;
            }
        }
    }

    if root_apis.is_empty() {
        return Ok(());
    }

    let name = unique_folder_name(conn, server_id, project_id, 0, "未分组")?;
    let folder_id = next_local_id(conn)?;
    let now = now_iso();
    conn.execute(
        "INSERT INTO folders(id, server_id, workspace_id, project_id, parent_id, name, sort_order, synced_at, is_local)
         VALUES(?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, 1)",
        params![
            folder_id,
            server_id,
            workspace_id,
            project_id,
            name,
            *folder_sort,
            now
        ],
    )?;
    *folder_sort += 1;
    result.folders += 1;
    let mut child_sort = 0i64;
    for node in root_apis {
        insert_local_api(
            conn,
            server_id,
            workspace_id,
            project_id,
            folder_id,
            child_sort,
            &node,
        )?;
        child_sort += 1;
        result.apis += 1;
    }
    Ok(())
}

fn insert_local_api(
    conn: &Connection,
    server_id: i64,
    workspace_id: i64,
    project_id: i64,
    folder_id: i64,
    sort_order: i64,
    node: &Value,
) -> AppResult<()> {
    let name = node
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("未命名接口")
        .trim();
    let name = if name.is_empty() { "未命名接口" } else { name };
    let method = node
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET")
        .to_uppercase();
    let url = node.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let status = node.get("status").and_then(|v| v.as_i64()).unwrap_or(1);
    let id = next_local_id(conn)?;
    let now = now_iso();
    conn.execute(
        "INSERT INTO interfaces(id, server_id, workspace_id, project_id, folder_id, name, method, url, status, sort_order, synced_at, is_local)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1)",
        params![
            id,
            server_id,
            workspace_id,
            project_id,
            folder_id,
            name,
            method,
            url,
            status,
            sort_order,
            now
        ],
    )?;
    let mut detail = node.clone();
    if let Some(obj) = detail.as_object_mut() {
        obj.insert("id".into(), json!(id));
        obj.insert("project_id".into(), json!(project_id));
        obj.insert("folder_id".into(), json!(folder_id));
        obj.insert("name".into(), json!(name));
        obj.insert("method".into(), json!(method));
        obj.insert("url".into(), json!(url));
        obj.insert("status".into(), json!(status));
    }
    let detail_str = serde_json::to_string(&detail)?;
    conn.execute(
        "INSERT INTO interface_details(server_id, interface_id, payload_json, synced_at)
         VALUES(?1, ?2, ?3, ?4)",
        params![server_id, id, detail_str, now],
    )?;
    replace_grpc_rpcs_from_detail(conn, server_id, id, &detail)?;
    Ok(())
}

fn next_folder_sort(conn: &Connection, server_id: i64, project_id: i64, parent_id: i64) -> AppResult<i64> {
    let max: i64 = conn.query_row(
        "SELECT IFNULL(MAX(sort_order), -1) FROM folders
         WHERE server_id = ?1 AND project_id = ?2 AND parent_id = ?3",
        params![server_id, project_id, parent_id],
        |r| r.get(0),
    )?;
    Ok(max + 1)
}

fn next_api_sort(conn: &Connection, server_id: i64, project_id: i64, folder_id: i64) -> AppResult<i64> {
    let max: i64 = conn.query_row(
        "SELECT IFNULL(MAX(sort_order), -1) FROM interfaces
         WHERE server_id = ?1 AND project_id = ?2 AND folder_id = ?3",
        params![server_id, project_id, folder_id],
        |r| r.get(0),
    )?;
    Ok(max + 1)
}

fn unique_folder_name(
    conn: &Connection,
    server_id: i64,
    project_id: i64,
    parent_id: i64,
    raw: &str,
) -> AppResult<String> {
    let base = {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            "未命名文件夹".to_string()
        } else {
            trimmed.chars().take(100).collect()
        }
    };
    let mut candidate = base.clone();
    let mut i = 2i64;
    loop {
        let exists: i64 = conn.query_row(
            "SELECT COUNT(1) FROM folders
             WHERE server_id = ?1 AND project_id = ?2 AND parent_id = ?3 AND name = ?4",
            params![server_id, project_id, parent_id, candidate],
            |r| r.get(0),
        )?;
        if exists == 0 {
            return Ok(candidate);
        }
        candidate = format!("{base} ({i})");
        i += 1;
    }
}
