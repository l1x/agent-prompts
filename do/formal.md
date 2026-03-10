```yaml
kind: prompt
name: formal
description: "Formal verification with Kani proof harnesses and TLA+ concurrency models"
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

### Phase 1 — Identify verification targets

1. Explore `{{worktree_path}}` and identify code that requires formal verification:
   - Manual buffer management, pointer arithmetic, or custom encoding logic
   - Arithmetic that could overflow or underflow
   - `unsafe` blocks and FFI boundaries
   - Concurrent or asynchronous state machines (background threads, channels, shared state)
   - Retry loops, bounded queues, or flush-on-drop patterns
2. Classify each target as **memory safety** (Kani) or **concurrency/liveness** (TLA+)

### Phase 2 — Kani proof harnesses

For each memory safety target:

1. Write a `#[kani::proof]` harness in the same module, gated behind `#[cfg(kani)]`
2. Use `kani::any()` to generate symbolic inputs covering the full domain
3. Prove the relevant properties:
   - No out-of-bounds access
   - No arithmetic overflow or underflow
   - No undefined behavior in `unsafe` blocks
   - Function contracts hold for all inputs (preconditions → postconditions)
4. Add `kani-verifier` to `[dev-dependencies]` if not already present
5. Verify harnesses pass locally with `cargo kani`

### Phase 3 — TLA+ concurrency models

For each concurrency target:

1. Write a TLA+ (or PlusCal) specification in a `specs/` directory
2. Model the relevant processes and their interactions
3. Define and verify these properties:
   - **Liveness** — the system eventually makes progress (buffers flush, tasks terminate)
   - **Boundedness** — under failure conditions, queues and buffers do not grow without limit
   - **No deadlock** — no circular wait between producers and consumers
4. Document the state space and any simplifying assumptions

### Phase 4 — Report findings

1. Summarize what was verified and any issues discovered
2. If a harness or model reveals a violation, describe the counterexample and suggest a fix

## Output

Your output MUST be valid JSON:

```json
{
  "verdict": "PASS",
  "summary": "One-sentence assessment of verification results",
  "kani_harnesses": [
    {
      "target": "function or module name",
      "file": "src/encoding.rs",
      "properties_verified": ["no overflow", "no out-of-bounds"],
      "status": "proven | counterexample | skipped",
      "notes": "Details or counterexample description"
    }
  ],
  "tla_models": [
    {
      "target": "component or interaction being modeled",
      "file": "specs/exporter.tla",
      "properties_verified": ["liveness", "boundedness", "no deadlock"],
      "status": "verified | violated | skipped",
      "notes": "State space size, assumptions, or violation details"
    }
  ],
  "issues": [
    {
      "severity": "critical | warning | nit",
      "location": "file:line or spec name",
      "message": "Description of the issue",
      "remediation": "Suggested fix"
    }
  ]
}
```

`verdict` must be exactly `PASS` or `FAIL`. Use `FAIL` if any proof finds a counterexample or a model reveals a property violation.

## Constraints

- Verification code must be strictly separated from production code via `#[cfg(kani)]`
- Do not use `unsafe` in harnesses — harnesses verify existing code, they do not add new risk
- Kani harnesses must be deterministic and self-contained (no I/O, no network)
- TLA+ models should be minimal — model only the interactions relevant to the target properties
- If a target cannot be verified with available tools, explain why and describe the best alternative
- Prioritize fire-and-forget and background flush paths — these are the most likely sources of integration bugs
- Do not modify production logic — only add verification artifacts

## Context from prior steps

{{context}}
