use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

pub struct Poetry;

impl Poetry {
    const NAME: &'static str = "poetry";
}

impl Find for Poetry {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean poetry's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "Poetry (version 2.1.2)" -> "2.1.2"
                    let v = version.trim_end_matches(|c: char| c == ')' || c.is_whitespace());
                    pm_info.version = v.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Categorizable for Poetry {
    fn category(&self) -> Category {
        Category::Python
    }
}
