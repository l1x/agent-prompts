  # Create Homebrew Tap Formula

  ## Task
  Create a Homebrew formula for the following project in this tap repository.

  ## Project Details
  - **Formula name**: [PROJECT_NAME]
  - **Source repository**: [GITHUB_URL]
  - **Description**: [SHORT_DESCRIPTION]
  - **Homepage**: [HOMEPAGE_URL]
  - **License**: [LICENSE_TYPE]
  - **Version**: [VERSION]
  - **Tarball URL**: [RELEASE_TARBALL_URL]

  ## Build Configuration
  - **Language**: [rust|go|python|etc.]
  - **Build dependencies**: [e.g., rust, go, cmake]
  - **Runtime dependencies**: [e.g., openssl, libpq, or "none"]
  - **Install command**: [e.g., "cargo install", "go build", "make install"]
  - **Binary name**: [name of the installed binary]
  - **Test command**: [e.g., "--version", "--help", "version"]

  ## Instructions

  1. **Compute SHA256** of the release tarball:
     ```bash
     curl -sL <TARBALL_URL> | shasum -a 256

  2. Create the formula at Formula/[PROJECT_NAME].rb:
  class [ProjectNameCamelCase] < Formula
    desc "[SHORT_DESCRIPTION]"
    homepage "[HOMEPAGE_URL]"
    url "[TARBALL_URL]"
    sha256 "[COMPUTED_SHA256]"
    license "[LICENSE_TYPE]"

    depends_on "[LANGUAGE]" => :build
    # Add any runtime dependencies here

    def install
      # For Rust:
      system "cargo", "install", *std_cargo_args
      # For Go:
      # system "go", "build", *std_go_args(ldflags: "-s -w")
    end

    test do
      assert_match "[PROJECT_NAME]", shell_output("#{bin}/[BINARY_NAME] [TEST_COMMAND]")
    end
  end
  3. Create or update README.md:
  # Homebrew Tap

  ## Installation

  ```bash
  brew tap [USERNAME]/[TAPNAME]
  brew install [PROJECT_NAME]

  Available Formulae

  | Formula        | Description         |
  |----------------|---------------------|
  | [PROJECT_NAME] | [SHORT_DESCRIPTION] |

  Updating

  brew update
  brew upgrade [PROJECT_NAME]

  4. Commit changes with message: [PROJECT_NAME] [VERSION]

  Tap Naming Reference

  - GitHub repo: github.com/[USERNAME]/homebrew-[TAPNAME]
  - Brew command: brew tap [USERNAME]/[TAPNAME]
  - Install: brew install [USERNAME]/[TAPNAME]/[PROJECT_NAME] or just brew install [PROJECT_NAME]

  ---

  **Example filled in for nacre:**

  ```markdown
  # Create Homebrew Tap Formula

  ## Project Details
  - **Formula name**: nacre
  - **Source repository**: https://github.com/l1x/nacre
  - **Description**: A modern shell written in Rust
  - **Homepage**: https://github.com/l1x/nacre
  - **License**: MIT
  - **Version**: 0.9.1
  - **Tarball URL**: https://github.com/l1x/nacre/archive/refs/tags/v0.9.1.tar.gz

  ## Build Configuration
  - **Language**: rust
  - **Build dependencies**: rust
  - **Runtime dependencies**: none
  - **Install command**: cargo install
  - **Binary name**: nacre
  - **Test command**: --version

