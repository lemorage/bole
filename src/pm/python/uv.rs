use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
        upstream::Upstream,
    },
};

/// uv - Extremely fast Python package manager
pub struct Uv;

impl Uv {
    const NAME: &'static str = "uv";
}

impl Find for Uv {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean uv's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "uv 0.8.4 (...)" -> "0.8.4"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let upstream = Upstream::GitHub {
            owner: "astral-sh",
            repo: "uv",
        };
        let http = ureq::agent();
        let latest = upstream.latest(&http).ok()?;

        // Determine update command based on installation method
        let cmd = match &pm_info.install_method {
            InstallMethod::Chain(origins) => {
                // Check the first origin in the chain
                if let Some(first) = origins.first() {
                    match first {
                        Origin::PackageManager("Homebrew") => "brew upgrade uv",
                        Origin::PackageManager("pip") => "pip install --upgrade uv",
                        Origin::PackageManager("pipx") => "pipx upgrade uv",
                        Origin::Direct(_) => "uv self update",
                        _ => "uv self update",
                    }
                } else {
                    "uv self update"
                }
            },
            _ => "uv self update",
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Uv {
    fn category(&self) -> Category {
        Category::Python
    }
}
