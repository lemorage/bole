use crate::{
    find::{Bump, Find},
    pm::{Categorizable, Category, PmInfo, find_all_pms, updater::update_cmd, upstream::Upstream},
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
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Npm {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}
