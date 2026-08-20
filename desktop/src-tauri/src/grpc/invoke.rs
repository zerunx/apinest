use std::path::Path;
use std::time::{Duration, Instant};

use bytes::{Buf, BufMut, Bytes};
use http::uri::PathAndQuery;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, MethodDescriptor};
use protox::file::{ChainFileResolver, File, FileResolver, GoogleFileResolver};
use protox::Compiler;
use serde_json::{json, Value};

use crate::grpc::well_known::EmbeddedFileResolver;
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};
use tonic::metadata::{KeyRef, MetadataKey, MetadataValue};
use tonic::transport::Endpoint;
use tonic::{Code, Request, Status};
use url::Url;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct GrpcCallResult {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub duration_ms: u128,
}

struct MemoryFileResolver {
    name: String,
    source: String,
}

impl FileResolver for MemoryFileResolver {
    fn open_file(&self, name: &str) -> Result<File, protox::Error> {
        if names_match(name, &self.name) {
            File::from_source(&self.name, &self.source)
        } else {
            Err(protox::Error::file_not_found(name))
        }
    }
}

fn names_match(requested: &str, owned: &str) -> bool {
    if requested == owned {
        return true;
    }
    Path::new(requested)
        .file_name()
        .and_then(|s| s.to_str())
        == Some(owned)
}

pub fn proto_file_name(name: &str) -> String {
    let trimmed = name.trim();
    let base = Path::new(trimmed)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .trim();
    if base.is_empty() {
        return "source.proto".to_string();
    }
    if base.ends_with(".proto") {
        base.to_string()
    } else {
        format!("{base}.proto")
    }
}

pub fn compile_descriptor_pool(name: &str, source: &str) -> AppResult<DescriptorPool> {
    let file_name = proto_file_name(name);
    let mut resolver = ChainFileResolver::new();
    resolver.add(MemoryFileResolver {
        name: file_name.clone(),
        source: source.to_string(),
    });
    resolver.add(EmbeddedFileResolver::google_common());
    resolver.add(GoogleFileResolver::new());
    let mut compiler = Compiler::with_file_resolver(resolver);
    compiler.include_imports(true);
    compiler.include_source_info(false);
    compiler
        .open_file(&file_name)
        .map_err(|e| AppError::msg(format!("failed to compile proto: {e}")))?;
    Ok(compiler.descriptor_pool())
}

pub fn find_method(pool: &DescriptorPool, rpc_path: &str) -> AppResult<MethodDescriptor> {
    let path = rpc_path.trim().trim_start_matches('/');
    let (service_name, method_name) = path.split_once('/').ok_or_else(|| {
        AppError::msg(format!("invalid rpc path: {rpc_path}"))
    })?;
    if let Some(svc) = pool.get_service_by_name(service_name) {
        if let Some(method) = svc.methods().find(|m| m.name() == method_name) {
            return Ok(method);
        }
    }
    for svc in pool.services() {
        let full = svc.full_name();
        if full == service_name
            || svc.name() == service_name
            || full.ends_with(&format!(".{service_name}"))
        {
            if let Some(method) = svc.methods().find(|m| m.name() == method_name) {
                return Ok(method);
            }
        }
    }
    Err(AppError::msg(format!(
        "method not found in proto: {rpc_path}"
    )))
}

pub fn parse_grpc_endpoint(raw: &str, tls_hint: bool) -> AppResult<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::msg("server address is empty"));
    }
    let parsed = if trimmed.contains("://") {
        Url::parse(trimmed).map_err(|e| AppError::msg(format!("invalid address: {e}")))?
    } else {
        Url::parse(&format!("http://{trimmed}"))
            .map_err(|e| AppError::msg(format!("invalid address: {e}")))?
    };
    let host = parsed.host_str().unwrap_or("").trim();
    if host.is_empty() {
        return Err(AppError::msg(
            "address must include a host, e.g. 127.0.0.1:9000",
        ));
    }
    let scheme = parsed.scheme().to_ascii_lowercase();
    let tls = tls_hint || scheme == "https" || scheme == "grpcs";
    let port = parsed.port().unwrap_or(if tls { 443 } else { 80 });
    let http_scheme = if tls { "https" } else { "http" };
    let host_fmt = if host.contains(':') {
        format!("[{host}]")
    } else {
        host.to_string()
    };
    Ok(format!("{http_scheme}://{host_fmt}:{port}"))
}

