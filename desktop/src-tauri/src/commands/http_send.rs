use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::Engine;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::Client;
use reqwest::Method;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::oneshot;
use url::Url;

use crate::db::{get_setting, now_iso, Db};
use crate::error::{AppError, AppResult};

pub const REQUEST_CANCELLED: &str = "request cancelled";

/// In-flight HTTP request cancel registry (request_id → oneshot).
pub struct HttpRequestHub {
    inner: Mutex<HashMap<String, oneshot::Sender<()>>>,
}

impl HttpRequestHub {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    fn register(&self, request_id: &str) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        if let Ok(mut map) = self.inner.lock() {
            if let Some(prev) = map.insert(request_id.to_string(), tx) {
                let _ = prev.send(());
            }
        }
        rx
    }

    fn unregister(&self, request_id: &str) {
        if let Ok(mut map) = self.inner.lock() {
            map.remove(request_id);
        }
    }

    fn cancel(&self, request_id: &str) -> bool {
        if let Ok(mut map) = self.inner.lock() {
            if let Some(tx) = map.remove(request_id) {
                let _ = tx.send(());
                return true;
            }
        }
        false
    }
}

pub(crate) const DEFAULT_TIMEOUT_SECS: u64 = 60;
pub(crate) const CONNECT_TIMEOUT_SECS: u64 = 10;
pub(crate) const MAX_TIMEOUT_SECS: u64 = 1800;

/// Clamp to 0..=1800. `0` means no request timeout.
pub(crate) fn clamp_timeout_secs(secs: u64) -> u64 {
    secs.min(MAX_TIMEOUT_SECS)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HttpVersionPref {
    /// ALPN negotiate (HTTP/2 when available, otherwise HTTP/1.1).
    Auto,
    Http11,
    Http2,
}

impl HttpVersionPref {
    pub(crate) fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "http1.1" | "http1" | "h1" | "1.1" | "1" => Self::Http11,
            "http2" | "h2" | "2" => Self::Http2,
            _ => Self::Auto,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Http11 => "http1.1",
            Self::Http2 => "http2",
        }
    }
}

fn error_chain(err: &dyn StdError) -> String {
    let mut out = err.to_string();
    let mut source = err.source();
    while let Some(inner) = source {
        out.push_str(": ");
        out.push_str(&inner.to_string());
        source = inner.source();
    }
    out
}

fn is_multipart_managed_header(name: &str) -> bool {
    name.eq_ignore_ascii_case("content-type")
        || name.eq_ignore_ascii_case("content-length")
        || name.eq_ignore_ascii_case("transfer-encoding")
}

#[derive(Debug, Deserialize, Clone)]
pub struct FormDataField {
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(rename = "type", default)]
    pub field_type: String,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub file_data: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SendRequestArgs {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderPair>,
    pub body: Option<String>,
    #[serde(default)]
    pub form_data: Option<Vec<FormDataField>>,
    pub interface_id: Option<i64>,
    /// Preferred folder for per-folder timeout / HTTP version; falls back to interface lookup.
    #[serde(default)]
    pub folder_id: Option<i64>,
    pub variables: Option<HashMap<String, String>>,
    /// Optional id so the frontend can cancel this in-flight request.
    #[serde(default)]
    pub request_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HeaderPair {
    pub key: String,
    pub value: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SendResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<HeaderPair>,
    pub body: String,
    pub duration_ms: u128,
    pub history_id: i64,
}

struct PreparedRequest {
    method: Method,
    url: String,
    stored_headers: Vec<HeaderPair>,
    /// Raw text body for history (non-multipart only).
    body_text: Option<String>,
    /// Exact bytes to send (raw UTF-8 or multipart).
    body_bytes: Option<Vec<u8>>,
    header_map: HeaderMap,
    timeout_secs: u64,
    http_version: HttpVersionPref,
}

pub(crate) fn apply_vars(input: &str, vars: &HashMap<String, String>) -> String {
    let mut out = input.to_string();
    for (k, v) in vars {
        let needle = format!("{{{{{k}}}}}");
        out = out.replace(&needle, v);
    }
    out
}

fn validate_request_url(raw: &str) -> AppResult<()> {
    let parsed = Url::parse(raw).map_err(|e| AppError::msg(format!("invalid URL: {e}")))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(AppError::msg("URL scheme must be http or https"));
    }
    let host = parsed.host_str().unwrap_or("").trim();
    if host.is_empty() {
        return Err(AppError::msg(
            "URL must include a host, e.g. https://api.example.com/path",
        ));
    }
    Ok(())
}

