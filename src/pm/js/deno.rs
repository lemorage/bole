use std::process::Command;

use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms_with_args,
        types::{InstallMethod, Origin},
    },
};

/// deno - Secure runtime for JavaScript and TypeScript
pub struct Deno;

impl Deno {
    const NAME: &'static str = "deno";
}

impl Find for Deno {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.deno/bin/deno"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms_with_args(Self::NAME, &["-V"])
            .into_iter()
            .map(|mut pm_info| {
                // Clean deno's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "deno 2.4.3" -> "2.4.3"
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
                "https://api.github.com/repos/denoland/deno/releases/latest",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json = String::from_utf8(output.stdout).ok()?;

        // Looking for: "tag_name": "v2.5.4"
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
                        Origin::PackageManager("Homebrew") => "brew upgrade deno",
                        Origin::PackageManager("MacPorts") => "sudo port upgrade deno",
                        Origin::PackageManager("npm") => "npm install -g deno@latest",
                        Origin::Direct(_) => "deno upgrade",
                        _ => "deno upgrade",
                    }
                } else {
                    "deno upgrade"
                }
            },
            _ => "deno upgrade", // System or Unknown
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Deno {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}
