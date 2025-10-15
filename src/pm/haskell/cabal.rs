use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
        upstream::Upstream,
    },
};

/// cabal - Haskell package manager and build tool
pub struct Cabal;

impl Cabal {
    const NAME: &'static str = "cabal";
}

impl Find for Cabal {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.cabal/bin/cabal", "~/.ghcup/bin/cabal"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean cabal's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "cabal-install version 3.12.1.0" -> "3.12.1.0"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let upstream = Upstream::GitHub {
            owner: "haskell",
            repo: "cabal",
        };
        let http = ureq::agent();
        let latest = upstream.latest(&http).ok()?;

        // Determine update command based on installation method
        let cmd = match &pm_info.install_method {
            InstallMethod::Chain(origins) => {
                // Check the first origin in the chain
                if let Some(first) = origins.first() {
                    match first {
                        Origin::PackageManager("Homebrew") => "brew upgrade cabal-install",
                        Origin::PackageManager("MacPorts") => "sudo port upgrade cabal",
                        Origin::Wrapper("Ghcup") => "ghcup install cabal latest",
                        _ => "cabal install cabal-install",
                    }
                } else {
                    "cabal install cabal-install"
                }
            },
            _ => "cabal install cabal-install",
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Cabal {
    fn category(&self) -> Category {
        Category::Haskell
    }
}
