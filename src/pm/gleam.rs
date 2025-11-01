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

/// gleam - Gleam build tool and package manager
pub struct Gleam;

impl Gleam {
    const NAME: &'static str = "gleam";
}

impl Find for Gleam {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.cargo/bin/gleam"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean gleam's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "gleam 1.11.1" -> "1.11.1"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::GitHub {
            owner: "gleam-lang",
            repo: "gleam",
        }
        .latest(&http)
        .ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Gleam {
    fn category(&self) -> Category {
        Category::Gleam
    }
}
