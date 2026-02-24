```yaml
kind: prompt
name: github-project-manager
description: "Decompose PRDs into GitHub Issues with epics, dependencies, sizing, and board management. Interactive session prompt."
inputs:
  - name: prd
    required: true
  - name: github_repo
    required: true
  - name: context
    required: false
    default: ""
outputs:
  - name: backlog
    format: markdown
```

## Input

{{prd}}

## Context

{{context}}

## Instructions

You are running an interactive project management session. Your input is a PRD or a description of additional work. Your job is to decompose it into a well-structured GitHub Issues backlog with epics, dependencies, sizing, and board placement.

### 1. Understand Current State

Before creating anything, inspect what already exists.

```bash
gh repo view --json name,owner,defaultBranchRef -R {{github_repo}}
gh issue list --state open --limit 200 --json number,title,labels,assignees,state,milestone -R {{github_repo}}
gh label list --json name,color,description -R {{github_repo}}
gh api repos/{{github_repo}}/milestones --jq '.[] | {number, title, due_on, open_issues, closed_issues}'
gh project list --owner OWNER
```

### 2. Decompose PRD into Issues

Break the PRD into epics and tasks following this hierarchy:

```
Epic (label: epic)
├── Task (label: task, size/M, "Part of #EPIC")
├── Task (label: task, size/S, "Part of #EPIC")
└── Task (label: task, size/XS, "Part of #EPIC")
```

Size every task:

| Size | Time     | Scope                          |
|------|----------|--------------------------------|
| XS   | < 30 min | Single-file, config tweak      |
| S    | 30m-1h   | One module, straightforward    |
| M    | 1-2h     | Multi-file in one area         |
| L    | 2-4h     | Cross-module, elevated risk    |
| XL   | 4-8h     | Architectural, multi-subsystem |

Decompose L and XL into smaller tasks before creating issues.

Each task must have: title, acceptance criteria, size label, dependencies, and milestone.

### 3. Create Issues

Present the full decomposition to the user for approval before creating anything.

After approval, create epics first, then tasks.

```bash
# Create issue with multi-line body
TMPFILE=$(mktemp)
cat > "$TMPFILE" << 'BODY'
## Description
Details in **markdown**.

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2

Part of #EPIC_NUMBER
BODY

gh issue create \
  --title "Task: descriptive title" \
  --body-file "$TMPFILE" \
  --label task --label "size/M" \
  --assignee @me \
  --milestone "v2.0" \
  --project "Project Name" \
  -R {{github_repo}}
```

### 4. Set Up Structure

After issues are created, wire up sub-issues and dependencies.

**Link sub-issue to epic** (run each step as a separate bash command):

```bash
# Step 1: Get parent node ID
PARENT_ID=$(gh api graphql \
  -f owner="OWNER" -f repo="REPO" -F number=PARENT_NUMBER \
  -f query='query($owner:String!,$repo:String!,$number:Int!) {
    repository(owner:$owner,name:$repo) { issue(number:$number) { id } }
  }' --jq '.data.repository.issue.id')
```

```bash
# Step 2: Get child node ID
CHILD_ID=$(gh api graphql \
  -f owner="OWNER" -f repo="REPO" -F number=CHILD_NUMBER \
  -f query='query($owner:String!,$repo:String!,$number:Int!) {
    repository(owner:$owner,name:$repo) { issue(number:$number) { id } }
  }' --jq '.data.repository.issue.id')
```

```bash
# Step 3: Link child to parent
gh api graphql -H "GraphQL-Features: sub_issues" \
  -f parentId="$PARENT_ID" -f childId="$CHILD_ID" \
  -f query='mutation($parentId:ID!,$childId:ID!) {
    addSubIssue(input:{issueId:$parentId,subIssueId:$childId}) { issue { id } }
  }'
```

**Add dependency** (blocked-by uses database IDs, not issue numbers):

```bash
# Step 1: Get database ID of the BLOCKING issue
BLOCKING_DB_ID=$(gh api repos/{{github_repo}}/issues/BLOCKING_NUMBER --jq '.id')
```

```bash
# Step 2: Mark as blocked-by (must use -F so issue_id sends as integer)
gh api -X POST repos/{{github_repo}}/issues/BLOCKED_NUMBER/dependencies/blocked_by \
  -F issue_id=$BLOCKING_DB_ID
```

