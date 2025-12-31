//! Docker Agent MCP Server
//!
//! Long-running Docker container manager accessible via MCP protocol

mod docker_manager;
mod mcp_server;
mod tools;

use argh::FromArgs;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(FromArgs, Debug)]
/// docker agent MCP server
struct Args {
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

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("docker-agent={}", args.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer().with_timer(
            tracing_subscriber::fmt::time::UtcTime::new(kiters::timestamp::get_utc_formatter()),
        ))
        .init();

    tracing::info!("Starting Docker Agent MCP Server");

    let manager = docker_manager::DockerManager::new().await?;
    mcp_server::run(manager).await?;

    Ok(())
}
