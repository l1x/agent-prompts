# AGENTS.md

This document provides guidance for AI agents working in the Docker Agent MCP Server codebase.

## Project Overview

This is a Rust-based MCP (Model Context Protocol) server for managing long-running Docker containers. The server exposes Docker operations as MCP tools that can be called by AI agents (like Claude) via JSON-RPC over stdio.

**Architecture**: AI Agent (Claude) → MCP Server → Docker Manager → Docker Daemon

## Essential Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Build with optimizations (release mode)
cargo build --release

# Run the MCP server with default settings
cargo run

# Run with custom Docker socket and log level
cargo run -- --socket /var/run/docker.sock --log-level info

# Run with debug logging
RUST_LOG=docker-agent=debug cargo run

# Run with specific log level for this crate only
RUST_LOG=docker-agent=trace cargo run

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Project Structure

```
docker-agent/
├── src/
│   ├── main.rs           # Entry point, CLI args, logging setup
│   ├── docker_manager.rs # Docker container lifecycle management
│   ├── mcp_server.rs     # MCP protocol (JSON-RPC over stdio)
│   └── tools.rs          # MCP tool definitions (serde structs)
├── Cargo.toml            # Project configuration and dependencies
├── Cargo.lock            # Locked dependency versions
├── .gitignore            # Standard Rust ignore patterns
└── README.md             # Project documentation
```

### Module Responsibilities

- **main.rs**: Application entry point, parses CLI arguments with `argh`, initializes `tracing` logging
- **docker_manager.rs**: Core Docker operations using `bollard` (Docker HTTP API client)
  - Manages container state in memory using `Arc<RwLock<HashMap>>`
  - Handles container lifecycle: start, stop, logs, exec, list
- **mcp_server.rs**: MCP protocol implementation
  - JSON-RPC 2.0 over stdio
  - Handles `initialize`, `tools/list`, `tools/call` requests
  - Routes tool calls to DockerManager methods
- **tools.rs**: MCP protocol tool definitions
  - Uses `serde` for JSON serialization/deserialization
  - Defines input/output structures for each Docker tool

## Code Conventions and Patterns

### Error Handling

All public functions return `Result<T, DockerError>` where `DockerError` is a custom enum:

```rust
#[derive(Debug, Error)]
pub enum DockerError {
    #[error("Failed to connect to Docker daemon: {0}")]
    Connection(#[from] bollard::errors::Error),

    #[error("Container not found: {0}")]
    ContainerNotFound(String),

    #[error("Container operation failed: {0}")]
    OperationFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
```

**Pattern**: Use `thiserror` derive macro for error types with automatic `From` conversions where appropriate.

### Async/Await

All I/O and Docker operations are async using `tokio` runtime. The application uses `#[tokio::main]` at the entry point.

### Serialization

Use `serde` for all MCP protocol structures:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerRunArgs {
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_vars: Vec<String>,
}
```

**Pattern**: Use `#[serde(skip_serializing_if = ...)]` for optional fields to avoid sending `null` values.

### State Management

The `DockerManager` uses `Arc<RwLock<HashMap<String, ContainerState>>>` for thread-safe, in-memory tracking of containers:

```rust
#[derive(Clone)]
pub struct DockerManager {
    docker: Docker,
    containers: Arc<RwLock<HashMap<String, ContainerState>>>,
}
```

**Pattern**: `Arc<RwLock<...>>` enables sharing state across async tasks with read/write locks.

### Time Handling

Use the `time` crate (0.3) with `OffsetDateTime` for timestamps:

```rust
pub struct ContainerState {
    pub started_at: OffsetDateTime,
}
```

Timestamps are formatted for ISO8601 output in logs.

### Logging

Use `tracing` for structured logging with the `RUST_LOG` environment variable for filtering:

```rust
tracing::info!("Starting Docker Agent MCP Server");
tracing::debug!("Container started: {}", container_id);
tracing::error!("Failed to start container: {}", error);
```

**Pattern**: Use appropriate log levels: `error!`, `warn!`, `info!`, `debug!`, `trace!`.

### CLI Arguments

Use `argh` for CLI argument parsing with documentation comments:

