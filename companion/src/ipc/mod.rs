use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

pub const PROTOCOL_VERSION: u32 = 1;

const MAX_PAYLOAD_BYTES: usize = 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcRequest {
    pub protocol_version: u32,
    pub request_id: String,
    pub operation: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcResponse {
    pub protocol_version: u32,
    pub request_id: String,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<IpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    MissingField(&'static str),
    InvalidProtocolVersion(u32),
    PayloadTooLarge(usize),
    MalformedJson(String),
    UnknownOperation(String),
}

impl IpcResponse {
    pub fn success(request_id: &str, data: serde_json::Value) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            request_id: request_id.to_string(),
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(request_id: &str, code: &str, message: &str) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            request_id: request_id.to_string(),
            success: false,
            data: None,
            error: Some(IpcError {
                code: code.to_string(),
                message: message.to_string(),
            }),
        }
    }
}

pub fn validate_request(request: &IpcRequest) -> Result<(), ValidationError> {
    if request.request_id.is_empty() {
        return Err(ValidationError::MissingField("request_id"));
    }

    if request.operation.is_empty() {
        return Err(ValidationError::MissingField("operation"));
    }

    if request.protocol_version != PROTOCOL_VERSION {
        return Err(ValidationError::InvalidProtocolVersion(
            request.protocol_version,
        ));
    }

    let payload_str = serde_json::to_string(&request.payload).unwrap_or_default();
    if payload_str.len() > MAX_PAYLOAD_BYTES {
        return Err(ValidationError::PayloadTooLarge(payload_str.len()));
    }

    Ok(())
}

pub fn is_known_operation(operation: &str) -> bool {
    matches!(
        operation,
        "vault.status"
            | "vault.unlock"
            | "vault.lock"
            | "job.save"
            | "job.get"
            | "job.list"
            | "job.compare"
            | "job.search"
    )
}

pub fn dispatch(request: &IpcRequest) -> IpcResponse {
    if let Err(e) = validate_request(request) {
        let (code, message) = match e {
            ValidationError::MissingField(f) => ("MISSING_FIELD", format!("Missing required field: {f}")),
            ValidationError::InvalidProtocolVersion(v) => {
                ("INVALID_PROTOCOL_VERSION", format!("Unsupported protocol version: {v}"))
            }
            ValidationError::PayloadTooLarge(s) => {
                ("PAYLOAD_TOO_LARGE", format!("Payload size {s} exceeds maximum {MAX_PAYLOAD_BYTES}"))
            }
            ValidationError::MalformedJson(m) => ("MALFORMED_JSON", m),
            ValidationError::UnknownOperation(o) => ("UNKNOWN_OPERATION", format!("Unknown operation: {o}")),
        };
        return IpcResponse::error(&request.request_id, code, &message);
    }

    if !is_known_operation(&request.operation) {
        return IpcResponse::error(
            &request.request_id,
            "UNKNOWN_OPERATION",
            &format!("Unknown operation: {}", request.operation),
        );
    }

    match request.operation.as_str() {
        "vault.status" => IpcResponse::success(
            &request.request_id,
            serde_json::json!({ "state": "locked" }),
        ),
        "vault.lock" => IpcResponse::success(
            &request.request_id,
            serde_json::json!({ "state": "locked" }),
        ),
        "job.save" => IpcResponse::success(
            &request.request_id,
            serde_json::json!({ "saved": true }),
        ),
        "job.get" | "job.list" | "job.compare" | "job.search" | "vault.unlock" => {
            IpcResponse::success(
                &request.request_id,
                serde_json::json!({ "status": "not_implemented" }),
            )
        }
        _ => unreachable!(),
    }
}

fn read_length_prefixed(reader: &mut impl Read) -> Result<Vec<u8>, String> {
    let mut len_bytes = [0u8; 4];
    reader
        .read_exact(&mut len_bytes)
        .map_err(|e| format!("Failed to read length: {e}"))?;
    let len = u32::from_ne_bytes(len_bytes) as usize;

    if len > MAX_PAYLOAD_BYTES {
        return Err(format!("Payload length {len} exceeds maximum {MAX_PAYLOAD_BYTES}"));
    }

    let mut buf = vec![0u8; len];
    reader
        .read_exact(&mut buf)
        .map_err(|e| format!("Failed to read payload: {e}"))?;
    Ok(buf)
}

fn write_length_prefixed(writer: &mut impl Write, data: &[u8]) -> Result<(), String> {
    let len = (data.len() as u32).to_ne_bytes();
    writer
        .write_all(&len)
        .map_err(|e| format!("Failed to write length: {e}"))?;
    writer
        .write_all(data)
        .map_err(|e| format!("Failed to write payload: {e}"))?;
    writer
        .flush()
        .map_err(|e| format!("Failed to flush: {e}"))
}

pub fn read_request(reader: &mut impl Read) -> Result<IpcRequest, String> {
    let bytes = read_length_prefixed(reader)?;
    let request: IpcRequest =
        serde_json::from_slice(&bytes).map_err(|e| format!("Malformed JSON: {e}"))?;
    Ok(request)
}

