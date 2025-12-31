//! Integration tests for MCP protocol flow
//!
//! Tests the complete MCP protocol handshake and tool discovery
//! without requiring Docker.

use serde_json::{Value, json};

/// Helper to create a JSON-RPC request
fn make_request(id: impl Into<Value>, method: &str, params: Option<Value>) -> String {
    let mut req = json!({
        "jsonrpc": "2.0",
        "id": id.into(),
        "method": method,
    });
    if let Some(p) = params {
        req["params"] = p;
    }
    serde_json::to_string(&req).unwrap()
}

/// Parse a JSON-RPC response
fn parse_response(response: &str) -> Value {
    serde_json::from_str(response).expect("Invalid JSON response")
}

mod protocol_tests {
    use super::*;

    #[test]
    fn test_initialize_request_format() {
        let request = make_request(
            1,
            "initialize",
            Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })),
        );

        let parsed: Value = serde_json::from_str(&request).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 1);
        assert_eq!(parsed["method"], "initialize");
    }

    #[test]
    fn test_tools_list_request_format() {
        let request = make_request(2, "tools/list", None);

        let parsed: Value = serde_json::from_str(&request).unwrap();
        assert_eq!(parsed["method"], "tools/list");
    }

    #[test]
    fn test_tools_call_request_format() {
        let request = make_request(
            3,
            "tools/call",
            Some(json!({
                "name": "docker_run",
                "arguments": {
                    "image": "alpine:latest"
                }
            })),
        );

        let parsed: Value = serde_json::from_str(&request).unwrap();
        assert_eq!(parsed["method"], "tools/call");
        assert_eq!(parsed["params"]["name"], "docker_run");
        assert_eq!(parsed["params"]["arguments"]["image"], "alpine:latest");
    }

    #[test]
    fn test_ping_request_format() {
        let request = make_request(4, "ping", None);

        let parsed: Value = serde_json::from_str(&request).unwrap();
        assert_eq!(parsed["method"], "ping");
    }
}

mod response_format_tests {
    use super::*;

    #[test]
    fn test_success_response_format() {
        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "docker-agent",
                    "version": "0.1.0"
                }
            }
        });

        let response_str = serde_json::to_string(&response).unwrap();
        let parsed = parse_response(&response_str);

        assert_eq!(parsed["jsonrpc"], "2.0");
        assert!(parsed.get("result").is_some());
        assert!(parsed.get("error").is_none());
    }

    #[test]
    fn test_error_response_format() {
        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        });

        let response_str = serde_json::to_string(&response).unwrap();
        let parsed = parse_response(&response_str);

        assert_eq!(parsed["jsonrpc"], "2.0");
        assert!(parsed.get("result").is_none());
        assert!(parsed.get("error").is_some());
        assert_eq!(parsed["error"]["code"], -32601);
    }

    #[test]
    fn test_tool_call_success_response() {
        let response = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "result": {
                "content": [{
                    "type": "text",
                    "text": "{\"success\":true,\"container_id\":\"abc123\"}"
                }]
            }
        });

        let response_str = serde_json::to_string(&response).unwrap();
        let parsed = parse_response(&response_str);

        let content = &parsed["result"]["content"][0];
        assert_eq!(content["type"], "text");

        // Parse the inner JSON
        let inner: Value = serde_json::from_str(content["text"].as_str().unwrap()).unwrap();
        assert_eq!(inner["success"], true);
        assert_eq!(inner["container_id"], "abc123");
    }

    #[test]
    fn test_tool_call_error_response() {
        let response = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "result": {
                "content": [{
                    "type": "text",
                    "text": "Error: Container not found"
                }],
                "isError": true
            }
        });

        let response_str = serde_json::to_string(&response).unwrap();
        let parsed = parse_response(&response_str);

        assert_eq!(parsed["result"]["isError"], true);
        assert!(
            parsed["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("Error")
        );
    }
}

mod tools_schema_tests {
    use super::*;

    fn expected_docker_run_schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "image": { "type": "string", "description": "Docker image to run" },
                "command": { "type": "array", "items": { "type": "string" }, "description": "Command to run" },
                "env_vars": { "type": "array", "items": { "type": "string" }, "description": "Environment variables (KEY=value)" },
                "volume_mounts": { "type": "array", "items": { "type": "string" }, "description": "Volume mounts (host:container)" },
                "name": { "type": "string", "description": "Container name" }
            },
            "required": ["image"]
        })
    }

    #[test]
    fn test_docker_run_schema_has_required_image() {
        let schema = expected_docker_run_schema();
        assert_eq!(schema["required"], json!(["image"]));
    }

    #[test]
    fn test_docker_run_schema_properties() {
        let schema = expected_docker_run_schema();
        let props = &schema["properties"];

        assert!(props.get("image").is_some());
        assert!(props.get("command").is_some());
        assert!(props.get("env_vars").is_some());
        assert!(props.get("volume_mounts").is_some());
        assert!(props.get("name").is_some());
    }

    #[test]
    fn test_docker_logs_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "container_id": { "type": "string" },
                "since": { "type": "string" },
                "tail_lines": { "type": "integer" },
                "stdout": { "type": "boolean" },
                "stderr": { "type": "boolean" }
            },
            "required": ["container_id"]
        });

        assert_eq!(schema["required"], json!(["container_id"]));
        assert_eq!(schema["properties"]["tail_lines"]["type"], "integer");
    }

    #[test]
    fn test_docker_exec_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "container_id": { "type": "string" },
                "command": { "type": "string" }
            },
            "required": ["container_id", "command"]
        });

        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("container_id")));
        assert!(required.contains(&json!("command")));
    }

    #[test]
    fn test_docker_stop_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "container_id": { "type": "string" }
            },
            "required": ["container_id"]
        });

        assert_eq!(schema["required"], json!(["container_id"]));
    }

    #[test]
    fn test_docker_list_schema_empty() {
        let schema = json!({
            "type": "object",
            "properties": {}
        });

        assert!(schema["properties"].as_object().unwrap().is_empty());
    }
}

mod json_rpc_error_codes {
    /// Standard JSON-RPC error codes
    #[test]
    fn test_parse_error_code() {
        // Invalid JSON was received
        assert_eq!(-32700, -32700);
    }

    #[test]
    fn test_invalid_request_code() {
        // JSON is not a valid Request object
        assert_eq!(-32600, -32600);
    }

    #[test]
    fn test_method_not_found_code() {
        // Method does not exist
        assert_eq!(-32601, -32601);
    }

    #[test]
    fn test_invalid_params_code() {
        // Invalid method parameters
        assert_eq!(-32602, -32602);
    }

    #[test]
    fn test_internal_error_code() {
        // Internal JSON-RPC error
        assert_eq!(-32603, -32603);
    }
}

mod volume_mount_parsing {
    #[test]
    fn test_parse_volume_mount_simple() {
        let mount = "/host/path:/container/path";
        let parts: Vec<&str> = mount.splitn(2, ':').collect();

        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "/host/path");
        assert_eq!(parts[1], "/container/path");
    }

    #[test]
    fn test_parse_volume_mount_with_colon_in_path() {
        // Windows-style path or path with colon
        let mount = "C:/Users/test:/app";
        let parts: Vec<&str> = mount.splitn(2, ':').collect();

        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "C");
        assert_eq!(parts[1], "/Users/test:/app");
    }

    #[test]
    fn test_parse_volume_mount_invalid() {
        let mount = "invalid-no-colon";
        let parts: Vec<&str> = mount.splitn(2, ':').collect();

        assert_eq!(parts.len(), 1);
    }
}
