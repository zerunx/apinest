use serde_json::{json, Map, Value};

use crate::error::{AppError, AppResult};

const METHODS: &[&str] = &[
    "get", "post", "put", "patch", "delete", "head", "options", "trace",
];

/// Convert OpenAPI 3.x / Swagger 2.0 JSON into an ApiNest export document.
pub fn convert_openapi_document(document: &Value) -> AppResult<Value> {
    let openapi = document
        .get("openapi")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let swagger = document
        .get("swagger")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let title = document
        .pointer("/info/title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let paths = document
        .get("paths")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    if openapi.is_empty() && swagger.is_empty() && (title.is_empty() || paths.is_empty()) {
        return Err(AppError::msg("不是 OpenAPI / Swagger 文件"));
    }
    if paths.is_empty() {
        return Err(AppError::msg("OpenAPI 文档没有 paths"));
    }

    let mut folder_order: Vec<String> = Vec::new();
    let mut folders: Map<String, Value> = Map::new();
    let mut untagged: Vec<Value> = Vec::new();

    for (path, path_item) in paths {
        if path.starts_with("x-") {
            continue;
        }
        let Some(path_obj) = path_item.as_object() else {
            continue;
        };
        let shared = path_obj.get("parameters").cloned().unwrap_or(Value::Null);
        for method in METHODS {
            let Some(op) = path_obj.get(*method) else {
                continue;
            };
            if op.is_null() {
                continue;
            }
            let node = convert_operation(document, &path, &method.to_uppercase(), op, &shared);
            let tag = op
                .get("tags")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            if let Some(tag) = tag {
                if !folders.contains_key(&tag) {
                    folder_order.push(tag.clone());
                    folders.insert(tag.clone(), Value::Array(Vec::new()));
                }
                if let Some(Value::Array(list)) = folders.get_mut(&tag) {
                    list.push(node);
                }
            } else {
                untagged.push(node);
            }
        }
    }

    let mut children = Vec::new();
    for tag in folder_order {
        let apis = folders.remove(&tag).unwrap_or_else(|| Value::Array(vec![]));
        children.push(json!({
            "type": "folder",
            "name": tag,
            "children": apis,
        }));
    }
    children.extend(untagged);
    if children.is_empty() {
        return Err(AppError::msg("OpenAPI 文档没有可导入的接口"));
    }

    let tree = if title.is_empty() {
        children
    } else {
        vec![json!({
            "type": "folder",
            "name": title,
            "children": children,
        })]
    };

    Ok(json!({
        "format": "apinest",
        "version": 1,
        "exported_at": crate::db::now_iso(),
        "scope": "project",
        "project": { "name": title },
        "tree": tree,
    }))
}

fn convert_operation(
    root: &Value,
    path: &str,
    method: &str,
    op: &Value,
    shared_params: &Value,
) -> Value {
    let name = op
        .get("summary")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            op.get("operationId")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
        })
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{method} {path}"));

    let (headers, query, mut body) = convert_parameters(root, op, shared_params);
    if let Some(request_body) = op.get("requestBody") {
        if let Some(converted) = convert_request_body(root, request_body) {
            body = converted;
        }
    }

    json!({
        "type": "api",
        "name": name,
        "method": method,
        "url": path,
        "status": 3,
        "request_headers": headers,
        "request_body": body,
        "query_params": query,
        "response_headers": [],
        "response_results": [],
        "response_examples": convert_responses(root, op.get("responses")),
    })
}

fn convert_parameters(
    root: &Value,
    op: &Value,
    shared_params: &Value,
) -> (Vec<Value>, Vec<Value>, Value) {
    let mut headers = Vec::new();
    let mut query = Vec::new();
    let mut body = default_body();

    let mut all = Vec::new();
    if let Some(arr) = shared_params.as_array() {
        all.extend(arr.iter().cloned());
    }
    if let Some(arr) = op.get("parameters").and_then(|v| v.as_array()) {
        all.extend(arr.iter().cloned());
    }

    for raw in all {
        let param = resolve_ref(root, &raw);
        let Some(obj) = param.as_object() else {
            continue;
        };
        let in_loc = obj
            .get("in")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() && in_loc != "body" {
            continue;
        }
        let required = obj.get("required").and_then(|v| v.as_bool()).unwrap_or(false);
        let desc = obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let mut example = value_to_string(obj.get("example"));
        if example.is_empty() {
            if let Some(schema) = obj.get("schema") {
                let schema = resolve_ref(root, schema);
                example = value_to_string(schema.get("example"));
                if example.is_empty() {
                    example = value_to_string(schema.get("default"));
                }
            }
        }
        let typ = param_type(root, obj);
        let item = json!({
            "name": name,
            "type": typ,
            "required": required,
            "enabled": true,
            "description": desc,
            "example": example,
        });
        match in_loc.as_str() {
            "header" => headers.push(item),
            "query" => query.push(item),
            "body" => {
                if let Some(schema) = obj.get("schema") {
                    body = schema_to_body(root, schema);
                }
            }
            "formdata" => {
                let mut fields = body
                    .get("fields")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                if body.get("format").and_then(|v| v.as_str()) != Some("form-data") {
                    fields.clear();
                }
                let field_type = if typ == "file" { "file" } else { "string" };
                fields.push(json!({
                    "parent_id": 0,
                    "name": name,
                    "type": field_type,
                    "required": required,
                    "enabled": true,
                    "description": desc,
                    "example": example,
                    "children": [],
                }));
                body = json!({
                    "format": "form-data",
                    "data_type": "Object",
                    "raw": "",
                    "fields": fields,
                });
            }
            _ => {}
        }
    }
    (headers, query, body)
}

