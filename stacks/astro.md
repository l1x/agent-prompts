```yaml
kind: prompt
name: astro
description: "Astro/TypeScript conventions and quality gates"
inputs: []
outputs: []
```

## Astro/TypeScript Conventions

- Use `bun` as the package manager — never npm, npx, yarn, or pnpm
- Follow existing Astro component patterns (check `src/components/` for reference)
- TypeScript types go in `src/lib/types.ts`
- API client functions go in `src/lib/api-client.ts`
- Use CSS variables from the design system (e.g., `var(--panel)`, `var(--line)`, `var(--muted)`)
- Keep components small and focused — one concern per file
- Use semantic HTML elements
- Commit only the files you changed

## Quality Gates

Run these before committing:

1. `bun run build` — ensure the project builds without TypeScript errors
2. `bun run check` — run any configured checks (if available)

If the build fails, fix the root cause.
