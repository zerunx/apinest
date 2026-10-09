use serde_json::{json, Map, Value};

use crate::error::{AppError, AppResult};

/// Convert an Eolink group/project export into ApiNest document.
/// Only groups and APIs are kept.
pub fn convert_eolink_document(document: &Value) -> AppResult<Value> {
    if document.as_array().is_some() {
        let groups = document.as_array().unwrap();
        if groups.is_empty() || !is_eolink_group(&groups[0]) {
            return Err(AppError::msg("不是 Eolink 导出文件"));
        }
        let children: Vec<Value> = groups.iter().map(convert_group).collect();
        return Ok(wrap_doc("", children));
    }

    if !is_eolink_group(document) {
        return Err(AppError::msg("不是 Eolink 导出文件"));
    }
    let name = document
        .pointer("/projectInfo/projectName")
        .and_then(|v| v.as_str())
        .or_else(|| document.get("groupName").and_then(|v| v.as_str()))
        .unwrap_or("")
        .trim()
        .to_string();
    Ok(wrap_doc(&name, vec![convert_group(document)]))
}

fn wrap_doc(name: &str, tree: Vec<Value>) -> Value {
    json!({
        "format": "apinest",
        "version": 1,
        "exported_at": crate::db::now_iso(),
        "scope": "project",
        "project": { "name": name },
        "tree": tree,
    })
}

fn is_eolink_group(v: &Value) -> bool {
    v.get("groupName").and_then(|x| x.as_str()).is_some()
        || v.get("groupID").is_some()
        || v.get("apiList").and_then(|x| x.as_array()).is_some()
        || v.get("childGroupList").and_then(|x| x.as_array()).is_some()
}

fn convert_group(g: &Value) -> Value {
    let mut children = Vec::new();
    if let Some(list) = g.get("childGroupList").and_then(|v| v.as_array()) {
        for child in list {
            children.push(convert_group(child));
        }
    }
    if let Some(list) = g.get("apiList").and_then(|v| v.as_array()) {
        for api in list {
            if let Some(node) = convert_api(api) {
                children.push(node);
            }
        }
    }
    json!({
        "type": "folder",
        "name": fallback(
            g.get("groupName").and_then(|v| v.as_str()).unwrap_or(""),
            "未命名文件夹",
        ),
        "children": children,
    })
}

fn convert_api(api: &Value) -> Option<Value> {
    let base = api.get("baseInfo").cloned().unwrap_or(Value::Null);
    let api_type = api
        .get("apiType")
        .and_then(|v| v.as_str())
        .unwrap_or("http")
        .to_ascii_lowercase();
    let name = base
        .get("apiName")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let url = base
        .get("apiURI")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if name.is_empty() && url.is_empty() {
        return None;
    }
    let method = eo_method(
        base.get("apiRequestType"),
        &api_type,
        base.get("apiProtocol"),
    );

    Some(json!({
        "type": "api",
        "name": fallback(&name, &format!("{method} {url}")),
        "method": method,
        "url": url,
        "status": 3,
        "request_headers": convert_headers(api.get("headerInfo")),
        "request_body": convert_body(&base, api.get("requestInfo")),
        "query_params": convert_params(api.get("urlParam")),
        "response_headers": [],
        "response_results": [],
        "response_examples": convert_results(api.get("resultInfo"), &base),
    }))
}

fn eo_method(request_type: Option<&Value>, api_type: &str, protocol: Option<&Value>) -> String {
    if api_type.contains("websocket") || eo_int(protocol) == 2 {
        return "WS".into();
    }
    match eo_int(request_type) {
        0 => "POST".into(),
        1 => "GET".into(),
        2 => "PUT".into(),
        3 => "DELETE".into(),
        4 => "HEAD".into(),
        5 => "OPTIONS".into(),
        6 => "PATCH".into(),
        _ => {
            if !api_type.is_empty() && api_type != "http" {
                api_type.to_uppercase()
            } else {
                "GET".into()
            }
        }
    }
}

