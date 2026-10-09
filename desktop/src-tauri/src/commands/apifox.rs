use serde_json::{json, Map, Value};

use crate::error::{AppError, AppResult};

/// Convert an Apifox project export into ApiNest document.
/// Only apiCollection folders/APIs are imported.
pub fn convert_apifox_document(document: &Value) -> AppResult<Value> {
    if !is_apifox(document) {
        return Err(AppError::msg("不是 Apifox 导出文件"));
    }
    let collection = document
        .get("apiCollection")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if collection.is_empty() {
        return Err(AppError::msg("Apifox 文件没有接口集合"));
    }

    let mut children = convert_items(&collection);
    if children.is_empty() {
        return Err(AppError::msg("Apifox 文件没有可导入的接口"));
    }

    // Skip meaningless single "根目录" wrapper.
    if children.len() == 1 {
        if let Some(obj) = children[0].as_object() {
            if obj.get("type").and_then(|v| v.as_str()) == Some("folder")
                && obj.get("name").and_then(|v| v.as_str()) == Some("根目录")
            {
                if let Some(inner) = obj.get("children").and_then(|v| v.as_array()) {
                    if !inner.is_empty() {
                        children = inner.clone();
                    }
                }
            }
        }
    }

    let name = document
        .pointer("/info/name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let tree = if name.is_empty() {
        children
    } else {
        vec![json!({
            "type": "folder",
            "name": name,
            "children": children,
        })]
    };

    Ok(json!({
        "format": "apinest",
        "version": 1,
        "exported_at": crate::db::now_iso(),
        "scope": "project",
        "project": { "name": name },
        "tree": tree,
    }))
}

fn is_apifox(document: &Value) -> bool {
    if document
        .get("apifoxProject")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
    {
        return true;
    }
    if let Some(app) = document.pointer("/$schema/app").and_then(|v| v.as_str()) {
        if app.eq_ignore_ascii_case("apifox") {
            return true;
        }
    }
    document
        .get("apiCollection")
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false)
        && document
            .pointer("/info/name")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
}

fn convert_items(items: &[Value]) -> Vec<Value> {
    let mut out = Vec::new();
    for item in items {
        if item.get("api").is_some() && !item.get("api").unwrap().is_null() {
            out.push(convert_api(item));
            continue;
        }
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let children = item
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if !children.is_empty() || !name.is_empty() {
            out.push(json!({
                "type": "folder",
                "name": fallback(name, "未命名文件夹"),
                "children": convert_items(&children),
            }));
        }
    }
    out
}

fn convert_api(item: &Value) -> Value {
    let api = item.get("api").cloned().unwrap_or(Value::Null);
    let name = item
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let method = api
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET")
        .trim()
        .to_ascii_uppercase();
    let url = api
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    let headers = merge_params(
        api.pointer("/commonParameters/header"),
        api.pointer("/parameters/header"),
    );
    let query = merge_params(
        api.pointer("/commonParameters/query"),
        api.pointer("/parameters/query"),
    );

    json!({
        "type": "api",
        "name": fallback(&name, &format!("{method} {url}")),
        "method": method,
        "url": url,
        "status": map_status(api.get("status").and_then(|v| v.as_str()).unwrap_or("")),
        "request_headers": convert_params(&headers),
        "request_body": convert_body(api.get("requestBody")),
        "query_params": convert_params(&query),
        "response_headers": [],
        "response_results": [],
        "response_examples": convert_examples(
            api.get("responses"),
            api.get("responseExamples"),
        ),
    })
}

fn merge_params(common: Option<&Value>, local: Option<&Value>) -> Vec<Value> {
    let mut out = Vec::new();
    if let Some(arr) = common.and_then(|v| v.as_array()) {
        out.extend(arr.iter().cloned());
    }
    if let Some(arr) = local.and_then(|v| v.as_array()) {
        out.extend(arr.iter().cloned());
    }
    out
}

fn convert_params(items: &[Value]) -> Vec<Value> {
    let mut out = Vec::new();
    for item in items {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            continue;
        }
        let enabled = item
            .get("enable")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        out.push(json!({
            "name": name,
            "type": apifox_type(item.get("type").and_then(|v| v.as_str()).unwrap_or("string")),
            "required": item.get("required").and_then(|v| v.as_bool()).unwrap_or(false),
            "enabled": enabled,
            "description": item.get("description").and_then(|v| v.as_str()).unwrap_or(""),
            "example": value_to_string(item.get("example")),
        }));
    }
    out
}

fn convert_body(body: Option<&Value>) -> Value {
    let Some(body) = body else {
        return empty_body();
    };
    let typ = body
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("none")
        .to_ascii_lowercase();
    let params = body
        .get("parameters")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let examples = body
        .get("examples")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if typ.is_empty() || typ == "none" {
        return empty_body();
    }
    if typ.contains("form-urlencoded") || typ.contains("multipart") || typ == "form-data" {
        return json!({
            "format": "form-data",
            "data_type": "Object",
            "raw": "",
            "fields": convert_body_fields(&params),
        });
    }
    if typ.contains("json") {
        if let Some(raw) = first_example_raw(&examples) {
            return json!({
                "format": "raw",
                "data_type": "JSON",
                "raw": raw,
                "fields": [],
            });
        }
        if !params.is_empty() {
            return json!({
                "format": "json",
                "data_type": "Object",
                "raw": "",
                "fields": convert_body_fields(&params),
            });
        }
        if let Some(schema) = body.get("jsonSchema") {
            if schema.is_object() {
                return json!({
                    "format": "json",
                    "data_type": "Object",
                    "raw": "",
                    "fields": schema_to_fields(schema),
                });
            }
        }
        return empty_body();
    }
    if typ.contains("xml") {
        return json!({
            "format": "raw",
            "data_type": "XML",
            "raw": first_example_raw(&examples).unwrap_or_default(),
            "fields": [],
        });
    }
    if let Some(raw) = first_example_raw(&examples) {
        return json!({
            "format": "raw",
            "data_type": "Text",
            "raw": raw,
            "fields": [],
        });
    }
    if !params.is_empty() {
        return json!({
            "format": "form-data",
            "data_type": "Object",
            "raw": "",
            "fields": convert_body_fields(&params),
        });
    }
    empty_body()
}

