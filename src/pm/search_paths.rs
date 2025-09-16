use std::{
    collections::HashSet,
    env,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use crate::pm::{PmInfo, try_detect_at_path};

/// Cached environment for path expansion.
struct SearchEnvironment {
    home: String,
    brew_prefix: &'static str,
}

impl SearchEnvironment {
    /// Initialize cached environment values.
    fn initialize() -> Self {
        let home = env::var("HOME").unwrap_or_default();
        let brew_prefix = if Path::new("/opt/homebrew").exists() {
            "/opt/homebrew"
        } else {
            "/usr/local"
        };

        Self { home, brew_prefix }
    }
}

static SEARCH_ENV: OnceLock<SearchEnvironment> = OnceLock::new();

const PM_LOCATIONS: &[(&str, &[&str])] = &[
    (
        "npm",
        &["~/.nvm/current/bin/npm", "{pkg}/bin/npm", "{sys}/bin/npm"],
    ),
    (
        "yarn",
        &[
            "~/.nvm/current/bin/yarn",
            "~/.npm/bin/yarn",
            "~/.yarn/bin/yarn",
            "{pkg}/bin/yarn",
            "{sys}/bin/yarn",
        ],
    ),
    (
        "pnpm",
        &[
            "~/.npm/bin/pnpm",
            "~/.local/share/pnpm/pnpm",
            "{pkg}/bin/pnpm",
            "{sys}/bin/pnpm",
        ],
    ),
    ("bun", &["~/.bun/bin/bun", "{pkg}/bin/bun"]),
    ("deno", &["~/.deno/bin/deno", "{pkg}/bin/deno"]),
    ("ni", &["~/.local/bin/ni", "{pkg}/bin/ni", "{sys}/bin/ni"]),
    (
        "corepack",
        &["{pkg}/bin/corepack", "~/.nvm/current/bin/corepack"],
    ),
    (
        "pip",
        &[
            "{sys}/bin/pip",
            "{sys}/bin/pip3",
            "~/.local/bin/pip",
            "{pkg}/bin/pip",
        ],
    ),
    (
        "pip3",
        &["{sys}/bin/pip3", "~/.local/bin/pip3", "{pkg}/bin/pip3"],
    ),
    (
        "poetry",
        &[
            "~/.local/bin/poetry",
            "~/.poetry/bin/poetry",
            "{pkg}/bin/poetry",
        ],
    ),
    (
        "uv",
        &["~/.local/bin/uv", "~/.cargo/bin/uv", "{pkg}/bin/uv"],
    ),
    (
        "conda",
        &[
            "~/miniconda3/bin/conda",
            "~/anaconda3/bin/conda",
            "/opt/miniconda3/bin/conda",
            "/opt/anaconda3/bin/conda",
        ],
    ),
    (
        "pipenv",
        &[
            "~/.local/bin/pipenv",
            "{pkg}/bin/pipenv",
            "{sys}/bin/pipenv",
        ],
    ),
    ("pipx", &["~/.local/bin/pipx", "{pkg}/bin/pipx"]),
    ("pdm", &["~/.local/bin/pdm", "{pkg}/bin/pdm"]),
    (
        "gem",
        &[
            "{sys}/bin/gem",
            "{pkg}/bin/gem",
            "~/.rbenv/shims/gem",
            "~/.rvm/rubies/default/bin/gem",
        ],
    ),
    (
        "bundle",
        &[
            "{sys}/bin/bundle",
            "{pkg}/bin/bundle",
            "~/.rbenv/shims/bundle",
            "~/.rvm/rubies/default/bin/bundle",
        ],
    ),
    (
        "bundler",
        &[
            "{sys}/bin/bundler",
            "{pkg}/bin/bundler",
            "~/.rbenv/shims/bundler",
            "~/.rvm/rubies/default/bin/bundler",
        ],
    ),
    ("rbenv", &["~/.rbenv/bin/rbenv", "{pkg}/bin/rbenv"]),
    (
        "rvm",
        &[
            "~/.rvm/bin/rvm",
            "~/.rvm/scripts/rvm",
            "/usr/local/rvm/bin/rvm",
        ],
    ),
    ("brew", &["{pkg}/bin/brew"]),
    ("port", &["/opt/local/bin/port"]),
    (
        "nix",
        &[
            "~/.nix-profile/bin/nix",
            "/nix/var/nix/profiles/default/bin/nix",
        ],
    ),
    (
        "cargo",
        &["~/.cargo/bin/cargo", "{pkg}/bin/cargo", "{sys}/bin/cargo"],
    ),
    (
        "go",
        &[
            "/usr/local/go/bin/go",
            "~/.local/bin/go",
            "{pkg}/bin/go",
            "{sys}/bin/go",
        ],
    ),
    (
        "composer",
        &[
            "~/.composer/vendor/bin/composer",
            "{pkg}/bin/composer",
            "{sys}/bin/composer",
        ],
    ),
    ("pecl", &["{pkg}/bin/pecl", "{sys}/bin/pecl"]),
    ("phpbrew", &["~/.phpbrew/bin/phpbrew", "{pkg}/bin/phpbrew"]),
    (
        "cabal",
        &[
            "~/.cabal/bin/cabal",
            "~/.local/bin/cabal",
            "{pkg}/bin/cabal",
        ],
    ),
    ("stack", &["~/.local/bin/stack", "{pkg}/bin/stack"]),
    (
        "gleam",
        &[
            "~/.local/bin/gleam",
            "~/.cargo/bin/gleam",
            "{pkg}/bin/gleam",
        ],
    ),
    ("asdf", &["~/.asdf/bin/asdf", "{pkg}/bin/asdf"]),
    ("volta", &["~/.volta/bin/volta", "{pkg}/bin/volta"]),
    (
        "mise",
        &["~/.local/bin/mise", "~/.cargo/bin/mise", "{pkg}/bin/mise"],
    ),
    ("pyenv", &["~/.pyenv/bin/pyenv", "{pkg}/bin/pyenv"]),
];

/// Expand a location template into `buf`.
///
/// Supports `~`, `{pkg}` (Homebrew prefix), and `{sys}` (`/usr`).
fn expand_template_to_buf(template: &str, buf: &mut String) {
    let env = SEARCH_ENV.get_or_init(SearchEnvironment::initialize);
    buf.clear();

    let expanded = template
        .replace("~", &env.home)
        .replace("{pkg}", env.brew_prefix)
        .replace("{sys}", "/usr");

    buf.push_str(&expanded);
}

/// Return candidate search paths for `name`.
///
/// Uses known templates when available, otherwise falls back to generic ones.
pub(crate) fn get_search_locations(name: &str) -> Vec<PathBuf> {
    let mut path_buf = String::with_capacity(128);
    let mut locations = Vec::new();

    if let Some((_, templates)) = PM_LOCATIONS.iter().find(|(tool, _)| *tool == name) {
        for &template in *templates {
            expand_template_to_buf(template, &mut path_buf);
            locations.push(PathBuf::from(&path_buf));
        }
    } else {
        // Generic fallback patterns.
        let fallback_templates = [
            "~/.local/bin/{name}",
            "~/.{name}/bin/{name}",
            "{pkg}/bin/{name}",
            "/usr/bin/{name}",
        ];

        for template in fallback_templates {
            let expanded = template.replace("{name}", name);
            expand_template_to_buf(&expanded, &mut path_buf);
            locations.push(PathBuf::from(&path_buf));
        }
    }

    locations
}

/// Scan common directories for `name` and detect installations.
///
/// Skips paths already present in `seen_paths` (by canonical path).
pub(crate) fn scan_common_directories(
    name: &str,
    seen_paths: &mut HashSet<PathBuf>,
) -> Vec<PmInfo> {
    let mut path_buf = String::with_capacity(128);
    let mut results = Vec::new();

    // Common directory templates.
    let common_templates = [
        "/usr/bin/{name}",
        "{pkg}/bin/{name}",
        "~/.local/bin/{name}",
        "~/.cargo/bin/{name}",
    ];

    for template in common_templates {
        let expanded = template.replace("{name}", name);
        expand_template_to_buf(&expanded, &mut path_buf);
        let target_path = PathBuf::from(&path_buf);

        if target_path.exists()
            && let Ok(canonical) = std::fs::canonicalize(&target_path)
                && !seen_paths.contains(&canonical)
                    && let Some(pm_info) = try_detect_at_path(&target_path, name, &["--version"]) {
                        seen_paths.insert(canonical);
                        results.push(pm_info);
                    }
    }
    results
}
