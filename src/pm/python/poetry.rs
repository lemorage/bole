use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
        upstream::Upstream,
    },
};

/// poetry - Python dependency management and packaging
pub struct Poetry;

impl Poetry {
    const NAME: &'static str = "poetry";
}

impl Find for Poetry {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.poetry/bin/poetry"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean poetry's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "Poetry (version 2.1.2)" -> "2.1.2"
                    let v = version.trim_end_matches(|c: char| c == ')' || c.is_whitespace());
                    pm_info.version = v.to_string();
                }
                pm_info
            })
            .collect()
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let upstream = Upstream::PyPI("poetry");
        let http = ureq::agent();
        let latest = upstream.latest(&http).ok()?;

        // Determine update command based on installation method
        let cmd = match &pm_info.install_method {
            InstallMethod::Chain(origins) => {
                // Check the first origin in the chain
                if let Some(first) = origins.first() {
                    match first {
                        Origin::PackageManager("Homebrew") => "brew upgrade poetry",
                        Origin::PackageManager("pip") => "pip install --upgrade poetry",
                        Origin::PackageManager("pipx") => "pipx upgrade poetry",
                        _ => "poetry self update",
                    }
                } else {
                    "poetry self update"
                }
            },
            _ => "poetry self update",
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Poetry {
    fn category(&self) -> Category {
        Category::Python
    }
}
