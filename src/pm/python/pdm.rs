use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
        upstream::Upstream,
    },
};

/// pdm - Modern Python dependency manager
pub struct Pdm;

impl Pdm {
    const NAME: &'static str = "pdm";
}

impl Find for Pdm {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean pdm's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "PDM, version 2.25.9" -> "2.25.9"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let upstream = Upstream::GitHub {
            owner: "pdm-project",
            repo: "pdm",
        };
        let http = ureq::agent();
        let latest = upstream.latest(&http).ok()?;

        // Determine update command based on installation method
        let cmd = match &pm_info.install_method {
            InstallMethod::Chain(origins) => {
                // Check the first origin in the chain
                if let Some(first) = origins.first() {
                    match first {
                        Origin::PackageManager("Homebrew") => "brew upgrade pdm",
                        Origin::PackageManager("pip") => "pip install --upgrade pdm",
                        Origin::PackageManager("pipx") => "pipx upgrade pdm",
                        _ => "pdm self update",
                    }
                } else {
                    "pdm self update"
                }
            },
            _ => "pdm self update",
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Pdm {
    fn category(&self) -> Category {
        Category::Python
    }
}