fn gen_multipart_boundary() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // Browser / Postman style boundary (no spaces).
    format!("----WebKitFormBoundary{nanos:x}")
}

fn escape_disposition_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "_")
        .replace('\n', "_")
}

/// Build multipart/form-data like browsers/Postman.
/// File fields are always appended last (required by OSS PostObject).
fn build_multipart_body(
    fields: &[FormDataField],
    vars: &HashMap<String, String>,
) -> AppResult<(String, Vec<u8>)> {
    let boundary = gen_multipart_boundary();
    let mut text_fields = Vec::new();
    let mut file_fields = Vec::new();

    for field in fields {
        let name = apply_vars(field.name.trim(), vars);
        if name.is_empty() {
            continue;
        }
        if field.field_type == "file" {
            file_fields.push((name, field));
        } else {
            text_fields.push((name, field));
        }
    }

    if text_fields.is_empty() && file_fields.is_empty() {
        return Err(AppError::msg("form-data request has no fields"));
    }

    let mut body = Vec::new();
    let write_boundary = |buf: &mut Vec<u8>| {
        buf.extend_from_slice(b"--");
        buf.extend_from_slice(boundary.as_bytes());
        buf.extend_from_slice(b"\r\n");
    };

    for (name, field) in text_fields {
        write_boundary(&mut body);
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"");
        body.extend_from_slice(escape_disposition_value(&name).as_bytes());
        body.extend_from_slice(b"\"\r\n\r\n");
        body.extend_from_slice(apply_vars(&field.value, vars).as_bytes());
        body.extend_from_slice(b"\r\n");
    }

    for (name, field) in file_fields {
        let encoded = field
            .file_data
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::msg(format!("missing file content for form field `{name}`")))?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| AppError::msg(format!("invalid file data for `{name}`: {e}")))?;
        let file_name = field
            .file_name
            .as_deref()
            .map(|s| apply_vars(s, vars))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "file".to_string());

        write_boundary(&mut body);
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"");
        body.extend_from_slice(escape_disposition_value(&name).as_bytes());
        body.extend_from_slice(b"\"; filename=\"");
        body.extend_from_slice(escape_disposition_value(&file_name).as_bytes());
        body.extend_from_slice(b"\"\r\n");
        body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
        body.extend_from_slice(&bytes);
        body.extend_from_slice(b"\r\n");
    }

    body.extend_from_slice(b"--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(b"--\r\n");

    let content_type = format!("multipart/form-data; boundary={boundary}");
    Ok((content_type, body))
}

