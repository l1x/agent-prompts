<!--file:review-rust-codebase.md-->

# Review Rust Codebase

## Objective

Perform a staff-engineer-level comprehensive review of a Rust codebase, evaluating code organization, reusability, testability, safety, security, and performance. Produce an actionable assessment with specific recommendations prioritized by impact.

## Input

- **Codebase Path**: Root directory of the Rust project (default: current working directory)
- **Focus Areas**: Optional specific modules or concerns to prioritize

## Process

### 1. Project Structure Analysis

**Explore the codebase layout:**
```bash
find . -name "*.rs" | head -50
cat Cargo.toml
ls -la src/
```

**Evaluate:**
- Workspace structure (single crate vs workspace with multiple crates)
- Module organization (`mod.rs` vs `module_name.rs` convention)
- Separation of concerns (lib vs bin, core logic vs IO, domain vs infrastructure)
- Feature flag organization and conditional compilation
- Build configuration (`build.rs`, `Cargo.toml` features)

### 2. Code Organization Review

**Module Architecture:**
- Is the public API surface minimal and intentional? (`pub` vs `pub(crate)` vs private)
- Are modules cohesive (single responsibility) or doing too much?
- Is the dependency graph between modules acyclic and logical?
- Are re-exports used appropriately to create clean public interfaces?

**Naming Conventions:**
- Do types, functions, and modules follow Rust naming conventions (snake_case, CamelCase)?
- Are names descriptive and domain-appropriate?
- Is terminology consistent across the codebase?

**File Organization:**
- Are files appropriately sized (< 500 lines as guideline)?
- Is related functionality colocated?
- Are integration tests in `tests/`, unit tests inline, benchmarks in `benches/`?

### 3. Reusability Assessment

**Abstraction Quality:**
- Are traits used effectively to define behavior contracts?
- Is there unnecessary duplication that should be abstracted?
- Are generics used appropriately (not over-engineered, not under-utilized)?
- Do abstractions have appropriate bounds (`where` clauses)?

**API Design:**
- Do public functions follow the principle of least surprise?
- Are builder patterns used for complex construction?
- Is the API idiomatic Rust (iterators, `Option`/`Result`, method chaining)?
- Are extension traits used appropriately?

**Code Patterns:**
- Is there copy-paste code that should be DRY?
- Are macros used judiciously (not when functions/traits suffice)?
- Is there dead code or unused dependencies?

```bash
cargo +nightly udeps  # Check unused dependencies
cargo clippy -- -W clippy::pedantic  # Aggressive linting
```

### 4. Testability Evaluation

**Test Coverage:**
```bash
cargo tarpaulin --out Html  # Coverage report
cargo test --all-features   # Run all tests
```

**Test Quality:**
- Unit tests for core logic?
- Integration tests for module boundaries?
- Property-based tests for invariants (`proptest`, `quickcheck`)?
- Fuzzing for parsers/deserializers (`cargo-fuzz`)?

**Design for Testability:**
- Is IO separated from business logic (ports and adapters)?
- Are dependencies injectable (trait objects, generics, or parameters)?
- Can components be tested in isolation?
- Are there test utilities/fixtures to reduce boilerplate?

**Test Organization:**
- Are tests close to the code they test?
- Do test names describe the behavior being verified?
- Are there doc tests for public API examples?
- Is there a clear distinction between fast unit tests and slow integration tests?

### 5. Safety Analysis

**Memory Safety:**
- Audit `unsafe` blocks: Are they necessary? Are invariants documented?
- Check raw pointer usage and FFI boundaries
- Review `std::mem::transmute`, `std::ptr::*` usage
- Verify lifetime annotations are correct (not just "make the compiler happy")

```bash
grep -rn "unsafe" src/
cargo +nightly miri test  # Runtime safety checks
```

**Concurrency Safety:**
- Are `Send`/`Sync` bounds correct?
- Is shared mutable state properly synchronized (`Mutex`, `RwLock`, `Atomic*`)?
- Are there potential deadlocks (lock ordering)?
- Is `Arc` vs `Rc` usage appropriate?

