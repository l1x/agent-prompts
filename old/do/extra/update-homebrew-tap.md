<!--file:update-homebrew-tap.md-->

# Update Homebrew Formula Version

## Task

Update an existing Homebrew formula to a new version.

## Input

- **Formula path**: [e.g., Formula/nacre.rb]
- **New version**: [e.g., 0.9.3]

## Instructions

1. **Read the existing formula** to extract:
   - Current version
   - URL pattern(s) - note if multi-arch (on_arm/on_intel) or single-arch
   - Binary name

2. **Construct new URL(s)** by replacing the version in the existing URL pattern:

   ```
   # Single-arch example:
   Old: https://github.com/user/repo/releases/download/v0.9.2/name-aarch64-apple-darwin.tar.gz
   New: https://github.com/user/repo/releases/download/v0.9.3/name-aarch64-apple-darwin.tar.gz

   # Multi-arch: do the same for both ARM and Intel URLs
   ```

3. **Compute SHA256** for each new tarball:

   ```bash
   # ARM
   curl -sL <NEW_ARM_URL> | shasum -a 256

   # Intel (if applicable)
   curl -sL <NEW_INTEL_URL> | shasum -a 256
   ```

4. **Update the formula**:
   - Change `version` to new version
   - Update `url` (or both URLs in on_arm/on_intel blocks)
   - Update `sha256` (or both hashes)

5. **Commit**: `[formula-name] [NEW_VERSION]`

6. **Push** (optional): `git push origin main`

---

## Example

**Input:**

- Formula path: `Formula/nacre.rb`
- New version: `0.9.3`

**Read existing formula** → finds:

- Current version: `0.9.2`
- URL: `https://github.com/l1x/nacre/releases/download/v0.9.2/nacre-aarch64-apple-darwin.tar.gz`
- Single-arch (ARM-only with `depends_on arch: :arm64`)

**Construct new URL:**

```
https://github.com/l1x/nacre/releases/download/v0.9.3/nacre-aarch64-apple-darwin.tar.gz
```

**Compute SHA256:**

```bash
curl -sL https://github.com/l1x/nacre/releases/download/v0.9.3/nacre-aarch64-apple-darwin.tar.gz | shasum -a 256
```

**Update formula** with new version, URL, and SHA256.

**Commit:** `nacre 0.9.3`
