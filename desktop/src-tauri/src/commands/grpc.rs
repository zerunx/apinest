use std::collections::HashMap;

use rusqlite::params;
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::State;

use crate::commands::auth::current_server_id;
use crate::commands::catalog::InterfaceRow;
use crate::commands::http_send::{
    apply_vars, HeaderPair, SendResponse, CONNECT_TIMEOUT_SECS, DEFAULT_TIMEOUT_SECS,
};
use crate::db::{get_setting, next_local_id, now_iso, Db};
use crate::error::{AppError, AppResult};
use crate::grpc::invoke::{
    compile_descriptor_pool, encode_message_json, find_method, invoke_unary, parse_grpc_endpoint,
};
use crate::grpc::proto::{parse_proto, ParsedProto};
use crate::grpc::store::replace_grpc_rpcs;

#[derive(Debug, Deserialize)]
pub struct ParseProtoArgs {
    pub filename: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct ImportGrpcProtoArgs {
    pub workspace_id: i64,
    pub project_id: i64,
    pub folder_id: i64,
    pub filename: String,
    pub source: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InvokeGrpcArgs {
    pub endpoint: String,
    pub rpc_path: String,
    pub proto_source: String,
    pub proto_name: Option<String>,
    pub message_json: String,
    pub metadata: Vec<HeaderPair>,
    pub tls: Option<bool>,
    pub interface_id: Option<i64>,
    pub variables: Option<HashMap<String, String>>,
}

#[tauri::command]
pub fn parse_proto_source(args: ParseProtoArgs) -> AppResult<ParsedProto> {
    parse_proto(&args.filename, &args.source)
}

#[tauri::command]
pub fn import_grpc_proto(db: State<'_, Db>, args: ImportGrpcProtoArgs) -> AppResult<InterfaceRow> {
    let parsed = parse_proto(&args.filename, &args.source)?;
    let mut name = parsed
        .proto_name
        .trim()
        .trim_end_matches(".proto")
        .to_string();
    if name.is_empty() {
        name = "gRPC".to_string();
    }
    let url = args.url.unwrap_or_default().trim().to_string();
    let selected = parsed
        .services
        .first()
        .and_then(|s| s.methods.first())
        .map(|m| m.rpc_path.clone())
        .unwrap_or_default();
    let detail = grpc_detail(&parsed, &args.source, &selected);
    let detail_str = serde_json::to_string(&detail)?;
    let server_id = current_server_id(&db)?;

    db.with_conn(|conn| {
        let id = next_local_id(conn)?;
        let now = now_iso();
        conn.execute(
            "INSERT INTO interfaces(id, server_id, workspace_id, project_id, folder_id, name, method, url, status, sort_order, synced_at, is_local)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'GRPC', ?7, 1, 0, ?8, 1)",
            params![
                id,
                server_id,
                args.workspace_id,
                args.project_id,
                args.folder_id,
                name,
                url,
                now
            ],
        )?;
        conn.execute(
            "INSERT INTO interface_details(server_id, interface_id, payload_json, synced_at)
             VALUES(?1, ?2, ?3, ?4)",
            params![server_id, id, detail_str, now],
        )?;
        replace_grpc_rpcs(conn, server_id, id, &parsed, &Value::Array(vec![]))?;
        Ok(InterfaceRow {
            id,
            workspace_id: args.workspace_id,
            project_id: args.project_id,
            folder_id: args.folder_id,
            name,
            method: "GRPC".into(),
            url,
            detail: Some(detail),
        })
    })
}

#[tauri::command]
pub async fn invoke_grpc(db: State<'_, Db>, args: InvokeGrpcArgs) -> AppResult<SendResponse> {
    if args.proto_source.trim().is_empty() {
        return Err(AppError::msg("import a .proto before invoking"));
    }
    if args.rpc_path.trim().is_empty() {
        return Err(AppError::msg("select a gRPC method"));
    }

    let timeout_secs = db.with_conn(|conn| {
        let secs = get_setting(conn, "request_timeout_secs")?
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(DEFAULT_TIMEOUT_SECS)
            .min(crate::commands::http_send::MAX_TIMEOUT_SECS);
        Ok(secs)
    })?;

    let vars = args.variables.clone().unwrap_or_default();
    let endpoint = parse_grpc_endpoint(&apply_vars(&args.endpoint, &vars), args.tls.unwrap_or(false))?;
    let rpc_path = apply_vars(&args.rpc_path, &vars);
    let message_json = apply_vars(&args.message_json, &vars);

    let mut stored_metadata = Vec::new();
    let mut call_metadata = Vec::new();
    for h in &args.metadata {
        if h.enabled == Some(false) || h.key.trim().is_empty() {
            continue;
        }
        let key = apply_vars(&h.key, &vars);
        let value = apply_vars(&h.value, &vars);
        stored_metadata.push(HeaderPair {
            key: key.clone(),
            value: value.clone(),
            enabled: Some(true),
        });
        call_metadata.push((key, value));
    }

    let proto_name = args.proto_name.clone().unwrap_or_default();
    let pool = compile_descriptor_pool(&proto_name, &args.proto_source)?;
    let method = find_method(&pool, &rpc_path)?;
    let body = encode_message_json(&method, &message_json)?;
    // 0 means "no timeout" in settings; tonic still needs a Duration, so use a week.
    let effective_timeout = if timeout_secs == 0 {
        7 * 24 * 3600
    } else {
        timeout_secs.max(1)
    };
    let result = invoke_unary(
        &endpoint,
        &rpc_path,
        &method,
        body,
        &call_metadata,
        effective_timeout,
        CONNECT_TIMEOUT_SECS,
    )
    .await?;

    let resp_headers: Vec<HeaderPair> = result
        .headers
        .into_iter()
        .map(|(key, value)| HeaderPair {
            key,
            value,
            enabled: Some(true),
        })
        .collect();

    let request_json = serde_json::to_string(&json!({
        "method": "GRPC",
        "url": format!("{endpoint}/{rpc_path}"),
        "headers": stored_metadata,
        "body": message_json,
        "rpc_path": rpc_path,
    }))?;
    let response_json = serde_json::to_string(&json!({
        "status": result.status,
        "status_text": result.status_text,
        "headers": resp_headers,
        "body": result.body,
    }))?;
    let created_at = now_iso();
    let interface_id = args.interface_id;
    let history_url = format!("{endpoint}/{rpc_path}");
    let status_code = result.status as i64;
    let duration_ms = result.duration_ms;

    let history_id = db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO request_history(
               interface_id, method, url, request_json, status_code, response_json, duration_ms, created_at
             ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                interface_id,
                "GRPC",
                history_url,
                request_json,
                status_code,
                response_json,
                duration_ms as i64,
                created_at
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })?;

    Ok(SendResponse {
        status: result.status,
        status_text: result.status_text,
        headers: resp_headers,
        body: result.body,
        duration_ms: result.duration_ms,
        history_id,
    })
}

pub fn grpc_detail(parsed: &ParsedProto, source: &str, selected_rpc: &str) -> Value {
    json!({
        "protocol": "GRPC",
        "grpc": {
            "proto_name": parsed.proto_name,
            "proto_source": source,
            "package": parsed.package,
            "selected_rpc": selected_rpc,
            "services": parsed.services,
        }
    })
}
