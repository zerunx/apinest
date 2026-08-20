use std::collections::HashMap;
use std::time::{Duration, Instant};

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Method;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;
use url::Url;

use crate::db::{get_setting, now_iso, Db};
use crate::error::{AppError, AppResult};

pub(crate) const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub(crate) const CONNECT_TIMEOUT_SECS: u64 = 10;

#[derive(Debug, Deserialize, Clone)]
pub struct SendRequestArgs {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderPair>,
    pub body: Option<String>,
    pub interface_id: Option<i64>,
    pub variables: Option<HashMap<String, String>>,
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
    body: Option<String>,
    header_map: HeaderMap,
    timeout_secs: u64,
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

fn prepare_request(args: &SendRequestArgs, timeout_secs: u64) -> AppResult<PreparedRequest> {
    let vars = args.variables.clone().unwrap_or_default();
    let url = apply_vars(&args.url, &vars);
    validate_request_url(&url)?;

    let body = args.body.as_ref().map(|b| apply_vars(b, &vars));
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

    Ok(PreparedRequest {
        method,
        url,
        stored_headers,
        body,
        header_map,
        timeout_secs,
    })
}

fn execute_http_request(prepared: PreparedRequest) -> AppResult<(u16, String, Vec<HeaderPair>, String, u128)> {
    let client = Client::builder()
        .timeout(Duration::from_secs(prepared.timeout_secs))
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()?;

    let mut req = client
        .request(prepared.method, &prepared.url)
        .headers(prepared.header_map);
    if let Some(body) = &prepared.body {
        req = req.body(body.clone());
    }

    let started = Instant::now();
    let resp = req.send().map_err(|e| {
        if e.is_timeout() {
            AppError::msg(format!(
                "request timed out after {}s (connect limit {}s)",
                prepared.timeout_secs, CONNECT_TIMEOUT_SECS
            ))
        } else if e.is_connect() {
            AppError::msg(format!("connection failed: {e}"))
        } else {
            AppError::msg(format!("request failed: {e}"))
        }
    })?;
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
    let resp_body = resp.text().unwrap_or_default();

    Ok((status_code, status_text, resp_headers, resp_body, duration_ms))
}

#[tauri::command]
pub async fn send_http_request(db: State<'_, Db>, args: SendRequestArgs) -> AppResult<SendResponse> {
    let timeout_secs = db.with_conn(|conn| {
        let secs = get_setting(conn, "request_timeout_secs")?
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(DEFAULT_TIMEOUT_SECS)
            .clamp(1, 600);
        Ok(secs)
    })?;

    let prepared = prepare_request(&args, timeout_secs)?;
    let method = args.method.clone();
    let interface_id = args.interface_id;
    let url = prepared.url.clone();
    let stored_headers = prepared.stored_headers.clone();
    let request_body = prepared.body.clone();

    let (status_code, status_text, resp_headers, resp_body, duration_ms) =
        tauri::async_runtime::spawn_blocking(move || execute_http_request(prepared))
            .await
            .map_err(|e| AppError::msg(format!("request task failed: {e}")))??
            ;

    let request_json = serde_json::to_string(&serde_json::json!({
        "method": method,
        "url": url,
        "headers": stored_headers,
        "body": request_body,
    }))?;
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
