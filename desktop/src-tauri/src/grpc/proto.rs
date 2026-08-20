use serde::Serialize;
use serde_json::{json, Value};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
pub struct ParsedProto {
    pub package: String,
    pub proto_name: String,
    pub services: Vec<ParsedService>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParsedService {
    pub name: String,
    pub full_name: String,
    pub methods: Vec<ParsedMethod>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParsedMethod {
    pub name: String,
    pub rpc_path: String,
    pub service_name: String,
    pub request_type: String,
    pub response_type: String,
    pub client_streaming: bool,
    pub server_streaming: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
struct MessageDef {
    full_name: String,
    fields: Vec<FieldDef>,
}

#[derive(Debug, Clone)]
struct FieldDef {
    name: String,
    json_name: String,
    ty: String,
    repeated: bool,
    is_map: bool,
}

#[derive(Debug, Clone)]
struct EnumDef {
    full_name: String,
    first: String,
}

struct Lexer<'a> {
    src: &'a [u8],
    i: usize,
    line: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    StringLit(String),
    Number(String),
    Sym(char),
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src: src.as_bytes(),
            i: 0,
            line: 1,
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.src.get(self.i).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek_byte()?;
        self.i += 1;
        if b == b'\n' {
            self.line += 1;
        }
        Some(b)
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while matches!(self.peek_byte(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
                self.bump();
            }
            match (self.peek_byte(), self.src.get(self.i + 1).copied()) {
                (Some(b'/'), Some(b'/')) => {
                    while !matches!(self.peek_byte(), Some(b'\n') | None) {
                        self.bump();
                    }
                }
                (Some(b'/'), Some(b'*')) => {
                    self.bump();
                    self.bump();
                    while self.peek_byte().is_some() {
                        if self.peek_byte() == Some(b'*')
                            && self.src.get(self.i + 1).copied() == Some(b'/')
                        {
                            self.bump();
                            self.bump();
                            break;
                        }
                        self.bump();
                    }
                }
                _ => return,
            }
        }
    }

    fn next_tok(&mut self) -> AppResult<Option<Tok>> {
        self.skip_ws_and_comments();
        let Some(b) = self.peek_byte() else {
            return Ok(None);
        };
        if is_ident_start(b) {
            let mut s = String::new();
            while matches!(self.peek_byte(), Some(c) if is_ident_part(c)) {
                s.push(self.bump().unwrap() as char);
            }
            return Ok(Some(Tok::Ident(s)));
        }
        if b == b'"' || b == b'\'' {
            return Ok(Some(Tok::StringLit(self.read_string()?)));
        }
        if b.is_ascii_digit() || (b == b'-' && self.src.get(self.i + 1).is_some_and(|c| c.is_ascii_digit()))
        {
            let mut s = String::new();
            if b == b'-' {
                s.push(self.bump().unwrap() as char);
            }
            while matches!(self.peek_byte(), Some(c) if c.is_ascii_digit() || c == b'.' || c == b'x' || c == b'X' || (b'a'..=b'f').contains(&c) || (b'A'..=b'F').contains(&c))
            {
                s.push(self.bump().unwrap() as char);
            }
            return Ok(Some(Tok::Number(s)));
        }
        self.bump();
        Ok(Some(Tok::Sym(b as char)))
    }
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_part(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

impl<'a> Lexer<'a> {
    fn read_string(&mut self) -> AppResult<String> {
        let quote = self.bump().unwrap();
        let mut out = String::new();
        loop {
            let Some(b) = self.bump() else {
                return Err(AppError::msg(format!(
                    "unterminated string at line {}",
                    self.line
                )));
            };
            if b == quote {
                break;
            }
            if b == b'\\' {
                if let Some(n) = self.bump() {
                    out.push(n as char);
                }
                continue;
            }
            out.push(b as char);
        }
        Ok(out)
    }
}

struct Parser<'a> {
    lex: Lexer<'a>,
    peeked: Option<Tok>,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            lex: Lexer::new(src),
            peeked: None,
        }
    }

