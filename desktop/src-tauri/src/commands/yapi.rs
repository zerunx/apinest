use serde_json::{json, Map, Value};

use crate::error::{AppError, AppResult};

/// Convert a YApi category/interface export into ApiNest document.
pub fn convert_yapi_document(document: &Value) -> AppResult<Value> {
    let cats = parse_categories(document)?;
    if cats.is_empty() {
        return Err(AppError::msg("YApi 文件没有分类"));
    }

    let mut tree = Vec::new();
    let mut api_count = 0;
    for cat in &cats {
        let folder = convert_category(cat);
        api_count += count_apis(&folder);
        tree.push(folder);
    }
    if api_count == 0 {
        return Err(AppError::msg("YApi 文件没有可导入的接口"));
    }

    Ok(json!({
        "format": "apinest",
        "version": 1,
        "exported_at": crate::db::now_iso(),
        "scope": "project",
        "project": { "name": "" },
        "tree": tree,
    }))
}

fn parse_categories(document: &Value) -> AppResult<Vec<Value>> {
    if let Some(arr) = document.as_array() {
        if !looks_like_categories(arr) {
            return Err(AppError::msg("不是 YApi 导出文件"));
        }
        return Ok(arr.clone());
    }

    if looks_like_category(document) {
        return Ok(vec![document.clone()]);
    }

    if let Some(obj) = document.as_object() {
        for key in ["data", "cats", "list", "categories"] {
            if let Some(payload) = obj.get(key) {
                if let Ok(cats) = parse_categories(payload) {
                    return Ok(cats);
                }
            }
        }
    }

    Err(AppError::msg("不是 YApi 导出文件"))
}

fn looks_like_categories(cats: &[Value]) -> bool {
    cats.iter().any(looks_like_category)
}

fn looks_like_category(v: &Value) -> bool {
    let name = v
        .get("name")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim();
    let list = v.get("list").and_then(|x| x.as_array());
    if !name.is_empty() && list.is_some() {
        return true;
    }
    list.map(|arr| {
        arr.iter().any(|api| {
            !api.get("title")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .trim()
                .is_empty()
                || !api
                    .get("path")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .trim()
                    .is_empty()
                || !api
                    .get("method")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .trim()
                    .is_empty()
        })
    })
    .unwrap_or(false)
}

fn convert_category(cat: &Value) -> Value {
    let children: Vec<Value> = cat
        .get("list")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(convert_api).collect())
        .unwrap_or_default();
    json!({
        "type": "folder",
        "name": fallback(
            cat.get("name").and_then(|v| v.as_str()).unwrap_or(""),
            "未命名分类",
        ),
        "children": children,
    })
}

fn convert_api(api: &Value) -> Value {
    let method = api
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET")
        .trim()
        .to_ascii_uppercase();
    let mut url = api
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if url.is_empty() {
        url = api
            .pointer("/query_path/path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
    }
    let title = api
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    json!({
        "type": "api",
        "name": fallback(&title, &format!("{method} {url}")),
        "method": method,
        "url": url,
        "status": map_status(api.get("status").and_then(|v| v.as_str()).unwrap_or("")),
        "request_headers": convert_params(api.get("req_headers")),
        "request_body": convert_body(api),
        "query_params": convert_params(api.get("req_query")),
        "response_headers": [],
        "response_results": [],
        "response_examples": convert_response(api),
    })
}

fn convert_params(items: Option<&Value>) -> Vec<Value> {
    let Some(arr) = items.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in arr {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            continue;
        }
        let mut example = item
            .get("example")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if example.is_empty() {
            example = item
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
        }
        out.push(json!({
            "name": name,
            "type": yapi_type(item.get("type").and_then(|v| v.as_str()).unwrap_or("string")),
            "required": yapi_required(item.get("required")),
            "enabled": true,
            "description": item.get("desc").and_then(|v| v.as_str()).unwrap_or(""),
            "example": example,
        }));
    }
    out
}

