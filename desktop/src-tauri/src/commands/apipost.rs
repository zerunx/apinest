use std::collections::HashMap;

use serde_json::{json, Map, Value};

use crate::error::{AppError, AppResult};

/// Convert ApiPost native JSON (project object or folder array) into ApiNest document.
/// Only folders/APIs are kept; envs/models/scripts are ignored.
pub fn convert_apipost_document(document: &Value) -> AppResult<Value> {
    let (nodes, project_name) = extract_nodes(document)?;
    if nodes.is_empty() {
        return Err(AppError::msg("ApiPost 文件没有可导入的文件夹或接口"));
    }

    let mut tree = convert_nodes(&nodes);
    if tree.is_empty() {
        tree = convert_flat(&nodes);
    }
    if tree.is_empty() {
        return Err(AppError::msg("ApiPost 文件没有可导入的接口"));
    }

    Ok(json!({
        "format": "apinest",
        "version": 1,
        "exported_at": crate::db::now_iso(),
        "scope": "project",
        "project": { "name": project_name },
        "tree": tree,
    }))
}

fn extract_nodes(document: &Value) -> AppResult<(Vec<Value>, String)> {
    if let Some(arr) = document.as_array() {
        if !looks_like_apipost(arr) {
            return Err(AppError::msg("不是 ApiPost 导出文件"));
        }
        return Ok((arr.clone(), String::new()));
    }

    if let Some(obj) = document.as_object() {
        if let Some(apis) = obj.get("apis").and_then(|v| v.as_array()) {
            if looks_like_apipost(apis) {
                let name = obj
                    .get("project")
                    .and_then(|v| v.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                return Ok((apis.clone(), name));
            }
        }
        let target_type = obj
            .get("target_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if !target_type.is_empty() {
            let name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            return Ok((vec![document.clone()], name));
        }
    }

    Err(AppError::msg("不是 ApiPost 导出文件"))
}

fn looks_like_apipost(nodes: &[Value]) -> bool {
    nodes.iter().any(|n| {
        let t = n
            .get("target_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        matches!(t.as_str(), "api" | "folder" | "dir" | "group")
            || n.get("children").and_then(|v| v.as_array()).is_some()
            || n.get("request").is_some()
    })
}

fn convert_nodes(nodes: &[Value]) -> Vec<Value> {
    let has_nested = nodes.iter().any(|n| {
        n.get("children")
            .and_then(|v| v.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false)
    });
    if !has_nested {
        let has_parent_links = nodes.iter().any(|n| {
            let parent = n
                .get("parent_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            !parent.is_empty() && parent != "0"
        });
        if has_parent_links {
            return Vec::new();
        }
    }

    let mut out = Vec::new();
    for n in nodes {
        if let Some(node) = convert_node(n) {
            out.push(node);
        }
    }
    out
}

fn convert_flat(nodes: &[Value]) -> Vec<Value> {
    let mut ids = std::collections::HashSet::new();
    for n in nodes {
        if let Some(id) = n.get("target_id").and_then(|v| v.as_str()) {
            if !id.trim().is_empty() {
                ids.insert(id.trim().to_string());
            }
        }
    }

    let mut by_parent: HashMap<String, Vec<Value>> = HashMap::new();
    for n in nodes {
        let mut parent = n
            .get("parent_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if parent.is_empty() || parent == "0" || parent == "1" || !ids.contains(&parent) {
            parent = "0".into();
        }
        by_parent.entry(parent).or_default().push(n.clone());
    }

    fn build(by_parent: &HashMap<String, Vec<Value>>, parent: &str) -> Vec<Value> {
        let Some(list) = by_parent.get(parent) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for n in list {
            let kind = n
                .get("target_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            match kind.as_str() {
                "folder" | "dir" | "group" => {
                    let id = n
                        .get("target_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    let mut children = build(by_parent, id);
                    if let Some(nested) = n.get("children").and_then(|v| v.as_array()) {
                        children.extend(convert_nodes(nested));
                    }
                    out.push(json!({
                        "type": "folder",
                        "name": fallback_name(n.get("name").and_then(|v| v.as_str()).unwrap_or(""), "未命名文件夹"),
                        "children": children,
                    }));
                }
                "api" | "" => {
                    if kind.is_empty()
                        && n.get("request").is_none()
                        && n.get("method").and_then(|v| v.as_str()).unwrap_or("").is_empty()
                    {
                        continue;
                    }
                    if let Some(api) = convert_api(n) {
                        out.push(api);
                    }
                }
                _ => {}
            }
        }
        out
    }

    build(&by_parent, "0")
}

fn convert_node(n: &Value) -> Option<Value> {
    let kind = n
        .get("target_type")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match kind.as_str() {
        "folder" | "dir" | "group" => {
            let children = n
                .get("children")
                .and_then(|v| v.as_array())
                .map(|arr| convert_nodes(arr))
                .unwrap_or_default();
            Some(json!({
                "type": "folder",
                "name": fallback_name(n.get("name").and_then(|v| v.as_str()).unwrap_or(""), "未命名文件夹"),
                "children": children,
            }))
        }
        "api" | "" => {
            if kind.is_empty()
                && n.get("request").is_none()
                && n.get("method").and_then(|v| v.as_str()).unwrap_or("").is_empty()
            {
                return None;
            }
            convert_api(n)
        }
        _ => None,
    }
}

fn convert_api(n: &Value) -> Option<Value> {
    let request = n.get("request").cloned().unwrap_or(Value::Null);
    let method = n
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET")
        .trim()
        .to_uppercase();
    let mut url = n
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if url.is_empty() {
        url = request
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
    }

    Some(json!({
        "type": "api",
        "name": fallback_name(n.get("name").and_then(|v| v.as_str()).unwrap_or(""), "未命名接口"),
        "method": if method.is_empty() { "GET".into() } else { method },
        "url": url,
        "status": 3,
        "request_headers": convert_params(request.pointer("/header/parameter")),
        "request_body": convert_body(request.get("body")),
        "query_params": convert_params(request.pointer("/query/parameter")),
        "response_headers": [],
        "response_results": [],
        "response_examples": convert_responses(n.get("response")),
    }))
}

fn convert_params(items: Option<&Value>) -> Vec<Value> {
    let Some(arr) = items.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in arr {
        let key = item
            .get("key")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if key.is_empty() {
            continue;
        }
        out.push(json!({
            "name": key,
            "type": field_type(item),
            "required": truthy(item.get("not_null")),
            "enabled": checked(item.get("is_checked")),
            "description": item.get("description").and_then(|v| v.as_str()).unwrap_or(""),
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
        "form-data" | "formdata" | "urlencoded" | "x-www-form-urlencoded" => {
            let mut fields = Vec::new();
            if let Some(arr) = body.get("parameter").and_then(|v| v.as_array()) {
                for item in arr {
                    let key = item
                        .get("key")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if key.is_empty() {
                        continue;
                    }
                    fields.push(json!({
                        "parent_id": 0,
                        "name": key,
                        "type": field_type(item),
                        "required": truthy(item.get("not_null")),
                        "enabled": checked(item.get("is_checked")),
                        "description": item.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                        "example": item.get("value").and_then(|v| v.as_str()).unwrap_or(""),
                        "children": [],
                    }));
                }
            }
            json!({
                "format": "form-data",
                "data_type": "Object",
                "raw": "",
                "fields": fields,
            })
        }
        "json" | "raw" | "javascript" | "xml" | "text" | "html" | "msgpack" => {
            let data_type = match mode.as_str() {
                "xml" => "XML",
                "html" => "HTML",
                "text" => "Text",
                _ => "JSON",
            };
            let mut raw = body
                .get("raw")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if raw.is_empty() {
                if let Some(arr) = body.get("parameter").and_then(|v| v.as_array()) {
                    let mut obj = Map::new();
                    for item in arr {
                        if let Some(key) = item.get("key").and_then(|v| v.as_str()) {
                            if !key.trim().is_empty() {
                                obj.insert(
                                    key.to_string(),
                                    json!(item.get("value").and_then(|v| v.as_str()).unwrap_or("")),
                                );
                            }
                        }
                    }
                    if !obj.is_empty() {
                        raw = serde_json::to_string_pretty(&Value::Object(obj)).unwrap_or_default();
                    }
                }
            }
            json!({
                "format": "raw",
                "data_type": data_type,
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
                    "data_type": "JSON",
                    "raw": raw,
                    "fields": [],
                })
            }
        }
    }
}

fn convert_responses(response: Option<&Value>) -> Vec<Value> {
    let Some(response) = response else {
        return Vec::new();
    };
    if let Some(arr) = response.as_array() {
        let mut out = Vec::new();
        for (i, item) in arr.iter().enumerate() {
            let mut name = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if name.is_empty() {
                name = format!("示例 {}", i + 1);
            }
            let mut code = item.get("code").and_then(|v| v.as_i64()).unwrap_or(200);
            let mut content_type = map_content_type(
                item.get("contentType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("json"),
            );
            let mut raw = item
                .get("raw")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if raw.is_empty() {
                raw = item
                    .get("mock")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
            }
            if let Some(expect) = item.get("expect") {
                if raw.is_empty() {
                    raw = expect
                        .get("mock")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                }
                if let Some(n) = expect.get("name").and_then(|v| v.as_str()) {
                    if !n.trim().is_empty() {
                        name = n.trim().to_string();
                    }
                }
                if let Some(c) = expect.get("code").and_then(|v| v.as_i64()) {
                    code = c;
                }
                if let Some(ct) = expect.get("contentType").and_then(|v| v.as_str()) {
                    content_type = map_content_type(ct);
                }
            }
            if raw.is_empty() {
                continue;
            }
            out.push(json!({
                "name": name,
                "status_code": code,
                "content_type": content_type,
                "raw": raw,
            }));
        }
        return out;
    }

    let Some(obj) = response.as_object() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (key, value) in obj {
        let expect = value.get("expect");
        let mut name = value
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            name = expect
                .and_then(|e| e.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
        }
        if name.is_empty() {
            name = match key.as_str() {
                "success" => "成功".into(),
                "error" => "失败".into(),
                other => other.to_string(),
            };
        }
        let code = expect
            .and_then(|e| e.get("code"))
            .and_then(|v| v.as_i64())
            .unwrap_or(200);
        let content_type = map_content_type(
            expect
                .and_then(|e| e.get("contentType"))
                .and_then(|v| v.as_str())
                .unwrap_or("json"),
        );
        let mut raw = value
            .get("raw")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if raw.is_empty() {
            raw = expect
                .and_then(|e| e.get("mock"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
        }
        if raw.is_empty() {
            continue;
        }
        out.push(json!({
            "name": name,
            "status_code": code,
            "content_type": content_type,
            "raw": raw,
        }));
    }
    out
}

fn map_content_type(ct: &str) -> String {
    match ct.trim().to_ascii_lowercase().as_str() {
        "json" | "application/json" => "application/json".into(),
        "xml" | "application/xml" => "application/xml".into(),
        "html" | "text/html" => "text/html".into(),
        "text" | "plain" | "text/plain" => "text/plain".into(),
        other => other.to_string(),
    }
}

fn field_type(item: &Value) -> &'static str {
    let t = item
        .get("field_type")
        .and_then(|v| v.as_str())
        .or_else(|| item.get("type").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_ascii_lowercase();
    match t.as_str() {
        "integer" | "number" | "int" | "float" | "double" => "number",
        "boolean" | "bool" => "boolean",
        "array" => "array",
        "object" => "object",
        "file" => "file",
        _ => "string",
    }
}

fn checked(v: Option<&Value>) -> bool {
    match v {
        None => true,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0) != 0.0,
        Some(Value::String(s)) => {
            let s = s.trim();
            !s.is_empty() && s != "0" && !s.eq_ignore_ascii_case("false")
        }
        _ => true,
    }
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0) != 0.0,
        Some(Value::String(s)) => {
            let s = s.trim();
            s == "1" || s.eq_ignore_ascii_case("true")
        }
        _ => false,
    }
}

fn default_body() -> Value {
    json!({
        "format": "json",
        "data_type": "Object",
        "raw": "",
        "fields": [],
    })
}

fn fallback_name(name: &str, fallback: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}
