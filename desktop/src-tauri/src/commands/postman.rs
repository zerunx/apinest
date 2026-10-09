use serde_json::{json, Value};

use crate::error::{AppError, AppResult};

/// Convert a Postman Collection v2.0/v2.1 document into an ApiNest export document.
pub fn convert_postman_collection(document: &Value) -> AppResult<Value> {
    let info = document.get("info").cloned().unwrap_or(Value::Null);
    let schema = info
        .get("schema")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let collection_name = info
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let items = document
        .get("item")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let looks_like_postman = schema.contains("postman")
        || schema.contains("collection")
        || (!collection_name.is_empty() && !items.is_empty());
    if !looks_like_postman {
        return Err(AppError::msg("不是 Postman Collection 文件"));
    }
    if items.is_empty() {
        return Err(AppError::msg("Postman Collection 没有可导入的内容"));
    }

    let children = convert_items(&items);
    let tree = if collection_name.is_empty() {
        children
    } else {
        vec![json!({
            "type": "folder",
            "name": collection_name,
            "children": children,
        })]
    };

    Ok(json!({
        "format": "apinest",
        "version": 1,
        "exported_at": crate::db::now_iso(),
        "scope": "project",
        "project": { "name": collection_name },
        "tree": tree,
    }))
}

fn convert_items(items: &[Value]) -> Vec<Value> {
    let mut nodes = Vec::with_capacity(items.len());
    for item in items {
        if item.get("request").is_some_and(|r| !r.is_null()) {
            nodes.push(convert_request(item));
            continue;
        }
        let children = item
            .get("item")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let name = fallback_name(
            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
            "未命名文件夹",
        );
        if !children.is_empty() || !name.is_empty() {
            nodes.push(json!({
                "type": "folder",
                "name": name,
                "children": convert_items(&children),
            }));
        }
    }
    nodes
}

fn convert_request(item: &Value) -> Value {
    let request = item.get("request").cloned().unwrap_or(Value::Null);
    let (method, url, query, headers, body) = if request.is_string() {
        (
            "GET".to_string(),
            request.as_str().unwrap_or("").to_string(),
            Vec::new(),
            Vec::new(),
            default_body(),
        )
    } else {
        let method = request
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .trim()
            .to_uppercase();
        let (url, query) = parse_url(request.get("url"));
        let headers = convert_params(request.get("header").and_then(|v| v.as_array()), false);
        let body = convert_body(request.get("body"));
        (method, url, query, headers, body)
    };

    json!({
        "type": "api",
        "name": fallback_name(
            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
            "未命名接口",
        ),
        "method": method,
        "url": url,
        "status": 3,
        "request_headers": headers,
        "request_body": body,
        "query_params": query,
        "response_headers": [],
        "response_results": [],
        "response_examples": convert_responses(item.get("response").and_then(|v| v.as_array())),
    })
}

fn parse_url(raw: Option<&Value>) -> (String, Vec<Value>) {
    let Some(raw) = raw else {
        return (String::new(), Vec::new());
    };
    if let Some(s) = raw.as_str() {
        return (s.to_string(), Vec::new());
    }
    let url = raw
        .get("raw")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| build_url(raw));
    let query = convert_params(raw.get("query").and_then(|v| v.as_array()), true);
    (url, query)
}

