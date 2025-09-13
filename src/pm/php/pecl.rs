use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms_with_args},
};

pub struct Pecl;

impl Pecl {
    const NAME: &'static str = "pecl";
}

impl Find for Pecl {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
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
}

impl Categorizable for Pecl {
    fn category(&self) -> Category {
        Category::PHP
    }
}
