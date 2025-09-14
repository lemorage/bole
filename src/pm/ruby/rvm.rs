use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms_with_args},
};

/// rvm - Ruby version manager
pub struct Rvm;

impl Rvm {
    const NAME: &'static str = "rvm";
}

impl Find for Rvm {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms_with_args(Self::NAME, &["--version"])
            .into_iter()
            .map(|mut pm_info| {
                // Clean rvm's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "rvm 1.29.12 (latest) by Wayne E. Seguin..." -> "1.29.12"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Categorizable for Rvm {
    fn category(&self) -> Category {
        Category::Tools
    }
}