fn prepare_request(
    args: &SendRequestArgs,
    timeout_secs: u64,
    http_version: HttpVersionPref,
) -> AppResult<PreparedRequest> {
    let vars = args.variables.clone().unwrap_or_default();
    let url = apply_vars(&args.url, &vars);
    validate_request_url(&url)?;

    let body_text = args.body.as_ref().map(|b| apply_vars(b, &vars));
    let multipart = match args.form_data.as_ref() {
        Some(fields) if !fields.is_empty() => Some(build_multipart_body(fields, &vars)?),
        _ => None,
    };
    if multipart.is_some() && body_text.is_some() {
        return Err(AppError::msg("request cannot include both body and form_data"));
    }

    let method = Method::from_bytes(args.method.to_uppercase().as_bytes())
        .map_err(|_| AppError::msg(format!("invalid method: {}", args.method)))?;

    let mut header_map = HeaderMap::new();
    let mut stored_headers = Vec::new();
    for h in &args.headers {
        if h.enabled == Some(false) || h.key.trim().is_empty() {
            continue;
        }
        let key = apply_vars(&h.key, &vars);
        let value = apply_vars(&h.value, &vars);
        if multipart.is_some() && is_multipart_managed_header(&key) {
            continue;
        }
        stored_headers.push(HeaderPair {
            key: key.clone(),
            value: value.clone(),
            enabled: Some(true),
        });
        let name = HeaderName::from_bytes(key.as_bytes())
            .map_err(|_| AppError::msg(format!("invalid header name: {key}")))?;
        let val = HeaderValue::from_str(&value)
            .map_err(|_| AppError::msg(format!("invalid header value for {key}")))?;
        header_map.insert(name, val);
    }

    let body_bytes = if let Some((content_type, bytes)) = multipart {
        header_map.remove(CONTENT_TYPE);
        header_map.remove(CONTENT_LENGTH);
        header_map.remove(HeaderName::from_static("transfer-encoding"));
        header_map.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(&content_type)
                .map_err(|_| AppError::msg("invalid multipart content-type"))?,
        );
        Some(bytes)
    } else {
        body_text.as_ref().map(|s| s.as_bytes().to_vec())
    };

    Ok(PreparedRequest {
        method,
        url,
        stored_headers,
        body_text,
        body_bytes,
        header_map,
        timeout_secs,
        http_version,
    })
}

fn map_reqwest_error(e: reqwest::Error, timeout_secs: u64) -> AppError {
    let detail = error_chain(&e);
    if e.is_timeout() {
        let timeout_label = if timeout_secs == 0 {
            "disabled".to_string()
        } else {
            format!("{timeout_secs}s")
        };
        AppError::msg(format!(
            "request timed out after {timeout_label} (connect limit {}s): {detail}",
            CONNECT_TIMEOUT_SECS
        ))
    } else if e.is_connect() {
        AppError::msg(format!("connection failed: {detail}"))
    } else if e.is_body() {
        AppError::msg(format!(
            "request failed: {detail} (peer closed the connection while the body was being sent)"
        ))
    } else {
        AppError::msg(format!("request failed: {detail}"))
    }
}

async fn execute_http_request(
    prepared: PreparedRequest,
) -> AppResult<(u16, String, Vec<HeaderPair>, String, u128)> {
    // native-tls (SChannel on Windows) matches Postman/browser TLS more closely than rustls.
    let mut builder = Client::builder()
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .redirect(reqwest::redirect::Policy::limited(10));
    // 0 = wait indefinitely (still bounded by connect_timeout for the TCP handshake).
    // Async ClientBuilder has no timeout(None); omit the call to disable.
    if prepared.timeout_secs > 0 {
        builder = builder.timeout(Duration::from_secs(prepared.timeout_secs));
    }
    builder = match prepared.http_version {
        HttpVersionPref::Http11 => builder.http1_only(),
        // Auto / HTTP/2: allow ALPN to pick HTTP/2 when the peer supports it.
        HttpVersionPref::Auto | HttpVersionPref::Http2 => builder,
    };
    let client = builder.build()?;

    let mut req = client
        .request(prepared.method, &prepared.url)
        .headers(prepared.header_map);
    if let Some(body) = prepared.body_bytes {
        req = req.body(body);
    }

    let started = Instant::now();
    let timeout_secs = prepared.timeout_secs;
    let resp = req
        .send()
        .await
        .map_err(|e| map_reqwest_error(e, timeout_secs))?;
    let duration_ms = started.elapsed().as_millis();
    let status = resp.status();
    let status_text = status.canonical_reason().unwrap_or("").to_string();
    let status_code = status.as_u16();

    let mut resp_headers = Vec::new();
    for (k, v) in resp.headers().iter() {
        resp_headers.push(HeaderPair {
            key: k.as_str().to_string(),
            value: v.to_str().unwrap_or("").to_string(),
            enabled: Some(true),
        });
    }
    let resp_body = resp.text().await.unwrap_or_default();

    Ok((status_code, status_text, resp_headers, resp_body, duration_ms))
}

