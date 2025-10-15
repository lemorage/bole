use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms_with_args,
        types::{InstallMethod, Origin},
        upstream::Upstream,
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

        // Determine update command based on installation method
        let cmd = match &pm_info.install_method {
            InstallMethod::Chain(origins) => {
                if let Some(first) = origins.first() {
                    match first {
                        Origin::PackageManager("Homebrew") => "brew upgrade php",
                        Origin::PackageManager("MacPorts") => "sudo port upgrade php +pear",
                        _ => "pecl channel-update pecl.php.net",
                    }
                } else {
                    "pecl channel-update pecl.php.net"
                }
            },
            _ => "pecl channel-update pecl.php.net",
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Pecl {
    fn category(&self) -> Category {
        Category::PHP
    }
}
