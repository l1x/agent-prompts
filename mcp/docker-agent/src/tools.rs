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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_docker_run_args_serialize_minimal() {
        let args = DockerRunArgs {
            image: "ubuntu:latest".to_string(),
            command: None,
            env_vars: vec![],
            volume_mounts: vec![],
            name: None,
        };

        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["image"], "ubuntu:latest");
        // Optional fields should be skipped when empty/None
        assert!(json.get("command").is_none());
        assert!(json.get("name").is_none());
    }

    #[test]
    fn test_docker_run_args_serialize_full() {
        let args = DockerRunArgs {
            image: "nginx:alpine".to_string(),
            command: Some(vec!["nginx".to_string(), "-g".to_string()]),
            env_vars: vec!["PORT=8080".to_string()],
            volume_mounts: vec!["/host:/container".to_string()],
            name: Some("my-nginx".to_string()),
        };

        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["image"], "nginx:alpine");
        assert_eq!(json["command"], json!(["nginx", "-g"]));
        assert_eq!(json["env_vars"], json!(["PORT=8080"]));
        assert_eq!(json["volume_mounts"], json!(["/host:/container"]));
        assert_eq!(json["name"], "my-nginx");
    }

    #[test]
    fn test_docker_run_args_deserialize() {
        let json = json!({
            "image": "redis:7",
            "command": ["redis-server"],
            "env_vars": ["REDIS_PORT=6379"],
            "volume_mounts": ["/data:/data"],
            "name": "my-redis"
        });

        let args: DockerRunArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.image, "redis:7");
        assert_eq!(args.command, Some(vec!["redis-server".to_string()]));
        assert_eq!(args.env_vars, vec!["REDIS_PORT=6379"]);
        assert_eq!(args.volume_mounts, vec!["/data:/data"]);
        assert_eq!(args.name, Some("my-redis".to_string()));
    }

    #[test]
    fn test_docker_logs_args_deserialize_minimal() {
        let json = json!({
            "container_id": "abc123"
        });

        let args: DockerLogsArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.container_id, "abc123");
        assert!(args.since.is_none());
        assert!(args.tail_lines.is_none());
        assert!(args.stdout.is_none());
        assert!(args.stderr.is_none());
    }

    #[test]
    fn test_docker_logs_args_deserialize_full() {
        let json = json!({
            "container_id": "abc123",
            "since": "2024-01-01T00:00:00Z",
            "tail_lines": 100,
            "stdout": true,
            "stderr": false
        });

        let args: DockerLogsArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.container_id, "abc123");
        assert_eq!(args.since, Some("2024-01-01T00:00:00Z".to_string()));
        assert_eq!(args.tail_lines, Some(100));
        assert_eq!(args.stdout, Some(true));
        assert_eq!(args.stderr, Some(false));
    }

    #[test]
    fn test_docker_exec_args() {
        let json = json!({
            "container_id": "xyz789",
            "command": "ls -la"
        });

        let args: DockerExecArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.container_id, "xyz789");
        assert_eq!(args.command, "ls -la");
    }

    #[test]
    fn test_docker_stop_args() {
        let json = json!({
            "container_id": "stop-me"
        });

        let args: DockerStopArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.container_id, "stop-me");
    }

    #[test]
    fn test_tool_result_serialize() {
        let result = ToolResult {
            success: true,
            output: "Command executed".to_string(),
            error: None,
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["output"], "Command executed");
        assert!(json.get("error").is_none());
    }

    #[test]
    fn test_tool_result_with_error() {
        let result = ToolResult {
            success: false,
            output: "".to_string(),
            error: Some("Container not found".to_string()),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["error"], "Container not found");
    }

    #[test]
    fn test_docker_run_result() {
        let result = DockerRunResult {
            success: true,
            container_id: "abc123def".to_string(),
            message: "Container started".to_string(),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["container_id"], "abc123def");
        assert_eq!(json["message"], "Container started");
    }

    #[test]
    fn test_docker_logs_result() {
        let result = DockerLogsResult {
            success: true,
            stdout: vec!["line1".to_string(), "line2".to_string()],
            stderr: vec!["error1".to_string()],
            timestamp: Some("2024-01-01T12:00:00Z".to_string()),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["stdout"], json!(["line1", "line2"]));
        assert_eq!(json["stderr"], json!(["error1"]));
        assert_eq!(json["timestamp"], "2024-01-01T12:00:00Z");
    }

    #[test]
    fn test_docker_tool_enum_serialize() {
        let tool = DockerTool::DockerRun(DockerRunArgs {
            image: "alpine".to_string(),
            command: None,
            env_vars: vec![],
            volume_mounts: vec![],
            name: None,
        });

        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["name"], "docker_run");
        assert_eq!(json["arguments"]["image"], "alpine");
    }

    #[test]
    fn test_docker_tool_enum_deserialize() {
        let json = json!({
            "name": "docker_exec",
            "arguments": {
                "container_id": "test123",
                "command": "whoami"
            }
        });

        let tool: DockerTool = serde_json::from_value(json).unwrap();
        match tool {
            DockerTool::DockerExec(args) => {
                assert_eq!(args.container_id, "test123");
                assert_eq!(args.command, "whoami");
            }
            _ => panic!("Expected DockerExec variant"),
        }
    }

    #[test]
    fn test_docker_list_variant() {
        let json = json!({
            "name": "docker_list"
        });

        let tool: DockerTool = serde_json::from_value(json).unwrap();
        assert!(matches!(tool, DockerTool::DockerList));
    }
}
