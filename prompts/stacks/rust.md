```yaml
kind: prompt
name: rust
description: "Rust conventions, idioms, and quality gates"
inputs: []
outputs: []
```

## Rust Conventions

- Write idiomatic Rust: proper error handling with `?` and `anyhow`/`thiserror`
- No `unwrap()` in production code — use `expect()` only where panics are acceptable and document why
- Use existing patterns in the codebase (if the project uses `tracing`, use `tracing`)
- Keep functions small and focused
- Match the project's public API documentation style
- Commit only the files you changed

## Quality Gates

Run these before committing, in order:

1. `cargo fmt` — format all code
2. `cargo clippy --workspace --all-targets -- -D warnings` — fix all warnings
3. `cargo test --workspace` — ensure all tests pass

If any gate fails, fix the issue and re-run.

## Patterns

- Use `&str` over `String` in function parameters when ownership is not needed
- Prefer iterators over manual loops
- Use `Cow<'_, str>` to avoid unnecessary cloning
- Use `#[derive(...)]` for standard traits when appropriate
- Avoid unnecessary `.clone()` calls