fn convert_headers(items: Option<&Value>) -> Vec<Value> {
    let Some(arr) = items.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in arr {
        let key = item
            .get("headerName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if key.is_empty() {
            continue;
        }
        out.push(json!({
            "name": key,
            "type": "string",
            "required": truthy(item.get("paramNotNull")),
            "enabled": true,
            "description": first_non_empty(&[
                item.get("paramNote").and_then(|v| v.as_str()).unwrap_or(""),
                item.get("paramName").and_then(|v| v.as_str()).unwrap_or(""),
            ]),
            "example": item.get("headerValue").and_then(|v| v.as_str()).unwrap_or(""),
        }));
    }
    out
}

fn convert_params(items: Option<&Value>) -> Vec<Value> {
    let Some(arr) = items.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in arr {
        let key = item
            .get("paramKey")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if key.is_empty() {
            continue;
        }
        let mut example = item
            .get("paramValue")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if example.is_empty() {
            example = item
                .get("paramInfo")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
        }
        out.push(json!({
            "name": key,
            "type": param_type(item.get("paramType")),
            "required": truthy(item.get("paramNotNull")),
            "enabled": true,
            "description": first_non_empty(&[
                item.get("paramNote").and_then(|v| v.as_str()).unwrap_or(""),
                item.get("paramName").and_then(|v| v.as_str()).unwrap_or(""),
            ]),
            "example": example,
        }));
    }
    out
}

fn convert_body(base: &Value, request_info: Option<&Value>) -> Value {
    let param_type = eo_int(base.get("apiRequestParamType"));
    let raw = base
        .get("apiRequestRaw")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let fields = convert_body_fields(request_info);

    match param_type {
        0 => json!({
            "format": "form-data",
            "data_type": "Object",
            "raw": "",
            "fields": fields,
        }),
        2 => {
            if !raw.is_empty() {
                json!({
                    "format": "raw",
                    "data_type": "JSON",
                    "raw": raw,
                    "fields": [],
                })
            } else if !fields.is_empty() {
                json!({
                    "format": "json",
                    "data_type": "Object",
                    "raw": "",
                    "fields": fields,
                })
            } else {
                default_body()
            }
        }
        3 => json!({
            "format": "raw",
            "data_type": "XML",
            "raw": raw,
            "fields": [],
        }),
        1 => json!({
            "format": "raw",
            "data_type": "Text",
            "raw": raw,
            "fields": [],
        }),
        _ => {
            if !fields.is_empty() {
                let has_nested = request_info
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter().any(|item| {
                            item.get("childList")
                                .and_then(|v| v.as_array())
                                .map(|c| !c.is_empty())
                                .unwrap_or(false)
                                || matches!(
                                    param_type_str(item.get("paramType")).as_str(),
                                    "object" | "array"
                                )
                        })
                    })
                    .unwrap_or(false);
                if has_nested {
                    json!({
                        "format": "json",
                        "data_type": "Object",
                        "raw": raw,
                        "fields": fields,
                    })
                } else {
                    json!({
                        "format": "form-data",
                        "data_type": "Object",
                        "raw": "",
                        "fields": fields,
                    })
                }
            } else if !raw.is_empty() {
                json!({
                    "format": "raw",
                    "data_type": "JSON",
                    "raw": raw,
                    "fields": [],
                })
            } else {
                default_body()
            }
        }
    }
}

fn convert_body_fields(items: Option<&Value>) -> Vec<Value> {
    let Some(arr) = items.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in arr {
        let key = item
            .get("paramKey")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if key.is_empty() {
            continue;
        }
        let mut example = item
            .get("paramValue")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if example.is_empty() {
            example = item
                .get("paramInfo")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
        }
        out.push(json!({
            "parent_id": 0,
            "name": key,
            "type": param_type(item.get("paramType")),
            "required": truthy(item.get("paramNotNull")),
            "enabled": true,
            "description": first_non_empty(&[
                item.get("paramNote").and_then(|v| v.as_str()).unwrap_or(""),
                item.get("paramName").and_then(|v| v.as_str()).unwrap_or(""),
            ]),
            "example": example,
            "children": convert_body_fields(item.get("childList")),
        }));
    }
    out
}

