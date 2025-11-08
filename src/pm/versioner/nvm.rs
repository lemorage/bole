use std::{path::Path, process::Command};

use crate::{
    find::{Bump, Find},
    pm::{
        core::{
            types::{Categorizable, Category, PmInfo},
            updater::update_cmd,
            upstream::Upstream,
        },
        determine_install_method,
    },
};

/// nvm - Node Version Manager (shell-based)
pub struct Nvm;

impl Nvm {
    const NAME: &'static str = "nvm";

    fn detect_shell_scripts(&self) -> Vec<PmInfo> {
        let mut instances = Vec::new();
        let mut seen_paths = std::collections::HashSet::new();

        let mut nvm_candidates = Vec::new();

        // Check NVM_DIR environment variable first
        if let Ok(nvm_dir) = std::env::var("NVM_DIR") {
            nvm_candidates.push((format!("{}/nvm.sh", nvm_dir), nvm_dir));
        }

        // Check default home directory location
        if let Ok(home) = std::env::var("HOME") {
            let default_nvm_dir = format!("{}/.nvm", home);
            nvm_candidates.push((format!("{}/nvm.sh", default_nvm_dir), default_nvm_dir));
        }

        // Check Homebrew installations
        if let Ok(output) = Command::new("brew").args(["--prefix", "nvm"]).output()
            && output.status.success()
        {
            let brew_nvm_prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let brew_nvm_dir = brew_nvm_prefix.clone();
            nvm_candidates.push((format!("{}/nvm.sh", brew_nvm_prefix), brew_nvm_dir));
        }

        // Also check common Homebrew locations as fallback
        for brew_prefix in &["/opt/homebrew", "/usr/local"] {
            let brew_nvm_dir = format!("{}/opt/nvm", brew_prefix);
            nvm_candidates.push((format!("{}/nvm.sh", brew_nvm_dir), brew_nvm_dir));
        }

        for (nvm_script, nvm_dir) in nvm_candidates {
            // Skip if already seen this path
            let Ok(canonical) = std::fs::canonicalize(&nvm_script) else {
                continue;
            };

            if !seen_paths.insert(canonical) {
                continue;
            }

            // Get NVM version by sourcing the script
            let output = Command::new("bash")
                .env("NVM_DIR", &nvm_dir)
                .args([
                    "-c",
                    &format!(". \"{}\" >/dev/null 2>&1 && nvm --version", nvm_script),
                ])
                .output()
                .ok();

            if let Some(output) = output
                && output.status.success()
            {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let install_method = determine_install_method(Path::new(&nvm_script));

                instances.push(PmInfo {
                    name: Self::NAME.to_string(),
                    version,
                    path: nvm_script,
                    install_method,
                    latest_version: None,
                });
            }
        }

        instances
    }
}

impl Find for Nvm {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        // Try standard detection for any nvm wrapper executables
        let mut instances = crate::pm::find_all_pms(Self::NAME);

        // If no binaries found, detect shell script installations
        if instances.is_empty() {
            instances = self.detect_shell_scripts();
        }

        instances
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::GitHub {
            owner: "nvm-sh",
            repo: "nvm",
        }
        .latest(&http)
        .ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Nvm {
    fn category(&self) -> Category {
        Category::Versioner
    }
}
