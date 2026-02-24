---
name: github-pm
description: "GitHub project management — create, view, edit, and close issues, set up dependencies (blocked-by/blocks), manage sub-issues (parent/child), milestones, labels, project boards, and comments. Use when working with GitHub Issues, epics, or project boards."
compatibility: "Requires gh and jq"
metadata:
  author: l1x
  version: "2.0.0"
---

# Critical Rules

## Use `--body-file` for multi-line content

Inline `--body` with heredoc breaks when content contains backticks or quotes. Always use `--body-file`:

```bash
TMPFILE=$(mktemp)
cat > "$TMPFILE" << 'BODY'
Markdown content — backticks, quotes, anything works.
BODY

gh issue create --title "..." --body-file "$TMPFILE" -R OWNER/REPO
```

## `-f` vs `-F` in `gh api`

- `-f key=value` — sends value as **string**
- `-F key=value` — infers type: numbers → integers, booleans → booleans, `@file` → reads file

Using `-f` for a numeric field (e.g., GraphQL `Int!`) causes `"Could not coerce value to Int"`. Use `-F` for numbers.

## GraphQL call hygiene

**Never chain** `gh api graphql` calls with `&&`, `;`, or `echo` in one bash invocation. Run each as a **separate** bash command. Chaining causes invisible encoding errors (`Expected VAR_SIGN, actual: UNKNOWN_CHAR`).

Always use the **multiline** query format.

## Use `Closes #N` in PR body

Link PRs to issues using `Closes #N` (or `Fixes #N`, `Resolves #N`) in the PR body. On merge, linked issues auto-close. Use `Related to #N` or `Part of #N` only when referencing without closing.

---

# Issues

## Create

```bash
TMPFILE=$(mktemp)
cat > "$TMPFILE" << 'BODY'
## Description
Details here in **markdown**.
BODY

gh issue create \
  --title "Bug: Login fails on mobile" \
  --body-file "$TMPFILE" \
  --label bug --label "high priority" \
  --assignee monalisa --assignee @me \
  --milestone "v2.0" \
  --project "Roadmap" \
  -R OWNER/REPO
```

- `--body-file` reads body from file (preferred over `--body`)
- `--project` auto-adds to the Projects v2 board
- `--label` and `--assignee` are repeatable
- `-R, --repo OWNER/REPO` targets a different repo

## View

```bash
gh issue view 42 -R OWNER/REPO                    # human-readable
gh issue view 42 --comments -R OWNER/REPO         # with comments
gh issue view 42 --comments --json number,title,body,state,labels,assignees,milestone,comments -R OWNER/REPO
gh issue view 42 --json assignees --jq '.assignees[].login' -R OWNER/REPO
```

Available `--json` fields: `assignees`, `author`, `body`, `closed`, `closedAt`, `comments`, `createdAt`, `id`, `labels`, `milestone`, `number`, `projectCards`, `projectItems`, `reactionGroups`, `state`, `stateReason`, `title`, `updatedAt`, `url`

## Edit

```bash
gh issue edit 42 --title "New title" --body "Updated body" -R OWNER/REPO
gh issue edit 42 --body-file ./updated-description.md -R OWNER/REPO
gh issue edit 42 --add-label "regression" --remove-label "in-progress" -R OWNER/REPO
gh issue edit 42 --add-assignee monalisa,@me --remove-assignee hubot -R OWNER/REPO
gh issue edit 42 --milestone "v3.0" -R OWNER/REPO
```

## Close / Reopen / Delete

```bash
gh issue close 42 --reason completed --comment "Fixed in v2.1" -R OWNER/REPO
gh issue reopen 42 -R OWNER/REPO
gh issue delete 42 --yes -R OWNER/REPO
```

## List

Defaults: open issues, limit 30.

```bash
gh issue list --label bug --assignee @me --milestone "v2.0" --state open --limit 100 -R OWNER/REPO
gh issue list --search 'no:assignee label:"help wanted" sort:created-asc' -R OWNER/REPO
gh issue list --json number,title,labels,assignees,state -R OWNER/REPO
```

Search qualifiers: `state:open|closed`, `assignee:USER`, `no:assignee`, `label:"name"`, `no:label`, `milestone:"v1.0"`, `no:milestone`, `author:USER`, `mentions:USER`, `created:2025-01-01..2026-02-11`, `updated:>2025-12-31`, `reactions:>5`, `comments:>10`, `sort:created-asc|updated-desc|comments-desc`

---

# Comments

```bash
# Add (multi-line — use --body-file pattern)
TMPFILE=$(mktemp)
cat > "$TMPFILE" << 'BODY'
Comment in **markdown** with `backticks` and other special chars.
BODY
gh api repos/OWNER/REPO/issues/42/comments -F body=@"$TMPFILE"

# Add (simple one-liner)
gh api repos/OWNER/REPO/issues/42/comments -f body="Simple comment"

# List
gh issue view 42 --comments --json comments --jq '.comments[] | {author: .author.login, body: .body, createdAt: .createdAt}' -R OWNER/REPO

# Edit
gh api -X PATCH repos/OWNER/REPO/issues/comments/COMMENT_ID -f body="Updated"

# Delete
gh api -X DELETE repos/OWNER/REPO/issues/comments/COMMENT_ID
```

---

# Epic Status

Check which sub-issues are ready, blocked, or done for an epic. Run each step as a **separate** bash command.

**Step 1: Fetch sub-issues**

