use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// rbenv - Ruby version manager
pub struct Rbenv;

impl Rbenv {
    const NAME: &'static str = "rbenv";
}

impl Find for Rbenv {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean rbenv's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "rbenv 1.2.0" -> "1.2.0"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Categorizable for Rbenv {
    fn category(&self) -> Category {
        Category::Tools
    }
}
