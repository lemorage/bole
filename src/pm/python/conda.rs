use std::process::Command;

use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
    },
};

/// conda - Package and environment manager
pub struct Conda;

impl Conda {
    const NAME: &'static str = "conda";
}

impl Find for Conda {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            // Home installs
            "~/miniconda3/bin/conda",
            "~/anaconda3/bin/conda",
            "~/miniconda/bin/conda",
            "~/anaconda/bin/conda",
            "~/miniforge3/bin/conda",
            "~/mambaforge/bin/conda",
            // /opt installs
            "/opt/miniconda3/bin/conda",
            "/opt/anaconda3/bin/conda",
            "/opt/miniforge3/bin/conda",
            "/opt/mambaforge/bin/conda",
            // /usr/local installs
            "/usr/local/miniconda3/bin/conda",
            "/usr/local/anaconda3/bin/conda",
            "/usr/local/miniforge3/bin/conda",
            "/usr/local/mambaforge/bin/conda",
        ]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean conda's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "conda 25.7.0" -> "25.7.0"
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
                "https://api.github.com/repos/conda/conda/releases/latest",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json = String::from_utf8(output.stdout).ok()?;

        // Looking for: "tag_name": "25.9.1" or "v25.9.1"
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
                        Origin::PackageManager("Homebrew") => "brew upgrade miniconda",
                        Origin::Direct(_) => "conda update -n base conda",
                        _ => "conda update -n base conda",
                    }
                } else {
                    "conda update -n base conda"
                }
            },
            _ => "conda update -n base conda",
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Conda {
    fn category(&self) -> Category {
        Category::Python
    }
}
