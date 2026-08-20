use rusqlite::{params, Connection};
use serde_json::{json, Value};

use crate::error::AppResult;
use crate::grpc::proto::ParsedProto;

pub fn replace_grpc_rpcs(
    conn: &Connection,
    server_id: i64,
    interface_id: i64,
    parsed: &ParsedProto,
    drafts: &Value,
) -> AppResult<()> {
    conn.execute(
        "DELETE FROM grpc_rpcs WHERE server_id = ?1 AND interface_id = ?2",
        params![server_id, interface_id],
    )?;
    let mut sort = 0i64;
    for svc in &parsed.services {
        for method in &svc.methods {
            let draft = draft_for(drafts, &method.rpc_path);
            let message = draft
                .get("message")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(method.message.as_str());
            let metadata = draft
                .get("metadata")
                .cloned()
                .unwrap_or_else(|| json!([]));
            let metadata_str = serde_json::to_string(&metadata)?;
            conn.execute(
                "INSERT INTO grpc_rpcs(
                   server_id, interface_id, rpc_path, service_name, method_name,
                   request_type, response_type, client_streaming, server_streaming,
                   message_json, metadata_json, sort_order
                 ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                params![
                    server_id,
                    interface_id,
                    method.rpc_path,
                    method.service_name,
                    method.name,
                    method.request_type,
                    method.response_type,
                    if method.client_streaming { 1 } else { 0 },
                    if method.server_streaming { 1 } else { 0 },
                    message,
                    metadata_str,
                    sort,
                ],
            )?;
            sort += 1;
        }
    }
    Ok(())
}

pub fn replace_grpc_rpcs_from_detail(
    conn: &Connection,
    server_id: i64,
    interface_id: i64,
    detail: &Value,
) -> AppResult<()> {
    let Some(grpc) = detail.get("grpc") else {
        conn.execute(
            "DELETE FROM grpc_rpcs WHERE server_id = ?1 AND interface_id = ?2",
            params![server_id, interface_id],
        )?;
        return Ok(());
    };
    conn.execute(
        "DELETE FROM grpc_rpcs WHERE server_id = ?1 AND interface_id = ?2",
        params![server_id, interface_id],
    )?;
    let mut sort = 0i64;
    let services = grpc.get("services").and_then(|v| v.as_array());
    let Some(services) = services else {
        return Ok(());
    };
    for svc in services {
        let methods = svc.get("methods").and_then(|v| v.as_array());
        let Some(methods) = methods else {
            continue;
        };
        let service_name = svc
            .get("full_name")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        for method in methods {
            let rpc_path = method
                .get("rpc_path")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if rpc_path.is_empty() {
                continue;
            }
            let method_name = method
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let request_type = method
                .get("request_type")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let response_type = method
                .get("response_type")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let client_streaming = method
                .get("client_streaming")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let server_streaming = method
                .get("server_streaming")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let message = method
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("{}");
            let metadata = method
                .get("metadata")
                .cloned()
                .unwrap_or_else(|| json!([]));
            let metadata_str = serde_json::to_string(&metadata)?;
            conn.execute(
                "INSERT INTO grpc_rpcs(
                   server_id, interface_id, rpc_path, service_name, method_name,
                   request_type, response_type, client_streaming, server_streaming,
                   message_json, metadata_json, sort_order
                 ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                params![
                    server_id,
                    interface_id,
                    rpc_path,
                    service_name,
                    method_name,
                    request_type,
                    response_type,
                    if client_streaming { 1 } else { 0 },
                    if server_streaming { 1 } else { 0 },
                    message,
                    metadata_str,
                    sort,
                ],
            )?;
            sort += 1;
        }
    }
    Ok(())
}

pub fn attach_grpc_rpcs(conn: &Connection, server_id: i64, interface_id: i64, detail: &mut Value) {
    let Ok(mut stmt) = conn.prepare(
        "SELECT rpc_path, service_name, method_name, request_type, response_type,
                client_streaming, server_streaming, message_json, metadata_json
         FROM grpc_rpcs
         WHERE server_id = ?1 AND interface_id = ?2
         ORDER BY sort_order, rpc_path",
    ) else {
        return;
    };
    let Ok(rows) = stmt.query_map(params![server_id, interface_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, i64>(5)?,
            r.get::<_, i64>(6)?,
            r.get::<_, String>(7)?,
            r.get::<_, String>(8)?,
        ))
    }) else {
        return;
    };

    let mut by_service: Vec<(String, Vec<Value>)> = Vec::new();
    for row in rows.flatten() {
        let (
            rpc_path,
            service_name,
            method_name,
            request_type,
            response_type,
            client_streaming,
            server_streaming,
            message_json,
            metadata_json,
        ) = row;
        let metadata: Value = serde_json::from_str(&metadata_json).unwrap_or_else(|_| json!([]));
        let method = json!({
            "name": method_name,
            "rpc_path": rpc_path,
            "service_name": service_name,
            "request_type": request_type,
            "response_type": response_type,
            "client_streaming": client_streaming == 1,
            "server_streaming": server_streaming == 1,
            "message": message_json,
            "metadata": metadata,
        });
        if let Some((_, methods)) = by_service.iter_mut().find(|(n, _)| n == &service_name) {
            methods.push(method);
        } else {
            by_service.push((service_name, vec![method]));
        }
    }
    if by_service.is_empty() {
        return;
    }

    let services: Vec<Value> = by_service
        .into_iter()
        .map(|(full_name, methods)| {
            let name = full_name.rsplit('.').next().unwrap_or(&full_name).to_string();
            json!({
                "name": name,
                "full_name": full_name,
                "methods": methods,
            })
        })
        .collect();

    if !detail.is_object() {
        *detail = json!({});
    }
    let grpc = detail.as_object_mut().unwrap().entry("grpc").or_insert(json!({}));
    if let Some(obj) = grpc.as_object_mut() {
        obj.insert("services".into(), Value::Array(services));
    }
}

pub fn delete_grpc_rpcs(conn: &Connection, server_id: i64, interface_id: i64) -> AppResult<()> {
    conn.execute(
        "DELETE FROM grpc_rpcs WHERE server_id = ?1 AND interface_id = ?2",
        params![server_id, interface_id],
    )?;
    Ok(())
}

pub fn delete_grpc_rpcs_for_folder(
    conn: &Connection,
    server_id: i64,
    folder_id: i64,
) -> AppResult<()> {
    conn.execute(
        "DELETE FROM grpc_rpcs WHERE server_id = ?1 AND interface_id IN (
           SELECT id FROM interfaces WHERE server_id = ?1 AND folder_id = ?2
         )",
        params![server_id, folder_id],
    )?;
    Ok(())
}

fn draft_for<'a>(drafts: &'a Value, rpc_path: &str) -> Value {
    if let Some(arr) = drafts.as_array() {
        for item in arr {
            if item.get("rpc_path").and_then(|v| v.as_str()) == Some(rpc_path) {
                return item.clone();
            }
        }
    }
    json!({})
}