fn convert_results(results: Option<&Value>, base: &Value) -> Vec<Value> {
    let mut out = Vec::new();
    if let Some(arr) = results.and_then(|v| v.as_array()) {
        for item in arr {
            let name = item
                .get("responseName")
                .and_then(|v| v.as_str())
                .unwrap_or("响应")
                .trim()
                .to_string();
            let code = item
                .get("responseCode")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(200);
            let mut raw = item
                .get("raw")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if raw.is_empty() {
                if let Some(params) = item.get("paramList") {
                    if let Some(example) = params_to_example(params) {
                        raw = serde_json::to_string_pretty(&example).unwrap_or_default();
                    }
                }
            }
            if raw.is_empty() {
                continue;
            }
            out.push(json!({
                "name": if name.is_empty() { "响应".into() } else { name },
                "status_code": code,
                "content_type": base
                    .get("apiSuccessContentType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("application/json"),
                "raw": raw,
            }));
        }
    }
    if out.is_empty() {
        if let Some(s) = base.get("apiSuccessMock").and_then(|v| v.as_str()) {
            if !s.trim().is_empty() {
                let mut code = eo_int(base.get("apiSuccessStatusCode"));
                if code == 0 {
                    code = 200;
                }
                out.push(json!({
                    "name": "成功",
                    "status_code": code,
                    "content_type": base
                        .get("apiSuccessContentType")
                        .and_then(|v| v.as_str())
                        .unwrap_or("application/json"),
                    "raw": s,
                }));
            }
        }
        if let Some(s) = base.get("apiFailureMock").and_then(|v| v.as_str()) {
            if !s.trim().is_empty() {
                let mut code = eo_int(base.get("apiFailureStatusCode"));
                if code == 0 {
                    code = 200;
                }
                out.push(json!({
                    "name": "失败",
                    "status_code": code,
                    "content_type": base
                        .get("apiFailureContentType")
                        .and_then(|v| v.as_str())
                        .unwrap_or("application/json"),
                    "raw": s,
                }));
            }
        }
    }
    out
}

fn params_to_example(items: &Value) -> Option<Value> {
    let arr = items.as_array()?;
    let mut obj = Map::new();
    for item in arr {
        let key = item.get("paramKey").and_then(|v| v.as_str())?.trim();
        if key.is_empty() {
            continue;
        }
        let typ = param_type_str(item.get("paramType"));
        let value = match typ.as_str() {
            "object" => params_to_example(item.get("childList").unwrap_or(&Value::Array(vec![])))
                .unwrap_or(json!({})),
            "array" => {
                if let Some(children) = item.get("childList").and_then(|v| v.as_array()) {
                    if !children.is_empty() {
                        json!([params_to_example(item.get("childList").unwrap()).unwrap_or(json!({}))])
                    } else {
                        json!([])
                    }
                } else {
                    json!([])
                }
            }
            "number" => {
                let s = item.get("paramValue").and_then(|v| v.as_str()).unwrap_or("0");
                json!(s.parse::<f64>().unwrap_or(0.0))
            }
            "boolean" => {
                let s = item.get("paramValue").and_then(|v| v.as_str()).unwrap_or("");
                json!(s == "true" || s == "1")
            }
            _ => json!(item.get("paramValue").and_then(|v| v.as_str()).unwrap_or("")),
        };
        obj.insert(key.to_string(), value);
    }
    Some(Value::Object(obj))
}

fn param_type(v: Option<&Value>) -> &'static str {
    match param_type_str(v).as_str() {
        "file" => "file",
        "number" => "number",
        "boolean" => "boolean",
        "array" => "array",
        "object" => "object",
        _ => "string",
    }
}

fn param_type_str(v: Option<&Value>) -> String {
    match eo_int(v) {
        1 => "file".into(),
        3 | 4 | 5 | 9 | 10 | 11 | 14 => "number".into(),
        8 => "boolean".into(),
        12 => "array".into(),
        13 => "object".into(),
        _ => "string".into(),
    }
}

fn eo_int(v: Option<&Value>) -> i64 {
    match v {
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0),
        Some(Value::String(s)) => s.trim().parse().unwrap_or(0),
        _ => 0,
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

fn fallback(name: &str, fb: &str) -> String {
    let t = name.trim();
    if t.is_empty() {
        fb.to_string()
    } else {
        t.to_string()
    }
}

fn first_non_empty(values: &[&str]) -> String {
    for v in values {
        if !v.trim().is_empty() {
            return v.trim().to_string();
        }
    }
    String::new()
}