    fn peek(&mut self) -> AppResult<Option<Tok>> {
        if self.peeked.is_none() {
            self.peeked = self.lex.next_tok()?;
        }
        Ok(self.peeked.clone())
    }

    fn bump(&mut self) -> AppResult<Option<Tok>> {
        if let Some(t) = self.peeked.take() {
            return Ok(Some(t));
        }
        self.lex.next_tok()
    }

    fn expect_sym(&mut self, c: char) -> AppResult<()> {
        match self.bump()? {
            Some(Tok::Sym(x)) if x == c => Ok(()),
            other => Err(self.err(&format!("expected '{c}', got {other:?}"))),
        }
    }

    fn eat_sym(&mut self, c: char) -> AppResult<bool> {
        if matches!(self.peek()?, Some(Tok::Sym(x)) if x == c) {
            self.bump()?;
            return Ok(true);
        }
        Ok(false)
    }

    fn expect_ident(&mut self) -> AppResult<String> {
        match self.bump()? {
            Some(Tok::Ident(s)) => Ok(s),
            other => Err(self.err(&format!("expected identifier, got {other:?}"))),
        }
    }

    fn err(&self, msg: &str) -> AppError {
        AppError::msg(format!("proto parse error at line {}: {msg}", self.lex.line))
    }

    fn skip_balanced(&mut self, open: char, close: char) -> AppResult<()> {
        let mut depth = 1;
        while depth > 0 {
            match self.bump()? {
                Some(Tok::Sym(c)) if c == open => depth += 1,
                Some(Tok::Sym(c)) if c == close => depth -= 1,
                None => return Err(self.err("unclosed block")),
                _ => {}
            }
        }
        Ok(())
    }

    fn skip_option_or_stmt(&mut self) -> AppResult<()> {
        loop {
            match self.bump()? {
                Some(Tok::Sym(';')) => return Ok(()),
                Some(Tok::Sym('{')) => {
                    self.skip_balanced('{', '}')?;
                    self.eat_sym(';')?;
                    return Ok(());
                }
                None => return Err(self.err("unterminated statement")),
                _ => {}
            }
        }
    }

    fn parse_type_name(&mut self) -> AppResult<String> {
        let _ = self.eat_sym('.')?;
        let mut name = self.expect_ident()?;
        while self.eat_sym('.')? {
            name.push('.');
            name.push_str(&self.expect_ident()?);
        }
        Ok(name)
    }
}

fn qualify(package: &str, parent: &[String], name: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if !package.is_empty() {
        parts.push(package);
    }
    for p in parent {
        parts.push(p.as_str());
    }
    parts.push(name);
    parts.join(".")
}

fn service_full_name(package: &str, name: &str) -> String {
    if package.is_empty() {
        name.to_string()
    } else {
        format!("{package}.{name}")
    }
}

pub fn parse_proto(filename: &str, source: &str) -> AppResult<ParsedProto> {
    let mut p = Parser::new(source);
    let mut package = String::new();
    let mut messages: Vec<MessageDef> = Vec::new();
    let mut enums: Vec<EnumDef> = Vec::new();
    let mut services: Vec<ParsedService> = Vec::new();

    while p.peek()?.is_some() {
        match p.bump()? {
            Some(Tok::Ident(k)) if k == "syntax" || k == "edition" => {
                p.skip_option_or_stmt()?;
            }
            Some(Tok::Ident(k)) if k == "package" => {
                package = p.parse_type_name()?;
                p.expect_sym(';')?;
            }
            Some(Tok::Ident(k)) if k == "import" => {
                let _ = p.bump()?;
                p.skip_option_or_stmt()?;
            }
            Some(Tok::Ident(k)) if k == "option" => {
                p.skip_option_or_stmt()?;
            }
            Some(Tok::Ident(k)) if k == "message" => {
                parse_message(&mut p, &package, &[], &mut messages, &mut enums)?;
            }
            Some(Tok::Ident(k)) if k == "enum" => {
                parse_enum(&mut p, &package, &[], &mut enums)?;
            }
            Some(Tok::Ident(k)) if k == "service" => {
                services.push(parse_service(&mut p, &package)?);
            }
            Some(Tok::Ident(_)) => {
                p.skip_option_or_stmt()?;
            }
            Some(Tok::Sym(';')) => {}
            None => break,
            other => return Err(p.err(&format!("unexpected token {other:?}"))),
        }
    }

    if services.is_empty() {
        return Err(AppError::msg("未在 .proto 中找到 service"));
    }

    let proto_name = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(filename)
        .to_string();

    for svc in &mut services {
        for method in &mut svc.methods {
            let example = example_json(&method.request_type, &package, &messages, &enums);
            method.message = to_pretty_json(&example);
        }
    }

    Ok(ParsedProto {
        package,
        proto_name,
        services,
    })
}

