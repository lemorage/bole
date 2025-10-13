use std::process::Command;

use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
    },
};

/// pdm - Modern Python dependency manager
pub struct Pdm;

impl Pdm {
    const NAME: &'static str = "pdm";
}

impl Find for Pdm {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean pdm's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "PDM, version 2.25.9" -> "2.25.9"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        // Use GitHub API to check latest pdm version
        let output = Command::new("curl")
            .args([
                "-s",
                "https://api.github.com/repos/pdm-project/pdm/releases/latest",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json = String::from_utf8(output.stdout).ok()?;

        // Looking for: "tag_name": "2.26.0"
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
                        Origin::PackageManager("Homebrew") => "brew upgrade pdm",
                        Origin::PackageManager("pip") => "pip install --upgrade pdm",
                        Origin::PackageManager("pipx") => "pipx upgrade pdm",
                        _ => "pdm self update",
                    }
                } else {
                    "pdm self update"
                }
            },
            _ => "pdm self update",
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Pdm {
    fn category(&self) -> Category {
        Category::Python
    }
}
