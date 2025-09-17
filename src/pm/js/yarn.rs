use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// yarn - JavaScript package manager
pub struct Yarn;

impl Yarn {
    const NAME: &'static str = "yarn";
}

impl Find for Yarn {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "~/.npm/bin/yarn",
            "~/.yarn/bin/yarn",
            "/opt/homebrew/bin/yarn",
            "/usr/local/bin/yarn",
            "/usr/bin/yarn",
        ]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }
}

impl Categorizable for Yarn {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}