**Error Handling:**
- Are errors typed and informative (not just `String` or `Box<dyn Error>`)?
- Is `unwrap()`/`expect()` used only where panics are acceptable?
- Are error chains preserved for debugging (`thiserror`, `anyhow`)?
- Is panic behavior documented for public APIs?

**Type Safety:**
- Are newtypes used to prevent primitive obsession?
- Are enums used to make invalid states unrepresentable?
- Is `NonZero*`, `NonNull`, etc. used where invariants exist?
- Are phantom types used for compile-time state tracking?

### 6. Security Assessment

**Input Validation:**
- Are external inputs validated at boundaries?
- Is deserialization safe (no arbitrary code execution, resource limits)?
- Are SQL/command injection vulnerabilities possible?

**Cryptography:**
- Are crypto primitives from vetted libraries (`ring`, `rustcrypto`)?
- Is `rand` vs `rand::rngs::OsRng` usage appropriate for security contexts?
- Are secrets zeroized after use (`zeroize` crate)?
- Are timing attacks considered for comparisons (`subtle` crate)?

**Dependencies:**
```bash
cargo audit              # Known vulnerabilities
cargo deny check         # License and security policy
cargo tree --duplicates  # Dependency conflicts
```

**Sensitive Data:**
- Are secrets/credentials handled securely (not logged, not in error messages)?
- Is PII appropriately protected?
- Are debug implementations safe (`#[debug_format = "..."]` or manual impl)?

**Denial of Service:**
- Are there unbounded allocations from untrusted input?
- Are recursion limits in place for parsers?
- Is there algorithmic complexity risk (regex, sorting)?

### 7. Performance Analysis

**Profiling:**
```bash
cargo bench                           # Run benchmarks
cargo flamegraph                      # CPU profiling
valgrind --tool=massif ./target/...   # Memory profiling
```

**Memory Efficiency:**
- Are allocations minimized in hot paths?
- Is `&str` vs `String`, `&[T]` vs `Vec<T>` used appropriately?
- Are small string/vec optimizations used where beneficial (`smallvec`, `smartstring`)?
- Is `Box` used to reduce struct size for large variants?
- Is `Cow<'_, T>` used to avoid unnecessary cloning?

**CPU Efficiency:**
- Are iterators used instead of manual loops (enables optimizations)?
- Is unnecessary cloning avoided (`.clone()` audit)?
- Are expensive operations cached where appropriate?
- Is SIMD used for data-parallel operations (`packed_simd`, auto-vectorization)?

**Async Performance:**
- Is blocking code kept off async executors (`spawn_blocking`)?
- Are streams used for large data sets (not collecting everything)?
- Is cancellation handled correctly?
- Are connection pools sized appropriately?

**Compilation:**
- Is incremental compilation working (reasonable rebuild times)?
- Are compile times acceptable? (generic monomorphization costs)
- Is LTO enabled for release builds where beneficial?

## Output Format

```markdown
## Rust Codebase Review: [Project Name]

### Executive Summary
[2-3 sentence assessment of overall code quality and maturity]

### Scores (1-5)
| Category       | Score | Notes                          |
|----------------|-------|--------------------------------|
| Organization   | X/5   | [Brief justification]          |
| Reusability    | X/5   | [Brief justification]          |
| Testability    | X/5   | [Brief justification]          |
| Safety         | X/5   | [Brief justification]          |
| Security       | X/5   | [Brief justification]          |
| Performance    | X/5   | [Brief justification]          |

### Critical Issues (Fix Immediately)
1. **[Issue Title]** - `path/to/file.rs:line`
   - Problem: [Description]
   - Impact: [Security/Safety/Correctness risk]
   - Recommendation: [Specific fix]

### High Priority Recommendations
1. **[Recommendation]**
   - Current state: [What exists now]
   - Proposed change: [What should change]
   - Benefit: [Why this matters]
   - Effort: [Low/Medium/High]

### Medium Priority Improvements
[Similar format]

### Low Priority / Nice-to-Have
[Similar format]

### Strengths
- [What the codebase does well]
- [Patterns worth preserving]

### Tooling Recommendations
- [ ] Add `cargo clippy` to CI with `-D warnings`
- [ ] Add `cargo audit` to CI
- [ ] Add `cargo fmt --check` to CI
- [ ] Set up `cargo tarpaulin` for coverage
- [ ] Consider `cargo deny` for dependency policy

### Appendix: Unsafe Audit
| Location | Purpose | Risk Level | Recommendation |
|----------|---------|------------|----------------|
| `file.rs:123` | FFI call | Medium | Add safety comment |
```

