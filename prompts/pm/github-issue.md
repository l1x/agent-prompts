# gh CLI Rules

## Use `--body-file` for multi-line content

Inline `--body` with heredoc breaks when content contains backticks or quotes. Always use `--body-file`:

```bash
TMPFILE=$(mktemp)
cat > "$TMPFILE" << 'BODY'
Markdown content — backticks, quotes, anything works.
BODY

gh issue create --title "..." --body-file "$TMPFILE" -R OWNER/REPO
```

## `-f` vs `-F` in `gh api`

- `-f key=value` — sends value as **string**
- `-F key=value` — infers type: numbers → integers, booleans → booleans, `@file` → reads file

Using `-f` for a numeric field (e.g., GraphQL `Int!`) causes `"Could not coerce value to Int"`. Use `-F` for numbers.

## GraphQL call hygiene

**Never chain** `gh api graphql` calls with `&&`, `;`, or `echo` in one bash invocation. Run each as a **separate** bash command. Chaining causes invisible encoding errors.

## Use `Closes #N` in PR body

Link PRs to issues using `Closes #N` (or `Fixes #N`, `Resolves #N`) in the PR body. On merge, linked issues auto-close.

---

# View Issue

```bash
gh issue view 42 -R OWNER/REPO
gh issue view 42 --comments -R OWNER/REPO
gh issue view 42 --json number,title,body,state,labels,assignees,comments -R OWNER/REPO
```

---

# Comments

```bash
# Add (multi-line)
TMPFILE=$(mktemp)
cat > "$TMPFILE" << 'BODY'
Comment in **markdown** with `backticks` and other special chars.
BODY
gh api repos/OWNER/REPO/issues/42/comments -F body=@"$TMPFILE"

# Add (one-liner)
gh api repos/OWNER/REPO/issues/42/comments -f body="Simple comment"

# List
gh issue view 42 --comments --json comments \
  --jq '.comments[] | {author: .author.login, body: .body, createdAt: .createdAt}' \
  -R OWNER/REPO
```

---

# Check Blockers

```bash
# What blocks issue #42?
gh api repos/OWNER/REPO/issues/42/dependencies/blocked_by \
  --jq '.[] | {number, title, state}'
```

If empty, the issue is **ready**. Otherwise, resolve blocking issues first.

---

# Close Issue

```bash
gh issue close 42 --reason completed --comment "Fixed in v2.1" -R OWNER/REPO
```
