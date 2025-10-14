use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
        upstream::Upstream,
    },
};

/// yarn - JavaScript package manager
pub struct Yarn;

impl Yarn {
    const NAME: &'static str = "yarn";
}

impl Find for Yarn {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "~/.yarn/bin/yarn",
            "~/.volta/bin/yarn",
            "~/.asdf/shims/yarn",
        ]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::Npm("yarn").latest(&http).ok()?;

        // Determine update command based on installation method
        let cmd = match &pm_info.install_method {
            InstallMethod::Chain(origins) => {
                // Check the first origin in the chain
                if let Some(first) = origins.first() {
                    match first {
                        Origin::PackageManager("Homebrew") => "brew upgrade yarn",
                        Origin::PackageManager("MacPorts") => "sudo port upgrade yarn",
                        Origin::PackageManager("npm") => "npm install -g yarn@latest",
                        Origin::Wrapper("Volta") => "volta install yarn@latest",
                        Origin::Wrapper("Asdf") => "asdf install yarn latest",
                        Origin::Direct(_) => "npm install -g yarn@latest",
                        _ => "npm install -g yarn@latest",
                    }
                } else {
                    "npm install -g yarn@latest"
                }
            },
            _ => "npm install -g yarn@latest", // System or Unknown
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Yarn {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}
