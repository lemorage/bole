use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// uv - Extremely fast Python package manager
pub struct Uv;

impl Uv {
    const NAME: &'static str = "uv";
}

impl Find for Uv {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean uv's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "uv 0.8.4 (...)" -> "0.8.4"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Categorizable for Uv {
    fn category(&self) -> Category {
        Category::Python
    }
}