fn sanitize_form_data_for_history(fields: &[FormDataField]) -> Vec<serde_json::Value> {
    fields
        .iter()
        .map(|field| {
            if field.field_type == "file" {
                serde_json::json!({
                    "name": field.name,
                    "type": "file",
                    "file_name": field.file_name,
                })
            } else {
                serde_json::json!({
                    "name": field.name,
                    "type": "string",
                    "value": field.value,
                })
            }
        })
        .collect()
}

/// Folder headers first; interface headers override same key (case-insensitive).
pub(crate) fn merge_headers(folder: &[HeaderPair], interface: &[HeaderPair]) -> Vec<HeaderPair> {
    let mut out: Vec<HeaderPair> = Vec::new();
    for h in folder {
        if h.enabled == Some(false) || h.key.trim().is_empty() {
            continue;
        }
        out.push(HeaderPair {
            key: h.key.trim().to_string(),
            value: h.value.clone(),
            enabled: Some(true),
        });
    }
    for h in interface {
        if h.enabled == Some(false) || h.key.trim().is_empty() {
            continue;
        }
        let key = h.key.trim().to_string();
        let next = HeaderPair {
            key: key.clone(),
            value: h.value.clone(),
            enabled: Some(true),
        };
        if let Some(pos) = out.iter().position(|x| x.key.eq_ignore_ascii_case(&key)) {
            out[pos] = next;
        } else {
            out.push(next);
        }
    }
    out
}

fn parse_folder_headers_json(raw: &str) -> Vec<HeaderPair> {
    serde_json::from_str::<Vec<HeaderPair>>(raw).unwrap_or_default()
}

fn resolve_request_options(
    conn: &rusqlite::Connection,
    folder_id: Option<i64>,
    interface_id: Option<i64>,
) -> AppResult<(u64, HttpVersionPref, Vec<HeaderPair>)> {
    let global_timeout = get_setting(conn, "request_timeout_secs")?
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    let global_timeout = clamp_timeout_secs(global_timeout);

    let server_id: Option<i64> = get_setting(conn, "active_server_id")?
        .and_then(|s| s.parse().ok());

    let global_http = HttpVersionPref::parse(
        &get_setting(conn, "http_version")?.unwrap_or_else(|| "auto".into()),
    );

    let mut folder_timeout: Option<u64> = None;
    let mut folder_http: Option<HttpVersionPref> = None;
    let mut folder_headers: Vec<HeaderPair> = Vec::new();

    let resolved_folder_id = if let Some(fid) = folder_id.filter(|id| *id != 0) {
        Some(fid)
    } else if let (Some(sid), Some(iid)) = (server_id, interface_id) {
        conn.query_row(
            "SELECT folder_id FROM interfaces WHERE server_id = ?1 AND id = ?2",
            params![sid, iid],
            |r| r.get::<_, i64>(0),
        )
        .ok()
        .filter(|id| *id != 0)
    } else {
        None
    };

    if let (Some(sid), Some(fid)) = (server_id, resolved_folder_id) {
        let row = conn.query_row(
            "SELECT request_timeout_secs, http_version, IFNULL(headers_json, '[]')
             FROM folders
             WHERE server_id = ?1 AND id = ?2",
            params![sid, fid],
            |r| {
                Ok((
                    r.get::<_, Option<i64>>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, String>(2)?,
                ))
            },
        );
        if let Ok((timeout, version, headers_json)) = row {
            if let Some(secs) = timeout {
                if secs >= 0 {
                    folder_timeout = Some(clamp_timeout_secs(secs as u64));
                }
            }
            if let Some(version) = version.filter(|v| !v.trim().is_empty()) {
                folder_http = Some(HttpVersionPref::parse(&version));
            }
            folder_headers = parse_folder_headers_json(&headers_json);
        }
    }

    Ok((
        folder_timeout.unwrap_or(global_timeout),
        folder_http.unwrap_or(global_http),
        folder_headers,
    ))
}

