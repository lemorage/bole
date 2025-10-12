use std::process::Command;

use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
    },
};

/// bun - All-in-one JavaScript runtime and toolkit
pub struct Bun;

impl Bun {
    const NAME: &'static str = "bun";
}

impl Find for Bun {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.bun/bin/bun"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let output = Command::new("curl")
            .args([
                "-s",
                "https://api.github.com/repos/oven-sh/bun/releases/latest",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json = String::from_utf8(output.stdout).ok()?;

        // Looking for: "tag_name": "bun-v1.2.23"
        let latest = json
            .lines()
            .find(|line| line.contains("\"tag_name\""))
            .and_then(|line| {
                line.split(':').nth(1).map(|s| {
                    let trimmed = s.trim().trim_end_matches(',').trim_matches('"');
                    // Remove "bun-v" prefix if present
                    if let Some(stripped) = trimmed.strip_prefix("bun-v") {
                        stripped.to_string()
                    } else if let Some(stripped) = trimmed.strip_prefix('v') {
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
                        Origin::PackageManager("Homebrew") => "brew upgrade bun",
                        Origin::PackageManager("MacPorts") => "sudo port upgrade bun",
                        Origin::PackageManager("npm") => "npm install -g bun@latest",
                        Origin::Direct(_) => "bun upgrade",
                        _ => "bun upgrade",
                    }
                } else {
                    "bun upgrade"
                }
            },
            _ => "bun upgrade", // System or Unknown
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Bun {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}
