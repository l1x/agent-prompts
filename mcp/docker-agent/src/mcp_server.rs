//! MCP (Model Context Protocol) server implementation
//!
//! Handles JSON-RPC 2.0 over stdio for tool discovery and execution.

use crate::docker_manager::{DockerManager, LogQuery, StartConfig};
use crate::tools::{
    DockerExecArgs, DockerLogsArgs, DockerLogsResult, DockerRunArgs, DockerRunResult,
    DockerStopArgs, ToolResult,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub struct JsonRpcRequest {
    #[allow(dead_code)] // Required by JSON-RPC spec but we don't validate the version
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

/// MCP tool definition for tools/list response
#[derive(Debug, Serialize)]
struct McpTool {
    name: &'static str,
    description: &'static str,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

/// Run the MCP server loop
pub async fn run(manager: DockerManager) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    tracing::info!("MCP server ready, waiting for requests");

    for line in stdin.lock().lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        tracing::info!(request = %line, "Received JSON-RPC request");

        let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => handle_request(&manager, request).await,
            Err(e) => {
                tracing::error!(error = %e, raw_request = %line, "Failed to parse JSON-RPC request");
                JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e))
            }
        };

        let output = serde_json::to_string(&response)?;
        writeln!(stdout, "{}", output)?;
        stdout.flush()?;
    }

    Ok(())
}

/// Handle a single JSON-RPC request
async fn handle_request(manager: &DockerManager, request: JsonRpcRequest) -> JsonRpcResponse {
    tracing::info!(method = %request.method, id = ?request.id, "Processing request");

    match request.method.as_str() {
        "initialize" => handle_initialize(request.id),
        "tools/list" => handle_tools_list(request.id),
        "tools/call" => handle_tools_call(manager, request.id, request.params).await,
        "ping" => JsonRpcResponse::success(request.id, json!({})),
        _ => JsonRpcResponse::error(request.id, -32601, "Method not found"),
    }
}

/// Handle initialize request
pub fn handle_initialize(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::success(
        id,
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "docker-agent",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

/// Handle tools/list request
pub fn handle_tools_list(id: Option<Value>) -> JsonRpcResponse {
    let tools = vec![
        McpTool {
            name: "docker_run",
            description: "Start a long-running Docker container",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "image": { "type": "string", "description": "Docker image to run" },
                    "command": { "type": "array", "items": { "type": "string" }, "description": "Command to run" },
                    "env_vars": { "type": "array", "items": { "type": "string" }, "description": "Environment variables (KEY=value)" },
                    "volume_mounts": { "type": "array", "items": { "type": "string" }, "description": "Volume mounts (host:container)" },
                    "name": { "type": "string", "description": "Container name" }
                },
                "required": ["image"]
            }),
        },
        McpTool {
            name: "docker_logs",
            description: "Fetch logs from a running container (supports incremental fetching)",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "container_id": { "type": "string", "description": "Container ID" },
                    "since": { "type": "string", "description": "ISO8601 timestamp to fetch logs since" },
                    "tail_lines": { "type": "integer", "description": "Number of lines from end" },
                    "stdout": { "type": "boolean", "description": "Include stdout (default: true)" },
                    "stderr": { "type": "boolean", "description": "Include stderr (default: true)" }
                },
                "required": ["container_id"]
            }),
        },
        McpTool {
            name: "docker_exec",
            description: "Execute a one-off command in a running container",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "container_id": { "type": "string", "description": "Container ID" },
                    "command": { "type": "string", "description": "Command to execute" }
                },
                "required": ["container_id", "command"]
            }),
        },
        McpTool {
            name: "docker_stop",
            description: "Stop and remove a container",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "container_id": { "type": "string", "description": "Container ID" }
                },
                "required": ["container_id"]
            }),
        },
        McpTool {
            name: "docker_list",
            description: "List all tracked containers",
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
    ];

    JsonRpcResponse::success(id, json!({ "tools": tools }))
}

/// Handle tools/call request
async fn handle_tools_call(
    manager: &DockerManager,
    id: Option<Value>,
    params: Value,
) -> JsonRpcResponse {
    let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    tracing::info!(tool = %tool_name, arguments = %arguments, "Executing tool");

    let result = match tool_name {
        "docker_run" => handle_docker_run(manager, arguments).await,
        "docker_logs" => handle_docker_logs(manager, arguments).await,
        "docker_exec" => handle_docker_exec(manager, arguments).await,
        "docker_stop" => handle_docker_stop(manager, arguments).await,
        "docker_list" => handle_docker_list(manager).await,
        _ => Err(format!("Unknown tool: {}", tool_name)),
    };

    match result {
        Ok(content) => {
            tracing::info!(tool = %tool_name, "Tool execution succeeded");
            JsonRpcResponse::success(
                id,
                json!({
                    "content": [{
                        "type": "text",
                        "text": content
                    }]
                }),
            )
        }
        Err(e) => {
            tracing::error!(tool = %tool_name, error = %e, "Tool execution failed");
            JsonRpcResponse::success(
                id,
                json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Error: {}", e)
                    }],
                    "isError": true
                }),
            )
        }
    }
}

