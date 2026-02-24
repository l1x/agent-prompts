<!--file:execute-task.md-->

# Execute Task (GitHub Issues)

## Objective

Implement a single, atomic task defined in a GitHub Issue, ensuring code quality and verification before completion.

## Input

- **Issue Number**: The GitHub issue number to execute (e.g., `42`).
- **Repository**: Owner/repo or current working directory.

## Tools

- use mise when feasible
- use gh CLI for issue management

## Process

1. **Setup env**
   - cd /home/agent
   - mise trust
   - mise git
   - mise agent

1. **Claim Task**
   - cd /home/agent/workspace
   - mise trust
   - Read the issue details: `gh issue view <number> --comments`
   - Create a branch: `git checkout -b issue-<number>-<short-slug>`
   - Mark the issue as in progress: `gh issue edit <number> --add-label in-progress`
   - git push -u origin HEAD

1. **Context Loading**
   - Read the parent Epic if referenced (check for `Part of #N` in the issue body) to understand the broader scope.
   - Explore relevant source code sections.
   - Ignore compiled or otherwise generated content.
   - Add a comment with a short summary of your plan: `gh issue comment <number> --body "Plan: ..."`

1. **Implementation**
   - **Create Test**: Write a failing test case that reproduces the requirement or bug.
   - **Implement**: Write the minimal code necessary to satisfy the requirements and to pass the test.
   - **Refactor**: Clean up code while keeping tests passing.

1. **Verification**
   - Run formatting: `mise run fmt`
   - Run linting: `mise run lint`
   - Run tests: `mise run tests`
   - Run security audit (if applicable): `mise run audit`

1. **Completion**
   - If verification fails, fix issues and repeat Step 4.
   - If verification passes:
     - Commit changes and push the branch
     - Create a PR linking the issue: `gh pr create --title "..." --body "Closes #<number>"`
     - Remove in-progress, add needs-review: `gh issue edit <number> --remove-label in-progress --add-label needs-review`

## Available mise Tasks

**Code Quality:**

- `mise run fmt` - Format code
- `mise run lint` - Lint with warnings as errors
- `mise run tests` - Run all tests with output
- `mise run audit` - Run security audit on dependencies

**Building:**

- `mise run build-dev` - Build development version
- `mise run build-prod` - Build release version

**Testing & Analysis:**

- `mise run coverage` - Run tests with coverage report
- `mise run coverage-html` - Generate HTML coverage report
- `mise run machete` - Find unused dependencies
- `mise run check-deps` - Run both audit and machete

## Constraints

- **Scope**: Focus ONLY on the specified issue. Do not implement extra features.
- **Quality**: Code must compile and pass all verification steps.
- **Atomic**: If the task is too large, stop and request to split it into smaller issues.

## Error Handling

- If you encounter a blocker (missing dependency, unclear requirement), add a comment to the issue (`gh issue comment <number> --body "Blocker: ..."`) and add the `blocked` label. Stop.