```bash
gh api graphql \
  -H "GraphQL-Features: sub_issues" \
  -f owner="OWNER" -f repo="REPO" -F number=EPIC_NUMBER \
  -f query='
  query($owner:String!, $repo:String!, $number:Int!) {
    repository(owner:$owner, name:$repo) {
      issue(number:$number) {
        title
        subIssues(first: 100) {
          nodes {
            number
            title
            state
            assignees(first: 5) { nodes { login } }
            labels(first: 10) { nodes { name } }
          }
        }
        subIssuesSummary { total completed percentCompleted }
      }
    }
  }'
```

**Step 2: For each open sub-issue, check blockers**

```bash
gh api repos/OWNER/REPO/issues/NUM/dependencies/blocked_by \
  --jq '[.[] | select(.state == "open")] | map("#" + (.number | tostring)) | join(", ")'
```

If the result is empty, the issue is **ready** (can be worked on now). Otherwise it is **blocked**.

---

# Dependencies

Uses REST API with **database IDs** (large integers), not issue numbers.

## Add dependency (issue blocked by another)

Run each step as a **separate** bash command.

```bash
# Step 1: Resolve the blocking issue number to its database ID
BLOCKING_DB_ID=$(gh api repos/OWNER/REPO/issues/BLOCKING_NUMBER --jq '.id')
```

```bash
# Step 2: Create the blocked-by relationship
gh api -X POST repos/OWNER/REPO/issues/BLOCKED_NUMBER/dependencies/blocked_by \
  -F issue_id=$BLOCKING_DB_ID
```

**Gotcha:** Must use `-F` (capital) so `issue_id` sends as integer. Using the issue number instead of the database ID will silently link the wrong issue.

## Add dependency chain (linear order)

To create a chain where #1 blocks #2 blocks #3:

1. Resolve all issue numbers to database IDs (one `gh api` call per issue)
2. For each consecutive pair, POST the blocked-by relationship

```bash
# For each pair (BLOCKING, BLOCKED) in the chain:
BLOCKING_DB_ID=$(gh api repos/OWNER/REPO/issues/BLOCKING_NUMBER --jq '.id')
gh api -X POST repos/OWNER/REPO/issues/BLOCKED_NUMBER/dependencies/blocked_by \
  -F issue_id=$BLOCKING_DB_ID
```

## Query dependencies

```bash
# What blocks issue #42?
gh api repos/OWNER/REPO/issues/42/dependencies/blocked_by \
  --jq '.[] | {number, title, state}'

# What does issue #42 block?
gh api repos/OWNER/REPO/issues/42/dependencies/blocking \
  --jq '.[] | {number, title, state}'
```

## Remove dependency

```bash
BLOCKING_DB_ID=$(gh api repos/OWNER/REPO/issues/10 --jq '.id')
gh api -X DELETE repos/OWNER/REPO/issues/42/dependencies/blocked_by/$BLOCKING_DB_ID
```

---

# Sub-Issues

Uses GraphQL API with **node IDs** (opaque strings like `I_kwDOABC123`), not issue numbers or database IDs. Requires header `-H "GraphQL-Features: sub_issues"`. Limits: 100 sub-issues per parent, 8 nesting levels.

Run each step as a **separate** bash command.

## Link existing issue as sub-issue

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

## List sub-issues

```bash
gh api graphql \
  -H "GraphQL-Features: sub_issues" \
  -f owner="OWNER" -f repo="REPO" -F number=PARENT_NUMBER \
  -f query='
  query($owner:String!, $repo:String!, $number:Int!) {
    repository(owner:$owner, name:$repo) {
      issue(number:$number) {
        title
        subIssues(first: 50) {
          nodes { number title state }
        }
        subIssuesSummary { total completed percentCompleted }
      }
    }
  }'
```

## Remove sub-issue

```bash
gh api graphql -H "GraphQL-Features: sub_issues" \
  -f parentId="$PARENT_ID" -f childId="$CHILD_ID" \
  -f query='mutation($parentId:ID!,$childId:ID!) {
    removeSubIssue(input:{issueId:$parentId,subIssueId:$childId}) { issue { id } }
  }'
```

---

# Project Board

## Add issue to board

```bash
ISSUE_URL="https://github.com/OWNER/REPO/issues/NUMBER"
gh project item-add PROJECT_NUMBER --owner OWNER --url "$ISSUE_URL"
```

To find your project number: `gh project list --owner OWNER`

## Update board status

Run each step as a **separate** bash command.

```bash
# Step 1: Find the project item ID and project ID for this issue
gh api graphql -F owner="OWNER" -F repo="REPO" -F issue=ISSUE_NUMBER -f query='
query($owner: String!, $repo: String!, $issue: Int!) {
  repository(owner: $owner, name: $repo) {
    issue(number: $issue) {
      projectItems(first: 10) {
        nodes {
          id
          project { id number }
        }
      }
    }
  }
}'
```

```bash
# Step 2: Get the Status field ID and option IDs for the project
gh project field-list PROJECT_NUMBER --owner OWNER --format json \
  | jq '.fields[] | select(.name == "Status") | {id, options}'
```

```bash
# Step 3: Update the status (match OPTION_ID from step 2)
gh project item-edit --id ITEM_ID --project-id PROJECT_ID \
  --field-id STATUS_FIELD_ID --single-select-option-id OPTION_ID
```

**Gotcha:** Only one field per `item-edit` call.

---

# Context Queries

Run these to understand the current state before taking actions.

```bash
gh repo view --json name,owner,defaultBranchRef --jq '{name: .name, owner: .owner.login, default_branch: .defaultBranchRef.name}'
gh api repos/OWNER/REPO/milestones --jq '.[] | {number, title, due_on, open_issues, closed_issues}'
gh label list --json name,color,description -R OWNER/REPO
gh project field-list PROJECT_NUMBER --owner OWNER --json id,name,type,options
gh project list --owner OWNER
```

---

For milestones management, raw API details, and advanced recipes see `advanced-reference.md`.