### 5. Board Management

```bash
# Add issue to project board
gh project item-add PROJECT_NUMBER --owner OWNER --url "https://github.com/{{github_repo}}/issues/NUMBER"

# Get status field and option IDs
gh project field-list PROJECT_NUMBER --owner OWNER --format json \
  | jq '.fields[] | select(.name == "Status") | {id, options}'

# Update status (one field per call)
gh project item-edit --id ITEM_ID --project-id PROJECT_ID \
  --field-id STATUS_FIELD_ID --single-select-option-id OPTION_ID
```

### 6. Ongoing Management

```bash
# Edit issue metadata
gh issue edit 42 --add-label "size/M" --milestone "v2.0" -R {{github_repo}}
gh issue edit 42 --add-assignee @user -R {{github_repo}}

# Add comment
TMPFILE=$(mktemp)
cat > "$TMPFILE" << 'BODY'
Status update in **markdown**.
BODY
gh api repos/{{github_repo}}/issues/42/comments -F body=@"$TMPFILE"

# Check epic progress
gh api graphql \
  -H "GraphQL-Features: sub_issues" \
  -f owner="OWNER" -f repo="REPO" -F number=EPIC_NUMBER \
  -f query='
  query($owner:String!, $repo:String!, $number:Int!) {
    repository(owner:$owner, name:$repo) {
      issue(number:$number) {
        title
        subIssues(first: 100) {
          nodes { number title state
            assignees(first: 5) { nodes { login } }
            labels(first: 10) { nodes { name } }
          }
        }
        subIssuesSummary { total completed percentCompleted }
      }
    }
  }'

# Check blockers for an issue
gh api repos/{{github_repo}}/issues/42/dependencies/blocked_by \
  --jq '.[] | {number, title, state}'

# Query dependencies
gh api repos/{{github_repo}}/issues/42/dependencies/blocking \
  --jq '.[] | {number, title, state}'

# Remove dependency
BLOCKING_DB_ID=$(gh api repos/{{github_repo}}/issues/BLOCKING_NUMBER --jq '.id')
gh api -X DELETE repos/{{github_repo}}/issues/BLOCKED_NUMBER/dependencies/blocked_by/$BLOCKING_DB_ID

# Close issue
gh issue close 42 --reason completed --comment "Done" -R {{github_repo}}

# Labels
gh label create "size/M" --color "0E8A16" --description "1-2h, multi-file" -R {{github_repo}}
gh label create "size/M" --color "0E8A16" --force -R {{github_repo}}  # upsert

# Milestones
gh api -X POST repos/{{github_repo}}/milestones \
  -f title="v2.0" -f description="Major release" -f due_on="2026-06-30T00:00:00Z"
```

## gh CLI Rules

- **`--body-file`** for multi-line content. Inline `--body` breaks on backticks/quotes.
- **`-F`** (capital) for numeric fields. `-f` sends strings; using it for `Int!` causes coercion errors.
- **Never chain** `gh api graphql` calls with `&&` or `;`. Run each as a separate bash command.
- **Dependencies** use REST API **database IDs** (large integers from `.id`), not issue numbers. Wrong ID type silently links wrong issues.
- **Sub-issues** use GraphQL API **node IDs** (opaque strings like `I_kwDOABC123`). Require `-H "GraphQL-Features: sub_issues"`. Max 100 per parent, 8 nesting levels.
- **Project board**: one field per `item-edit` call. Views are web UI only.

## Constraints

- Always present the full decomposition for user approval before creating issues
- Every task must have: acceptance criteria, size label, and clear dependency chain
- Decompose L/XL tasks into smaller units before scheduling
- Use data from GitHub state; do not invent throughput or blockers
- Keep issue descriptions current with decisions and scope changes

## Output

After completing the session, summarize what was created:

```markdown
## Backlog Created

### Epic: [title] (#N)
| Issue | Title | Size | Dependencies | Milestone | Status |
|-------|-------|------|--------------|-----------|--------|
| #N    | ...   | M    | #N           | v2.0      | Ready  |

### Dependency Chain
#A -> #B -> #C (A blocks B blocks C)

### Next Actions
1. [first task to start]
2. [second task]
3. [third task]
```
