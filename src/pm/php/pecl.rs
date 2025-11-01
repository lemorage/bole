use crate::{
    find::{Bump, Find},
    pm::{
        core::{
            types::{Categorizable, Category, PmInfo},
            updater::update_cmd,
            upstream::Upstream,
        },
        find_all_pms_with_args,
    },
};

/// pecl - PHP extension manager
pub struct Pecl;

impl Pecl {
    const NAME: &'static str = "pecl";
}

impl Find for Pecl {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "/opt/homebrew/opt/php/bin/pecl",
            "/usr/local/opt/php/bin/pecl",
            "/home/linuxbrew/.linuxbrew/opt/php/bin/pecl",
        ]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms_with_args(Self::NAME, &["-V"])
            .into_iter()
            .map(|mut pm_info| {
                // Clean pecl's verbose output
                if let Some(version_line) = pm_info.version.lines().next()
                    && let Some(version) = version_line.split(':').nth(1)
                {
                    // "PEAR Version: 1.10.16" -> "1.10.16"
                    pm_info.version = version.trim().to_string();
                }
                pm_info
            })
            .collect()
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        // PECL is part of PEAR, so we check PEAR version
        let upstream = Upstream::Pear("pear");
        let http = ureq::agent();
        let latest = upstream.latest(&http).ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Pecl {
    fn category(&self) -> Category {
        Category::PHP
    }
}
