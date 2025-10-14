use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
        upstream::Upstream,
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
        let http = ureq::agent();
        let latest = Upstream::GitHub {
            owner: "oven-sh",
            repo: "bun",
        }
        .latest(&http)
        .ok()?;

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