pub fn write_response(writer: &mut impl Write, response: &IpcResponse) -> Result<(), String> {
    let bytes = serde_json::to_vec(response).map_err(|e| format!("Failed to serialize: {e}"))?;
    write_length_prefixed(writer, &bytes)
}

pub fn run() -> Result<(), String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    loop {
        let request = match read_request(&mut reader) {
            Ok(r) => r,
            Err(e) => {
                let response = IpcResponse::error("unknown", "READ_ERROR", &e);
                write_response(&mut writer, &response)?;
                continue;
            }
        };

        let response = dispatch(&request);
        write_response(&mut writer, &response)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serializes() {
        let req = IpcRequest {
            protocol_version: 1,
            request_id: "test-1".into(),
            operation: "vault.status".into(),
            payload: serde_json::json!({}),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("vault.status"));
    }

    #[test]
    fn validate_accepts_valid_request() {
        let req = IpcRequest {
            protocol_version: 1,
            request_id: "r1".into(),
            operation: "vault.status".into(),
            payload: serde_json::json!({}),
        };
        assert!(validate_request(&req).is_ok());
    }

    #[test]
    fn validate_rejects_empty_request_id() {
        let req = IpcRequest {
            protocol_version: 1,
            request_id: "".into(),
            operation: "vault.status".into(),
            payload: serde_json::json!({}),
        };
        assert_eq!(
            validate_request(&req),
            Err(ValidationError::MissingField("request_id"))
        );
    }

    #[test]
    fn validate_rejects_empty_operation() {
        let req = IpcRequest {
            protocol_version: 1,
            request_id: "r1".into(),
            operation: "".into(),
            payload: serde_json::json!({}),
        };
        assert_eq!(
            validate_request(&req),
            Err(ValidationError::MissingField("operation"))
        );
    }

    #[test]
    fn validate_rejects_wrong_protocol_version() {
        let req = IpcRequest {
            protocol_version: 99,
            request_id: "r1".into(),
            operation: "vault.status".into(),
            payload: serde_json::json!({}),
        };
        assert_eq!(
            validate_request(&req),
            Err(ValidationError::InvalidProtocolVersion(99))
        );
    }

    #[test]
    fn dispatch_rejects_unknown_operation() {
        let req = IpcRequest {
            protocol_version: 1,
            request_id: "r1".into(),
            operation: "system.exec".into(),
            payload: serde_json::json!({}),
        };
        let resp = dispatch(&req);
        assert!(!resp.success);
        assert_eq!(resp.error.as_ref().unwrap().code, "UNKNOWN_OPERATION");
    }

    #[test]
    fn dispatch_accepts_known_operation() {
        let req = IpcRequest {
            protocol_version: 1,
            request_id: "r1".into(),
            operation: "vault.status".into(),
            payload: serde_json::json!({}),
        };
        let resp = dispatch(&req);
        assert!(resp.success);
    }

    #[test]
    fn dispatch_returns_error_for_invalid_request() {
        let req = IpcRequest {
            protocol_version: 1,
            request_id: "".into(),
            operation: "vault.status".into(),
            payload: serde_json::json!({}),
        };
        let resp = dispatch(&req);
        assert!(!resp.success);
        assert_eq!(resp.error.as_ref().unwrap().code, "MISSING_FIELD");
    }

    #[test]
    fn response_success_has_data() {
        let resp = IpcResponse::success("r1", serde_json::json!({"ok": true}));
        assert!(resp.success);
        assert!(resp.data.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn response_error_has_error() {
        let resp = IpcResponse::error("r1", "ERR", "boom");
        assert!(!resp.success);
        assert!(resp.data.is_none());
        assert!(resp.error.is_some());
    }

    #[test]
    fn roundtrip_length_prefixed() {
        let data = b"hello world";
        let mut buf = Vec::new();
        write_length_prefixed(&mut buf, data).unwrap();

        let mut reader = &buf[..];
        let result = read_length_prefixed(&mut reader).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn roundtrip_request_response() {
        let req = IpcRequest {
            protocol_version: 1,
            request_id: "rt-1".into(),
            operation: "vault.status".into(),
            payload: serde_json::json!({}),
        };

        let mut buf = Vec::new();
        let req_bytes = serde_json::to_vec(&req).unwrap();
        write_length_prefixed(&mut buf, &req_bytes).unwrap();

        let mut reader = &buf[..];
        let read_req = read_request(&mut reader).unwrap();
        assert_eq!(read_req.request_id, "rt-1");

        let resp = dispatch(&read_req);
        let mut out = Vec::new();
        write_response(&mut out, &resp).unwrap();

        let mut reader = &out[..];
        let resp_bytes = read_length_prefixed(&mut reader).unwrap();
        let read_resp: IpcResponse = serde_json::from_slice(&resp_bytes).unwrap();
        assert!(read_resp.success);
    }
}
