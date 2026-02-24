# Agent Prompts v2 — Full Plan

## Context

Redesigning the agent prompt collection to serve as the foundation for a webUI that orchestrates multiple agents on the same project. The key architectural insight: **a Role is composed of 1 or many Prompts**. Prompts are atomic, reusable instruction modules. Roles bundle prompts into agent personas with identity, permissions, and behavioral rules.

GitHub Issues is the sole task backend. Language-specific roles (rust, astro) are kept separate.

## Design Principles

1. **Lifecycle phase ≠ Stack specialization.** Phase prompts are verbs (plan, implement, review). Stack prompts are modifiers (rust, astro). Never combine axes in a filename.
2. **Orthogonal axes, not folders.** Prompts live on independent axes: concern type (what), stack (how), environment (where). Directories group by concern type. Frontmatter encodes all axes.
3. **Early structure is cheap, late structure is expensive.** Subdirectories now prevent breaking role references later.
4. **Deterministic composition order.** Not random concatenation — ordered layers with clear precedence.

## Composition Model

```
Final Agent = Role body
            + Core prompt(s)        ← phase behavior
            + Stack prompt(s)       ← language/framework modifier
```

### Composition Order (deterministic precedence)

```
1. Role body              — identity, rules, behavioral constraints (highest precedence)
1. Core prompt(s)         — phase instructions (plan, implement, review, test, pr)
1. Stack prompt(s)        — language conventions, quality gates (rust, astro)
1. Template resolution    — {{mission}}, {{worktree_path}}, {{context}}
```

### Assembly Algorithm

1. Load Role file → extract frontmatter (model, permissionMode, maxTurns, prompts[])
1. Partition prompts by `type`: core prompts first, then stack prompts (from frontmatter metadata)
1. Compose in deterministic order: Role body → Core prompts → Stack prompts
1. Resolve template variables: `{{mission}}`, `{{worktree_path}}`, `{{context}}`

### Dynamic Resolution

Workflows can resolve prompts semantically instead of by filename:

```
Workflow Step:
  role: worker
  stack: rust
  concern: implement

Resolves → core/implement.md + stacks/rust.md
```

This makes the workflow builder semantic instead of file-based.

## Directory Structure

```
agent-prompts/
├── prompts/                        # Atomic, reusable instruction modules
│   ├── core/                       # Phase behavior (verbs)
│   │   ├── plan.md                 # Explore codebase, produce implementation plan
│   │   ├── implement.md            # Implement changes, quality gates, commit
│   │   ├── review.md               # Review diff, PASS/FAIL verdict
│   │   ├── test.md                 # Write and run tests
│   │   └── pr.md                   # Create pull request
│   ├── stacks/                     # Language/framework modifiers
│   │   ├── rust.md                 # Rust conventions and quality gates
│   │   └── astro.md                # Astro/TypeScript conventions
│   ├── github/                     # Environment specialization
│   │   └── review-pr.md            # Full PR review workflow
│   └── pm/                         # Project management
│       ├── create-prd.md           # Generate PRD from feature request
│       └── generate-tasks.md       # Decompose PRD into GitHub Issues with complexity
│
├── roles/                          # Agent personas (compose prompts)
│   ├── planner.md                  # Read-only planning agent
│   ├── rust-worker.md              # Rust implementation agent
│   ├── astro-worker.md             # Astro/TypeScript implementation agent
│   ├── reviewer.md                 # Code review agent
│   ├── tester.md                   # Test writing agent
│   ├── github-user.md              # GitHub interaction agent
│   └── github-pm.md                # GitHub project manager agent
│
├── workflows/                      # DAG definitions for multi-step execution
│   ├── dev-task.toml               # plan → implement → review → [fix|pr]
│   └── pm-epic.toml                # create-prd → generate-tasks → execute
```

## Complexity Classification

Each task MUST be assigned a complexity label:

| Size | Time     | Scope                                      | Signals                                     |
| ---- | -------- | ------------------------------------------ | ------------------------------------------- |
| XS   | < 30 min | Single-file change, config tweak, typo fix | 1 file, no new deps, no tests needed        |
| S    | 30m - 1h | One module, straightforward logic          | 1-2 files, unit tests only                  |
| M    | 1 - 2h   | Multiple files in one module               | 2-5 files, unit + integration tests         |
| L    | 2 - 4h   | Cross-module changes, complex logic        | 5+ files, multiple test types, 1-2 new deps |
| XL   | 4 - 8h   | Architectural change, multiple subsystems  | Many files, 3+ new deps, high breakage risk |

Any task classified as L or XL should be reviewed for decomposition into smaller tasks.

## Issue Hierarchy

```
#10 Epic: Feature scope (label: epic)
├── #11 Task: Atomic unit (label: task, size/M, body: "Part of #10")
└── #12 Task: Atomic unit (label: task, size/S, body: "Part of #10")
```

## Implementation Sequence

1. Create `prompts/` directory and write all 10 prompt files
2. Create `roles/` directory and write all 7 role files
3. Create `workflows/` directory and write both workflow files
4. Write `README.md` documenting the architecture and composition model

## Verification

- Every role's `prompts` list references prompt files that exist in `prompts/`
- Every workflow step's `role` references a role file that exists in `roles/`
- All template variables (`{{mission}}`, `{{worktree_path}}`, `{{context}}`) are defined in prompt `inputs`
- No prompt file re-declares agent identity (that belongs in the role)
- No role file contains task-specific instructions (that belongs in the prompt)
