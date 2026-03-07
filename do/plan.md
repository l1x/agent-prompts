```yaml
kind: prompt
name: plan
description: "Explore codebase, design implementation approach, and produce precise context for the worker"
inputs:
  - name: mission
    required: true
  - name: worktree_path
    required: true
  - name: context
    required: false
    default: ""
outputs:
  - name: implementation_prompt
    format: xml
```

## Mission

{{mission}}

## Instructions

1. Read and understand the mission above
2. Explore the codebase in `{{worktree_path}}` to understand the relevant code
3. Map the architecture — identify key modules, patterns, and conventions
4. Identify files that need to be created or modified
5. Design the implementation approach — consider edge cases and existing patterns
6. Curate the precise context the worker agent needs — no more, no less

## Output

Produce a structured XML context block for the worker agent:

```xml
<task>
  <instructions>
    <summary>What needs to happen</summary>
    <objective>What success looks like</objective>
    <steps>
      1. Concrete implementation step
      2. ...
    </steps>
    <commit>feat(scope): description</commit>
    <constraints>Any limitations or requirements</constraints>
  </instructions>

  <context>
    <tree root="src/">
        src/
        ├── relevant files only
    </tree>

    <files>
      <file path="src/main.rs" mode="full"/>
      <file path="src/lib.rs" mode="codemap"/>
      <file path="src/large_file.rs" mode="slice" lines="45-120"/>
    </files>
  </context>
</task>
```

### File selection modes

| Mode    | Content               | When to use                         |
| ------- | --------------------- | ----------------------------------- |
| full    | Complete file         | Files being actively edited         |
| codemap | Signatures, types, and public interfaces only | Reference files, APIs, dependencies |
| slice   | Specific line ranges  | Large files where only part matters |

### Context curation principles

- Codemaps give structural understanding at ~10x fewer tokens than full files
- Slices turn a 2000-line file into a 50-line extract of the relevant function
- Budget tokens — leave room for the worker's response
- Include only what the worker needs to complete the task

## Constraints

- Do NOT implement anything — only plan and curate context
- Be specific: reference exact file paths, function names, and line numbers
- Consider backward compatibility and existing patterns
- If the mission is ambiguous, state assumptions clearly
- Prefer modifying existing code over creating new files

## Context from prior steps

{{context}}
