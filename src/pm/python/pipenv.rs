use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

pub struct Pipenv;

impl Pipenv {
    const NAME: &'static str = "pipenv";
}

impl Find for Pipenv {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean pipenv's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "pipenv, version 2023.10.24" -> "2023.10.24"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Categorizable for Pipenv {
    fn category(&self) -> Category {
        Category::Python
    }
}
