//! Docker container lifecycle and log management

use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
use bollard::models::HostConfig;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use time::OffsetDateTime;
use tokio::sync::RwLock;

/// Errors that can occur during Docker operations
#[derive(Debug, Error)]
#[allow(dead_code)] // Variants will be used as operations are implemented
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

/// Manages long-running Docker containers
#[derive(Clone)]
pub struct DockerManager {
    docker: Docker,
    containers: Arc<RwLock<HashMap<String, ContainerState>>>,
}

/// State of a tracked container
#[derive(Clone)]
pub struct ContainerState {
    pub id: String,
    pub name: String,
    pub image: String,
    pub started_at: OffsetDateTime,
    pub status: ContainerStatus,
}

/// Container lifecycle status
#[derive(Clone)]
#[allow(dead_code)] // Variants will be used as container lifecycle tracking is implemented
pub enum ContainerStatus {
    Running,
    Stopped,
    Exited(i32), // Exit code
    Error(String),
}

/// Configuration for starting a new container
pub struct StartConfig {
    pub image: String,
    pub command: Option<Vec<String>>,
    pub env_vars: Vec<String>,
    pub volume_mounts: Vec<(String, String)>, // (host_path, container_path)
    pub name: Option<String>,
}

/// Log query options for incremental fetching
#[allow(dead_code)] // Fields will be used when log fetching is implemented
pub struct LogQuery {
    pub container_id: String,
    pub since: Option<OffsetDateTime>,
    pub tail_lines: Option<u64>,
    pub include_stdout: bool,
    pub include_stderr: bool,
}

/// Log output from container
pub struct LogsOutput {
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    pub timestamp: Option<OffsetDateTime>,
}

impl DockerManager {
    /// Create new Docker manager with connection to Docker daemon using default socket
    #[allow(dead_code)] // Convenience method for when default socket is acceptable
    pub async fn new() -> Result<Self, DockerError> {
        let docker = Docker::connect_with_socket_defaults()?;
        Ok(Self {
            docker,
            containers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create new Docker manager with connection to a specific socket path
    pub async fn new_with_socket(socket_path: &str) -> Result<Self, DockerError> {
        let docker = Docker::connect_with_socket(socket_path, 120, bollard::API_DEFAULT_VERSION)?;
        Ok(Self {
            docker,
            containers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Start a long-running container
    pub async fn start_container(&self, config: StartConfig) -> Result<String, DockerError> {
        // Parse volume mounts into Docker bind format
        let binds: Vec<String> = config
            .volume_mounts
            .iter()
            .map(|(host, container)| format!("{}:{}", host, container))
            .collect();

        let host_config = HostConfig {
            binds: if binds.is_empty() { None } else { Some(binds) },
            ..Default::default()
        };

        // Expand environment variables: if entry has no '=', read from server env
        let expanded_env: Vec<String> = config
            .env_vars
            .iter()
            .filter_map(|e| {
                if e.contains('=') {
                    Some(e.clone())
                } else {
                    // No '=' means read from server environment
                    std::env::var(e).ok().map(|val| format!("{}={}", e, val))
                }
            })
            .collect();

        let container_config = Config {
            image: Some(config.image.clone()),
            cmd: config.command.clone(),
            env: if expanded_env.is_empty() {
                None
            } else {
                Some(expanded_env)
            },
            host_config: Some(host_config),
            tty: Some(true),
            open_stdin: Some(true),
            ..Default::default()
        };

        // Create container with optional name
        let create_options = config.name.as_ref().map(|n| CreateContainerOptions {
            name: n.as_str(),
            platform: None,
        });

        let container = self
            .docker
            .create_container(create_options, container_config)
            .await
            .map_err(DockerError::Connection)?;

        let container_id = container.id.clone();

        // Start the container
        self.docker
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(DockerError::Connection)?;

        // Track container state
        let container_name = config
            .name
            .unwrap_or_else(|| container_id[..12].to_string());

        let state = ContainerState {
            id: container_id.clone(),
            name: container_name,
            image: config.image,
            started_at: OffsetDateTime::now_utc(),
            status: ContainerStatus::Running,
        };

        self.containers
            .write()
            .await
            .insert(container_id.clone(), state);

        tracing::info!(container_id = %container_id, "Container started successfully");

        Ok(container_id)
    }

    /// Fetch logs from container (can be called repeatedly)
    pub async fn get_logs(&self, query: LogQuery) -> Result<LogsOutput, DockerError> {
        use bollard::container::LogsOptions;
        use tokio_stream::StreamExt;

        let options = LogsOptions::<String> {
            stdout: query.include_stdout,
            stderr: query.include_stderr,
            since: query.since.map(|t| t.unix_timestamp()).unwrap_or(0),
            tail: query
                .tail_lines
                .map(|n| n.to_string())
                .unwrap_or_else(|| "all".to_string()),
            ..Default::default()
        };

        let mut stream = self.docker.logs(&query.container_id, Some(options));

        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => {
                    let line = output.to_string();
                    match output {
                        bollard::container::LogOutput::StdOut { .. } => {
                            stdout_lines.push(line);
                        }
                        bollard::container::LogOutput::StdErr { .. } => {
                            stderr_lines.push(line);
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Error reading log stream");
                    break;
                }
            }
        }

        Ok(LogsOutput {
            stdout: stdout_lines,
            stderr: stderr_lines,
            timestamp: Some(OffsetDateTime::now_utc()),
        })
    }

    /// Execute one-off command in container
    pub async fn exec_command(
        &self,
        container_id: &str,
        command: &str,
    ) -> Result<String, DockerError> {
        use bollard::exec::{CreateExecOptions, StartExecResults};
        use tokio_stream::StreamExt;

        // Parse command string into args (simple split on whitespace)
        let cmd: Vec<&str> = command.split_whitespace().collect();

        let exec_config = CreateExecOptions {
            cmd: Some(cmd),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec = self
            .docker
            .create_exec(container_id, exec_config)
            .await
            .map_err(DockerError::Connection)?;

        let start_result = self
            .docker
            .start_exec(&exec.id, None)
            .await
            .map_err(DockerError::Connection)?;

        let mut output = String::new();

        if let StartExecResults::Attached {
            output: mut stream, ..
        } = start_result
        {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        output.push_str(&chunk.to_string());
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Error reading exec output");
                        break;
                    }
                }
            }
        }

        Ok(output)
    }

    /// Stop and remove container
    pub async fn stop_container(&self, container_id: &str) -> Result<(), DockerError> {
        use bollard::container::{RemoveContainerOptions, StopContainerOptions};

        // Stop the container (with 10 second timeout)
        self.docker
            .stop_container(container_id, Some(StopContainerOptions { t: 10 }))
            .await
            .map_err(DockerError::Connection)?;

        // Remove the container
        self.docker
            .remove_container(
                container_id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await
            .map_err(DockerError::Connection)?;

        // Remove from our tracking
        self.containers.write().await.remove(container_id);

        tracing::info!(container_id = %container_id, "Container stopped and removed");

        Ok(())
    }

    /// List all tracked containers
    pub async fn list_containers(&self) -> Vec<ContainerState> {
        self.containers.read().await.values().cloned().collect()
    }

    /// Get container state by ID
    #[allow(dead_code)] // Will be used when container lookup is needed
    pub async fn get_container(&self, container_id: &str) -> Option<ContainerState> {
        self.containers.read().await.get(container_id).cloned()
    }
}
