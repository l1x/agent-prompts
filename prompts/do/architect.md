```yaml
kind: prompt
name: architect
description: "Design system architecture with components, interfaces, data flow, and trade-offs"
inputs:
  - name: mission
    required: true
  - name: worktree_path
    required: true
  - name: context
    required: false
    default: ""
outputs:
  - name: architecture
    format: markdown
```

## Mission

{{mission}}

## Instructions

1. Clarify requirements — extract functional and non-functional needs from the mission
2. Explore the codebase in `{{worktree_path}}` to understand the current architecture
3. Design the system structure using these principles:
   - **Partitioning**: divide into autonomous subsystems that minimize interdependencies
   - **Simplification**: remove non-essential components to reduce overhead and attack surfaces
   - **Iteration**: plan delivery one subset at a time for faster value and adaptability
4. Produce structured documentation (see Output below)

## Output

### Required sections

1. **Summary** — one-paragraph overview of the design
2. **Components** — each component with purpose, responsibilities, and interfaces
3. **Data flow** — how data moves between components (include a Mermaid diagram)
4. **Trade-offs** — decisions made and alternatives considered, with rationale
5. **ADRs** — Architecture Decision Records for major choices:
   - Title, Status, Context, Decision, Consequences
6. **Non-functional requirements** — scalability, reliability, security, operational cost implications
7. **Open questions** — unresolved items that need input

### Diagrams

Use Mermaid for system visualization. Include at minimum:
- Component/service diagram showing boundaries and interfaces
- Data flow diagram showing request/response paths

## Constraints

- Focus on the "what," not the "how" — define structure, not implementation details
- Consider long-term maintainability, scalability, and operational costs
- Simplicity above all — the least complex system that meets the requirements wins
- Do NOT write implementation code
- Be specific about component boundaries and API contracts
- Quantify where possible (expected load, data volume, latency requirements)

## Context from prior steps

{{context}}
