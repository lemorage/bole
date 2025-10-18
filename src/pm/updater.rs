//! Update command generation based on installation context.
//!
//! Provides a unified way to determine the appropriate update command
//! for a package manager based on how it was installed.

use super::types::{InstallMethod, Origin};

/// Generate update command based on package manager and installation method.
pub(crate) fn update_cmd(pm_name: &str, method: &InstallMethod) -> &'static str {
    // Extract the first origin if it's a chain
    let origin = match method {
        InstallMethod::Chain(origins) => origins.first(),
        _ => None,
    };

    // Match on package manager and origin
    match (pm_name, origin) {
        // JavaScript ecosystem
        ("npm", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade node",
        ("npm", Some(Origin::PackageManager("MacPorts"))) => "sudo port upgrade nodejs",
        ("npm", Some(Origin::Wrapper("Volta"))) => "volta install node@latest",
        ("npm", Some(Origin::Wrapper("Asdf"))) => "asdf install nodejs latest",
        ("npm", _) => "npm install -g npm@latest",

        ("yarn", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade yarn",
        ("yarn", Some(Origin::PackageManager("MacPorts"))) => "sudo port upgrade yarn",
        ("yarn", Some(Origin::Wrapper("Volta"))) => "volta install yarn@latest",
        ("yarn", _) => "npm install -g yarn@latest",

        ("pnpm", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade pnpm",
        ("pnpm", Some(Origin::Wrapper("Volta"))) => "volta install pnpm@latest",
        ("pnpm", _) => "npm install -g pnpm@latest",

        ("bun", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade bun",
        ("bun", _) => "bun upgrade",

        ("deno", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade deno",
        ("deno", Some(Origin::PackageManager("MacPorts"))) => "sudo port upgrade deno",
        ("deno", Some(Origin::Wrapper("Asdf"))) => "asdf install deno latest",
        ("deno", _) => "deno upgrade",

        ("ni", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade ni",
        ("ni", _) => "npm install -g @antfu/ni@latest",

        // Python ecosystem
        ("pip", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade python",
        ("pip", Some(Origin::PackageManager("MacPorts"))) => "sudo port upgrade python",
        ("pip", Some(Origin::Wrapper("Pyenv"))) => "pyenv install --skip-existing",
        ("pip", Some(Origin::Wrapper("Asdf"))) => "asdf install python latest",
        ("pip", _) => "python -m pip install --upgrade pip",

        ("poetry", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade poetry",
        ("poetry", Some(Origin::PackageManager("pip"))) => "pip install --upgrade poetry",
        ("poetry", Some(Origin::PackageManager("pipx"))) => "pipx upgrade poetry",
        ("poetry", _) => "poetry self update",

        ("uv", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade uv",
        ("uv", Some(Origin::PackageManager("pip"))) => "pip install --upgrade uv",
        ("uv", Some(Origin::PackageManager("pipx"))) => "pipx upgrade uv",
        ("uv", _) => "uv self update",

        ("conda", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade --cask miniconda",
        ("conda", Some(Origin::PackageManager("MacPorts"))) => "sudo port upgrade py-conda",
        ("conda", _) => "conda update conda",

        ("pipenv", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade pipenv",
        ("pipenv", Some(Origin::PackageManager("pip"))) => "pip install --upgrade pipenv",
        ("pipenv", Some(Origin::PackageManager("pipx"))) => "pipx upgrade pipenv",
        ("pipenv", _) => "pip install --upgrade pipenv",

        ("pdm", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade pdm",
        ("pdm", Some(Origin::PackageManager("pip"))) => "pip install --upgrade pdm",
        ("pdm", Some(Origin::PackageManager("pipx"))) => "pipx upgrade pdm",
        ("pdm", _) => "pdm self update",

        // PHP ecosystem
        ("composer", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade composer",
        ("composer", Some(Origin::PackageManager("MacPorts"))) => "sudo port upgrade composer",
        ("composer", _) => "composer self-update",

        ("pecl", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade php",
        ("pecl", Some(Origin::PackageManager("MacPorts"))) => "sudo port upgrade php +pear",
        ("pecl", _) => "pecl channel-update pecl.php.net",

        // Haskell ecosystem
        ("cabal", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade cabal-install",
        ("cabal", Some(Origin::PackageManager("MacPorts"))) => "sudo port upgrade cabal",
        ("cabal", Some(Origin::Wrapper("Ghcup"))) => "ghcup install cabal latest",
        ("cabal", _) => "cabal install cabal-install",

        ("stack", Some(Origin::PackageManager("Homebrew"))) => "brew upgrade haskell-stack",
        ("stack", Some(Origin::PackageManager("MacPorts"))) => "sudo port upgrade stack",
        ("stack", Some(Origin::Wrapper("Ghcup"))) => "ghcup install stack latest",
        ("stack", _) => "stack upgrade",

        // System package managers
        ("brew", _) => "brew update && brew upgrade",
        ("port", _) => "sudo port selfupdate && sudo port upgrade outdated",
        ("nix", _) => "nix-channel --update && nix-env --upgrade",

        // Default fallback
        _ => "# No update command available",
    }
}
