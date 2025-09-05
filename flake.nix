{
  description = "Bole - Manage all package managers on your system";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";

  outputs = { nixpkgs, ... }: let
    forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" "x86_64-windows" ];
  in {
    devShells = forAllSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      default = pkgs.mkShell {
        packages = with pkgs; [ typos ];

        shellHook = ''
          echo "Bole development environment"
          echo "Available: cargo fmt, cargo test, cargo clippy, typos"

          # Install git hooks automatically
          if [ -d .git ] && [ ! -f .git/hooks/commit-msg ]; then
            mkdir -p .git/hooks
            cat > .git/hooks/commit-msg << 'EOF'
#!/usr/bin/env bash
set -euo pipefail
msg=$(head -1 "$1" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
if [[ ! "$msg" =~ ^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\([a-z0-9-]+\))?(!)?:[[:space:]].+ ]]; then
  echo "❌ Invalid commit format. Use: <type>[scope][!]: <description>" >&2
  echo "   Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert" >&2
  exit 1
fi
if [ ''${#msg} -gt 72 ]; then
  echo "❌ Commit message too long (''${#msg}/72 chars)" >&2
  exit 1
fi
echo "✅ Conventional commit format valid"
EOF
            chmod +x .git/hooks/commit-msg
            echo "Git commit-msg hook installed"
          fi
        '';
      };
    });

    # Simple check command for CI
    checks = forAllSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      lint = pkgs.writeShellApplication {
        name = "bole-lint";
        runtimeInputs = with pkgs; [ cargo rustc rustfmt clippy typos ];
        text = ''
          echo "Running format check..."
          cargo fmt -- --check

          echo "Running clippy..."
          cargo clippy -- -D warnings

          echo "Running tests..."
          cargo test

          echo "Running spell check..."
          typos

          echo "All checks passed!"
        '';
      };
    });
  };
}