fn parse_message(
    p: &mut Parser,
    package: &str,
    parent: &[String],
    messages: &mut Vec<MessageDef>,
    enums: &mut Vec<EnumDef>,
) -> AppResult<()> {
    let name = p.expect_ident()?;
    let full_name = qualify(package, parent, &name);
    p.expect_sym('{')?;
    let mut fields = Vec::new();
    let mut nested_parent = parent.to_vec();
    nested_parent.push(name);

    while !p.eat_sym('}')? {
        match p.peek()? {
            Some(Tok::Ident(k)) if k == "message" => {
                p.bump()?;
                parse_message(p, package, &nested_parent, messages, enums)?;
            }
            Some(Tok::Ident(k)) if k == "enum" => {
                p.bump()?;
                parse_enum(p, package, &nested_parent, enums)?;
            }
            Some(Tok::Ident(k)) if k == "oneof" => {
                p.bump()?;
                let _ = p.expect_ident()?;
                p.expect_sym('{')?;
                while !p.eat_sym('}')? {
                    if matches!(p.peek()?, Some(Tok::Ident(k)) if k == "option") {
                        p.bump()?;
                        p.skip_option_or_stmt()?;
                        continue;
                    }
                    fields.push(parse_field(p, false)?);
                }
            }
            Some(Tok::Ident(k)) if k == "option" || k == "reserved" || k == "extensions" => {
                p.bump()?;
                p.skip_option_or_stmt()?;
            }
            Some(Tok::Ident(k)) if k == "extend" => {
                p.bump()?;
                p.skip_option_or_stmt()?;
            }
            Some(Tok::Ident(_)) | Some(Tok::Sym('<')) => {
                fields.push(parse_field(p, false)?);
            }
            Some(Tok::Sym(';')) => {
                p.bump()?;
            }
            other => return Err(p.err(&format!("unexpected token in message {other:?}"))),
        }
    }

    messages.push(MessageDef { full_name, fields });
    Ok(())
}

fn parse_enum(
    p: &mut Parser,
    package: &str,
    parent: &[String],
    enums: &mut Vec<EnumDef>,
) -> AppResult<()> {
    let name = p.expect_ident()?;
    let full_name = qualify(package, parent, &name);
    p.expect_sym('{')?;
    let mut first = String::new();
    while !p.eat_sym('}')? {
        match p.bump()? {
            Some(Tok::Ident(k)) if k == "option" || k == "reserved" => {
                p.skip_option_or_stmt()?;
            }
            Some(Tok::Ident(k)) => {
                if first.is_empty() && k != "option" {
                    first = k;
                }
                p.skip_option_or_stmt()?;
            }
            Some(Tok::Sym(';')) => {}
            None => return Err(p.err("unclosed enum")),
            _ => {}
        }
    }
    enums.push(EnumDef { full_name, first });
    Ok(())
}

