<!--file:generate-tasks.md-->

# Generate Tasks (GitHub Issues)

## Objective

Decompose a PRD into granular, actionable GitHub Issues with epic/task hierarchy suitable for iterative implementation by multiple agents.

## Input

- PRD file: `prds/prd-[feature-name]-[YYYY-MM-DD].md`
- Repository: Owner/repo or current working directory.

## Process

1. **Parse PRD**: Extract functional requirements, non-functional requirements, and acceptance criteria.
1. **Identify Epic**: Create one or multiple epics per PRD representing the full feature scope.
1. **Decompose into Tasks**
   - Break each requirement into atomic units of work (completable in one PR) that must have verifiable completion criteria derived from the PRD.
1. **Create Issues**: Output issues using `gh issue create`.

## Issue Hierarchy

GitHub Issues support sub-issues (parent/child). Use this structure:

```
#10  Epic: Feature scope              (label: epic)
├── #11  Task: Logical grouping       (label: task, body: "Part of #10")
└── #12  Task: Logical grouping       (label: task, body: "Part of #10")
```

After creating child issues, add them as sub-issues of the epic:

```bash
gh api graphql -f query='
  mutation {
    addSubIssue(input: {issueId: "EPIC_NODE_ID", subIssueId: "TASK_NODE_ID"}) {
      issue { id }
    }
  }'
```

## Issue Structure

Each issue must include:

```
Title: [Action verb] + [Component/Feature]
Labels: epic | task, P1 | P2 | P3
Milestone: (if applicable)
Body:
  ## Description
  [What needs to be done]

  ## Acceptance Criteria
  - [ ] [Verifiable condition 1]
  - [ ] [Verifiable condition 2]

  ## Dependencies
  Blocked by #N (if applicable)
  Part of #N (if child of an epic)
```

## Creating Issues

Use `--body-file` for multi-line content to avoid quoting issues:

```bash
TMPFILE=$(mktemp)
cat > "$TMPFILE" << 'BODY'
## Description
Details here in **markdown**.

## Acceptance Criteria
- [ ] Condition 1
- [ ] Condition 2
BODY

gh issue create \
  --title "Task: Implement session counter" \
  --body-file "$TMPFILE" \
  --label task --label P1 \
  --milestone "v1.0.0"
```

## Setting Dependencies

Use comments to document blocking relationships, and the `blocked` label when a task cannot proceed:

```bash
gh issue comment 12 --body "Blocked by #11 — needs the shared component first."
```

## Task Granularity Rules

- **Epic** — Full feature, multiple tasks
- **Task** — Atomic unit, one PR, max 2 hours of work

## Output

- **Format:** GitHub Issues with labels, milestones, and parent/child relationships
- **Epics:** Labeled `epic` with sub-issues linked
- **Tasks:** Labeled `task` with `Part of #N` in body

## Constraints

- Each task must be completable in a single PR
- Tasks must be independently verifiable
- Dependencies must be explicit (documented in comments and/or body)
- Do NOT skip acceptance criteria
