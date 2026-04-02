use serde::{Deserialize, Serialize};

/// Worker status in the coordination system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    Running,
    Completed,
    Failed,
}

/// Information about a registered worker Claude Code instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    pub worker_id: String,
    pub task_description: String,
    pub cwd: String,
    pub status: WorkerStatus,
    pub registered_at: i64,
    pub result: Option<WorkerResult>,
}

/// Result reported by a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResult {
    pub status: WorkerStatus,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub reported_at: i64,
}

/// A pending permission prompt detected on a worker pane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPrompt {
    pub worker_pane_id: u64,
    pub prompt_text: String,
    pub detected_at: i64,
}

// --- JSON-RPC 2.0 ---

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcResponse {
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: serde_json::Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
            }),
        }
    }
}

/// MCP tool description for the `tools/list` response.
#[derive(Debug, Serialize)]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_status_serialization() {
        let json = serde_json::to_string(&WorkerStatus::Completed).unwrap();
        assert_eq!(json, "\"completed\"");
        let parsed: WorkerStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, WorkerStatus::Completed);
    }

    #[test]
    fn json_rpc_response_success() {
        let resp = JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"ok": true}));
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["result"]["ok"], true);
        assert!(json.get("error").is_none());
    }

    #[test]
    fn json_rpc_response_error() {
        let resp = JsonRpcResponse::error(serde_json::json!(2), -32601, "Not found");
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["error"]["code"], -32601);
        assert!(json.get("result").is_none());
    }
}
