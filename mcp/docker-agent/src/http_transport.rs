//! Streamable HTTP transport for MCP
//!
//! Implements the MCP Streamable HTTP transport specification with:
//! - Single endpoint supporting POST and GET
//! - SSE streams for server-to-client messages
//! - Session management with Mcp-Session-Id
//! - Security: Origin validation, localhost binding

use crate::docker_manager::DockerManager;
use crate::mcp_server::{JsonRpcRequest, JsonRpcResponse};
use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response, Sse, sse::Event},
    routing::{delete, get, post},
};
use serde_json::{Value, json};
use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{RwLock, broadcast};
use tokio_stream::StreamExt;
use uuid::Uuid;

/// Session state for a connected client
#[derive(Debug)]
#[allow(dead_code)] // id and created_at will be used for session management
pub struct Session {
    pub id: String,
    pub created_at: time::OffsetDateTime,
    /// Broadcast channel for sending SSE events to this session
    pub tx: broadcast::Sender<SseMessage>,
}

/// Message sent over SSE
#[derive(Debug, Clone)]
pub struct SseMessage {
    pub event_id: String,
    pub data: String,
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub manager: DockerManager,
    pub sessions: Arc<RwLock<HashMap<String, Session>>>,
    /// Allowed origins for CORS/security (None = allow all for local dev)
    pub allowed_origins: Option<Vec<String>>,
}

impl AppState {
    pub fn new(manager: DockerManager) -> Self {
        Self {
            manager,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            allowed_origins: None,
        }
    }

    /// Create a new session and return the session ID
    pub async fn create_session(&self) -> String {
        let session_id = Uuid::new_v4().to_string();
        let (tx, _rx) = broadcast::channel(100);

        let session = Session {
            id: session_id.clone(),
            created_at: time::OffsetDateTime::now_utc(),
            tx,
        };

        self.sessions
            .write()
            .await
            .insert(session_id.clone(), session);
        tracing::info!(session_id = %session_id, "Created new session");
        session_id
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<broadcast::Sender<SseMessage>> {
        self.sessions
            .read()
            .await
            .get(session_id)
            .map(|s| s.tx.clone())
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &str) -> bool {
        let removed = self.sessions.write().await.remove(session_id).is_some();
        if removed {
            tracing::info!(session_id = %session_id, "Session terminated");
        }
        removed
    }

    /// Check if session exists
    pub async fn session_exists(&self, session_id: &str) -> bool {
        self.sessions.read().await.contains_key(session_id)
    }
}

/// Custom header names for MCP
const MCP_SESSION_ID_HEADER: &str = "mcp-session-id";
#[allow(dead_code)] // Will be used for protocol version negotiation
const MCP_PROTOCOL_VERSION_HEADER: &str = "mcp-protocol-version";

/// Run the HTTP transport server
pub async fn run(
    manager: DockerManager,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::new(manager);

    let app = Router::new()
        .route("/mcp", post(handle_post))
        .route("/mcp", get(handle_get))
        .route("/mcp", delete(handle_delete))
        .with_state(state);

    tracing::info!("Starting MCP HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Validate Origin header for security
fn validate_origin(headers: &HeaderMap, allowed_origins: &Option<Vec<String>>) -> bool {
    // If no allowed origins configured, only allow localhost
    let origin = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok());

    match origin {
        None => true, // No origin header (e.g., curl, non-browser clients)
        Some(origin) => {
            if let Some(allowed) = allowed_origins {
                allowed.iter().any(|a| a == origin)
            } else {
                // Default: only allow localhost origins
                origin.starts_with("http://localhost")
                    || origin.starts_with("http://127.0.0.1")
                    || origin.starts_with("https://localhost")
                    || origin.starts_with("https://127.0.0.1")
            }
        }
    }
}

/// Extract session ID from headers
fn get_session_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get(MCP_SESSION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Handle POST requests - client sends JSON-RPC messages
async fn handle_post(State(state): State<AppState>, headers: HeaderMap, body: String) -> Response {
    // Validate Origin
    if !validate_origin(&headers, &state.allowed_origins) {
        return (StatusCode::FORBIDDEN, "Invalid origin").into_response();
    }

    tracing::info!(request = %body, "Received HTTP POST request");

    // Parse JSON-RPC request
    let request: JsonRpcRequest = match serde_json::from_str(&body) {
        Ok(req) => req,
        Err(e) => {
            tracing::error!(error = %e, raw_request = %body, "Failed to parse JSON-RPC request");
            let error = JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e));
            return (
                StatusCode::BAD_REQUEST,
                [(header::CONTENT_TYPE, "application/json")],
                serde_json::to_string(&error).unwrap_or_default(),
            )
                .into_response();
        }
    };

    tracing::info!(method = %request.method, id = ?request.id, "Processing HTTP request");

    // Handle initialize specially - creates session
    if request.method == "initialize" {
        let session_id = state.create_session().await;
        let response = crate::mcp_server::handle_initialize(request.id);
        let body = serde_json::to_string(&response).unwrap_or_default();

        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .header(MCP_SESSION_ID_HEADER, &session_id)
            .body(body)
            .unwrap()
            .into_response();
    }

    // For non-initialize requests, log session status (validation optional for dev)
    let session_id = get_session_id(&headers);
    match &session_id {
        Some(sid) => {
            if state.session_exists(sid).await {
                tracing::debug!(session_id = %sid, "Valid session");
            } else {
                tracing::warn!(session_id = %sid, "Stale session ID, proceeding anyway");
            }
        }
        None => {
            tracing::debug!("No session ID provided, proceeding without session");
        }
    }

    // Handle the request
    let response = handle_json_rpc_request(&state, request).await;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&response).unwrap_or_default(),
    )
        .into_response()
}

