{
  description = "Bole - Manage all package managers on your system";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";

  outputs = { nixpkgs, ... }: let
    forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
  in {
    devShells = forAllSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      default = pkgs.mkShellNoCC {
        packages = with pkgs; [ typos ];

        shellHook = ''
          echo "Bole development environment"

          if [ -d .git ]; then
            mkdir -p .git/hooks
            HOOK_PATH=.git/hooks/commit-msg
            HOOK_TMP=.git/hooks/commit-msg.tmp
            cat > "$HOOK_TMP" << 'EOF'
#!/usr/bin/env bash
set -euo pipefail
msg=$(head -1 "$1" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
if [[ ! "$msg" =~ ^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\([a-z0-9-]+\))?(!)?:[[:space:]].+ ]]; then
  echo "✗ Invalid commit format. Use: <type>[scope][!]: <description>" >&2
  echo "   Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert" >&2
  exit 1
fi
if [ ''${#msg} -gt 72 ]; then
  echo "✗ Commit message too long (''${#msg}/72 chars)" >&2
  exit 1
fi
echo "✓ Conventional commit format valid"
EOF
            if [ ! -f "$HOOK_PATH" ] || ! cmp -s "$HOOK_TMP" "$HOOK_PATH"; then
              mv -f "$HOOK_TMP" "$HOOK_PATH"
              chmod +x "$HOOK_PATH"
              echo "Git commit-msg hook all settled"
            else
              rm -f "$HOOK_TMP"
            fi
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
          set -euo pipefail

          echo "Running format check..."
          cargo fmt --all -- --check

          echo "Running clippy..."
          cargo clippy --all-targets --all-features --frozen -D warnings

          echo "Running tests..."
          cargo test --all-targets --all-features --frozen

          echo "Running spell check..."
          typos

          echo "All checks passed!"
        '';
      };
    });

    formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.alejandra);
  };
}