pub fn encode_message_json(method: &MethodDescriptor, json: &str) -> AppResult<Bytes> {
    let value: Value = if json.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(json).map_err(|e| AppError::msg(format!("invalid request JSON: {e}")))?
    };
    let message = DynamicMessage::deserialize(method.input(), &value)
        .map_err(|e| AppError::msg(format!("request JSON does not match proto: {e}")))?;
    Ok(Bytes::from(message.encode_to_vec()))
}

fn decode_message_json(method: &MethodDescriptor, bytes: &[u8]) -> AppResult<String> {
    let message = DynamicMessage::decode(method.output(), bytes)
        .map_err(|e| AppError::msg(format!("failed to decode response: {e}")))?;
    serde_json::to_string_pretty(&message)
        .map_err(|e| AppError::msg(format!("failed to encode response JSON: {e}")))
}

fn grpc_code_name(code: Code) -> &'static str {
    match code {
        Code::Ok => "OK",
        Code::Cancelled => "CANCELLED",
        Code::Unknown => "UNKNOWN",
        Code::InvalidArgument => "INVALID_ARGUMENT",
        Code::DeadlineExceeded => "DEADLINE_EXCEEDED",
        Code::NotFound => "NOT_FOUND",
        Code::AlreadyExists => "ALREADY_EXISTS",
        Code::PermissionDenied => "PERMISSION_DENIED",
        Code::ResourceExhausted => "RESOURCE_EXHAUSTED",
        Code::FailedPrecondition => "FAILED_PRECONDITION",
        Code::Aborted => "ABORTED",
        Code::OutOfRange => "OUT_OF_RANGE",
        Code::Unimplemented => "UNIMPLEMENTED",
        Code::Internal => "INTERNAL",
        Code::Unavailable => "UNAVAILABLE",
        Code::DataLoss => "DATA_LOSS",
        Code::Unauthenticated => "UNAUTHENTICATED",
    }
}

fn grpc_status_number(code: Code) -> i32 {
    match code {
        Code::Ok => 0,
        Code::Cancelled => 1,
        Code::Unknown => 2,
        Code::InvalidArgument => 3,
        Code::DeadlineExceeded => 4,
        Code::NotFound => 5,
        Code::AlreadyExists => 6,
        Code::PermissionDenied => 7,
        Code::ResourceExhausted => 8,
        Code::FailedPrecondition => 9,
        Code::Aborted => 10,
        Code::OutOfRange => 11,
        Code::Unimplemented => 12,
        Code::Internal => 13,
        Code::Unavailable => 14,
        Code::DataLoss => 15,
        Code::Unauthenticated => 16,
    }
}

fn grpc_http_status(code: Code) -> u16 {
    match code {
        Code::Ok => 200,
        Code::Cancelled => 499,
        Code::InvalidArgument | Code::FailedPrecondition | Code::OutOfRange => 400,
        Code::DeadlineExceeded => 504,
        Code::NotFound => 404,
        Code::AlreadyExists | Code::Aborted => 409,
        Code::PermissionDenied => 403,
        Code::ResourceExhausted => 429,
        Code::Unimplemented => 501,
        Code::Unavailable => 503,
        Code::Unauthenticated => 401,
        _ => 500,
    }
}

fn metadata_headers(metadata: &tonic::metadata::MetadataMap) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    for key in metadata.keys() {
        match key {
            KeyRef::Ascii(key) => {
                for value in metadata.get_all(key) {
                    headers.push((
                        key.as_str().to_string(),
                        value.to_str().unwrap_or_default().to_string(),
                    ));
                }
            }
            KeyRef::Binary(_) => {}
        }
    }
    headers
}

fn apply_request_metadata(
    request: &mut Request<Bytes>,
    metadata: &[(String, String)],
) -> AppResult<()> {
    for (key, value) in metadata {
        let name = key.trim();
        if name.is_empty() {
            continue;
        }
        let lower = name.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "content-type" | "te" | "host" | "connection" | "grpc-encoding" | "grpc-accept-encoding"
        ) || lower.ends_with("-bin")
        {
            continue;
        }
        let parsed_key: MetadataKey<_> = lower
            .parse()
            .map_err(|_| AppError::msg(format!("invalid metadata key: {name}")))?;
        let parsed_val = MetadataValue::try_from(value.as_str())
            .map_err(|_| AppError::msg(format!("invalid metadata value for {name}")))?;
        request.metadata_mut().insert(parsed_key, parsed_val);
    }
    Ok(())
}

#[derive(Debug, Clone, Default)]
struct BytesCodec;

impl Codec for BytesCodec {
    type Encode = Bytes;
    type Decode = Bytes;
    type Encoder = BytesEncoder;
    type Decoder = BytesDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        BytesEncoder
    }

    fn decoder(&mut self) -> Self::Decoder {
        BytesDecoder
    }
}

