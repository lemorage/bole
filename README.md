# bole

<div align="center">
  <a href="#bole">
    <img src="assets/bole-logo.png" alt="Logo" width="180" height="180">
  </a>

<div>&nbsp;</div>

> <em>Package managers<br>
> scattered like autumn leaves fall—<br>
> the bole holds them firm.</em>
</div>

## What

A unified CLI for discovering and managing all package managers on your system. Find what's installed, check their health, and detect outdated versions.

## Install

```bash
cargo install bole
```

## Commands

### `bole check` - Health monitoring

```bash
# Check all package managers
bole check

# Verbose output with details
bole check -v

# Very verbose (shows all PMs by category)
bole check -vv

# Show only broken PMs
bole check --broken

# Show only outdated PMs
bole check --outdated
```

Example output:
```
⠏ Checking package managers...
Summary: 33 total, 30 healthy, 1 broken, 2 outdated
```

### `bole show` - Discovery and listing

```bash
# Show all package managers (default table format)
bole show

# Filter by category
bole show javascript   # npm, yarn, pnpm, bun, deno, ni
bole show python       # pip, poetry, uv, conda, pdm, pipx, pipenv
bole show system       # brew, nix, macports
bole show rust         # cargo
bole show tools        # asdf, volta, mise, corepack, pyenv

# Show all instances (including duplicates)
bole show --all

# Output formats
bole show --tree       # Tree view
bole show --json       # JSON output
bole show --csv        # CSV output
```

## Supported

- **System:** Homebrew, MacPorts, Nix
- **JavaScript:** npm, yarn, pnpm, bun, deno, ni
- **Python:** pip, poetry, uv, conda, pdm, pipx, pipenv
- **PHP:** composer, pecl
- **Ruby:** gem, bundler, rvm, rbenv
- **PHP:** composer, pecl
- **Other:** Rust (cargo), Zig, Go, Haskell (cabal/stack), Gleam
- **Tools:** asdf, volta, mise, corepack, pyenv, phpbrew

## License

MIT
