<!--file:create-docker-image.md-->

# Create Docker Image for Agent Workers

## Objective

Build a Docker image that provides an isolated execution environment for agent workers, with efficient git repository setup from a read-only host mount.

## Fetching the base image

```bash
docker pull debian:trixie-slim
```

## Building Image

```bash
DOCKER_BUILDKIT=1 docker build --progress=plain --no-cache -t image-name:0.1.0 -f containers/image-name.platform.Dockerfile containers/
```
