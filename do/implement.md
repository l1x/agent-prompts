```yaml
kind: prompt
name: implement
description: "Implement changes with TDD, run quality gates, commit"
inputs:
  - name: mission
    required: true
  - name: worktree_path
    required: true
  - name: context
    required: false
    default: ""
outputs:
  - name: summary
    format: markdown
  - name: files_changed
    format: list
```

## Mission

{{mission}}

## Instructions

1. Read the plan and context from prior steps
2. Work in `{{worktree_path}}`
3. Break the requested behavior into acceptance criteria
4. For each acceptance criterion, run a TDD loop:
   - RED: write or update a test that fails for the missing behavior
   - GREEN: implement the minimal production change to pass that test
   - REFACTOR: clean up while keeping tests green
5. After all criteria are implemented, run full quality gates (tests, lint, type checks, and formatting if available)
6. Fix any failing checks before committing
7. Commit code and tests together with the commit message from the plan (Conventional Commits format)

## Output

Return a summary of:

- What was implemented
- Tests added or changed per acceptance criterion
- Quality gate results

## Constraints

- Follow the plan; do not add features or refactor beyond what is specified
- Prefer small RED/GREEN iterations over big-bang implementation
- If a criterion cannot be expressed as an automated test, explain why and describe the best available verification
- Keep commits atomic and well-described
- If something in the plan is unclear, make a reasonable choice and document it
- Do NOT push or create PRs
- Stop after committing

## Context from prior steps

{{context}}