fn parse_field(p: &mut Parser, in_map: bool) -> AppResult<FieldDef> {
    let mut repeated = false;
    let mut is_map = false;
    let ty;

    loop {
        match p.peek()? {
            Some(Tok::Ident(k)) if k == "repeated" => {
                p.bump()?;
                repeated = true;
            }
            Some(Tok::Ident(k)) if k == "optional" || k == "required" => {
                p.bump()?;
            }
            Some(Tok::Ident(k)) if k == "map" => {
                p.bump()?;
                is_map = true;
                p.expect_sym('<')?;
                let map_key = p.parse_type_name()?;
                p.expect_sym(',')?;
                let map_value = p.parse_type_name()?;
                p.expect_sym('>')?;
                ty = format!("map<{map_key},{map_value}>");
                break;
            }
            _ => {
                ty = p.parse_type_name()?;
                break;
            }
        }
    }

    let name = if in_map {
        String::new()
    } else {
        p.expect_ident()?
    };
    let mut json_name = name.clone();
    p.expect_sym('=')?;
    match p.bump()? {
        Some(Tok::Number(_)) => {}
        other => return Err(p.err(&format!("expected field number, got {other:?}"))),
    }
    if p.eat_sym('[')? {
        loop {
            if p.eat_sym(']')? {
                break;
            }
            match p.peek()? {
                Some(Tok::Ident(k)) if k == "json_name" => {
                    p.bump()?;
                    p.expect_sym('=')?;
                    if let Some(Tok::StringLit(s)) = p.bump()? {
                        json_name = s;
                    }
                }
                Some(Tok::Sym(']')) => {
                    p.bump()?;
                    break;
                }
                Some(Tok::Sym(',')) => {
                    p.bump()?;
                }
                None => return Err(p.err("unclosed field options")),
                _ => {
                    p.bump()?;
                }
            }
        }
    }
    p.expect_sym(';')?;
    Ok(FieldDef {
        name,
        json_name,
        ty,
        repeated,
        is_map,
    })
}

fn parse_service(p: &mut Parser, package: &str) -> AppResult<ParsedService> {
    let name = p.expect_ident()?;
    let full_name = service_full_name(package, &name);
    p.expect_sym('{')?;
    let mut methods = Vec::new();
    while !p.eat_sym('}')? {
        match p.peek()? {
            Some(Tok::Ident(k)) if k == "option" => {
                p.bump()?;
                p.skip_option_or_stmt()?;
            }
            Some(Tok::Ident(k)) if k == "rpc" => {
                p.bump()?;
                methods.push(parse_rpc(p, &full_name)?);
            }
            Some(Tok::Sym(';')) => {
                p.bump()?;
            }
            other => return Err(p.err(&format!("unexpected token in service {other:?}"))),
        }
    }
    Ok(ParsedService {
        name,
        full_name,
        methods,
    })
}

fn parse_rpc(p: &mut Parser, service_full: &str) -> AppResult<ParsedMethod> {
    let name = p.expect_ident()?;
    p.expect_sym('(')?;
    let mut client_streaming = false;
    if matches!(p.peek()?, Some(Tok::Ident(k)) if k == "stream") {
        p.bump()?;
        client_streaming = true;
    }
    let request_type = p.parse_type_name()?;
    p.expect_sym(')')?;
    let returns = p.expect_ident()?;
    if returns != "returns" {
        return Err(p.err("expected 'returns'"));
    }
    p.expect_sym('(')?;
    let mut server_streaming = false;
    if matches!(p.peek()?, Some(Tok::Ident(k)) if k == "stream") {
        p.bump()?;
        server_streaming = true;
    }
    let response_type = p.parse_type_name()?;
    p.expect_sym(')')?;
    if p.eat_sym('{')? {
        p.skip_balanced('{', '}')?;
        p.eat_sym(';')?;
    } else {
        p.expect_sym(';')?;
    }
    Ok(ParsedMethod {
        name: name.clone(),
        rpc_path: format!("{service_full}/{name}"),
        service_name: service_full.to_string(),
        request_type,
        response_type,
        client_streaming,
        server_streaming,
        message: "{}".into(),
    })
}

fn example_json(ty: &str, package: &str, messages: &[MessageDef], enums: &[EnumDef]) -> Value {
    example_json_inner(ty, package, messages, enums, 0)
}

fn to_pretty_json(value: &Value) -> String {
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    match value.serialize(&mut ser) {
        Ok(()) => String::from_utf8(buf).unwrap_or_else(|_| "{}".into()),
        Err(_) => "{}".into(),
    }
}

