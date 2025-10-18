use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms_with_args, updater::update_cmd,
        upstream::Upstream,
    },
};

/// zig - Zig toolchain and built-in package manager
pub struct Zig;

impl Zig {
    const NAME: &'static str = "zig";
}

impl Find for Zig {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms_with_args(Self::NAME, &["version"])
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::GitHub {
            owner: "ziglang",
            repo: "zig",
        }
        .latest(&http)
        .ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Zig {
    fn category(&self) -> Category {
        Category::Zig
    }
}
