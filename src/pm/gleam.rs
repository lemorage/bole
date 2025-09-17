use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
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
        &[
            "~/.local/bin/gleam",
            "~/.cargo/bin/gleam",
            "/opt/homebrew/bin/gleam",
            "/usr/local/bin/gleam",
        ]
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
}

impl Categorizable for Gleam {
    fn category(&self) -> Category {
        Category::Gleam
    }
}
