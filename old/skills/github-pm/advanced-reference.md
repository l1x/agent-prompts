# Advanced Reference — GitHub Project Management

Read this file when you need to manage milestones, dependencies, sub-issues, labels, or project boards.

> **ID types -- don't mix them up!**
>
> - **Dependencies** use the REST API and require **database IDs** -- large integers obtained via `gh api repos/OWNER/REPO/issues/N --jq '.id'` (e.g., `2894561234`).
> - **Sub-issues** use the GraphQL API and require **node IDs** -- opaque strings obtained via GraphQL queries (e.g., `I_kwDOABC123`).
> - Using the wrong ID type will silently link the wrong issue or produce an API error.

---

## Quick Recipes

### Recipe 1: Create an epic with sub-issues

```bash
# 1. Create the parent (epic) issue
TMPFILE=$(mktemp)
cat > "$TMPFILE" << 'BODY'
Epic description here.
BODY
gh issue create --title "Epic: ..." --body-file "$TMPFILE" --label epic --project "Project Name" -R OWNER/REPO
```

```bash
# 2. Get the parent node ID (separate bash call)
PARENT_ID=$(gh api graphql \
  -f owner="OWNER" -f repo="REPO" -F number=PARENT_NUMBER \
  -f query='query($owner:String!,$repo:String!,$number:Int!) {
    repository(owner:$owner,name:$repo) { issue(number:$number) { id } }
  }' --jq '.data.repository.issue.id')
```

```bash
# 3. Get a child node ID (separate bash call, repeat per child)
CHILD_ID=$(gh api graphql \
  -f owner="OWNER" -f repo="REPO" -F number=CHILD_NUMBER \
  -f query='query($owner:String!,$repo:String!,$number:Int!) {
    repository(owner:$owner,name:$repo) { issue(number:$number) { id } }
  }' --jq '.data.repository.issue.id')
```

```bash
# 4. Link child to parent (separate bash call, repeat per child)
gh api graphql -H "GraphQL-Features: sub_issues" \
  -f parentId="$PARENT_ID" -f childId="$CHILD_ID" \
  -f query='mutation($parentId:ID!,$childId:ID!) {
    addSubIssue(input:{issueId:$parentId,subIssueId:$childId}) { issue { id } }
  }'
```

### Recipe 2: Set up issue dependencies (blocked-by)

```bash
# 1. Get the database ID of the BLOCKING issue (not the issue number!)
BLOCKING_DB_ID=$(gh api repos/OWNER/REPO/issues/BLOCKING_NUMBER --jq '.id')

# 2. Mark the blocked issue as blocked-by
gh api -X POST repos/OWNER/REPO/issues/BLOCKED_NUMBER/dependencies/blocked_by \
  -F issue_id=$BLOCKING_DB_ID
```

### Recipe 3: Bulk-create sub-issues for an epic

Create multiple child issues and link them all to a parent in one flow.

**Important:** Each `gh api graphql` call must be a **separate** bash invocation (see GraphQL Call Hygiene in SKILL.md).

```bash
# 1. Create the parent (epic) issue and capture its number
TMPFILE=$(mktemp)
cat > "$TMPFILE" << 'BODY'
Epic description here.
BODY
PARENT_NUM=$(gh issue create --title "Epic: ..." --body-file "$TMPFILE" --label epic -R OWNER/REPO | grep -oE '[0-9]+$')
```

```bash
# 2. Create child issues and collect their numbers
CHILD_NUMS=()
for TITLE in "Task A" "Task B" "Task C"; do
  TMPFILE=$(mktemp)
  cat > "$TMPFILE" << BODY
Description for ${TITLE}.
BODY
  NUM=$(gh issue create --title "$TITLE" --body-file "$TMPFILE" -R OWNER/REPO | grep -oE '[0-9]+$')
  CHILD_NUMS+=("$NUM")
done
```

```bash
# 3. Get the parent node ID (separate bash call)
PARENT_ID=$(gh api graphql \
  -f owner="OWNER" -f repo="REPO" -F number=$PARENT_NUM \
  -f query='query($owner:String!,$repo:String!,$number:Int!) {
    repository(owner:$owner,name:$repo) { issue(number:$number) { id } }
  }' --jq '.data.repository.issue.id')
```

