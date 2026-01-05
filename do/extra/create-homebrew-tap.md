<!--file:create-homebrew-tap.md-->

# Create Homebrew Tap Formula

## Task

Create a Homebrew formula for the following project using pre-built binaries.

## Project Details

- **Formula name**: [PROJECT_NAME]
- **Source repository**: [GITHUB_URL]
- **Description**: [SHORT_DESCRIPTION]
- **Homepage**: [HOMEPAGE_URL]
- **License**: [LICENSE_TYPE]
- **Version**: [VERSION]

## Binary Configuration

- **Binary name**: [name of the installed binary]
- **ARM tarball URL**: [URL to aarch64-apple-darwin.tar.gz]
- **Intel tarball URL**: [URL to x86_64-apple-darwin.tar.gz, or "N/A" if ARM-only]
- **Test command**: [e.g., "--version", "--help"]

## Instructions

1. **Compute SHA256** for each architecture:

   ```bash
   # ARM (Apple Silicon)
   curl -sL <ARM_TARBALL_URL> | shasum -a 256

   # Intel (if applicable)
   curl -sL <INTEL_TARBALL_URL> | shasum -a 256
   ```

2. **Create the formula** at `Formula/[PROJECT_NAME].rb`:

   ```ruby
   class [ProjectNameCamelCase] < Formula
     desc "[SHORT_DESCRIPTION]"
     homepage "[HOMEPAGE_URL]"
     version "[VERSION]"
     license "[LICENSE_TYPE]"

     on_arm do
       url "[ARM_TARBALL_URL]"
       sha256 "[ARM_SHA256]"
     end

     on_intel do
       url "[INTEL_TARBALL_URL]"
       sha256 "[INTEL_SHA256]"
     end

     def install
       bin.install "[BINARY_NAME]"
     end

     test do
       assert_match "[BINARY_NAME]", shell_output("#{bin}/[BINARY_NAME] [TEST_COMMAND]")
     end
   end
   ```

   **For ARM-only projects**, use this simpler format:

   ```ruby
   class [ProjectNameCamelCase] < Formula
     desc "[SHORT_DESCRIPTION]"
     homepage "[HOMEPAGE_URL]"
     version "[VERSION]"
     license "[LICENSE_TYPE]"

     depends_on arch: :arm64

     url "[ARM_TARBALL_URL]"
     sha256 "[ARM_SHA256]"

     def install
       bin.install "[BINARY_NAME]"
     end

     test do
       assert_match "[BINARY_NAME]", shell_output("#{bin}/[BINARY_NAME] [TEST_COMMAND]")
     end
   end
   ```

3. **Create or update README.md**:

   ```markdown
   # Homebrew Tap

   ## Installation

   ```bash
   brew tap [USERNAME]/[TAPNAME]
   brew install [PROJECT_NAME]
   ```

   ## Available Formulae

   | Formula        | Description         |
   |----------------|---------------------|
   | [PROJECT_NAME] | [SHORT_DESCRIPTION] |

   ## Updating

   ```bash
   brew update
   brew upgrade [PROJECT_NAME]
   ```
   ```

4. **Commit changes** with message: `[PROJECT_NAME] [VERSION]`

## Tap Naming Reference

- GitHub repo: `github.com/[USERNAME]/homebrew-[TAPNAME]`
- Brew command: `brew tap [USERNAME]/[TAPNAME]`
- Install: `brew install [USERNAME]/[TAPNAME]/[PROJECT_NAME]` or just `brew install [PROJECT_NAME]`

---

## Example: nacre

```markdown
# Create Homebrew Tap Formula

## Project Details
- **Formula name**: nacre
- **Source repository**: https://github.com/l1x/nacre
- **Description**: Agentic project management UI
- **Homepage**: https://github.com/l1x/nacre
- **License**: MIT
- **Version**: 0.9.2

## Binary Configuration
- **Binary name**: nacre
- **ARM tarball URL**: https://github.com/l1x/nacre/releases/download/v0.9.2/nacre-aarch64-apple-darwin.tar.gz
- **Intel tarball URL**: N/A (ARM-only)
- **Test command**: --version
```

**Resulting formula:**

```ruby
class Nacre < Formula
  desc "Agentic project management UI"
  homepage "https://github.com/l1x/nacre"
  version "0.9.2"
  license "MIT"

  depends_on arch: :arm64

  url "https://github.com/l1x/nacre/releases/download/v0.9.2/nacre-aarch64-apple-darwin.tar.gz"
  sha256 "722a06192dc56f0ad73a6e9db3f4fde93fa2c22038e43663eac3487186014007"

  def install
    bin.install "nacre"
  end

  test do
    assert_match "nacre", shell_output("#{bin}/nacre --version")
  end
end
```