```rust
#[derive(FromArgs, Debug)]
/// Docker Agent MCP Server
struct Args {
    /// docker socket path (default: /var/run/docker.sock)
    #[argh(option, default = "String::from(\"/var/run/docker.sock\")")]
    socket: String,
}
```

**Important**: Argh requires documentation strings to start with lowercase letters (enforced by the library).

## MCP Tool Design

The server exposes these tools via MCP protocol:

| Tool | Purpose | Key Features |
|------|---------|--------------|
| `docker_run` | Start long-running container | Image, command, env vars, volumes, name |
| `docker_logs` | Fetch logs (incremental) | Supports since timestamps, tail lines, stdout/stderr selection |
| `docker_exec` | Execute one-off command | Runs command in running container |
| `docker_stop` | Stop/remove container | Cleanup container resources |
| `docker_list` | List tracked containers | Returns id, name, image, status, started_at |

**Key Design Principle**: Logs are fetched incrementally using timestamps to avoid re-fetching old log data.

## Key Dependencies

- `bollard`: Docker HTTP API client - all Docker operations
- `tokio`: Async runtime with full features
- `serde`/`serde_json`: JSON serialization for MCP protocol
- `thiserror`: Error derive macros
- `time`: Time handling with serde, formatting, and parsing features
- `tracing`/`tracing-subscriber`: Structured logging with env-filter support
- `argh`: CLI argument parsing
- `kiters`: Custom UTC timestamp formatting
- `futures`: Async utilities

## Current Implementation Status

The MCP server infrastructure is complete. The code compiles successfully.

**Implemented:**
- `mcp_server.rs` - Full MCP protocol with JSON-RPC 2.0
  - `initialize` - Server handshake
  - `tools/list` - Tool discovery with JSON schemas
  - `tools/call` - Tool execution routing
  - `ping` - Health check

**Stub implementations** (`docker_manager.rs` - will panic with `todo!()`):
- `start_container()` - Start long-running container
- `get_logs()` - Fetch container logs (incremental)
- `exec_command()` - Execute one-off command
- `stop_container()` - Stop and remove container
- `list_containers()` - List tracked containers
- `get_container()` - Get container by ID

### Linter Warnings

Clippy reports warnings about unused fields/variants in stub code. These are expected and will resolve when Docker operations are implemented.

## Code Style

- Use Rust 2024 edition
- Follow standard Rust naming conventions: `snake_case` for functions/variables, `PascalCase` for types
- Use `///` for public API documentation
- Use `//!` for module-level documentation
- All public structs and enums should derive `Debug`, `Clone`, and `Serialize`/`Deserialize` if used in MCP protocol
- Use descriptive error messages with context

## Testing

No test files exist yet. When adding tests:
- Use `cargo test` to run all tests
- Test modules should be in `tests/` directory or inline with `#[cfg(test)]`
- Test async code with `tokio::test`

## Docker Integration Notes

- Uses Docker HTTP API via Unix socket (`/var/run/docker.sock` by default)
- Containers are managed as long-running processes (not one-off)
- Volume mounts use format `host_path:container_path` as strings
- Environment variables use `KEY=value` format
- Container names are auto-generated if not provided

### Connecting from Claude Desktop

To use this MCP server with Claude Desktop:

```bash
# Add the MCP server to Claude configuration
claude mcp add docker-agent -- cargo run --bin docker-agent
```

The server communicates via JSON-RPC over stdio.

## Common Gotchas

1. **Argh docstring capitalization**: Argh requires `/// docker socket path` not `/// Docker socket path`
2. **EnvFilter type**: When providing a default filter, use `.into()`: `format!("docker-agent={}", level).into()`
3. **Serde enum with tag/content**: MCP tools use `#[serde(tag = "name", content = "arguments")]` pattern
4. **Time crate version**: Using `time` 0.3, not the newer 0.4
5. **Clippy unused variable warnings**: Functions with `todo!()` trigger warnings. Prefix with underscore: `_config`

## Development Workflow

1. Make changes to source files
2. Run `cargo check` to verify compilation
3. Run `cargo clippy` to catch common issues
4. Run `cargo fmt` to format code
5. Run `cargo test` to verify tests (once added)
6. Build and run locally with `cargo run -- --log-level debug`
7. Test MCP tools by connecting from Claude Desktop