fn build_url(raw: &Value) -> String {
    let host = raw
        .get("host")
        .and_then(|v| v.as_array())
        .map(|parts| {
            parts
                .iter()
                .filter_map(|p| p.as_str())
                .collect::<Vec<_>>()
                .join(".")
        })
        .unwrap_or_default();
    let path = raw
        .get("path")
        .and_then(|v| v.as_array())
        .map(|parts| {
            parts
                .iter()
                .filter_map(|p| p.as_str())
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_default();
    if !host.is_empty() && !path.is_empty() {
        format!("{host}/{path}")
    } else if !host.is_empty() {
        host
    } else {
        path
    }
}

fn convert_params(items: Option<&Vec<Value>>, _allow_empty: bool) -> Vec<Value> {
    let Some(items) = items else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in items {
        let key = item
            .get("key")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if key.is_empty() {
            continue;
        }
        let disabled = item
            .get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let typ = item
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("text");
        let field_type = if typ.eq_ignore_ascii_case("file") {
            "file"
        } else {
            "string"
        };
        out.push(json!({
            "name": key,
            "type": field_type,
            "required": false,
            "enabled": !disabled,
            "description": description_text(item.get("description")),
            "example": item.get("value").and_then(|v| v.as_str()).unwrap_or(""),
        }));
    }
    out
}

fn convert_body(body: Option<&Value>) -> Value {
    let Some(body) = body else {
        return default_body();
    };
    let mode = body
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match mode.as_str() {
        "raw" => {
            let language = body
                .pointer("/options/raw/language")
                .and_then(|v| v.as_str())
                .unwrap_or("json");
            json!({
                "format": "raw",
                "data_type": map_raw_language(language),
                "raw": body.get("raw").and_then(|v| v.as_str()).unwrap_or(""),
                "fields": [],
            })
        }
        "urlencoded" => json!({
            "format": "form-data",
            "data_type": "Object",
            "raw": "",
            "fields": convert_body_fields(body.get("urlencoded").and_then(|v| v.as_array())),
        }),
        "formdata" => json!({
            "format": "form-data",
            "data_type": "Object",
            "raw": "",
            "fields": convert_body_fields(body.get("formdata").and_then(|v| v.as_array())),
        }),
        "graphql" => {
            let query = body
                .pointer("/graphql/query")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let variables = body
                .pointer("/graphql/variables")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            let raw = if variables.is_empty() {
                query.to_string()
            } else {
                format!("{query}\n\n# variables\n{variables}")
            };
            let raw = if raw.trim().is_empty() {
                body.get("raw")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            } else {
                raw
            };
            json!({
                "format": "raw",
                "data_type": "JSON",
                "raw": raw,
                "fields": [],
            })
        }
        _ => {
            let raw = body.get("raw").and_then(|v| v.as_str()).unwrap_or("");
            if raw.is_empty() {
                default_body()
            } else {
                json!({
                    "format": "raw",
                    "data_type": "Text",
                    "raw": raw,
                    "fields": [],
                })
            }
        }
    }
}

fn convert_body_fields(items: Option<&Vec<Value>>) -> Vec<Value> {
    let Some(items) = items else {
        return Vec::new();
    };
    let mut fields = Vec::new();
    for item in items {
        let key = item
            .get("key")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if key.is_empty() {
            continue;
        }
        let disabled = item
            .get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let typ = item
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("text");
        let field_type = if typ.eq_ignore_ascii_case("file") {
            "file"
        } else {
            "string"
        };
        fields.push(json!({
            "parent_id": 0,
            "name": key,
            "type": field_type,
            "required": false,
            "enabled": !disabled,
            "description": description_text(item.get("description")),
            "example": item.get("value").and_then(|v| v.as_str()).unwrap_or(""),
            "children": [],
        }));
    }
    fields
}

fn convert_responses(items: Option<&Vec<Value>>) -> Vec<Value> {
    let Some(items) = items else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (index, resp) in items.iter().enumerate() {
        let name = resp
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("示例 {}", index + 1));
        let status_code = resp.get("code").and_then(|v| v.as_i64()).unwrap_or(200);
        let mut content_type = content_type_from_headers(resp.get("header").and_then(|v| v.as_array()));
        if content_type.is_empty() {
            content_type = content_type_from_preview(
                resp.get("_postman_previewlanguage")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
            );
        }
        out.push(json!({
            "name": name,
            "status_code": status_code,
            "content_type": content_type,
            "raw": resp.get("body").and_then(|v| v.as_str()).unwrap_or(""),
        }));
    }
    out
}

fn default_body() -> Value {
    json!({
        "format": "json",
        "data_type": "Object",
        "raw": "",
        "fields": [],
    })
}

fn map_raw_language(language: &str) -> &'static str {
    match language.trim().to_ascii_lowercase().as_str() {
        "json" | "javascript" | "js" => "JSON",
        "xml" => "XML",
        "html" => "HTML",
        "text" | "plain" => "Text",
        "" => "JSON",
        _ => "Text",
    }
}

fn content_type_from_headers(headers: Option<&Vec<Value>>) -> String {
    let Some(headers) = headers else {
        return String::new();
    };
    for item in headers {
        let key = item.get("key").and_then(|v| v.as_str()).unwrap_or("");
        if key.eq_ignore_ascii_case("content-type") {
            return item
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
        }
    }
    String::new()
}

fn content_type_from_preview(lang: &str) -> String {
    match lang.trim().to_ascii_lowercase().as_str() {
        "json" => "application/json".into(),
        "xml" => "application/xml".into(),
        "html" => "text/html".into(),
        "text" => "text/plain".into(),
        _ => "application/json".into(),
    }
}

fn description_text(raw: Option<&Value>) -> String {
    let Some(raw) = raw else {
        return String::new();
    };
    if let Some(s) = raw.as_str() {
        return s.to_string();
    }
    raw.get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn fallback_name(name: &str, fallback: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}
