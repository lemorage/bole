# Contributing to Bole

Thank you for contributing! This guide covers the development setup and workflow.

## Quick Start

### Prerequisites
- **Nix with flakes enabled** (recommended) OR manual tool installation
- **Git** for version control

### Option 1: Nix + direnv (Recommended)
```bash
# Clone the repository
git clone https://github.com/lemorage/bole.git
cd bole

# Enable automatic environment loading
direnv allow
```

The development environment loads automatically when you enter the directory. Git commit hooks are installed automatically.

### Option 2: Manual Nix
```bash
# Enter development environment manually
nix develop
```

### Option 3: Manual Installation
```bash
# Install Rust nightly toolchain (required for formatting and Edition 2024)
rustup toolchain install nightly
rustup component add rustfmt clippy rust-src rust-analyzer --toolchain nightly
rustup override set nightly

# Install additional tools
cargo install typos-cli

# Install git hook manually
mkdir -p .git/hooks
# Copy commit validation script to .git/hooks/commit-msg
```

## Development Workflow

1. Fork and clone the repository
2. Set up development environment: `direnv allow` (or `nix develop`)
3. Create a feature branch: `git checkout -b feat/your-feature`
4. Make changes
5. Commit with proper format: `git commit -m "feat: your change"`
6. Push and create pull request

### Development Commands

**Contributors:**
```bash
# All checks (same as CI)
nix run .#checks.aarch64-darwin.lint   # macOS (Apple Silicon)
nix run .#checks.x86_64-darwin.lint    # macOS (Intel)
nix run .#checks.x86_64-linux.lint     # Linux

# Individual fixes
cargo fmt
cargo clippy --fix
typos --write-changes
cargo test

# Or use xtask to check (format, lint, test, build)
cargo xtask check
```
> Note: Nix does not support native Windows yet. Use WSL2 and run `nix` inside your WSL shell.

**Maintainers:**
```bash
cargo xtask release patch    # Patch version bump
cargo xtask release minor    # Minor version bump
cargo xtask release major    # Major version bump
```

## Commit Message Format

Use Conventional Commits format:
```
<type>[optional scope]: <description>
```

- **Types**: `feat` (new feature), `fix` (bug fix), `docs`, `chore`, `test`, `refactor`, etc.
- **Scope**: Optional, in parentheses (e.g., `feat(parser)`)
- **Description**: Brief summary of changes, max 72 characters

**Examples**:
- `feat: add user login`
- `fix(api): handle null response`
- `docs: fix typo in README`

**Valid scopes**: `cli`, `core`, `pm`, `config`, `deps`, `nix`

Details: https://www.conventionalcommits.org/en/v1.0.0/

## Troubleshooting

### Commit message rejected
Ensure your commit message follows the conventional format:
```bash
git commit -m "feat: describe your change"
# Not: git commit -m "added stuff"
```

### Code quality issues
```bash
# Fix all issues
cargo fmt
typos --write-changes
cargo clippy --fix
```

### Nix environment issues
```bash
# Reload environment
direnv reload

# Update flake dependencies
nix flake update

# Rebuild environment
nix develop
```

---

Thank you for contributing to Bole!
