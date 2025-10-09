use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// phpbrew - PHP version manager
pub struct Phpbrew;

impl Phpbrew {
    const NAME: &'static str = "phpbrew";
}

impl Find for Phpbrew {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.phpbrew/bin/phpbrew"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean phpbrew's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "phpbrew 1.27.0" -> "1.27.0"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Categorizable for Phpbrew {
    fn category(&self) -> Category {
        Category::Tools
    }
}