```bash
# 4. For each child: get node ID then link to parent (separate bash call per child)
for NUM in "${CHILD_NUMS[@]}"; do
  CHILD_ID=$(gh api graphql \
    -f owner="OWNER" -f repo="REPO" -F number=$NUM \
    -f query='query($owner:String!,$repo:String!,$number:Int!) {
      repository(owner:$owner,name:$repo) { issue(number:$number) { id } }
    }' --jq '.data.repository.issue.id')

  gh api graphql -H "GraphQL-Features: sub_issues" \
    -f parentId="$PARENT_ID" -f childId="$CHILD_ID" \
    -f query='mutation($parentId:ID!,$childId:ID!) {
      addSubIssue(input:{issueId:$parentId,subIssueId:$childId}) { issue { id } }
    }'
done
```

> **Caveat:** This contains two `gh api graphql` calls in the same bash loop. While the "one GraphQL call per bash invocation" rule is ideal, this loop pattern works reliably since the two calls are sequential (not chained with `&&`). If you hit encoding errors, split each iteration into its own bash call.

---

## Labels

```bash
gh label create "bug-critical" --color "FF0000" --description "Critical bugs" -R OWNER/REPO
gh label create "bug-critical" --color "FF0000" --force -R OWNER/REPO   # upsert
gh label list --json name,color,description -R OWNER/REPO
gh label list --search "bug" -R OWNER/REPO
gh label edit "bug-critical" --description "New desc" --color "FF5500" -R OWNER/REPO
gh label delete "help wanted" -R OWNER/REPO
```

---

## Dependencies — Blocked By / Blocks (REST API)

No CLI flags exist yet. Uses **database IDs** (large integers), NOT issue numbers.

### Add dependency

```bash
# Get database ID of the BLOCKING issue
BLOCKING_DB_ID=$(gh api repos/OWNER/REPO/issues/10 --jq '.id')

# Mark issue #42 as blocked by issue #10
gh api -X POST repos/OWNER/REPO/issues/42/dependencies/blocked_by \
  -F issue_id=$BLOCKING_DB_ID
```

**Gotcha:** Must use `-F` (capital) for `issue_id` so it sends as integer, not string.

**Gotcha:** Using the issue number instead of the database ID will silently link the wrong issue.

### Query dependencies

```bash
# What blocks issue #42?
gh api repos/OWNER/REPO/issues/42/dependencies/blocked_by --jq '.[] | {number: .number, title: .title, state: .state}'
# What does issue #42 block?
gh api repos/OWNER/REPO/issues/42/dependencies/blocking --jq '.[] | {number: .number, title: .title, state: .state}'
```

### Remove dependency

```bash
BLOCKING_DB_ID=$(gh api repos/OWNER/REPO/issues/10 --jq '.id')
gh api -X DELETE repos/OWNER/REPO/issues/42/dependencies/blocked_by/$BLOCKING_DB_ID
```

---

## Sub-Issues — Parent/Child (GraphQL API)

Requires header `-H "GraphQL-Features: sub_issues"`. Limits: 100 sub-issues per parent, 8 nesting levels.

**Remember: each `gh api graphql` call must be a separate bash invocation.**

### Link existing issue as sub-issue

Each step below is a **separate** bash call. Use `-F` (capital) for `number` since it is `Int!`.

```bash
# Call 1: get parent node ID
PARENT_ID=$(gh api graphql \
  -f owner="OWNER" -f repo="REPO" -F number=10 \
  -f query='
  query($owner:String!, $repo:String!, $number:Int!) {
    repository(owner:$owner, name:$repo) {
      issue(number:$number) { id }
    }
  }' --jq '.data.repository.issue.id')
```

```bash
# Call 2: get child node ID
CHILD_ID=$(gh api graphql \
  -f owner="OWNER" -f repo="REPO" -F number=42 \
  -f query='
  query($owner:String!, $repo:String!, $number:Int!) {
    repository(owner:$owner, name:$repo) {
      issue(number:$number) { id }
    }
  }' --jq '.data.repository.issue.id')
```

```bash
# Call 3: link them
gh api graphql \
  -H "GraphQL-Features: sub_issues" \
  -f parentId="$PARENT_ID" \
  -f childId="$CHILD_ID" \
  -f query='
  mutation($parentId:ID!, $childId:ID!) {
    addSubIssue(input:{issueId:$parentId, subIssueId:$childId}) {
      issue { id }
    }
  }'
```

### Create new issue and link as sub-issue

Each step is a **separate** bash call. Get PARENT_ID first (see above).

```bash
# Call 1: get repo ID
REPO_ID=$(gh api graphql \
  -f owner="OWNER" -f repo="REPO" \
  -f query='
  query($owner:String!, $repo:String!) {
    repository(owner:$owner, name:$repo) { id }
  }' --jq '.data.repository.id')
```

