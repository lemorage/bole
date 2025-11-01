use crate::{
    find::{Bump, Find},
    pm::{
        core::{
            types::{Categorizable, Category, PmInfo},
            updater::update_cmd,
            upstream::Upstream,
        },
        find_all_pms,
    },
};

/// pipenv - Python virtual environment and dependency manager
pub struct Pipenv;

impl Pipenv {
    const NAME: &'static str = "pipenv";
}

impl Find for Pipenv {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean pipenv's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "pipenv, version 2023.10.24" -> "2023.10.24"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let upstream = Upstream::GitHub {
            owner: "pypa",
            repo: "pipenv",
        };
        let http = ureq::agent();
        let latest = upstream.latest(&http).ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Pipenv {
    fn category(&self) -> Category {
        Category::Python
    }
}