/// Handle a JSON-RPC request (reuses mcp_server logic)
async fn handle_json_rpc_request(state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => crate::mcp_server::handle_initialize(request.id),
        "tools/list" => crate::mcp_server::handle_tools_list(request.id),
        "tools/call" => handle_tools_call(state, request.id, request.params).await,
        "ping" => JsonRpcResponse::success(request.id, json!({})),
        _ => JsonRpcResponse::error(request.id, -32601, "Method not found"),
    }
}

/// Handle tools/call - delegates to docker operations
async fn handle_tools_call(state: &AppState, id: Option<Value>, params: Value) -> JsonRpcResponse {
    use crate::docker_manager::{LogQuery, StartConfig};
    use crate::tools::{
        DockerExecArgs, DockerLogsArgs, DockerLogsResult, DockerRunArgs, DockerRunResult,
        DockerStopArgs, ToolResult,
    };

    let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    tracing::info!(tool = %tool_name, arguments = %arguments, "Executing tool (HTTP)");

    let result: Result<String, String> = match tool_name {
        "docker_run" => {
            let args: DockerRunArgs = match serde_json::from_value(arguments) {
                Ok(a) => a,
                Err(e) => {
                    return JsonRpcResponse::error(id, -32602, format!("Invalid params: {}", e));
                }
            };

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

            match state.manager.start_container(config).await {
                Ok(container_id) => {
                    let result = DockerRunResult {
                        success: true,
                        container_id: container_id.clone(),
                        message: format!("Container started: {}", container_id),
                    };
                    serde_json::to_string(&result).map_err(|e| e.to_string())
                }
                Err(e) => Err(e.to_string()),
            }
        }
        "docker_logs" => {
            let args: DockerLogsArgs = match serde_json::from_value(arguments) {
                Ok(a) => a,
                Err(e) => {
                    return JsonRpcResponse::error(id, -32602, format!("Invalid params: {}", e));
                }
            };

            let query = LogQuery {
                container_id: args.container_id,
                since: args.since.and_then(|s| {
                    time::OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339)
                        .ok()
                }),
                tail_lines: args.tail_lines,
                include_stdout: args.stdout.unwrap_or(true),
                include_stderr: args.stderr.unwrap_or(true),
            };

            match state.manager.get_logs(query).await {
                Ok(logs) => {
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
                Err(e) => Err(e.to_string()),
            }
        }
        "docker_exec" => {
            let args: DockerExecArgs = match serde_json::from_value(arguments) {
                Ok(a) => a,
                Err(e) => {
                    return JsonRpcResponse::error(id, -32602, format!("Invalid params: {}", e));
                }
            };

            match state
                .manager
                .exec_command(&args.container_id, &args.command)
                .await
            {
                Ok(output) => {
                    let result = ToolResult {
                        success: true,
                        output,
                        error: None,
                    };
                    serde_json::to_string(&result).map_err(|e| e.to_string())
                }
                Err(e) => Err(e.to_string()),
            }
        }
        "docker_stop" => {
            let args: DockerStopArgs = match serde_json::from_value(arguments) {
                Ok(a) => a,
                Err(e) => {
                    return JsonRpcResponse::error(id, -32602, format!("Invalid params: {}", e));
                }
            };

            match state.manager.stop_container(&args.container_id).await {
                Ok(()) => {
                    let result = ToolResult {
                        success: true,
                        output: format!("Container {} stopped", args.container_id),
                        error: None,
                    };
                    serde_json::to_string(&result).map_err(|e| e.to_string())
                }
                Err(e) => Err(e.to_string()),
            }
        }
        "docker_list" => {
            let containers = state.manager.list_containers().await;
            let list: Vec<Value> = containers.iter().map(|c| {
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
            }).collect();
            serde_json::to_string(&json!({ "containers": list })).map_err(|e| e.to_string())
        }
        _ => Err(format!("Unknown tool: {}", tool_name)),
    };

    match result {
        Ok(content) => {
            tracing::info!(tool = %tool_name, "Tool execution succeeded (HTTP)");
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
            tracing::error!(tool = %tool_name, error = %e, "Tool execution failed (HTTP)");
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

/// Handle GET requests - opens SSE stream for server-to-client messages
async fn handle_get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    // Validate Origin
    if !validate_origin(&headers, &state.allowed_origins) {
        return (StatusCode::FORBIDDEN, "Invalid origin").into_response();
    }

    // Check Accept header
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !accept.contains("text/event-stream") {
        return (StatusCode::NOT_ACCEPTABLE, "Must accept text/event-stream").into_response();
    }

    // Get session ID
    let session_id = match get_session_id(&headers) {
        Some(sid) => sid,
        None => return (StatusCode::BAD_REQUEST, "Missing Mcp-Session-Id header").into_response(),
    };

    // Get session's broadcast channel
    let tx = match state.get_session(&session_id).await {
        Some(tx) => tx,
        None => return (StatusCode::NOT_FOUND, "Session not found").into_response(),
    };

    // Subscribe to the broadcast channel
    let rx = tx.subscribe();

    // Create SSE stream
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| {
        match result {
            Ok(msg) => Some(Ok::<_, Infallible>(
                Event::default().id(msg.event_id).data(msg.data),
            )),
            Err(_) => None, // Skip lagged messages
        }
    });

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text("ping"),
        )
        .into_response()
}

