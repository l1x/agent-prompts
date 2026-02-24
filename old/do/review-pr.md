<!--file:review-pr.md-->

# Review PR

## Objective

Perform a thorough code review of a pull request, evaluating code quality, correctness, security, and maintainability. Provide actionable feedback and a clear recommendation.

## Input

- **PR Identifier**: GitHub PR URL or number (e.g., `#123` or `https://github.com/owner/repo/pull/123`)
- **Repository**: Current working directory (if PR number provided without URL)

## Process

1. **Fetch PR Details**
   - Get PR metadata: `gh pr view <id> --json title,body,author,baseRefName,headRefName,files,additions,deletions`
   - Get the diff: `gh pr diff <id>`
   - Check CI status: `gh pr checks <id>`

2. **Understand Context**
   - Read the PR description and linked issues
   - Identify the purpose: bug fix, feature, refactor, docs, etc.
   - Check if there's a related Beads ticket or Epic

3. **Review Changes**
   - **Correctness**: Does the code do what it claims? Are edge cases handled?
   - **Security**: Check for OWASP Top 10 vulnerabilities (injection, XSS, auth issues, etc.)
   - **Performance**: Identify potential bottlenecks, N+1 queries, memory leaks
   - **Maintainability**: Is the code readable? Are names descriptive? Is complexity appropriate?
   - **Tests**: Are there adequate tests? Do they cover the changes? Are they meaningful?
   - **Documentation**: Are public APIs documented? Are complex algorithms explained?

4. **Check for Common Issues**
   - Hardcoded secrets or credentials
   - Debug code left in (console.log, print statements, TODO/FIXME)
   - Breaking changes without migration path
   - Missing error handling
   - Race conditions or concurrency issues
   - Inconsistent code style

5. **Formulate Feedback**
   - Categorize findings:
     - **Blocking**: Must fix before merge (bugs, security issues, breaking changes)
     - **Suggestion**: Should consider (performance, maintainability improvements)
     - **Nitpick**: Minor style or preference (optional to address)
   - Be specific: reference file paths and line numbers
   - Be constructive: explain *why* something is problematic and suggest alternatives

6. **Render Decision**
   - **Approve**: No blocking issues, code is ready to merge
   - **Request Changes**: Blocking issues exist, must be addressed
   - **Comment**: Non-blocking feedback, author decides

## Output Format

```markdown
## PR Review: [PR Title]

### Summary
[1-2 sentence overview of the PR and your assessment]

### Verdict: [Approve | Request Changes | Comment]

### Blocking Issues
- [ ] [Issue description with file:line reference]

### Suggestions
- [Suggestion with rationale]

### Nitpicks
- [Minor observation]

### What's Good
- [Positive observations - acknowledge good patterns]
```

## Review Checklist

Use this mental checklist while reviewing:

- [ ] PR description clearly explains the change
- [ ] Code compiles/builds without errors
- [ ] Tests pass (check CI status)
- [ ] No security vulnerabilities introduced
- [ ] Error handling is appropriate
- [ ] No unnecessary complexity added
- [ ] Changes are scoped appropriately (not doing too much)
- [ ] Breaking changes are documented
- [ ] Dependencies are justified and vetted

## Constraints

- **Scope**: Review only what's in the PR. Don't request unrelated refactors.
- **Tone**: Be respectful and constructive. Critique code, not people.
- **Objectivity**: Base feedback on established patterns, not personal preference.
- **Completeness**: Review all changed files, not just a sample.
- **Action**: If using `gh`, submit the review: `gh pr review <id> --approve|--request-changes|--comment -b "feedback"`

## Examples

### Blocking Issue Example
```
**Blocking**: SQL injection vulnerability in `src/db/users.rs:45`
The query uses string interpolation instead of parameterized queries:
`format!("SELECT * FROM users WHERE id = {}", user_id)`
Should use prepared statements to prevent injection attacks.
```

### Suggestion Example
```
**Suggestion**: Consider extracting the validation logic in `src/api/handler.rs:120-145`
into a separate function. This would improve testability and reduce the handler's
complexity from 45 to ~20 lines.
```

### Nitpick Example
```
**Nitpick**: `src/utils/helpers.rs:23` - variable `x` could have a more descriptive
name like `retry_count` to improve readability.
```