fn convert_body(api: &Value) -> Value {
    let typ = api
        .get("req_body_type")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let form = api
        .get("req_body_form")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let other = api
        .get("req_body_other")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let is_schema = api
        .get("req_body_is_json_schema")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    match typ.as_str() {
        "" | "none" => {
            if form.is_empty() && other.is_empty() {
                return empty_body();
            }
        }
        "form" | "form-data" | "file" => {
            let mut fields = convert_body_fields(&form);
            if fields.is_empty() && is_schema {
                if let Some(schema) = parse_json_schema(&other) {
                    fields = schema_to_fields(&schema);
                }
            }
            return json!({
                "format": "form-data",
                "data_type": "Object",
                "raw": "",
                "fields": fields,
            });
        }
        "json" => {
            if is_schema {
                if let Some(schema) = parse_json_schema(&other) {
                    if has_properties(&schema) {
                        return json!({
                            "format": "json",
                            "data_type": "Object",
                            "raw": "",
                            "fields": schema_to_fields(&schema),
                        });
                    }
                    return json!({
                        "format": "raw",
                        "data_type": "JSON",
                        "raw": schema_example_json(&schema),
                        "fields": [],
                    });
                }
            }
            if other.is_empty() {
                return empty_body();
            }
            return json!({
                "format": "raw",
                "data_type": "JSON",
                "raw": other,
                "fields": [],
            });
        }
        "raw" | "text" | "xml" => {
            let data_type = if typ == "xml" { "XML" } else { "Text" };
            return json!({
                "format": "raw",
                "data_type": data_type,
                "raw": other,
                "fields": [],
            });
        }
        _ => {}
    }

    if !form.is_empty() {
        return json!({
            "format": "form-data",
            "data_type": "Object",
            "raw": "",
            "fields": convert_body_fields(&form),
        });
    }
    if !other.is_empty() {
        return json!({
            "format": "raw",
            "data_type": "Text",
            "raw": other,
            "fields": [],
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
        let mut example = item
            .get("example")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if example.is_empty() {
            example = item
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
        }
        out.push(json!({
            "parent_id": 0,
            "name": name,
            "type": yapi_type(item.get("type").and_then(|v| v.as_str()).unwrap_or("string")),
            "required": yapi_required(item.get("required")),
            "enabled": true,
            "description": item.get("desc").and_then(|v| v.as_str()).unwrap_or(""),
            "example": example,
            "children": [],
        }));
    }
    out
}

fn convert_response(api: &Value) -> Vec<Value> {
    let mut raw = api
        .get("res_body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if raw.is_empty() {
        return Vec::new();
    }
    let mut content_type =
        map_content_type(api.get("res_body_type").and_then(|v| v.as_str()).unwrap_or(""));
    let is_schema = api
        .get("res_body_is_json_schema")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if is_schema {
        if let Some(schema) = parse_json_schema(&raw) {
            let example = schema_example_json(&schema);
            if !example.is_empty() {
                raw = example;
            }
            content_type = "application/json".into();
        }
    }
    vec![json!({
        "name": "成功",
        "status_code": 200,
        "content_type": content_type,
        "raw": raw,
    })]
}

fn parse_json_schema(raw: &str) -> Option<Value> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let schema: Value = serde_json::from_str(raw).ok()?;
    let obj = schema.as_object()?;
    if obj.get("type").is_none()
        && obj.get("properties").is_none()
        && obj.get("$schema").is_none()
        && obj.get("items").is_none()
    {
        return None;
    }
    Some(schema)
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
        let typ = yapi_type(prop.get("type").and_then(|v| v.as_str()).unwrap_or("string"));
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

fn has_properties(schema: &Value) -> bool {
    schema
        .get("properties")
        .and_then(|v| v.as_object())
        .map(|m| !m.is_empty())
        .unwrap_or(false)
}

fn schema_example_json(schema: &Value) -> String {
    match schema_to_example(schema, 0) {
        Some(v) => pretty_json(&v),
        None => String::new(),
    }
}

fn schema_to_example(schema: &Value, depth: usize) -> Option<Value> {
    if depth > 6 {
        return None;
    }
    if let Some(ex) = schema.get("example") {
        return Some(ex.clone());
    }
    if let Some(def) = schema.get("default") {
        return Some(def.clone());
    }
    match yapi_type(schema.get("type").and_then(|v| v.as_str()).unwrap_or("")).as_str() {
        "object" => {
            let mut obj = Map::new();
            if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
                for (k, v) in props {
                    obj.insert(
                        k.clone(),
                        schema_to_example(v, depth + 1).unwrap_or(Value::Null),
                    );
                }
            }
            Some(Value::Object(obj))
        }
        "array" => {
            let item = schema
                .get("items")
                .and_then(|items| schema_to_example(items, depth + 1))
                .unwrap_or(Value::Null);
            Some(Value::Array(vec![item]))
        }
        "number" => Some(json!(0)),
        "boolean" => Some(json!(false)),
        _ => {
            if schema.get("properties").is_some() {
                return schema_to_example(
                    &json!({ "type": "object", "properties": schema.get("properties") }),
                    depth,
                );
            }
            Some(json!(""))
        }
    }
}

fn empty_body() -> Value {
    json!({
        "format": "json",
        "data_type": "Object",
        "raw": "",
        "fields": [],
    })
}

fn yapi_type(t: &str) -> String {
    match t.trim().to_ascii_lowercase().as_str() {
        "number" | "integer" | "int" | "float" | "double" => "number".into(),
        "boolean" | "bool" => "boolean".into(),
        "array" => "array".into(),
        "object" => "object".into(),
        "file" => "file".into(),
        "text" | "string" | "" => "string".into(),
        _ => "string".into(),
    }
}

fn map_content_type(ct: &str) -> String {
    match ct.trim().to_ascii_lowercase().as_str() {
        "" | "json" | "application/json" => "application/json".into(),
        "xml" | "application/xml" | "text/xml" => "application/xml".into(),
        "html" | "text/html" => "text/html".into(),
        "text" | "plain" | "text/plain" | "raw" => "text/plain".into(),
        other => other.to_string(),
    }
}

fn map_status(status: &str) -> u8 {
    match status.trim().to_ascii_lowercase().as_str() {
        "done" | "released" | "published" => 1,
        "testing" | "test" => 2,
        "deprecated" | "obsolete" => 4,
        _ => 3,
    }
}

fn yapi_required(v: Option<&Value>) -> bool {
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

fn pretty_json(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_default()
}

fn count_apis(node: &Value) -> usize {
    if node.get("type").and_then(|v| v.as_str()) == Some("api") {
        return 1;
    }
    node.get("children")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(count_apis).sum())
        .unwrap_or(0)
}