fn convert_body_fields(items: &[Value]) -> Vec<Value> {
    let mut out = Vec::new();
    for item in items {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            continue;
        }
        let enabled = item
            .get("enable")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        out.push(json!({
            "parent_id": 0,
            "name": name,
            "type": apifox_type(item.get("type").and_then(|v| v.as_str()).unwrap_or("string")),
            "required": item.get("required").and_then(|v| v.as_bool()).unwrap_or(false),
            "enabled": enabled,
            "description": item.get("description").and_then(|v| v.as_str()).unwrap_or(""),
            "example": value_to_string(item.get("example")),
            "children": [],
        }));
    }
    out
}

fn schema_to_fields(schema: &Value) -> Vec<Value> {
    let required = schema
        .get("required")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    let Some(props) = schema.get("properties").and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    let mut fields = Vec::new();
    for (name, prop) in props {
        let typ = apifox_type(prop.get("type").and_then(|v| v.as_str()).unwrap_or("string"));
        let mut children = Vec::new();
        if typ == "object" {
            children = schema_to_fields(prop);
        }
        fields.push(json!({
            "parent_id": 0,
            "name": name,
            "type": typ,
            "required": required.iter().any(|r| r == name),
            "enabled": true,
            "description": prop.get("description").and_then(|v| v.as_str()).unwrap_or(""),
            "example": value_to_string(prop.get("example")),
            "children": children,
        }));
    }
    fields
}

fn convert_examples(responses: Option<&Value>, examples: Option<&Value>) -> Vec<Value> {
    let mut code_by_id: Map<String, Value> = Map::new();
    let mut name_by_id: Map<String, Value> = Map::new();
    let mut content_by_id: Map<String, Value> = Map::new();
    if let Some(arr) = responses.and_then(|v| v.as_array()) {
        for resp in arr {
            let id = value_to_string(resp.get("id"));
            code_by_id.insert(id.clone(), json!(as_i64(resp.get("code"))));
            name_by_id.insert(
                id.clone(),
                json!(resp.get("name").and_then(|v| v.as_str()).unwrap_or("")),
            );
            content_by_id.insert(
                id,
                json!(map_content_type(
                    resp.get("contentType").and_then(|v| v.as_str()).unwrap_or("")
                )),
            );
        }
    }

    let mut out = Vec::new();
    if let Some(arr) = examples.and_then(|v| v.as_array()) {
        for ex in arr {
            let mut raw = ex
                .get("data")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if raw.is_empty() {
                if let Some(v) = ex.get("value") {
                    raw = pretty_json(v);
                }
            }
            if raw.is_empty() {
                continue;
            }
            let id = value_to_string(ex.get("responseId"));
            let mut code = code_by_id
                .get(&id)
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            if code == 0 {
                code = 200;
            }
            let mut name = ex
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if name.is_empty() {
                name = name_by_id
                    .get(&id)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
            }
            if name.is_empty() {
                name = "示例".into();
            }
            let content_type = content_by_id
                .get(&id)
                .and_then(|v| v.as_str())
                .unwrap_or("application/json")
                .to_string();
            out.push(json!({
                "name": name,
                "status_code": code,
                "content_type": content_type,
                "raw": raw,
            }));
        }
    }
    out
}

fn first_example_raw(examples: &[Value]) -> Option<String> {
    for ex in examples {
        let data = ex
            .get("data")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if !data.is_empty() {
            return Some(data.to_string());
        }
        if let Some(v) = ex.get("value") {
            let s = pretty_json(v);
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    None
}

fn empty_body() -> Value {
    json!({
        "format": "json",
        "data_type": "Object",
        "raw": "",
        "fields": [],
    })
}

fn apifox_type(t: &str) -> String {
    match t.trim().to_ascii_lowercase().as_str() {
        "integer" | "number" | "int" | "float" | "double" => "number".into(),
        "boolean" | "bool" => "boolean".into(),
        "array" => "array".into(),
        "object" => "object".into(),
        "file" => "file".into(),
        _ => "string".into(),
    }
}

fn map_content_type(ct: &str) -> String {
    match ct.trim().to_ascii_lowercase().as_str() {
        "" | "json" | "application/json" => "application/json".into(),
        "xml" | "application/xml" => "application/xml".into(),
        "html" | "text/html" => "text/html".into(),
        "text" | "plain" | "text/plain" => "text/plain".into(),
        other => other.to_string(),
    }
}

fn map_status(status: &str) -> u8 {
    match status.trim().to_ascii_lowercase().as_str() {
        "released" | "done" | "published" => 1,
        "testing" | "test" => 2,
        "deprecated" | "obsolete" => 4,
        _ => 3,
    }
}

fn fallback(name: &str, fb: &str) -> String {
    let t = name.trim();
    if t.is_empty() {
        fb.to_string()
    } else {
        t.to_string()
    }
}

fn value_to_string(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => n.to_string(),
        Some(other) => pretty_json(other),
    }
}

fn as_i64(v: Option<&Value>) -> i64 {
    match v {
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0),
        Some(Value::String(s)) => s.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

fn pretty_json(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_default()
}
