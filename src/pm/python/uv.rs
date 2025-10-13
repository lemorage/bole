use std::process::Command;

use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
    },
};

/// uv - Extremely fast Python package manager
pub struct Uv;

impl Uv {
    const NAME: &'static str = "uv";
}

impl Find for Uv {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean uv's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "uv 0.8.4 (...)" -> "0.8.4"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let output = Command::new("curl")
            .args([
                "-s",
                "https://api.github.com/repos/astral-sh/uv/releases/latest",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json = String::from_utf8(output.stdout).ok()?;

        // Looking for: "tag_name": "0.5.4" or "v0.5.4"
        let latest = json
            .lines()
            .find(|line| line.contains("\"tag_name\""))
            .and_then(|line| {
                line.split(':').nth(1).map(|s| {
                    let trimmed = s.trim().trim_end_matches(',').trim_matches('"');
                    // Remove "v" prefix if present
                    if let Some(stripped) = trimmed.strip_prefix('v') {
                        stripped.to_string()
                    } else {
                        trimmed.to_string()
                    }
                })
            })?;

        // Determine update command based on installation method
        let cmd = match &pm_info.install_method {
            InstallMethod::Chain(origins) => {
                // Check the first origin in the chain
                if let Some(first) = origins.first() {
                    match first {
                        Origin::PackageManager("Homebrew") => "brew upgrade uv",
                        Origin::PackageManager("pip") => "pip install --upgrade uv",
                        Origin::PackageManager("pipx") => "pipx upgrade uv",
                        Origin::Direct(_) => "uv self update",
                        _ => "uv self update",
                    }
                } else {
                    "uv self update"
                }
            },
            _ => "uv self update",
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Uv {
    fn category(&self) -> Category {
        Category::Python
    }
}
