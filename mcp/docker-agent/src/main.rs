//! Docker Agent MCP Server
//!
//! Long-running Docker container manager accessible via MCP protocol.
//! Supports both stdio and Streamable HTTP transports.

mod docker_manager;
mod http_transport;
mod mcp_server;
mod tools;

use argh::FromArgs;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(FromArgs, Debug)]
/// docker agent MCP server
struct Args {
    /// transport type: "stdio" or "http" (default: stdio)
    #[argh(option, short = 't', default = "String::from(\"stdio\")")]
    transport: String,

    /// HTTP port for Streamable HTTP transport (default: 3000)
    #[argh(option, short = 'p', default = "3000")]
    port: u16,

    /// HTTP host to bind to (default: 127.0.0.1)
    #[argh(option, default = "String::from(\"127.0.0.1\")")]
    host: String,

    /// docker socket path (default: /var/run/docker.sock)
    #[argh(option, default = "String::from(\"/var/run/docker.sock\")")]
    socket: String,

    /// log level (default: info)
    #[argh(option, default = "String::from(\"info\")")]
    log_level: String,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Args = argh::from_env();

    // For HTTP transport, log to stderr so stdout isn't polluted
    // For stdio transport, also log to stderr (stdout is for JSON-RPC)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("docker_agent={}", args.log_level).into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_timer(tracing_subscriber::fmt::time::UtcTime::new(
                    kiters::timestamp::get_utc_formatter(),
                )),
        )
        .init();

    tracing::info!("Starting Docker Agent MCP Server");
    tracing::info!(socket = %args.socket, "Connecting to Docker daemon");

    let manager = docker_manager::DockerManager::new_with_socket(&args.socket).await?;

    match args.transport.as_str() {
        "stdio" => {
            tracing::info!("Using stdio transport");
            mcp_server::run(manager).await?;
        }
        "http" => {
            let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
            tracing::info!("Using HTTP transport on {}", addr);
            http_transport::run(manager, addr).await?;
        }
        other => {
            eprintln!("Unknown transport: {}. Use 'stdio' or 'http'.", other);
            std::process::exit(1);
        }
    }

    Ok(())
}
