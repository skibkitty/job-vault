use serde::{Deserialize, Serialize};

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

pub fn run() -> Result<(), String> {
    // TODO: read from stdin, dispatch operations, write to stdout
    Ok(())
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
}
