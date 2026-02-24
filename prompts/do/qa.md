```yaml
kind: prompt
name: qa
description: "Write tests, run them, and verify coverage against acceptance criteria"
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
```

## Mission

{{mission}}

## Instructions

1. Read the plan and context from prior steps
2. Explore `{{worktree_path}}` to understand:
   - The testing framework already in use (do not introduce a new one)
   - Existing test patterns, file locations, and naming conventions
   - What code was changed or added (via `git diff` or recent commits)
3. Identify what needs testing:
   - New or modified public functions and methods
   - Edge cases, error paths, and boundary conditions
   - Integration points between changed and existing code
4. Write tests following existing project conventions:
   - Place test files where the project already puts them
   - Match the assertion style, setup/teardown patterns, and naming used elsewhere
   - Prefer testing behavior over implementation details
5. Run the full test suite:
   - Use the project's test runner (e.g., `mise run test`, `cargo test`, `bun test`)
   - Fix any failing tests — both new and existing
   - Run linting and type checks if available
6. Verify coverage against acceptance criteria from the mission or linked issue

## Output

Your output MUST be valid JSON:

```json
{
  "verdict": "PASS",
  "summary": "One-sentence assessment of test coverage and results",
  "tests_written": [
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

`verdict` must be exactly `PASS` or `FAIL`. FAIL if any test fails or if critical acceptance criteria lack coverage.

## Constraints

- Use the existing test framework — do not add a new one
- Match the project's test style and conventions exactly
- Do not modify production code — only add or fix tests
- If production code has a bug that causes test failure, report it in `issues` and FAIL
- Keep tests focused and independent — no test should depend on another's state
- Do not test private internals unless the project already does so
- Commit test files with message format: `test(scope): description`

## Context from prior steps

{{context}}