#[tauri::command]
pub fn cancel_http_request(hub: State<'_, HttpRequestHub>, request_id: String) -> AppResult<()> {
    let id = request_id.trim();
    if id.is_empty() {
        return Err(AppError::msg("request_id is required"));
    }
    let _ = hub.cancel(id);
    Ok(())
}

#[tauri::command]
pub async fn send_http_request(
    db: State<'_, Db>,
    hub: State<'_, HttpRequestHub>,
    args: SendRequestArgs,
) -> AppResult<SendResponse> {
    let (timeout_secs, http_version, folder_headers) = db.with_conn(|conn| {
        resolve_request_options(conn, args.folder_id, args.interface_id)
    })?;

    let mut args = args;
    args.headers = merge_headers(&folder_headers, &args.headers);

    let prepared = prepare_request(&args, timeout_secs, http_version)?;
    let method = args.method.clone();
    let interface_id = args.interface_id;
    let url = prepared.url.clone();
    let stored_headers = prepared.stored_headers.clone();
    let request_body = prepared.body_text.clone();
    let form_data = args.form_data.clone();
    let request_id = args
        .request_id
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let cancel_rx = request_id.as_ref().map(|id| hub.register(id));

    let send_result = if let Some(rx) = cancel_rx {
        tokio::select! {
            biased;
            _ = rx => Err(AppError::msg(REQUEST_CANCELLED)),
            res = execute_http_request(prepared) => res,
        }
    } else {
        execute_http_request(prepared).await
    };

    if let Some(id) = request_id.as_ref() {
        hub.unregister(id);
    }

    let (status_code, status_text, resp_headers, resp_body, duration_ms) = send_result?;

    let request_json = if let Some(ref fields) = form_data {
        serde_json::to_string(&serde_json::json!({
            "method": method,
            "url": url,
            "headers": stored_headers,
            "form_data": sanitize_form_data_for_history(fields),
        }))?
    } else {
        serde_json::to_string(&serde_json::json!({
            "method": method,
            "url": url,
            "headers": stored_headers,
            "body": request_body,
        }))?
    };
    let response_json = serde_json::to_string(&serde_json::json!({
        "status": status_code,
        "headers": resp_headers,
        "body": resp_body,
    }))?;
    let created_at = now_iso();

    let history_id = db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO request_history(
               interface_id, method, url, request_json, status_code, response_json, duration_ms, created_at
             ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                interface_id,
                method.to_uppercase(),
                url,
                request_json,
                status_code as i64,
                response_json,
                duration_ms as i64,
                created_at
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })?;

    Ok(SendResponse {
        status: status_code,
        status_text,
        headers: resp_headers,
        body: resp_body,
        duration_ms,
        history_id,
    })
}

#[derive(Debug, Serialize)]
pub struct HistoryItem {
    pub id: i64,
    pub interface_id: Option<i64>,
    pub method: String,
    pub url: String,
    pub status_code: Option<i64>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
}

#[tauri::command]
pub fn list_request_history(db: State<'_, Db>, limit: Option<i64>) -> AppResult<Vec<HistoryItem>> {
    let limit = limit.unwrap_or(50);
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, interface_id, method, url, status_code, duration_ms, created_at
             FROM request_history ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt
            .query_map(params![limit], |r| {
                Ok(HistoryItem {
                    id: r.get(0)?,
                    interface_id: r.get(1)?,
                    method: r.get(2)?,
                    url: r.get(3)?,
                    status_code: r.get(4)?,
                    duration_ms: r.get(5)?,
                    created_at: r.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}