```bash
# Call 2: create child issue
CHILD_ID=$(gh api graphql \
  -f repoId="$REPO_ID" -f title="Subtask title" -f body="Subtask body" \
  -f query='
  mutation($repoId:ID!, $title:String!, $body:String) {
    createIssue(input:{repositoryId:$repoId, title:$title, body:$body}) {
      issue { id }
    }
  }' --jq '.data.createIssue.issue.id')
```

```bash
# Call 3: link to parent
gh api graphql \
  -H "GraphQL-Features: sub_issues" \
  -f parentId="$PARENT_ID" -f childId="$CHILD_ID" \
  -f query='
  mutation($parentId:ID!, $childId:ID!) {
    addSubIssue(input:{issueId:$parentId, subIssueId:$childId}) {
      issue { id }
    }
  }'
```

### List sub-issues

```bash
gh api graphql \
  -H "GraphQL-Features: sub_issues" \
  -f owner="OWNER" -f repo="REPO" -F number=10 \
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

### Remove sub-issue

```bash
gh api graphql \
  -H "GraphQL-Features: sub_issues" \
  -f parentId="$PARENT_ID" -f childId="$CHILD_ID" \
  -f query='
  mutation($parentId:ID!, $childId:ID!) {
    removeSubIssue(input:{issueId:$parentId, subIssueId:$childId}) {
      issue { id }
    }
  }'
```

---

## Milestones (REST API — no native `gh milestone` command)

```bash
gh api -X POST repos/OWNER/REPO/milestones \
  -f title="v2.0" -f description="Major release" -f due_on="2026-06-30T00:00:00Z"
gh api repos/OWNER/REPO/milestones \
  --jq '.[] | {number: .number, title: .title, due_on: .due_on, open_issues: .open_issues, closed_issues: .closed_issues}'
gh api repos/OWNER/REPO/milestones -f state=closed --jq '.[] | {number: .number, title: .title}'  # list closed
gh api -X PATCH repos/OWNER/REPO/milestones/1 -f title="v2.1" -f due_on="2026-07-30T00:00:00Z"
gh api -X PATCH repos/OWNER/REPO/milestones/1 -f state="closed"   # close
gh api -X DELETE repos/OWNER/REPO/milestones/1                    # delete (permanent)
gh issue edit 42 --milestone "v2.0" -R OWNER/REPO                    # assign to issue
```

---

## Projects v2

Projects are scoped to an **org or user**, not a repo. Use `--owner ORG_NAME`.

**Auth:** If you get permission errors, run `gh auth refresh -s project,read:project --hostname github.com` (the `--hostname` flag is required for non-interactive/agent use).

```bash
gh project create --owner ORG_NAME --title "My Project" --format json
gh project list --owner OWNER
gh project field-list PROJECT_NUMBER --owner OWNER --json id,name,type,options
gh project item-list PROJECT_NUMBER --owner OWNER --json id,title,status
gh issue create --title "..." --project "Roadmap" -R OWNER/REPO   # adds to project at creation

# Board operations — see SKILL.md "Project Board" section
```

Add existing issue to project (GraphQL):

```bash
gh api graphql \
  -f projectId="PVT_xxxxx" -f issueId="I_xxxxx" \
  -f query='
  mutation($projectId:ID!, $issueId:ID!) {
    addProjectV2ItemById(input:{projectId:$projectId, contentId:$issueId}) {
      item { id }
    }
  }'
```

Update project item field (one field per call):

```bash
gh project item-edit --id ITEM_ID --project-id PROJECT_ID \
  --field-id FIELD_ID --single-select-option-id OPTION_ID
# Other types: --text "val", --number 42, --date "2026-06-30", --clear
```

**Gotcha:** Only one field per `item-edit` call. Views (Table/Board/Roadmap) are web UI only.

---

## What Does NOT Work from CLI

| Operation                             | Workaround                                         |
| ------------------------------------- | -------------------------------------------------- |
| File attachments on issues            | Web UI only; link external URLs in markdown        |
| Sub-issue creation in one step        | Create issue then link via GraphQL                 |
| Clear milestone (`--milestone ""`)    | Use GraphQL `updateIssue` with `milestoneId: null` |
| Dependency CLI flags (`--blocked-by`) | Use `gh api` REST calls                            |
| Multiple project fields at once       | Loop `item-edit` calls                             |
| Project view creation                 | Web UI only                                        |
