<!--file:execute-task.md-->

# Execute Task

## Objective

Implement a single, atomic task defined in a Beads ticket, ensuring code quality and verification before completion.

## Input

- **Ticket ID**: The `bd` ticket ID to execute (e.g., `bd-a3f8.1`).

## Tools

- use mise when feasible
- use bd for tasks management

## Process

1. **Setup env**
   - cd /home/agent
   - mise trust
   - mise git
   - mise agent

1. **Claim Task**
   - cd /home/agent/workspace
   - mise trust
   - bd init
   - git push -u origin BRANCH_NAME
   - Read the task details: `bd show <id>`
   - Mark the task as in progress: `bd update <id> --status=in_progress`
   - run `bd sync`

1. **Context Loading**
   - Read the parent Epic or PRD if referenced to understand the broader scope.
   - Explore relevant source code sections.
   - Ignore compiled or otherwise generated content.
   - Update the task description with a short summary and

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
     - Update the ticket description: prepend "**PENDING REVIEW**\n\n" to the top, then add a summary of changes made, keeping the original description intact below
     - Mark ticket blocked: `bd update <id> --status=blocked`
     - Run `bd sync`

## Available mise Tasks

**Code Quality:**

- `mise run fmt` - Format code with cargo fmt
- `mise run lint` - Lint with Clippy, failing on warnings
- `mise run tests` - Run all tests with output
- `mise run audit` - Run security audit on dependencies

**Building:**

- `mise run build-dev` - Build development version
- `mise run build-prod` - Build release version (includes frontend)
- `mise run build-js-prod` - Build frontend TypeScript only

**Testing & Analysis:**

- `mise run coverage` - Run tests with coverage report
- `mise run coverage-html` - Generate HTML coverage report
- `mise run machete` - Find unused dependencies
- `mise run check-deps` - Run both audit and machete

## Constraints

- **Scope**: Focus ONLY on the specified ticket. Do not implement extra features.
- **Quality**: Code must compile and pass all verification steps.
- **Atomic**: If the task is too large, stop and request to split it into smaller tickets.

## Error Handling

- If you encounter a blocker (missing dependency, unclear requirement), add a comment to the ticket (`bd comment <id> "Blocker: ..."`) and move the ticket to be blocked and stop.
- Run `bd sync`
