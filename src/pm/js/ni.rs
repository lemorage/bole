use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
        upstream::Upstream,
    },
};

/// ni - Package manager agnostic scripts runner
pub struct Ni;

impl Ni {
    const NAME: &'static str = "ni";
}

impl Find for Ni {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::Npm("@antfu/ni").latest(&http).ok()?;

        // Determine update command based on installation method
        let cmd = match &pm_info.install_method {
            InstallMethod::Chain(origins) => {
                // Check the first origin in the chain
                if let Some(first) = origins.first() {
                    match first {
                        Origin::PackageManager("Homebrew") => "brew upgrade @antfu/ni",
                        Origin::PackageManager("MacPorts") => "sudo port upgrade ni",
                        Origin::PackageManager("npm") => "npm install -g @antfu/ni@latest",
                        Origin::PackageManager("pnpm") => "pnpm add -g @antfu/ni@latest",
                        Origin::PackageManager("yarn") => "yarn global add @antfu/ni@latest",
                        Origin::Direct(_) => "npm install -g @antfu/ni@latest",
                        _ => "npm install -g @antfu/ni@latest",
                    }
                } else {
                    "npm install -g @antfu/ni@latest"
                }
            },
            _ => "npm install -g @antfu/ni@latest", // System or Unknown
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Ni {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}