#[derive(Debug, Clone, Default)]
struct BytesEncoder;

impl Encoder for BytesEncoder {
    type Item = Bytes;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        dst.put(item);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
struct BytesDecoder;

impl Decoder for BytesDecoder {
    type Item = Bytes;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        Ok(Some(src.copy_to_bytes(src.remaining())))
    }
}

pub async fn invoke_unary(
    endpoint: &str,
    rpc_path: &str,
    method: &MethodDescriptor,
    body: Bytes,
    metadata: &[(String, String)],
    timeout_secs: u64,
    connect_timeout_secs: u64,
) -> AppResult<GrpcCallResult> {
    if method.is_client_streaming() || method.is_server_streaming() {
        return Err(AppError::msg(
            "streaming RPCs are not supported yet; pick a unary method",
        ));
    }

    let path = format!("/{}", rpc_path.trim().trim_start_matches('/'));
    let grpc_path = PathAndQuery::try_from(path)
        .map_err(|e| AppError::msg(format!("invalid rpc path: {e}")))?;

    let channel = Endpoint::from_shared(endpoint.to_string())
        .map_err(|e| AppError::msg(format!("invalid address: {e}")))?
        .connect_timeout(Duration::from_secs(connect_timeout_secs))
        .timeout(Duration::from_secs(timeout_secs))
        .connect()
        .await
        .map_err(|e| AppError::msg(format!("connection failed: {e}")))?;

    let mut client = tonic::client::Grpc::new(channel);
    client
        .ready()
        .await
        .map_err(|e| AppError::msg(format!("connection failed: {e}")))?;

    let mut request = Request::new(body);
    apply_request_metadata(&mut request, metadata)?;

    let started = Instant::now();
    let result = client.unary(request, grpc_path, BytesCodec).await;
    let duration_ms = started.elapsed().as_millis();

    match result {
        Ok(response) => {
            let (metadata, body, _ext) = response.into_parts();
            let mut headers = metadata_headers(&metadata);
            headers.push(("grpc-status".into(), "0".into()));
            headers.push(("grpc-message".into(), String::new()));
            Ok(GrpcCallResult {
                status: 200,
                status_text: "OK".into(),
                headers,
                body: decode_message_json(method, &body)?,
                duration_ms,
            })
        }
        Err(status) => {
            let mut headers = metadata_headers(status.metadata());
            let code_name = grpc_code_name(status.code());
            headers.push(("grpc-status".into(), grpc_status_number(status.code()).to_string()));
            headers.push(("grpc-message".into(), status.message().to_string()));
            let body = serde_json::to_string_pretty(&json!({
                "code": code_name,
                "message": status.message(),
            }))?;
            Ok(GrpcCallResult {
                status: grpc_http_status(status.code()),
                status_text: code_name.to_string(),
                headers,
                body,
                duration_ms,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_simple_proto_and_finds_method() {
        let src = r#"
            syntax = "proto3";
            package hello;
            service Greeter {
                rpc SayHello (HelloRequest) returns (HelloReply);
            }
            message HelloRequest { string name = 1; }
            message HelloReply { string message = 1; }
        "#;
        let pool = compile_descriptor_pool("hello.proto", src).unwrap();
        let method = find_method(&pool, "hello.Greeter/SayHello").unwrap();
        assert_eq!(method.name(), "SayHello");
        assert!(!method.is_client_streaming());
        let bytes = encode_message_json(&method, r#"{"name":"Ada"}"#).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn compiles_proto_with_google_api_annotations() {
        let src = r#"
            syntax = "proto3";
            package hello;
            import "google/api/annotations.proto";
            service Greeter {
                rpc SayHello (HelloRequest) returns (HelloReply) {
                    option (google.api.http) = {
                        post: "/v1/hello"
                        body: "*"
                    };
                }
            }
            message HelloRequest { string name = 1; }
            message HelloReply { string message = 1; }
        "#;
        let pool = compile_descriptor_pool("hello.proto", src).unwrap();
        let method = find_method(&pool, "hello.Greeter/SayHello").unwrap();
        assert_eq!(method.name(), "SayHello");
    }

    #[test]
    fn parses_endpoints() {
        assert_eq!(
            parse_grpc_endpoint("grpc://127.0.0.1:9000/", false).unwrap(),
            "http://127.0.0.1:9000"
        );
        assert_eq!(
            parse_grpc_endpoint("127.0.0.1:9000", false).unwrap(),
            "http://127.0.0.1:9000"
        );
        assert_eq!(
            parse_grpc_endpoint("https://example.com", true).unwrap(),
            "https://example.com:443"
        );
    }
}
