<!--file:execute-epic.md-->

# Execute Epic (Orchestrator, GitHub Issues)

## Objective

Manage the execution of an Epic by analyzing the dependency graph of its child issues and spawning isolated, parallel agentic workers for individual tasks.

## Input

- **Epic Issue Number**: The GitHub issue number for the Epic (e.g., `#33`).
- **Instruction Source**: `gh/execute-task.md` (Content to be injected into worker containers).
- **Max Concurrency**: `4` (Maximum concurrent Docker containers).
- **Repository**: Owner/repo or current working directory.

## Process

1. **Initialization**

- Read Epic details: `gh issue view <number> --comments`
- Mark Epic as in progress: `gh issue edit <number> --add-label in-progress`
- Verify `gh/execute-task.md` exists and is readable.

2. **Dependency Analysis & Scheduling**

- **Loop** (Repeat until all child issues are resolved):

1. **Fetch State**: List child issues for the Epic.

```bash
# List open sub-issues
gh issue list --search "label:task state:open" --json number,title,labels,state
```

Or if using sub-issues:

```bash
gh api graphql -f query='
  query {
    repository(owner: "OWNER", name: "REPO") {
      issue(number: EPIC_NUMBER) {
        subIssues(first: 50) {
          nodes { number title state labels(first:10) { nodes { name } } }
        }
      }
    }
  }'
```

2. **Filter Ready Tasks**: Identify issues that are:
   - State: `open`
   - NOT labeled `in-progress` (not claimed by another worker)
   - NOT labeled `blocked` (no unresolved blockers)
   - NOT labeled `needs-review` (already completed, awaiting review)

3. **Check Completion**:
   - If all children are `closed` or labeled `needs-review` -> **Proceed to Step 4**.
   - If no tasks are ready but some are `in-progress` -> **Wait** (poll every 60s).
   - If no tasks are ready and nothing is running -> **Error** (Deadlock detected).

3. **Worker Execution (Parallel)**

- For each **Ready Task** identified in Step 2:
- **Launch Container**:
  Start a new agent process in Docker. Pass the content of `execute-task.md` as the prompt and the Issue Number as the target.

```bash
docker run \
  --rm \
  --name "agent-issue-<number>" \
  -v $(pwd):/repo \
  -w /app \
  -e GH_TOKEN=$GH_TOKEN \
  -e TARGET_ISSUE="<number>" \
  -e AGENT_SYSTEM_PROMPT="$(cat gh/execute-task.md)" \
  agent-image:latest \
  run-agent
```

- **Monitor**:
  - Track container exit codes.
  - If exit code `0`: Assume task completed (agent should have created PR and labeled `needs-review`).
  - If exit code `!= 0`: Log error, post comment on Epic, and remove `in-progress` label (do not retry in infinite loops).

4. **Epic Finalization**

- Verify all child issues are labeled `needs-review` or `closed`.
- Add a summary comment to the Epic with work completed.
- Label the Epic `needs-review`: `gh issue edit <number> --remove-label in-progress --add-label needs-review`

## Constraints

- **Concurrency**: Limit to **4** concurrent Docker containers (see Max Concurrency input).
- **Context Isolation**: Workers must strictly use the `execute-task.md` protocol. They do not share memory; they only coordinate via GitHub Issue labels and state.
- **Safe Failover**: If the orchestrator crashes, it must be able to restart and resume from the current state of the GitHub Issues (stateless orchestration).

## Error Handling

- **Orphaned Tasks**: If a container dies without updating the issue, the Orchestrator must detect the timeout, kill the container, and remove the `in-progress` label with a comment explaining the failure.
- **Scope Creep**: If a worker requests to create new issues, the Orchestrator must approve them and link them to the Epic as sub-issues.