async fn handle_docker_run(manager: &DockerManager, args: Value) -> Result<String, String> {
    let args: DockerRunArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;

    let config = StartConfig {
        image: args.image,
        command: args.command,
        env_vars: args.env_vars,
        volume_mounts: args
            .volume_mounts
            .iter()
            .filter_map(|m| {
                let parts: Vec<&str> = m.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect(),
        name: args.name,
    };

    let container_id = manager
        .start_container(config)
        .await
        .map_err(|e| e.to_string())?;

    let result = DockerRunResult {
        success: true,
        container_id: container_id.clone(),
        message: format!("Container started: {}", container_id),
    };

    serde_json::to_string(&result).map_err(|e| e.to_string())
}

async fn handle_docker_logs(manager: &DockerManager, args: Value) -> Result<String, String> {
    let args: DockerLogsArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;

    let query = LogQuery {
        container_id: args.container_id,
        since: args.since.and_then(|s| {
            time::OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339).ok()
        }),
        tail_lines: args.tail_lines,
        include_stdout: args.stdout.unwrap_or(true),
        include_stderr: args.stderr.unwrap_or(true),
    };

    let logs = manager.get_logs(query).await.map_err(|e| e.to_string())?;

    let result = DockerLogsResult {
        success: true,
        stdout: logs.stdout,
        stderr: logs.stderr,
        timestamp: logs.timestamp.map(|t| {
            t.format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default()
        }),
    };

    serde_json::to_string(&result).map_err(|e| e.to_string())
}

async fn handle_docker_exec(manager: &DockerManager, args: Value) -> Result<String, String> {
    let args: DockerExecArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;

    let output = manager
        .exec_command(&args.container_id, &args.command)
        .await
        .map_err(|e| e.to_string())?;

    let result = ToolResult {
        success: true,
        output,
        error: None,
    };

    serde_json::to_string(&result).map_err(|e| e.to_string())
}

async fn handle_docker_stop(manager: &DockerManager, args: Value) -> Result<String, String> {
    let args: DockerStopArgs = serde_json::from_value(args).map_err(|e| e.to_string())?;

    manager
        .stop_container(&args.container_id)
        .await
        .map_err(|e| e.to_string())?;

    let result = ToolResult {
        success: true,
        output: format!("Container {} stopped", args.container_id),
        error: None,
    };

    serde_json::to_string(&result).map_err(|e| e.to_string())
}

async fn handle_docker_list(manager: &DockerManager) -> Result<String, String> {
    let containers = manager.list_containers().await;

    let list: Vec<Value> = containers
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "name": c.name,
                "image": c.image,
                "started_at": c.started_at.format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
                "status": match &c.status {
                    crate::docker_manager::ContainerStatus::Running => "running",
                    crate::docker_manager::ContainerStatus::Stopped => "stopped",
                    crate::docker_manager::ContainerStatus::Exited(_) => "exited",
                    crate::docker_manager::ContainerStatus::Error(_) => "error",
                }
            })
        })
        .collect();

    serde_json::to_string(&json!({ "containers": list })).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(Some(json!(1)), json!({"result": "ok"}));

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
        assert_eq!(response.result, Some(json!({"result": "ok"})));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse::error(Some(json!(2)), -32600, "Invalid Request");

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(2)));
        assert!(response.result.is_none());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
    }

    #[test]
    fn test_json_rpc_request_parse() {
        let json_str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, Some(json!(1)));
        assert_eq!(request.method, "initialize");
    }

    #[test]
    fn test_json_rpc_request_parse_no_params() {
        let json_str = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();

        assert_eq!(request.method, "ping");
        assert_eq!(request.params, Value::Null);
    }

    #[test]
    fn test_json_rpc_request_parse_string_id() {
        let json_str = r#"{"jsonrpc":"2.0","id":"request-1","method":"tools/list"}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();

        assert_eq!(request.id, Some(json!("request-1")));
        assert_eq!(request.method, "tools/list");
    }

    #[test]
    fn test_handle_initialize() {
        let response = handle_initialize(Some(json!(1)));

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert!(result["capabilities"]["tools"].is_object());
        assert_eq!(result["serverInfo"]["name"], "docker-agent");
    }

    #[test]
    fn test_handle_initialize_null_id() {
        let response = handle_initialize(None);

        assert!(response.id.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn test_handle_tools_list() {
        let response = handle_tools_list(Some(json!(2)));

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(2)));
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();

        // Should have 5 tools
        assert_eq!(tools.len(), 5);

        // Check tool names
        let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(tool_names.contains(&"docker_run"));
        assert!(tool_names.contains(&"docker_logs"));
        assert!(tool_names.contains(&"docker_exec"));
        assert!(tool_names.contains(&"docker_stop"));
        assert!(tool_names.contains(&"docker_list"));
    }

    #[test]
    fn test_handle_tools_list_schema_structure() {
        let response = handle_tools_list(Some(json!(1)));
        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();

        // Find docker_run tool
        let docker_run = tools.iter().find(|t| t["name"] == "docker_run").unwrap();

        // Check schema structure
        assert!(docker_run.get("description").is_some());
        assert!(docker_run.get("inputSchema").is_some());

        let schema = &docker_run["inputSchema"];
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["image"].is_object());
        assert_eq!(schema["required"], json!(["image"]));
    }

    #[test]
    fn test_json_rpc_response_serialization() {
        let response = JsonRpcResponse::success(Some(json!(1)), json!({"status": "ok"}));
        let json_str = serde_json::to_string(&response).unwrap();

        // Parse back and verify
        let parsed: Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 1);
        assert_eq!(parsed["result"]["status"], "ok");
        // error should not be present (skip_serializing_if)
        assert!(parsed.get("error").is_none());
    }

    #[test]
    fn test_json_rpc_error_serialization() {
        let response = JsonRpcResponse::error(Some(json!(1)), -32601, "Method not found");
        let json_str = serde_json::to_string(&response).unwrap();

        let parsed: Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["error"]["code"], -32601);
        assert_eq!(parsed["error"]["message"], "Method not found");
        // result should not be present
        assert!(parsed.get("result").is_none());
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = serde_json::from_str::<JsonRpcRequest>("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_required_fields() {
        // Missing method field
        let result = serde_json::from_str::<JsonRpcRequest>(r#"{"jsonrpc":"2.0","id":1}"#);
        assert!(result.is_err());
    }
}
