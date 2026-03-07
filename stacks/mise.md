```yaml
kind: prompt
name: mise
description: "mise conventions for tool version management and task running"
inputs: []
outputs: []
```

## mise Conventions

- Use `mise.toml` (or `.mise.toml`) for tool versions and tasks
- Pin exact tool versions — no ranges or `latest`
- Place a `mise.toml` in each folder that has tasks to run, not just the repo root
- **Never use shell scripts from mise** — define all tasks inline in `mise.toml`
- Use `mise install` to install all project tools
- Use `mise run <task>` for project tasks instead of direct commands

## Tool Versions

```toml
[tools]
rust = "1.91.0"
node = "22.14.0"
bun = "1.2.4"
terraform = "1.10.5"
```

- Keep versions consistent across the team — commit `mise.toml` to the repo
- Update versions intentionally — test after upgrading

## Monorepo Setup

For monorepos, place a root `mise.toml` with monorepo config and a `mise.toml` in each subproject folder.

Root `mise.toml`:

```toml
[settings]
experimental = true

[env]
_.python.venv = { path = ".venv", create = true }
MISE_EXPERIMENTAL = true
```

Root `.mise/config.toml` (monorepo routing):

```toml
experimental_monorepo_root = true

[monorepo]
config_roots = [
    "docs",
    "infrastructure/mgmt",
    "infrastructure/tf/aws/envs/prod/960682158808/eu-west-1/*",
    "packages/*",
    "services/*/api",
    "services/*/sdk",
    "services/*/dashboard",
    "sites/*",
    "services/insights/lambdas/*",
    "sites/costs-dashboard/lambdas/*",
    "tools/infra/*",
]
```

Each path in `config_roots` is a subproject that gets its own `mise.toml` with its own tools and tasks. When you `cd` into a subproject, mise activates that folder's config automatically.

## Tasks

Define tasks inline in `mise.toml` — no shell script files.

```toml
[tasks.fmt]
run = "cargo fmt"
description = "Format code"

[tasks.lint]
run = "cargo clippy --workspace --all-targets -- -D warnings"
description = "Run linter"

[tasks.test]
run = "cargo test --workspace"
description = "Run tests"
```

Add `description` to every task.

### Task Ordering

**`depends` does not guarantee execution order.** Tasks in `depends` may run in parallel or in any order:

```toml
# BAD: order is not guaranteed
[tasks.check]
depends = ["fmt", "lint", "test"]
description = "Run all quality gates"
```

**Use `run` with a task list for ordered sequential execution:**

```toml
# GOOD: build, then sync, then invalidate — guaranteed order
[tasks.deploy]
description = "Build, sync to S3, and invalidate CloudFront cache"
run = [
  { task = "build" },
  { task = "sync" },
  { task = "invalidate-cache" },
]

# GOOD: fmt before lint before test — guaranteed order
[tasks.check]
description = "Run all quality gates in order"
run = [
  { task = "fmt" },
  { task = "lint" },
  { task = "test" },
]
```

Use `depends` only when order does not matter (e.g., independent setup steps). Use `run` with `{ task = "..." }` entries when steps must execute sequentially.

## Environment Variables

```toml
[env]
RUST_LOG = "info"
```

- Never put secrets in `mise.toml` — use `.env.local` (gitignored) or a secret manager
- For per-developer values, use `.env.local`

## Quality Gates

When a project uses mise, prefer:

1. `mise run fmt` — format
2. `mise run lint` — lint
3. `mise run test` — test
4. `mise run check` — all gates (if defined)

Over direct tool invocations, to ensure consistent tool versions.
