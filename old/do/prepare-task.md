# Precise Context Specification

Purpose: Curate precise context for AI coding agents - providing exactly what they need to understand and complete the task, without overwhelming them with irrelevant code.

## Context Components

A well-structured context block for AI coding agents (delivered as XML) consists of:

1.  Instructions - What the agent should do
1.  File Tree - Project structure for navigation context
1.  Codemaps - Structural summaries (signatures, types, interfaces) without implementation (~10x fewer tokens than full files)
1.  Selected Files - Full content of files being modified
1.  Slices - Specific line ranges from large files (e.g., L45-120)

File Selection Modes

| Mode    | Content               | Use Case                            |
| ------- | --------------------- | ----------------------------------- |
| Full    | Complete file         | Files being actively edited         |
| Slices  | Line ranges only      | Large files where only part matters |
| Codemap | Signatures/types only | Reference files, APIs, dependencies |

Key Principles

- Codemaps extract function signatures, class definitions, type declarations - giving structural understanding without implementation bloat
- Slices can turn a 2000-line file into a 50-line extract containing just the relevant function
- Token budgeting matters - leave room for the agent's response (10-20k for plans, more for code generation)

## XML generation

The xml should go to the description section of the task as quoted xml (using the triple backtick method)

The XML should specify:

- Task instructions
- Files needed (with mode: full, codemap, or slice with line ranges)
- Relevant dependencies to include as codemaps
- Any git diff context if continuing work
- Behavioral guidance (role, constraints)

## Example:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<task>
  <instructions>
    <summary>Brief description of the task</summary>
    <objective>What success looks like</objective>
    <constraints>Any limitations or requirements</constraints>
  </instructions>

  <context>
    <tree root="src/">
        src/
        ├── app.rs
        ├── beads.rs
        ├── error.rs
        ├── handlers
        │   ├── board.rs
        │   ├── general.rs
        │   ├── graph.rs
        │   ├── landing.rs
        │   ├── metrics.rs
        │   ├── prds.rs
        │   └── tasks.rs
        ├── handlers.rs
        ├── lib.rs
        ├── main.rs
        ├── markdown.rs
        └── templates.rs
    </tree>

    <files>
      <file path="src/main.rs" mode="full"/>
      <file path="src/lib.rs" mode="codemap"/>
      <file path="src/beads.rs" mode="slice" lines="45-120"/>
    </files>

    <dependencies>
      <!-- External crates/APIs to include as codemaps -->
      <dep name="tokio" items="spawn,select,JoinHandle"/>
    </dependencies>

    <diff>
      <![CDATA[
      --- a/src/lib.rs
      +++ b/src/lib.rs
      @@ -10,3 +10,5 @@
      ...
      ]]>
    </diff>

  </context>

  <guidance>
    <role>e.g., senior Rust engineer</role>
    <style>e.g., idiomatic, minimal unsafe</style>
  </guidance>
</task>
```
