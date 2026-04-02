use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::routing::post;
use axum::{Json, Router};
use tokio::sync::RwLock;

use crate::protocol::{
    JsonRpcRequest, JsonRpcResponse, McpToolInfo, PermissionPrompt, WorkerInfo, WorkerResult,
    WorkerStatus,
};

/// Shared state for the MCP coordination server.
pub struct CoordState {
    pub workers: RwLock<HashMap<String, WorkerInfo>>,
    pub permissions: RwLock<Vec<PermissionPrompt>>,
}

impl CoordState {
    pub fn new() -> Self {
        Self {
            workers: RwLock::new(HashMap::new()),
            permissions: RwLock::new(Vec::new()),
        }
    }
}

impl Default for CoordState {
    fn default() -> Self {
        Self::new()
    }
}

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Start the MCP HTTP server. Returns the actual bound address.
pub async fn start_server(state: Arc<CoordState>, port: u16) -> std::io::Result<SocketAddr> {
    let app = Router::new()
        .route("/mcp", post(handle_mcp_request))
        .with_state(state);

    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let bound = listener.local_addr()?;

    eprintln!("MCP server listening on http://{bound}/mcp");

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("MCP server error: {e}");
        }
    });

    Ok(bound)
}

async fn handle_mcp_request(
    axum::extract::State(state): axum::extract::State<Arc<CoordState>>,
    Json(req): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    let response = match req.method.as_str() {
        "initialize" => handle_initialize(&req),
        "tools/list" => handle_tools_list(&req),
        "tools/call" => handle_tools_call(&state, &req).await,
        _ => JsonRpcResponse::error(req.id, -32601, format!("Method not found: {}", req.method)),
    };
    Json(response)
}

