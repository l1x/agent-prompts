# AGENTS.md

## Project Overview

Reusable prompt library for AI agents. Designed as a git submodule (`.agent-prompts/`) for any project. Workflows compose atomic prompts directly — no roles layer.

## Architecture

```
Final Agent = Workflow step + Core prompt + Included docs + Template resolution
```

Composition at expansion time:
```
rendered = prompt_file
         + "\n\n---\n\n" + each workflow.include entry (or step.include override)
         + template resolution
```

## Orchestration Flow

GitHub Issues is the shared state between sessions. The orchestrator dispatches workflows — no single agent runs end-to-end.

```
Orchestrator
    |
    v
[prd-to-issue] ---- PRD in, GitHub Issues out
    |  (for each issue)
    v
[precise-context] -- Issue in, XML context written to issue
    |
    v
[develop-feature] -- Issue+context in, code committed
    |
    v
[issue-review] ----- Code reviewed, verdict out
    |
    +-- PASS --> done
    +-- FAIL --> orchestrator dispatches new [develop-feature]
```

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
│   ├── prd-to-issue.toml           # decompose PRD into issues
│   ├── precise-context.toml        # plan (explore + write XML context)
│   ├── develop-feature.toml        # implement ↔ qa loop
│   └── issue-review.toml           # review + security-review
└── docs/
    └── architecture.svg            # Visual architecture diagram
```

## Workflows

### prd-to-issue.toml

Decomposes a PRD into GitHub Issues with epics, dependencies, and sizing.

- Single step: `decompose`
- `github-project-manager.md` already contains all `gh` CLI patterns for issue creation
- `include = []` — no additional docs needed

### precise-context.toml

Explores the codebase and writes structured XML context into a GitHub issue.

- Single step: `plan`
- `include = []` — caller provides stacks and helpers at dispatch time
- Output is written directly to the GitHub issue

### develop-feature.toml

Implements a feature from pre-planned context, loops QA until tests pass.

- `implement → qa` — if QA fails, loops back to `implement` (max 3 retries)
- `include = []` — caller provides stacks and helpers at dispatch time
- Context is required — must come from a prior `precise-context` run

### issue-review.toml

Reviews implementation and PR for code quality, security, and issue alignment.

- `review` and `security-review` run in parallel (no `depends_on`)
- Diagnostic only — no fix step; FAIL verdict goes back to the orchestrator

## Workflow Schema

| Field | Type | Location | Purpose |
|-------|------|----------|---------|
| `workflow.include` | string[] | workflow | Default `[]`; caller provides paths relative to repo root at dispatch time |
| `workflow.inputs` | table | workflow | Template variables needed |
| `step.prompt_file` | string | step | Path relative to repo root |
| `step.include` | string[] | step | Override `workflow.include` for this step |
| `step.depends_on` | string[] | step | Step IDs that must complete first |
| `step.on_fail` | string | step | Step ID to loop back to on failure |
| `step.max_retries` | int | step | Max loop iterations before exit |

## Prompt Conventions

- **File naming:** `kebab-case.md`
- **Axes are orthogonal:** Phase prompts are verbs (plan, implement). Stack prompts are modifiers (rust, astro). Never combine axes in a filename.
- **No role/identity in prompts:** Prompts contain task instructions only
- **No runtime config in workflows:** No `model`, `permissionMode`, or `maxTurns` — those are runtime concerns

## Template Variables

| Variable | Used by |
|----------|---------|
| `{{mission}}` | precise-context, develop-feature, issue-review |
| `{{worktree_path}}` | precise-context, develop-feature, issue-review |
| `{{github_issue}}` | precise-context, develop-feature, issue-review |
| `{{github_repo}}` | prd-to-issue, precise-context |
| `{{context}}` | all workflows |
| `{{prd}}` | prd-to-issue |

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