fn example_json_inner(
    ty: &str,
    package: &str,
    messages: &[MessageDef],
    enums: &[EnumDef],
    depth: usize,
) -> Value {
    if depth > 8 {
        return json!({});
    }
    let ty = ty.trim_start_matches('.');
    if ty.starts_with("map<") {
        return json!({});
    }
    match scalar_example(ty) {
        Some(v) => return v,
        None => {}
    }
    if let Some(msg) = find_message(ty, package, messages) {
        let mut obj = serde_json::Map::new();
        for f in &msg.fields {
            let key = if f.json_name.is_empty() {
                f.name.clone()
            } else {
                f.json_name.clone()
            };
            if f.is_map {
                obj.insert(key, json!({}));
            } else if f.repeated {
                obj.insert(key, json!([]));
            } else {
                obj.insert(
                    key,
                    example_json_inner(&f.ty, package, messages, enums, depth + 1),
                );
            }
        }
        return Value::Object(obj);
    }
    if let Some(en) = find_enum(ty, package, enums) {
        if en.first.is_empty() {
            return json!(0);
        }
        return json!(en.first);
    }
    json!({})
}

fn scalar_example(ty: &str) -> Option<Value> {
    Some(match ty {
        "double" | "float" | "int32" | "int64" | "uint32" | "uint64" | "sint32" | "sint64"
        | "fixed32" | "fixed64" | "sfixed32" | "sfixed64" => json!(0),
        "bool" => json!(false),
        "string" | "bytes" => json!(""),
        "google.protobuf.Empty" => json!({}),
        "google.protobuf.Timestamp" | "google.protobuf.Duration" => json!(""),
        "google.protobuf.StringValue" | "google.protobuf.BytesValue" => json!(""),
        "google.protobuf.BoolValue" => json!(false),
        "google.protobuf.Int32Value"
        | "google.protobuf.Int64Value"
        | "google.protobuf.UInt32Value"
        | "google.protobuf.UInt64Value"
        | "google.protobuf.FloatValue"
        | "google.protobuf.DoubleValue" => json!(0),
        "google.protobuf.Struct" => json!({}),
        "google.protobuf.Value" => Value::Null,
        "google.protobuf.ListValue" => json!([]),
        _ => return None,
    })
}

fn find_message<'a>(ty: &str, package: &str, messages: &'a [MessageDef]) -> Option<&'a MessageDef> {
    messages.iter().find(|m| m.full_name == ty).or_else(|| {
        if !package.is_empty() {
            let q = format!("{package}.{ty}");
            if let Some(m) = messages.iter().find(|m| m.full_name == q) {
                return Some(m);
            }
        }
        messages.iter().find(|m| {
            m.full_name == ty
                || m.full_name.ends_with(&format!(".{ty}"))
                || m.full_name.rsplit('.').next() == Some(ty)
        })
    })
}

fn find_enum<'a>(ty: &str, package: &str, enums: &'a [EnumDef]) -> Option<&'a EnumDef> {
    enums.iter().find(|m| m.full_name == ty).or_else(|| {
        if !package.is_empty() {
            let q = format!("{package}.{ty}");
            if let Some(m) = enums.iter().find(|m| m.full_name == q) {
                return Some(m);
            }
        }
        enums.iter().find(|m| m.full_name.rsplit('.').next() == Some(ty))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_service_and_example() {
        let src = r#"
            syntax = "proto3";
            package product;
            message CreateReq { string name = 1; int32 price = 2; repeated string tags = 3; }
            service Product {
              rpc Create (CreateReq) returns (google.protobuf.Empty);
              rpc Items (stream CreateReq) returns (stream CreateReq) {}
            }
        "#;
        let parsed = parse_proto("product.proto", src).unwrap();
        assert_eq!(parsed.package, "product");
        assert_eq!(parsed.services.len(), 1);
        assert_eq!(parsed.services[0].full_name, "product.Product");
        assert_eq!(parsed.services[0].methods[0].rpc_path, "product.Product/Create");
        assert!(parsed.services[0].methods[0].message.contains("    \"name\""));
        assert!(parsed.services[0].methods[1].client_streaming);
        assert!(parsed.services[0].methods[1].server_streaming);
    }
}
