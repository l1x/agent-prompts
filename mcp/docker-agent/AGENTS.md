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
│   └── tools.rs          # MCP tool definitions (serde structs)
├── Cargo.toml            # Project configuration and dependencies
├── Cargo.lock            # Locked dependency versions (not committed for binary)
├── .gitignore            # Standard Rust ignore patterns
└── README.md             # Project documentation
```

**Note**: `Cargo.lock` is not committed as this is a binary crate (see `.gitignore` line 5-6).

### Module Responsibilities

- **main.rs**: Application entry point, parses CLI arguments with `argh`, initializes `tracing` logging
- **docker_manager.rs**: Core Docker operations using `bollard` (Docker HTTP API client)
  - Manages container state in memory using `Arc<RwLock<HashMap>>`
  - Handles container lifecycle: start, stop, logs, exec, list
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

The project structure is in place but core Docker operations are not yet implemented:

- `docker_manager.rs` has stub methods with `todo!()` for:
  - `start_container()`
  - `get_logs()`
  - `exec_command()`
  - `stop_container()`
  - `list_containers()`
  - `get_container()`

- `main.rs` references `mcp_server::run()` but `src/mcp_server.rs` does not exist yet

### Compilation Errors

The codebase currently has compilation errors that must be resolved before the server can run:

1. **Missing module**: `src/main.rs:7` - `mod mcp_server;` references non-existent file
   - Fix: Create `src/mcp_server.rs` or `src/mcp_server/mod.rs`

2. **EnvFilter type mismatch**: `src/main.rs:33` - `format!()` returns `String`, but needs `EnvFilter`
   - Fix: Add `.into()` conversion: `format!("docker-agent={}", args.log_level).into()`

3. **Argh docstring capitalization**: `src/main.rs:16` and `:20` - Docstrings must start with lowercase
   - Fix: Change `/// Docker socket path` to `/// docker socket path`

### Linter Warnings

Clippy reports 7 warnings about unused variables in stub functions:

```
warning: unused variable: `config`
warning: unused variable: `query`
warning: unused variable: `container_id` (multiple)
warning: unused variable: `command`
```

These are expected for stub implementations with `todo!()`. To suppress, prefix with underscore:
```rust
pub async fn start_container(&self, _config: StartConfig) -> Result<String, DockerError> {
    todo!()
}
```

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
2. **EnvFilter type**: `tracing_subscriber::EnvFilter::try_from_default_env()` returns `Result`, not `EnvFilter` directly. When providing a default, need `.into()` conversion: `format!("docker-agent={}", args.log_level).into()`
3. **Serde enum with tag/content**: MCP tools use `#[serde(tag = "name", content = "arguments")]` pattern
4. **Time crate version**: Using `time` 0.3, not the newer 0.4
5. **Missing mcp_server module**: The main.rs references a module that doesn't exist yet
6. **Clippy unused variable warnings**: Functions with `todo!()` implementations will trigger unused variable warnings. Prefix with underscore if intentional: `_config`, `_container_id`, etc.

## Development Workflow

1. Make changes to source files
2. Run `cargo check` to verify compilation
3. Run `cargo clippy` to catch common issues
   - Note: Stub functions with `todo!()` will generate unused variable warnings
   - Prefix with underscore (`_`) to suppress: `_config`, `_container_id`, etc.
4. Run `cargo fmt` to format code
   - Note: `cargo fmt` will fail if the code doesn't compile
5. Run `cargo test` to verify tests (once added)
6. Build and run locally with `cargo run -- --socket /var/run/docker.sock --log-level debug`
7. Test MCP tools by connecting from Claude Desktop

### Linting Notes

- **cargo fmt**: Requires code to compile successfully before running
- **cargo clippy**: Reports 7 warnings for unused variables in stub functions (expected)
- Use `#[allow(unused_variables)]` or underscore prefixes for intentional unused variables
