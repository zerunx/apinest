use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{HeaderName, HeaderValue};
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;

use crate::db::{get_setting, Db};
use crate::error::{AppError, AppResult};

use super::http_send::{apply_vars, HeaderPair, CONNECT_TIMEOUT_SECS, DEFAULT_TIMEOUT_SECS};

/// Protocol-level ping interval so idle connections are not reset by proxies/NAT.
const WS_PING_INTERVAL: Duration = Duration::from_secs(30);

static SESSION_SEQ: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WsEventKind {
    Connected,
    Sent,
    Message,
    Error,
    Closed,
}

#[derive(Debug, Clone, Serialize)]
pub struct WsSessionEvent {
    pub session_id: String,
    pub kind: WsEventKind,
    pub direction: String,
    pub data: String,
    pub message_type: String,
    pub at_ms: u128,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WsConnectArgs {
    pub url: String,
    pub headers: Vec<HeaderPair>,
    pub variables: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct WsConnectResult {
    pub session_id: String,
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<HeaderPair>,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WsSendArgs {
    pub session_id: String,
    pub body: String,
    pub variables: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WsDisconnectArgs {
    pub session_id: String,
}

enum WsCmd {
    SendText(String),
    Close,
}

struct WsSessionHandle {
    session_id: String,
    cmd_tx: mpsc::UnboundedSender<WsCmd>,
}

pub struct WsHub {
    current: Mutex<Option<WsSessionHandle>>,
}

impl WsHub {
    pub fn new() -> Self {
        Self {
            current: Mutex::new(None),
        }
    }

    fn take_if_match(&self, session_id: &str) -> Option<WsSessionHandle> {
        let mut guard = self.current.lock().unwrap_or_else(|e| e.into_inner());
        if guard.as_ref().map(|s| s.session_id.as_str()) == Some(session_id) {
            guard.take()
        } else {
            None
        }
    }

    fn clear_if_match(&self, session_id: &str) {
        let mut guard = self.current.lock().unwrap_or_else(|e| e.into_inner());
        if guard.as_ref().map(|s| s.session_id.as_str()) == Some(session_id) {
            *guard = None;
        }
    }

    fn send_cmd(&self, session_id: &str, cmd: WsCmd) -> AppResult<()> {
        let guard = self.current.lock().unwrap_or_else(|e| e.into_inner());
        let Some(session) = guard.as_ref() else {
            return Err(AppError::msg("websocket is not connected"));
        };
        if session.session_id != session_id {
            return Err(AppError::msg("websocket session mismatch"));
        }
        session
            .cmd_tx
            .send(cmd)
            .map_err(|_| AppError::msg("websocket connection is closed"))
    }
}

fn validate_ws_url(raw: &str) -> AppResult<Url> {
    let parsed = Url::parse(raw).map_err(|e| AppError::msg(format!("invalid URL: {e}")))?;
    let scheme = parsed.scheme();
    if scheme != "ws" && scheme != "wss" {
        return Err(AppError::msg("URL scheme must be ws or wss"));
    }
    let host = parsed.host_str().unwrap_or("").trim();
    if host.is_empty() {
        return Err(AppError::msg(
            "URL must include a host, e.g. wss://api.example.com/ws",
        ));
    }
    Ok(parsed)
}

fn message_payload(msg: Message) -> (String, &'static str) {
    match msg {
        Message::Text(text) => (text.to_string(), "text"),
        Message::Binary(bin) => (
            format!("(binary {} bytes) {}", bin.len(), base64_preview(&bin)),
            "binary",
        ),
        Message::Ping(payload) => (format!("(ping {} bytes)", payload.len()), "ping"),
        Message::Pong(payload) => (format!("(pong {} bytes)", payload.len()), "pong"),
        Message::Close(frame) => {
            let detail = frame
                .map(|f| format!("{} {}", f.code, f.reason))
                .unwrap_or_default();
            (
                format!("(close) {detail}").trim().to_string(),
                "close",
            )
        }
        Message::Frame(_) => ("(frame)".to_string(), "frame"),
    }
}

fn base64_preview(bytes: &[u8]) -> String {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    if encoded.len() <= 512 {
        encoded
    } else {
        format!("{}…", &encoded[..512])
    }
}

fn emit_event(app: &AppHandle, event: WsSessionEvent) {
    let _ = app.emit("ws-session-event", event);
}

fn now_ms(started: Instant) -> u128 {
    started.elapsed().as_millis()
}

fn format_ws_connect_error(err: tokio_tungstenite::tungstenite::Error) -> String {
    use tokio_tungstenite::tungstenite::Error as WsError;
    match err {
        WsError::Http(response) => {
            format!("Unexpected server response: {}", response.status().as_u16())
        }
        other => format!("websocket connect failed: {other}"),
    }
}

#[tauri::command]
pub async fn ws_connect(
    app: AppHandle,
    hub: State<'_, WsHub>,
    db: State<'_, Db>,
    args: WsConnectArgs,
) -> AppResult<WsConnectResult> {
    let timeout_secs = db.with_conn(|conn| {
        let secs = get_setting(conn, "request_timeout_secs")?
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(DEFAULT_TIMEOUT_SECS)
            .min(crate::commands::http_send::MAX_TIMEOUT_SECS);
        Ok(secs)
    })?;

    let vars = args.variables.clone().unwrap_or_default();
    let url = apply_vars(&args.url, &vars);
    let parsed = validate_ws_url(&url)?;
    let url = parsed.to_string();

    let mut request = url
        .as_str()
        .into_client_request()
        .map_err(|e| AppError::msg(format!("invalid websocket request: {e}")))?;
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
            .map_err(|e| AppError::msg(format!("invalid header name `{key}`: {e}")))?;
        let val = HeaderValue::from_str(&value)
            .map_err(|e| AppError::msg(format!("invalid header value for `{key}`: {e}")))?;
        request.headers_mut().append(name, val);
    }

    // Close any previous session before opening a new one.
    {
        let prev = {
            let mut guard = hub.current.lock().unwrap_or_else(|e| e.into_inner());
            guard.take()
        };
        if let Some(prev) = prev {
            let _ = prev.cmd_tx.send(WsCmd::Close);
        }
    }

    let connect_timeout = Duration::from_secs(if timeout_secs == 0 {
        CONNECT_TIMEOUT_SECS
    } else {
        CONNECT_TIMEOUT_SECS.min(timeout_secs).max(1)
    });
    let (ws_stream, response) = tokio::time::timeout(connect_timeout, connect_async(request))
        .await
        .map_err(|_| AppError::msg("websocket connect timed out"))?
        .map_err(|e| AppError::msg(format_ws_connect_error(e)))?;

    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Switching Protocols")
        .to_string();
    let resp_headers: Vec<HeaderPair> = response
        .headers()
        .iter()
        .map(|(k, v)| HeaderPair {
            key: k.to_string(),
            value: v.to_str().unwrap_or_default().to_string(),
            enabled: Some(true),
        })
        .collect();

    let session_id = format!("ws-{}", SESSION_SEQ.fetch_add(1, Ordering::Relaxed));
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<WsCmd>();
    {
        let mut guard = hub.current.lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(WsSessionHandle {
            session_id: session_id.clone(),
            cmd_tx,
        });
    }

    let (mut write, mut read) = ws_stream.split();
    let app_reader = app.clone();
    let session_for_task = session_id.clone();
    let connected_url = url.clone();

    tokio::spawn(async move {
        let started = Instant::now();
        emit_event(
            &app_reader,
            WsSessionEvent {
                session_id: session_for_task.clone(),
                kind: WsEventKind::Connected,
                direction: "system".into(),
                data: format!("Connected to {connected_url}"),
                message_type: "system".into(),
                at_ms: now_ms(started),
            },
        );

        let mut close_reason = String::new();
        let mut ping_tick = tokio::time::interval_at(
            tokio::time::Instant::now() + WS_PING_INTERVAL,
            WS_PING_INTERVAL,
        );
        ping_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(WsCmd::SendText(text)) => {
                            if let Err(e) = write.send(Message::Text(text.clone().into())).await {
                                close_reason = format!("send failed: {e}");
                                emit_event(
                                    &app_reader,
                                    WsSessionEvent {
                                        session_id: session_for_task.clone(),
                                        kind: WsEventKind::Error,
                                        direction: "system".into(),
                                        data: close_reason.clone(),
                                        message_type: "error".into(),
                                        at_ms: now_ms(started),
                                    },
                                );
                                break;
                            }
                            emit_event(
                                &app_reader,
                                WsSessionEvent {
                                    session_id: session_for_task.clone(),
                                    kind: WsEventKind::Sent,
                                    direction: "out".into(),
                                    data: text,
                                    message_type: "text".into(),
                                    at_ms: now_ms(started),
                                },
                            );
                        }
                        Some(WsCmd::Close) | None => {
                            close_reason = String::from("Disconnected");
                            let _ = write.close().await;
                            break;
                        }
                    }
                }
                _ = ping_tick.tick() => {
                    // Keepalive only — do not surface in the message list.
                    if let Err(e) = write.send(Message::Ping(Vec::new().into())).await {
                        close_reason = format!("connection error: {e}");
                        emit_event(
                            &app_reader,
                            WsSessionEvent {
                                session_id: session_for_task.clone(),
                                kind: WsEventKind::Error,
                                direction: "system".into(),
                                data: close_reason.clone(),
                                message_type: "error".into(),
                                at_ms: now_ms(started),
                            },
                        );
                        break;
                    }
                }
                incoming = read.next() => {
                    match incoming {
                        Some(Ok(msg)) => {
                            // Server ping must be answered or the peer may drop the socket.
                            if let Message::Ping(payload) = &msg {
                                if let Err(e) =
                                    write.send(Message::Pong(payload.clone())).await
                                {
                                    close_reason = format!("connection error: {e}");
                                    emit_event(
                                        &app_reader,
                                        WsSessionEvent {
                                            session_id: session_for_task.clone(),
                                            kind: WsEventKind::Error,
                                            direction: "system".into(),
                                            data: close_reason.clone(),
                                            message_type: "error".into(),
                                            at_ms: now_ms(started),
                                        },
                                    );
                                    break;
                                }
                            }
                            // Hide protocol keepalive frames from the UI.
                            if matches!(msg, Message::Ping(_) | Message::Pong(_)) {
                                continue;
                            }
                            let is_close = matches!(msg, Message::Close(_));
                            let (data, message_type) = message_payload(msg);
                            if is_close {
                                close_reason = if data.is_empty() {
                                    String::from("Disconnected")
                                } else {
                                    data
                                };
                                break;
                            }
                            emit_event(
                                &app_reader,
                                WsSessionEvent {
                                    session_id: session_for_task.clone(),
                                    kind: WsEventKind::Message,
                                    direction: "in".into(),
                                    data,
                                    message_type: message_type.into(),
                                    at_ms: now_ms(started),
                                },
                            );
                        }
                        Some(Err(e)) => {
                            close_reason = format!("connection error: {e}");
                            emit_event(
                                &app_reader,
                                WsSessionEvent {
                                    session_id: session_for_task.clone(),
                                    kind: WsEventKind::Error,
                                    direction: "system".into(),
                                    data: close_reason.clone(),
                                    message_type: "error".into(),
                                    at_ms: now_ms(started),
                                },
                            );
                            break;
                        }
                        None => {
                            close_reason = String::from("Connection closed by remote");
                            break;
                        }
                    }
                }
            }
        }

        if let Some(state) = app_reader.try_state::<WsHub>() {
            state.clear_if_match(&session_for_task);
        }
        if close_reason.is_empty() {
            close_reason = String::from("Disconnected");
        }
        emit_event(
            &app_reader,
            WsSessionEvent {
                session_id: session_for_task,
                kind: WsEventKind::Closed,
                direction: "system".into(),
                data: close_reason,
                message_type: "system".into(),
                at_ms: now_ms(started),
            },
        );
    });

    Ok(WsConnectResult {
        session_id,
        status,
        status_text,
        headers: resp_headers,
        url,
    })
}

#[tauri::command]
pub async fn ws_send(hub: State<'_, WsHub>, args: WsSendArgs) -> AppResult<()> {
    let vars = args.variables.unwrap_or_default();
    let body = apply_vars(&args.body, &vars);
    if body.is_empty() {
        return Err(AppError::msg("message body is empty"));
    }
    hub.send_cmd(&args.session_id, WsCmd::SendText(body))
}

#[tauri::command]
pub async fn ws_disconnect(hub: State<'_, WsHub>, args: WsDisconnectArgs) -> AppResult<()> {
    if let Some(session) = hub.take_if_match(&args.session_id) {
        let _ = session.cmd_tx.send(WsCmd::Close);
    }
    Ok(())
}