fn convert_request_body(root: &Value, request_body: &Value) -> Option<Value> {
    let obj = resolve_ref(root, request_body);
    let content = obj.get("content")?.as_object()?;
    let prefer = [
        "application/json",
        "application/xml",
        "text/plain",
        "multipart/form-data",
        "application/x-www-form-urlencoded",
    ];
    let (media_type, media) = prefer
        .iter()
        .find_map(|key| content.get(*key).map(|v| (*key, v)))
        .or_else(|| content.iter().next().map(|(k, v)| (k.as_str(), v)))?;

    let schema = media.get("schema").map(|s| resolve_ref(root, s));

    if media_type.contains("multipart/form-data")
        || media_type.contains("application/x-www-form-urlencoded")
    {
        let fields = schema
            .as_ref()
            .map(|s| schema_to_fields(root, s))
            .unwrap_or_default();
        return Some(json!({
            "format": "form-data",
            "data_type": "Object",
            "raw": "",
            "fields": fields,
        }));
    }

    let mut raw = String::new();
    if let Some(ex) = media.get("example") {
        raw = pretty_json(ex);
    } else if let Some(examples) = media.get("examples").and_then(|v| v.as_object()) {
        for ex in examples.values() {
            if let Some(v) = ex.get("value") {
                raw = pretty_json(v);
                break;
            }
        }
    }
    if raw.is_empty() {
        if let Some(schema) = &schema {
            if let Some(ex) = schema.get("example") {
                raw = pretty_json(ex);
            } else if has_properties(schema) {
                return Some(json!({
                    "format": "json",
                    "data_type": "Object",
                    "raw": "",
                    "fields": schema_to_fields(root, schema),
                }));
            } else {
                raw = schema_example_json(root, schema);
            }
        }
    }

    let data_type = if media_type.contains("xml") {
        "XML"
    } else if media_type.contains("text/plain") {
        "Text"
    } else {
        "JSON"
    };
    Some(json!({
        "format": "raw",
        "data_type": data_type,
        "raw": raw,
        "fields": [],
    }))
}

fn convert_responses(root: &Value, responses: Option<&Value>) -> Vec<Value> {
    let Some(obj) = responses.and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (code, resp) in obj {
        if code.starts_with("x-") {
            continue;
        }
        let resp = resolve_ref(root, resp);
        let mut name = resp
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            name = code.clone();
        }
        let status_code = if code == "default" {
            200
        } else {
            code.parse::<i64>().unwrap_or(200)
        };
        let mut content_type = "application/json".to_string();
        let mut raw = String::new();
        if let Some(content) = resp.get("content").and_then(|v| v.as_object()) {
            if let Some((mt, media)) = content.iter().next() {
                content_type = mt.clone();
                if let Some(ex) = media.get("example") {
                    raw = pretty_json(ex);
                } else if let Some(schema) = media.get("schema") {
                    let schema = resolve_ref(root, schema);
                    if let Some(ex) = schema.get("example") {
                        raw = pretty_json(ex);
                    } else {
                        raw = schema_example_json(root, &schema);
                    }
                }
            }
        } else if let Some(schema) = resp.get("schema") {
            let schema = resolve_ref(root, schema);
            if let Some(ex) = schema.get("example") {
                raw = pretty_json(ex);
            } else {
                raw = schema_example_json(root, &schema);
            }
        }
        if raw.is_empty() && name == *code {
            continue;
        }
        out.push(json!({
            "name": name,
            "status_code": status_code,
            "content_type": content_type,
            "raw": raw,
        }));
    }
    out
}

fn schema_to_body(root: &Value, schema: &Value) -> Value {
    let schema = resolve_ref(root, schema);
    if has_properties(&schema) {
        json!({
            "format": "json",
            "data_type": "Object",
            "raw": "",
            "fields": schema_to_fields(root, &schema),
        })
    } else {
        json!({
            "format": "raw",
            "data_type": "JSON",
            "raw": schema_example_json(root, &schema),
            "fields": [],
        })
    }
}

