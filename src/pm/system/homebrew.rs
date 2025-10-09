use crate::{
    find::Find,
    pm::{
        Categorizable, Category, PmInfo, find_all_pms,
        types::{AsOrigin, Origin},
    },
};

/// Homebrew package manager for macOS and Linux.
pub struct Homebrew;

impl Homebrew {
    const NAME: &'static str = "brew";
}

impl AsOrigin for Homebrew {
    fn as_origin() -> Origin {
        Origin::PackageManager("Homebrew")
    }
}

impl Find for Homebrew {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "/opt/homebrew/bin/brew",
            "/usr/local/bin/brew",
            "~/.linuxbrew/bin/brew",
            "/home/linuxbrew/.linuxbrew/bin/brew",
        ]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean homebrew's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "Homebrew 4.6.3" -> "4.6.3"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Categorizable for Homebrew {
    fn category(&self) -> Category {
        Category::System
    }
}
