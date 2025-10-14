use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
        upstream::Upstream,
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
        let http = ureq::agent();
        let latest = Upstream::Npm("npm").latest(&http).ok()?;

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
