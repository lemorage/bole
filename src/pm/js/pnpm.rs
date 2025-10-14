use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{InstallMethod, Origin},
        upstream::Upstream,
    },
};

/// pnpm - Fast disk space efficient package manager
pub struct Pnpm;

impl Pnpm {
    const NAME: &'static str = "pnpm";
}

impl Find for Pnpm {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "~/.local/share/pnpm/pnpm",
            "~/Library/pnpm/pnpm",
            "~/.volta/bin/pnpm",
            "~/.asdf/shims/pnpm",
        ]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::Npm("pnpm").latest(&http).ok()?;

        // Determine update command based on installation method
        let cmd = match &pm_info.install_method {
            InstallMethod::Chain(origins) => {
                // Check the first origin in the chain
                if let Some(first) = origins.first() {
                    match first {
                        Origin::PackageManager("Homebrew") => "brew upgrade pnpm",
                        Origin::PackageManager("MacPorts") => "sudo port upgrade pnpm",
                        Origin::PackageManager("npm") => "npm install -g pnpm@latest",
                        Origin::Wrapper("Volta") => "volta install pnpm@latest",
                        Origin::Wrapper("Asdf") => "asdf install pnpm latest",
                        Origin::Direct(_) => "pnpm add -g pnpm",
                        _ => "npm install -g pnpm@latest",
                    }
                } else {
                    "npm install -g pnpm@latest"
                }
            },
            _ => "npm install -g pnpm@latest", // System or Unknown
        };

        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Pnpm {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}
