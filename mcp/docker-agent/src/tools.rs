//! MCP tool definitions for Docker operations

use serde::{Deserialize, Serialize};

/// Available Docker tools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name", content = "arguments")]
pub enum DockerTool {
    /// Start a new long-running container
    #[serde(rename = "docker_run")]
    DockerRun(DockerRunArgs),

    /// Fetch logs from a running container
    #[serde(rename = "docker_logs")]
    DockerLogs(DockerLogsArgs),

    /// Execute one-off command in container
    #[serde(rename = "docker_exec")]
    DockerExec(DockerExecArgs),

    /// Stop and remove a container
    #[serde(rename = "docker_stop")]
    DockerStop(DockerStopArgs),

    /// List all tracked containers
    #[serde(rename = "docker_list")]
    DockerList,
}

/// Arguments for starting a container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerRunArgs {
    /// Docker image to run
    pub image: String,
    /// Command to run (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    /// Environment variables (e.g., ["KEY=value"])
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_vars: Vec<String>,
    /// Volume mounts (host_path:container_path)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub volume_mounts: Vec<String>,
    /// Container name (optional, auto-generated if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Arguments for fetching logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerLogsArgs {
    /// Container ID
    pub container_id: String,
    /// Fetch logs since this ISO8601 timestamp (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    /// Number of lines from end (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tail_lines: Option<u64>,
    /// Include stdout (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<bool>,
    /// Include stderr (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<bool>,
}

/// Arguments for executing command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerExecArgs {
    /// Container ID
    pub container_id: String,
    /// Command to execute
    pub command: String,
}

/// Arguments for stopping container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerStopArgs {
    /// Container ID
    pub container_id: String,
}

/// Tool result response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Container run result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerRunResult {
    pub success: bool,
    pub container_id: String,
    pub message: String,
}

/// Logs fetch result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerLogsResult {
    pub success: bool,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    pub timestamp: Option<String>,
}
