# bole

Discover all package managers on your system.

> <em>Package managers<br>
> scattered like autumn leaves fall—<br>
> the bole holds them firm.</em>

## What

Shows which package managers are installed, their versions, locations, and how they were installed.

```
┌────────┬─────────┬──────────────────────────┬──────────────────┬─────────────────┐
│Name    │Version  │Path                      │Via               │Others           │
├────────┼─────────┼──────────────────────────┼──────────────────┼─────────────────┤
│cargo   │1.91.0   │~/.cargo/bin/cargo        │Rustup            │-                │
├────────┼─────────┼──────────────────────────┼──────────────────┼─────────────────┤
│npm     │11.5.1   │/opt/homebrew/bin/npm     │Homebrew          │1 other location │
├────────┼─────────┼──────────────────────────┼──────────────────┼─────────────────┤
│pnpm    │10.14.0  │/usr/local/bin/pnpm       │Node.js → Corepack│-                │
└────────┴─────────┴──────────────────────────┴──────────────────┴─────────────────┘
```

## Install

```bash
cargo install bole
```

## Use

```bash
# Show primary installations
bole

# Show all installations  
bole --all

# Tree format
bole --tree
```

## Supported

**System:** Homebrew, MacPorts, Nix  
**JavaScript:** npm, yarn, pnpm, bun, deno  
**Python:** pip, poetry, uv, conda, pdm, pipx  
**Other:** Rust (cargo), Go, Haskell (cabal/stack), Gleam

## License

MIT