/// Handle DELETE requests - terminates session
async fn handle_delete(State(state): State<AppState>, headers: HeaderMap) -> Response {
    // Validate Origin
    if !validate_origin(&headers, &state.allowed_origins) {
        return (StatusCode::FORBIDDEN, "Invalid origin").into_response();
    }

    let session_id = match get_session_id(&headers) {
        Some(sid) => sid,
        None => return (StatusCode::BAD_REQUEST, "Missing Mcp-Session-Id header").into_response(),
    };

    if state.remove_session(&session_id).await {
        (StatusCode::OK, "Session terminated").into_response()
    } else {
        (StatusCode::NOT_FOUND, "Session not found").into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_origin_no_header() {
        let headers = HeaderMap::new();
        assert!(validate_origin(&headers, &None));
    }

    #[test]
    fn test_validate_origin_localhost() {
        let mut headers = HeaderMap::new();
        headers.insert(header::ORIGIN, "http://localhost:3000".parse().unwrap());
        assert!(validate_origin(&headers, &None));
    }

    #[test]
    fn test_validate_origin_127() {
        let mut headers = HeaderMap::new();
        headers.insert(header::ORIGIN, "http://127.0.0.1:8080".parse().unwrap());
        assert!(validate_origin(&headers, &None));
    }

    #[test]
    fn test_validate_origin_external_rejected() {
        let mut headers = HeaderMap::new();
        headers.insert(header::ORIGIN, "https://evil.com".parse().unwrap());
        assert!(!validate_origin(&headers, &None));
    }

    #[test]
    fn test_validate_origin_allowed_list() {
        let mut headers = HeaderMap::new();
        headers.insert(header::ORIGIN, "https://myapp.com".parse().unwrap());
        let allowed = Some(vec!["https://myapp.com".to_string()]);
        assert!(validate_origin(&headers, &allowed));
    }
}
