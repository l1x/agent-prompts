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
RUN curl -sSL https://raw.githubusercontent.com/steveyegge/beads/main/scripts/install.sh | bash || echo 'ok'
RUN mv /home/agent/.local/bin/bd /home/agent/bin/bd

# mise config
COPY --chown=agent:agent mise.config.toml /home/agent/.config/mise/config.toml

# env
ENV PATH="/home/agent/bin:$PATH"

# mise tasks
RUN echo 'eval "$(mise activate bash)"' >> ~/.bashrc
RUN mise activate | bash
RUN mise trust
RUN mise install
RUN rustc --version
RUN cargo --version
RUN mise --version
RUN bd --version

# mise commands for runtime
COPY --chown=agent:agent mise.toml /home/agent/mise.toml
