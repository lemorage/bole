use std::process::Command;

use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
    },
};

/// npm - Node.js package manager
pub struct Npm;

impl Npm {
    const NAME: &'static str = "npm";
}

impl Find for Npm {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.volta/bin/npm", "~/.asdf/shims/npm"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let output = Command::new("curl")
            .args(["-s", "https://registry.npmjs.org/-/package/npm/dist-tags"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json = String::from_utf8(output.stdout).ok()?;
        let latest = json
            .split("\"latest\":\"")
            .nth(1)?
            .split('"')
            .next()?
            .to_string();

        // Determine update command based on installation method
        let cmd = match &pm_info.install_method {
            InstallMethod::Chain(origins) => {
                // Check the first origin in the chain
                if let Some(first) = origins.first() {
                    match first {
                        Origin::PackageManager("Homebrew") => "brew upgrade node",
                        Origin::PackageManager("MacPorts") => "sudo port upgrade nodejs",
                        Origin::Wrapper("Volta") => "volta install node@latest",
                        Origin::Wrapper("Asdf") => "asdf install nodejs latest",
                        _ => "npm install -g npm@latest",
                    }
                } else {
                    "npm install -g npm@latest"
                }
            },
            _ => "npm install -g npm@latest", // System or Unknown
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Npm {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}
