<!--file:create-docker-image.md-->

# Create Docker Image for Agent Workers

## Objective

Build a Docker image that provides an isolated execution environment for agent workers, with efficient git repository setup from a read-only host mount.

## Git Strategy

Mount the host repository **read-only** and use `git clone --local` inside the container. This approach:

- Uses hardlinks for git objects (fast, minimal disk usage)
- Provides the container with a clean, writable git repo
- Requires no network access or credentials for initial setup
- Isolates container changes completely from the host

## Using Docker

### Fetching the base image

```bash
docker pull debian:trixie-slim
```

### Dockerfile

```dockerfile
# -*- mode: dockerfile -*-

# This is non-reproducible because apt update makes it random
FROM debian:trixie-slim AS base-image

# installing packages
RUN apt update -y && \
  apt --no-install-suggests --no-install-recommends install rustup git bash curl ca-certificates -y && \
  rm -rf /var/lib/apt/lists/*

# user creation
RUN groupadd -g 1000 agent
RUN useradd -u 1000 -g agent -s /bin/bash -m agent

# Unpriviledged mode
USER agent
WORKDIR /home/agent
# debug
RUN echo -n "current user:" && whoami && \
  echo -n "current folder:" && pwd
# mise
RUN mkdir -p /home/agent/.config/mise/ && \
  mkdir -p bin/ && curl https://mise.run | MISE_DEBUG=1 MISE_INSTALL_PATH=/home/agent/bin/mise sh
COPY --chown=agent:agent mise.config.toml /home/agent/.config/mise/config.toml
ENV PATH="/home/agent/bin:$PATH"
RUN echo 'eval "$(mise activate bash)"' >> ~/.bashrc
RUN mise install
RUN rustc --version
```

## Building Image

```bash
DOCKER_BUILDKIT=1 docker build --progress=plain --no-cache -t image-name:0.1.0 -f containers/image-name.platform.Dockerfile containers/
```
