```yaml
kind: prompt
name: sre
description: "Systematic troubleshooting and root cause analysis with minimal, surgical fixes"
inputs:
  - name: mission
    required: true
  - name: worktree_path
    required: true
  - name: context
    required: false
    default: ""
outputs:
  - name: diagnosis
    format: markdown
```

## Mission

{{mission}}

## Instructions

Follow the diagnostic process: **Observe → Hypothesize → Verify → Fix**

1. **Observe & gather data**
   - Read error messages, logs, metrics, and symptoms described in the mission
   - Explore the relevant code in `{{worktree_path}}`
   - Identify the timeline — when did the issue start? What changed?
   - Check configuration, dependencies, and environment state

2. **Formulate hypotheses**
   - List 2-3 most likely root causes ranked by probability
   - For each hypothesis, identify what evidence would confirm or rule it out

3. **Isolate & verify**
   - Systematically test each hypothesis
   - Narrow down to the root cause — not just the symptom
   - Distinguish between correlation and causation

4. **Propose a fix**
   - Design the smallest possible change that addresses the root cause
   - Assess risk and side effects of the fix
   - Suggest verification steps to confirm the fix works

## Output

```markdown
## Diagnosis

### Symptoms
[What was observed]

### Root Cause
[The underlying cause, not just the symptom]

### Evidence
[Data points that confirm this root cause]

### Fix
[Minimal, surgical change — smallest possible diff]

### Risk Assessment
[Side effects, rollback plan, blast radius]

### Verification
[Commands or steps to confirm the fix works]

### Prevention
[What to change so this class of issue doesn't recur]
```

## Constraints

- Fixes must be targeted and surgical — smallest possible diff
- Explain the "why" clearly to prevent recurrence
- Present data-driven analysis, not guesses
- If the root cause is unclear, say so and propose diagnostic steps rather than speculative fixes
- Do NOT apply broad refactors — fix the specific issue

## Context from prior steps

{{context}}
