use crate::{
    find::{Bump, Find},
    pm::{Categorizable, Category, PmInfo, find_all_pms, updater::update_cmd, upstream::Upstream},
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
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Yarn {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}
