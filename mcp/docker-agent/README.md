# Docker Agent MCP Server

MCP (Model Context Protocol) server for managing long-running Docker containers.

## Architecture

```
AI Agent (Claude)
    │
    │ JSON-RPC over stdio
    ▼
MCP Server (this project)
    │
    ├── Tool: docker_run    ──► Start container
    ├── Tool: docker_logs   ◄─── Fetch logs (incremental)
    ├── Tool: docker_exec   ──► Run one-off command
    ├── Tool: docker_stop   ──► Stop/remove container
    └── Tool: docker_list   ◄─── List containers
    │
    ▼
Docker Manager (docker_manager.rs)
    │
    ├── Track containers in memory
    ├── Manage lifecycle (start/stop)
    └── Stream logs via Bollard
```

## Tools

| Tool | Input | Output |
|------|-------|--------|
| `docker_run` | image, command?, env_vars[], volume_mounts[], name? | `{container_id, message}` |
| `docker_logs` | container_id, since?, tail_lines?, stdout?, stderr? | `{stdout[], stderr[], timestamp?}` |
| `docker_exec` | container_id, command | `{exit_code, output}` |
| `docker_stop` | container_id | `{success}` |
| `docker_list` | (none) | `[{id, name, image, status, started_at}]` |

## Usage

```bash
# Run MCP server
cargo run -- --socket /var/run/docker.sock --log-level info

# Connect from Claude
claude mcp add docker-agent -- cargo run --bin docker-agent
```

## Development

```bash
# Build
cargo build

# Test
cargo test

# Run with debug logging
RUST_LOG=docker-agent=debug cargo run
```

## Key Design Decisions

1. **Long-running containers**: Containers stay alive between tool calls
2. **Incremental logs**: Use timestamps/cursors to fetch new logs only
3. **No Dagger**: Uses Bollard (Docker HTTP API) directly
4. **No git worktrees**: Works with fully checked-out repos in container
5. **No service management**: Focus on container lifecycle and log streaming

## Dependencies

- `bollard`: Docker HTTP API client
- `tokio`: Async runtime
- `argh`: CLI argument parsing
- `kiters`: Custom timestamp formatting
- `time`: Time handling
- `tracing` + `tracing-subscriber`: Logging
