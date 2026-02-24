```yaml
kind: prompt
name: review
description: "Review recent commits and diffs, verify against GitHub issue, and produce a structured PASS/FAIL verdict"
inputs:
  - name: mission
    required: true
  - name: worktree_path
    required: true
  - name: github_issue
    required: true
  - name: context
    required: false
    default: ""
outputs:
  - name: verdict
    format: json
```

## Mission

{{mission}}

## Instructions

1. Read recent commits in `{{worktree_path}}`:
   - Identify task-relevant commits (prefer commits since branch point; fallback to latest 5 commits)
   - Review commit messages and patches for each commit
2. Read current workspace changes in `{{worktree_path}}` using `git diff` and `git diff --cached`
3. Use `{{github_issue}}` as the target GitHub issue (issue number or URL)
4. Fetch and read the GitHub issue details (title, description, acceptance criteria)
5. Read the plan and context from prior steps
6. Evaluate the changes:
   - **Issue alignment**: Does implementation satisfy the GitHub issue scope and acceptance criteria?
   - **Correctness**: Does the code do what the plan specified?
   - **Style**: Does it follow existing codebase conventions?
   - **Security**: Any injection, XSS, or other vulnerabilities?
   - **Performance**: Any obvious inefficiencies or regressions?
   - **Tests**: Are there adequate tests for the changes?
7. Run quality gates if stack-specific instructions are present

## Output

Your output MUST be valid JSON:

```json
{
  "verdict": "PASS",
  "summary": "One-sentence overall assessment",
  "reviewed_commits": [
    "abc1234 feat(scope): summary"
  ],
  "issue_validation": {
    "issue": "#123",
    "status": "met",
    "notes": "How the implementation maps to issue requirements"
  },
  "issues": [
    {
      "severity": "critical | warning | nit",
      "file": "path/to/file",
      "line": 42,
      "message": "Description of the issue"
    }
  ]
}
```

`verdict` must be exactly `PASS` or `FAIL`.
`issue_validation.status` must be one of `met`, `partially_met`, `not_met`, or `unreadable`.

## Constraints

- Be constructive — if you fail the review, explain exactly what needs fixing
- Only FAIL for real issues, not style preferences
- PASS if the implementation is correct and complete, even if minor improvements are possible
- Review both recent commits and current diffs before deciding
- Validate against the GitHub issue before returning PASS
- If issue details are missing/unreadable, set `issue_validation.status` to `unreadable` and return FAIL with a critical issue
- Do not rewrite the code — only review it

## Context from prior steps

{{context}}