fn handle_initialize(req: &JsonRpcRequest) -> JsonRpcResponse {
    JsonRpcResponse::success(
        req.id.clone(),
        serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": "clux-coord",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

fn handle_tools_list(req: &JsonRpcRequest) -> JsonRpcResponse {
    let tools = build_tool_definitions();
    JsonRpcResponse::success(req.id.clone(), serde_json::json!({ "tools": tools }))
}

async fn handle_tools_call(state: &CoordState, req: &JsonRpcRequest) -> JsonRpcResponse {
    let tool_name = req.params.get("name").and_then(|v| v.as_str());
    let args = req
        .params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    let Some(tool_name) = tool_name else {
        return JsonRpcResponse::error(req.id.clone(), -32602, "Missing 'name' parameter");
    };

    let result = match tool_name {
        "clux_register_worker" => tool_register_worker(state, &args).await,
        "clux_report_result" => tool_report_result(state, &args).await,
        "clux_list_workers" => tool_list_workers(state).await,
        "clux_check_permissions" => tool_check_permissions(state).await,
        _ => Err(format!("Unknown tool: {tool_name}")),
    };

    match result {
        Ok(content) => JsonRpcResponse::success(
            req.id.clone(),
            serde_json::json!({
                "content": [{ "type": "text", "text": content }]
            }),
        ),
        Err(e) => JsonRpcResponse::success(
            req.id.clone(),
            serde_json::json!({
                "content": [{ "type": "text", "text": e }],
                "isError": true
            }),
        ),
    }
}

// --- Tool implementations ---

async fn tool_register_worker(
    state: &CoordState,
    args: &serde_json::Value,
) -> Result<String, String> {
    let worker_id = args
        .get("worker_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'worker_id'")?;
    let task_description = args
        .get("task_description")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'task_description'")?;
    let cwd = args
        .get("cwd")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'cwd'")?;

    let info = WorkerInfo {
        worker_id: worker_id.to_string(),
        task_description: task_description.to_string(),
        cwd: cwd.to_string(),
        status: WorkerStatus::Running,
        registered_at: now_epoch(),
        result: None,
    };

    state
        .workers
        .write()
        .await
        .insert(worker_id.to_string(), info);

    Ok(format!("Worker '{worker_id}' registered"))
}

async fn tool_report_result(
    state: &CoordState,
    args: &serde_json::Value,
) -> Result<String, String> {
    let worker_id = args
        .get("worker_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'worker_id'")?;
    let status_str = args
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'status'")?;
    let summary = args
        .get("summary")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'summary'")?;
    let details = args.get("details").and_then(|v| v.as_str());

    let status = match status_str {
        "completed" => WorkerStatus::Completed,
        "failed" => WorkerStatus::Failed,
        _ => return Err(format!("Invalid status: '{status_str}'. Use 'completed' or 'failed'")),
    };

    let mut workers = state.workers.write().await;
    let worker = workers
        .get_mut(worker_id)
        .ok_or_else(|| format!("Worker '{worker_id}' not found"))?;

    worker.status = status;
    worker.result = Some(WorkerResult {
        status,
        summary: summary.to_string(),
        details: details.map(String::from),
        reported_at: now_epoch(),
    });

    Ok(format!("Result reported for worker '{worker_id}'"))
}

async fn tool_list_workers(state: &CoordState) -> Result<String, String> {
    let workers = state.workers.read().await;
    let list: Vec<&WorkerInfo> = workers.values().collect();
    serde_json::to_string_pretty(&list).map_err(|e| e.to_string())
}

async fn tool_check_permissions(state: &CoordState) -> Result<String, String> {
    let permissions = state.permissions.read().await;
    serde_json::to_string_pretty(&*permissions).map_err(|e| e.to_string())
}

// --- Tool definitions ---

fn build_tool_definitions() -> Vec<McpToolInfo> {
    vec![
        McpToolInfo {
            name: "clux_register_worker".into(),
            description: "Register this Claude Code instance as a worker. Call this when starting a task.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "worker_id": { "type": "string", "description": "Unique worker ID (use pane ID)" },
                    "task_description": { "type": "string", "description": "Description of the assigned task" },
                    "cwd": { "type": "string", "description": "Current working directory" }
                },
                "required": ["worker_id", "task_description", "cwd"]
            }),
        },
        McpToolInfo {
            name: "clux_report_result".into(),
            description: "Report the result of your task. Call this when your task is completed or failed.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "worker_id": { "type": "string", "description": "Your worker ID" },
                    "status": { "type": "string", "enum": ["completed", "failed"], "description": "Task outcome" },
                    "summary": { "type": "string", "description": "Brief summary of what was done" },
                    "details": { "type": "string", "description": "Detailed output or error message (optional)" }
                },
                "required": ["worker_id", "status", "summary"]
            }),
        },
        McpToolInfo {
            name: "clux_list_workers".into(),
            description: "List all registered workers and their current status/results.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpToolInfo {
            name: "clux_check_permissions".into(),
            description: "Check if any worker panes have pending permission prompts that need approval.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

// --- Internal API for terminal-side permission detection ---

/// Register a detected permission prompt. Called by the terminal's output monitor.
pub async fn register_permission_prompt(
    state: &CoordState,
    worker_pane_id: u64,
    prompt_text: String,
) {
    let prompt = PermissionPrompt {
        worker_pane_id,
        prompt_text,
        detected_at: now_epoch(),
    };
    state.permissions.write().await.push(prompt);
}

/// Clear a resolved permission prompt. Called after the central agent responds.
pub async fn clear_permission_prompt(state: &CoordState, worker_pane_id: u64) {
    state
        .permissions
        .write()
        .await
        .retain(|p| p.worker_pane_id != worker_pane_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> Arc<CoordState> {
        Arc::new(CoordState::new())
    }

    #[tokio::test]
    async fn initialize_response() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(1),
            method: "initialize".into(),
            params: serde_json::Value::Null,
        };
        let resp = handle_initialize(&req);
        let result = resp.result.unwrap();
        assert_eq!(result["serverInfo"]["name"], "clux-coord");
        assert_eq!(result["protocolVersion"], "2025-03-26");
    }

    #[tokio::test]
    async fn tools_list_has_all_tools() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(1),
            method: "tools/list".into(),
            params: serde_json::Value::Null,
        };
        let resp = handle_tools_list(&req);
        let tools = resp.result.unwrap()["tools"].as_array().unwrap().clone();
        assert_eq!(tools.len(), 4);
        let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        assert!(names.contains(&"clux_register_worker"));
        assert!(names.contains(&"clux_report_result"));
        assert!(names.contains(&"clux_list_workers"));
        assert!(names.contains(&"clux_check_permissions"));
    }

    #[tokio::test]
    async fn register_and_list_worker() {
        let state = test_state();

        let args = serde_json::json!({
            "worker_id": "pane-5",
            "task_description": "run tests",
            "cwd": "/project"
        });
        let result = tool_register_worker(&state, &args).await.unwrap();
        assert!(result.contains("pane-5"));

        let list = tool_list_workers(&state).await.unwrap();
        let workers: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap();
        assert_eq!(workers.len(), 1);
        assert_eq!(workers[0]["worker_id"], "pane-5");
        assert_eq!(workers[0]["status"], "running");
    }

    #[tokio::test]
    async fn report_result_updates_worker() {
        let state = test_state();

        let reg_args = serde_json::json!({
            "worker_id": "pane-5",
            "task_description": "run tests",
            "cwd": "/project"
        });
        tool_register_worker(&state, &reg_args).await.unwrap();

        let report_args = serde_json::json!({
            "worker_id": "pane-5",
            "status": "completed",
            "summary": "All tests passed",
            "details": "36 tests, 0 failures"
        });
        tool_report_result(&state, &report_args).await.unwrap();

        let list = tool_list_workers(&state).await.unwrap();
        let workers: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap();
        assert_eq!(workers[0]["status"], "completed");
        assert_eq!(workers[0]["result"]["summary"], "All tests passed");
    }

    #[tokio::test]
    async fn report_result_unknown_worker() {
        let state = test_state();
        let args = serde_json::json!({
            "worker_id": "nonexistent",
            "status": "completed",
            "summary": "done"
        });
        let result = tool_report_result(&state, &args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn report_result_invalid_status() {
        let state = test_state();
        let reg_args = serde_json::json!({
            "worker_id": "pane-1",
            "task_description": "test",
            "cwd": "/"
        });
        tool_register_worker(&state, &reg_args).await.unwrap();

        let args = serde_json::json!({
            "worker_id": "pane-1",
            "status": "invalid",
            "summary": "done"
        });
        let result = tool_report_result(&state, &args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn permission_prompt_lifecycle() {
        let state = test_state();

        register_permission_prompt(&state, 3, "Allow Bash(cargo test)?".into()).await;
        register_permission_prompt(&state, 5, "Allow Write(src/main.rs)?".into()).await;

        let list = tool_check_permissions(&state).await.unwrap();
        let prompts: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap();
        assert_eq!(prompts.len(), 2);

        clear_permission_prompt(&state, 3).await;

        let list = tool_check_permissions(&state).await.unwrap();
        let prompts: Vec<serde_json::Value> = serde_json::from_str(&list).unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0]["worker_pane_id"], 5);
    }

    #[tokio::test]
    async fn unknown_tool_returns_error() {
        let state = test_state();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(1),
            method: "tools/call".into(),
            params: serde_json::json!({ "name": "nonexistent" }),
        };
        let Json(resp) = handle_mcp_request(axum::extract::State(state), Json(req)).await;
        let result = resp.result.unwrap();
        assert_eq!(result["isError"], true);
    }

    #[tokio::test]
    async fn unknown_method_returns_error() {
        let state = test_state();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(1),
            method: "unknown/method".into(),
            params: serde_json::Value::Null,
        };
        let Json(resp) = handle_mcp_request(axum::extract::State(state), Json(req)).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn full_workflow() {
        let state = test_state();

        // Worker registers
        let args = serde_json::json!({
            "worker_id": "w1",
            "task_description": "cargo test -p clux-coord",
            "cwd": "/home/user/clux-term"
        });
        tool_register_worker(&state, &args).await.unwrap();

        // List shows running worker
        let list = tool_list_workers(&state).await.unwrap();
        assert!(list.contains("running"));

        // Worker reports success
        let args = serde_json::json!({
            "worker_id": "w1",
            "status": "completed",
            "summary": "All 10 tests passed"
        });
        tool_report_result(&state, &args).await.unwrap();

        // List shows completed worker with result
        let list = tool_list_workers(&state).await.unwrap();
        assert!(list.contains("completed"));
        assert!(list.contains("All 10 tests passed"));
    }
}