fn schema_to_fields(root: &Value, schema: &Value) -> Vec<Value> {
    let schema = resolve_ref(root, schema);
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
        let prop = resolve_ref(root, prop);
        let typ = schema_type(&prop);
        let mut example = value_to_string(prop.get("example"));
        if example.is_empty() {
            example = value_to_string(prop.get("default"));
        }
        let mut children = Vec::new();
        if typ == "object" {
            children = schema_to_fields(root, &prop);
        } else if typ == "array" {
            if let Some(items) = prop.get("items") {
                let items = resolve_ref(root, items);
                if schema_type(&items) == "object" {
                    children = schema_to_fields(root, &items);
                }
            }
        }
        fields.push(json!({
            "parent_id": 0,
            "name": name,
            "type": typ,
            "required": required.iter().any(|r| r == name),
            "enabled": true,
            "description": prop.get("description").and_then(|v| v.as_str()).unwrap_or(""),
            "example": example,
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

fn schema_example_json(root: &Value, schema: &Value) -> String {
    match schema_to_example(root, schema, 0) {
        Some(v) => pretty_json(&v),
        None => String::new(),
    }
}

fn schema_to_example(root: &Value, schema: &Value, depth: usize) -> Option<Value> {
    if depth > 6 {
        return None;
    }
    let schema = resolve_ref(root, schema);
    if let Some(ex) = schema.get("example") {
        return Some(ex.clone());
    }
    if let Some(def) = schema.get("default") {
        return Some(def.clone());
    }
    match schema_type(&schema).as_str() {
        "object" => {
            let mut obj = Map::new();
            if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
                for (k, v) in props {
                    obj.insert(k.clone(), schema_to_example(root, v, depth + 1).unwrap_or(Value::Null));
                }
            }
            Some(Value::Object(obj))
        }
        "array" => {
            let item = schema
                .get("items")
                .and_then(|items| schema_to_example(root, items, depth + 1))
                .unwrap_or(Value::Null);
            Some(Value::Array(vec![item]))
        }
        "number" => Some(json!(0)),
        "boolean" => Some(json!(false)),
        _ => {
            if let Some(arr) = schema.get("enum").and_then(|v| v.as_array()) {
                if let Some(first) = arr.first() {
                    return Some(first.clone());
                }
            }
            Some(json!(""))
        }
    }
}

fn schema_type(schema: &Value) -> String {
    if let Some(t) = schema.get("type").and_then(|v| v.as_str()) {
        return match t.to_ascii_lowercase().as_str() {
            "integer" | "number" => "number".into(),
            "boolean" => "boolean".into(),
            "array" => "array".into(),
            "object" => "object".into(),
            "file" => "file".into(),
            _ => "string".into(),
        };
    }
    if schema.get("properties").is_some() {
        return "object".into();
    }
    if schema.get("items").is_some() {
        return "array".into();
    }
    "string".into()
}

fn param_type(root: &Value, param: &Map<String, Value>) -> String {
    if let Some(schema) = param.get("schema") {
        return schema_type(&resolve_ref(root, schema));
    }
    match param
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("string")
        .to_ascii_lowercase()
        .as_str()
    {
        "integer" | "number" => "number".into(),
        "boolean" => "boolean".into(),
        "array" => "array".into(),
        "file" => "file".into(),
        _ => "string".into(),
    }
}

fn resolve_ref(root: &Value, value: &Value) -> Value {
    let Some(ref_str) = value.get("$ref").and_then(|v| v.as_str()) else {
        return value.clone();
    };
    let Some(target) = lookup_ref(root, ref_str) else {
        return value.clone();
    };
    let Some(target_obj) = target.as_object() else {
        return target;
    };
    let Some(local) = value.as_object() else {
        return target;
    };
    let mut out = target_obj.clone();
    for (k, v) in local {
        if k != "$ref" {
            out.insert(k.clone(), v.clone());
        }
    }
    Value::Object(out)
}

fn lookup_ref(root: &Value, ref_str: &str) -> Option<Value> {
    let ref_str = ref_str.trim();
    if !ref_str.starts_with("#/") {
        return None;
    }
    let mut cur = root;
    for part in ref_str.trim_start_matches("#/").split('/') {
        let part = part.replace("~1", "/").replace("~0", "~");
        cur = cur.get(&part)?;
    }
    Some(cur.clone())
}

fn default_body() -> Value {
    json!({
        "format": "json",
        "data_type": "Object",
        "raw": "",
        "fields": [],
    })
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
    if let Some(s) = v.as_str() {
        return s.to_string();
    }
    serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
}
