use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// composer - PHP dependency manager
pub struct Composer;

impl Composer {
    const NAME: &'static str = "composer";
}

impl Find for Composer {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean composer's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "Composer version 2.6.5 2023-10-06..." -> "2.6.5"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Categorizable for Composer {
    fn category(&self) -> Category {
        Category::PHP
    }
}
