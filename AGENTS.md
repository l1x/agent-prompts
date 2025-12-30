# AGENTS.md

## Project Overview

This repository contains reusable prompts for AI agents. It is designed to be included as a git submodule (typically at `.agent-prompts/`) in other projects.

## Repository Structure

```
agent-prompts/
├── do/           # Action prompts (tasks the agent should perform)
│   ├── create-prd.md
│   ├── generate-tasks.md
│   ├── execute-epic.md
│   └── execute-task.md
└── context/      # Context prompts (background information for agents)
    └── project-management.md
```

## Prompt Conventions

- **File naming:** Use `kebab-case.md` for all prompt files
- **File header:** Start each prompt with an HTML comment containing the filename: `<!--file:filename.md-->`
- **Structure:** Include clear sections: Objective, Process, Output, Constraints
- **Target audience:** Write for junior developers with strong technical skills but limited domain knowledge
- **Be explicit:** Avoid jargon, explain concepts clearly

## Adding New Prompts

1. Determine the category: `do/` for actions, `context/` for background information
2. Create a new `.md` file following the naming convention
3. Include the file header comment
4. Follow the structure of existing prompts in that category

## Testing Prompts

Since this is a prompt collection (not code), there are no automated tests. Instead:

- Review prompts for clarity and completeness before committing
- Test prompts manually with an AI agent to verify they produce expected outputs
- Ensure prompts are self-contained and don't assume external context

## PR Instructions

- **Title format:** `[prompts] <Brief description>`
- **Description:** Explain what the prompt does and when to use it
- Review your markdown for formatting issues before submitting
- Ensure the prompt follows existing conventions in this repository

## Using This Repository

In your project:

```bash
# Add as submodule
git submodule add https://github.com/l1x/agent-prompts.git .agent-prompts

# Update to latest
git submodule update --remote
```

Reference prompts from `.agent-prompts/do/` or `.prompts/context/` as needed.
