```yaml
kind: prompt
name: qa
description: "Verify implementation quality and acceptance criteria coverage"
inputs:
  - name: mission
    required: true
  - name: worktree_path
    required: true
  - name: context
    required: false
    default: ""
outputs:
  - name: report
    format: json
    schema:
      type: object
      properties:
        verdict: { type: string, enum: [PASS, FAIL] }
        summary: { type: string }
        test_run: { type: object, properties: { passed: { type: integer }, failed: { type: integer } } }
      required: [verdict, summary, test_run]
```

## Mission

{{mission}}

## Instructions

1. Read the plan and context from prior steps
2. Explore `{{worktree_path}}` to understand:
   - The testing framework already in use
   - Existing test patterns, file locations, and naming conventions
   - What code and tests were changed (via `git diff` or recent commits)
3. Review test adequacy for changed behavior:
   - New or modified public functions and methods
   - Edge cases, error paths, and boundary conditions
   - Integration points between changed and existing code
4. Run the full test suite:
   - Use the project's test runner (for example: `mise run test`, `cargo test`, `bun test`)
   - Run linting and type checks if available
5. Validate acceptance criteria coverage from the mission or linked issue
6. If validation fails, report concrete gaps so the workflow can return to `implement`.
7. **Action:** Post your final verdict (PASS/FAIL) and a brief summary of the results as a comment to the GitHub issue referenced in the Mission using the `gh` CLI.

## Output

Your output MUST be valid JSON:

```json
{
  "verdict": "PASS",
  "summary": "One-sentence assessment of quality and coverage",
  "tests_reviewed": [
    {
      "file": "tests/test_feature.rs",
      "tests": ["test_happy_path", "test_invalid_input", "test_edge_case"]
    }
  ],
  "test_run": {
    "passed": 42,
    "failed": 0,
    "skipped": 1
  },
  "coverage_notes": "What is covered and any gaps",
  "issues": [
    {
      "severity": "critical | warning | nit",
      "message": "Description of concern"
    }
  ]
}
```

`verdict` must be exactly `PASS` or `FAIL`. Use `FAIL` if any test/check fails or if critical acceptance criteria are not covered.

## Constraints

- **Tools:** Use the `gh` CLI to post your verdict.

- Use the existing test framework; do not introduce a new one
- Match the project's test style and conventions exactly
- QA is verification-only: do not modify production code or tests
- Do not create commits
- If a production bug is found, report it in `issues` and set `verdict` to `FAIL`
- Keep findings concrete and actionable

## Context from prior steps

{{context}}
