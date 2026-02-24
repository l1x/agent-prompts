# AGENTS.md

## Project Overview

Reusable prompt library for AI agents. Designed as a git submodule (`.agent-prompts/`) for any project. Workflows compose atomic prompts directly — no roles layer.

## Architecture

```
Final Agent = Workflow step + Core prompt + Stack prompt(s) + Template resolution
```

Composition at expansion time: `prompt_file` content + `\n\n---\n\n` + each `stacks/{name}.md`.

## Repository Structure

```
agent-prompts/
├── prompts/                        # Atomic, reusable instruction modules
│   ├── do/                         # Development lifecycle (verbs)
│   │   ├── plan.md                 # Explore codebase, produce implementation plan
│   │   ├── implement.md            # Execute changes, quality gates, commit
│   │   ├── review.md               # Review diff, PASS/FAIL verdict
│   │   ├── qa.md                   # Write tests, run suite, verify coverage
│   │   ├── architect.md            # System design, ADRs, Mermaid diagrams
│   │   ├── security-review.md      # STRIDE threat model, JSON findings
│   │   └── sre.md                  # Observe-Hypothesize-Verify-Fix diagnosis
│   ├── stacks/                     # Language/framework modifiers
│   │   ├── rust.md                 # Rust idioms, clippy, cargo test
│   │   ├── astro.md                # Astro/TypeScript, bun, CSS variables
│   │   ├── aws.md                  # Terraform IaC, AWS services, tfsec
│   │   └── mise.md                 # Tool versions, monorepo, task runner
│   ├── pm/                         # Project management
│   │   ├── github-project-manager.md  # PRD decomposition, epics, board
│   │   └── github-issue.md         # Dev gh CLI: view, comment, close
│   └── biz/                        # Business
│       └── gtm.md                  # Go-to-market, messaging, positioning
├── workflows/                      # DAG definitions for multi-step execution
│   ├── dev-task.toml               # plan → implement → qa → review → [fix]
│   └── pm-epic.toml                # decompose → track
└── docs/
    └── architecture.svg            # Visual architecture diagram
```

## Workflows

### dev-task.toml

Development lifecycle: `plan → implement → qa → review → fix (conditional)`.

- `workflow.stack` sets default stack for all steps (e.g., `["rust"]`, `["astro"]`)
- `step.stack` overrides `workflow.stack` when present
- The `fix` step re-runs `implement.md` when review verdict is FAIL (max 3 retries)

### pm-epic.toml

Project management: `decompose → track`.

- No stack — PM prompts don't need language modifiers
- Decomposes PRD into GitHub Issues with epics, dependencies, and sizing

## Workflow Schema

| Field | Type | Location | Purpose |
|-------|------|----------|---------|
| `workflow.stack` | string[] | workflow | Default stack prompts for all steps |
| `workflow.inputs` | table | workflow | Template variables needed |
| `step.prompt_file` | string | step | Path relative to `prompts/` |
| `step.stack` | string[] | step | Override workflow stack |
| `step.depends_on` | string[] | step | Step IDs that must complete first |
| `step.condition` | string | step | Expression on prior step output |
| `step.max_retries` | int | step | Retry count for conditional loops |

## Prompt Conventions

- **File naming:** `kebab-case.md`
- **Axes are orthogonal:** Phase prompts are verbs (plan, implement). Stack prompts are modifiers (rust, astro). Never combine axes in a filename.
- **No role/identity in prompts:** Prompts contain task instructions only
- **No runtime config in workflows:** No `model`, `permissionMode`, or `maxTurns` — those are runtime concerns

## Template Variables

| Variable | Used by |
|----------|---------|
| `{{mission}}` | dev-task |
| `{{worktree_path}}` | dev-task |
| `{{github_issue}}` | dev-task |
| `{{context}}` | dev-task, pm-epic |
| `{{prd}}` | pm-epic |
| `{{github_repo}}` | pm-epic |

## Task Sizing

| Size | Time | Scope |
|------|------|-------|
| XS | < 30 min | Single-file change, config tweak |
| S | 30m - 1h | 1-2 files |
| M | 1 - 2h | 2-5 files |
| L | 2 - 4h | 5+ files |
| XL | 4 - 8h | Architectural scope |

L/XL tasks should be decomposed into smaller tasks.

## Using This Repository

```bash
git submodule add https://github.com/l1x/agent-prompts.git .agent-prompts
git submodule update --remote
```
