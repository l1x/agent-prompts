```yaml
kind: prompt
name: implement
description: "Implement changes per plan, run quality gates, commit"
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
3. Implement the changes described in the plan
4. Run quality gates (see stack-specific instructions if present)
5. Fix any failing checks before committing
6. Commit changes with the commit message from the plan (Conventional Commits format)

## Output

Return a summary of:

- What was done
- Files changed
- Quality gate results

## Constraints

- Follow the plan — do not add features or refactor beyond what is specified
- Keep commits atomic and well-described
- If something in the plan is unclear, make a reasonable choice and document it
- Do NOT push or create PRs
- Stop after committing

## Context from prior steps

{{context}}