## Review Checklist

### Organization
- [ ] Workspace/crate structure is logical
- [ ] Module hierarchy reflects domain
- [ ] Public API surface is intentional and minimal
- [ ] Naming is consistent and idiomatic

### Reusability
- [ ] Traits define clear behavior contracts
- [ ] Generics are neither over- nor under-used
- [ ] No significant code duplication
- [ ] Macros are justified

### Testability
- [ ] Core logic has unit tests
- [ ] Module boundaries have integration tests
- [ ] IO is separated from business logic
- [ ] Dependencies are injectable

### Safety
- [ ] `unsafe` blocks are minimal and documented
- [ ] Error handling is comprehensive
- [ ] Panics are intentional and documented
- [ ] Lifetimes are correct, not just appeasing the compiler

### Security
- [ ] Dependencies have no known vulnerabilities
- [ ] Input validation at boundaries
- [ ] Secrets are handled appropriately
- [ ] No injection vulnerabilities

### Performance
- [ ] No unnecessary allocations in hot paths
- [ ] No unnecessary cloning
- [ ] Async code doesn't block
- [ ] Benchmarks exist for critical paths

## Constraints

- **Depth vs Breadth**: For large codebases, focus on core modules and public API rather than exhaustive file-by-file review
- **Actionability**: Every finding should have a concrete recommendation
- **Prioritization**: Rank findings by impact and effort to help teams plan
- **Objectivity**: Base recommendations on Rust idioms, documented best practices, and measurable impact
- **Context**: Consider the project's stage (prototype vs production) and domain requirements

## Tools Reference

```bash
# Essential
cargo clippy -- -D warnings
cargo fmt --check
cargo test
cargo doc --no-deps

# Security
cargo audit
cargo deny check

# Quality
cargo +nightly udeps
cargo machete  # unused dependencies

# Performance
cargo bench
cargo flamegraph
cargo bloat  # binary size

# Safety
cargo +nightly miri test
cargo careful test  # extra UB checks

# Coverage
cargo tarpaulin --out Html
cargo llvm-cov
```

## Example Findings

### Critical: Unsound Unsafe Code
```rust
// src/buffer.rs:45
unsafe fn get_unchecked(&self, idx: usize) -> &T {
    &*self.ptr.add(idx)  // No bounds check!
}
```
**Problem**: Public function with unsafe that can cause UB if `idx >= len`
**Fix**: Make private, add bounds check, or require `idx < self.len()` precondition with debug_assert

### High: Missing Error Context
```rust
// src/config.rs:23
let config = fs::read_to_string(path)?;  // Which file failed?
```
**Fix**: Use `anyhow` or `thiserror` with `.with_context(|| format!("Failed to read config: {}", path))`

### Medium: Unnecessary Clone
```rust
// src/handler.rs:89
let name = request.name.clone();  // request is owned, could move
process(name);
```
**Fix**: `process(request.name)` to avoid allocation

### Low: Non-idiomatic Option Handling
```rust
// src/utils.rs:12
if option.is_some() {
    let value = option.unwrap();
    // ...
}
```
**Fix**: `if let Some(value) = option { ... }`
