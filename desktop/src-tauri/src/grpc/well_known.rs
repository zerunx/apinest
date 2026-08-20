use std::collections::HashMap;

use protox::file::{File, FileResolver};

const ANNOTATIONS_PROTO: &str = r#"
syntax = "proto3";
package google.api;
import "google/api/http.proto";
import "google/protobuf/descriptor.proto";
extend google.protobuf.MethodOptions {
  HttpRule http = 72295728;
}
"#;

const HTTP_PROTO: &str = r#"
syntax = "proto3";
package google.api;
message Http {
  repeated HttpRule rules = 1;
  bool fully_decode_reserved_expansion = 2;
}
message HttpRule {
  string selector = 1;
  oneof pattern {
    string get = 2;
    string put = 3;
    string post = 4;
    string delete = 5;
    string patch = 6;
    CustomHttpPattern custom = 8;
  }
  string body = 7;
  string response_body = 12;
  repeated HttpRule additional_bindings = 11;
}
message CustomHttpPattern {
  string kind = 1;
  string path = 2;
}
"#;

const FIELD_BEHAVIOR_PROTO: &str = r#"
syntax = "proto3";
package google.api;
import "google/protobuf/descriptor.proto";
extend google.protobuf.FieldOptions {
  repeated google.api.FieldBehavior field_behavior = 1052 [packed = false];
}
enum FieldBehavior {
  FIELD_BEHAVIOR_UNSPECIFIED = 0;
  OPTIONAL = 1;
  REQUIRED = 2;
  OUTPUT_ONLY = 3;
  INPUT_ONLY = 4;
  IMMUTABLE = 5;
  UNORDERED_LIST = 6;
  NON_EMPTY_DEFAULT = 7;
  IDENTIFIER = 8;
}
"#;

const HTTPBODY_PROTO: &str = r#"
syntax = "proto3";
package google.api;
import "google/protobuf/any.proto";
message HttpBody {
  string content_type = 1;
  bytes data = 2;
  repeated google.protobuf.Any extensions = 3;
}
"#;

const STATUS_PROTO: &str = r#"
syntax = "proto3";
package google.rpc;
import "google/protobuf/any.proto";
message Status {
  int32 code = 1;
  string message = 2;
  repeated google.protobuf.Any details = 3;
}
"#;

pub struct EmbeddedFileResolver {
    files: HashMap<&'static str, &'static str>,
}

impl EmbeddedFileResolver {
    pub fn google_common() -> Self {
        let mut files = HashMap::new();
        files.insert("google/api/annotations.proto", ANNOTATIONS_PROTO);
        files.insert("google/api/http.proto", HTTP_PROTO);
        files.insert("google/api/field_behavior.proto", FIELD_BEHAVIOR_PROTO);
        files.insert("google/api/httpbody.proto", HTTPBODY_PROTO);
        files.insert("google/rpc/status.proto", STATUS_PROTO);
        Self { files }
    }
}

impl FileResolver for EmbeddedFileResolver {
    fn open_file(&self, name: &str) -> Result<File, protox::Error> {
        match self.files.get(name) {
            Some(source) => File::from_source(name, source),
            None => Err(protox::Error::file_not_found(name)),
        }
    }
}
