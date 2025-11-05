use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// mise - Modern asdf alternative
///
/// mise (formerly rtx) is a fast, polyglot tool version manager.
/// It's compatible with asdf but faster and more feature-rich.
pub struct Mise;

impl Mise {
    const NAME: &'static str = "mise";
}

impl Find for Mise {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.cargo/bin/mise"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean mise's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(0) {
                    // "2025.9.6 ... (...)" -> "2025.9.6"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Categorizable for Mise {
    fn category(&self) -> Category {
        Category::Tools
    }
}
