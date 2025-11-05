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

/// nix - Functional package manager
pub struct Nix;

impl Nix {
    const NAME: &'static str = "nix";
}

impl Find for Nix {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "~/.nix-profile/bin/nix",
            "/nix/var/nix/profiles/default/bin/nix",
        ]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean nix's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "nix (Nix) 2.30.2" -> "2.30.2"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::GitHub {
            owner: "NixOS",
            repo: "nix",
        }
        .latest(&http)
        .ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Nix {
    fn category(&self) -> Category {
        Category::System
    }
}
