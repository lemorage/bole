use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// bundler - Ruby dependency manager
pub struct Bundler;

impl Bundler {
    const NAME: &'static str = "bundler";
}

impl Find for Bundler {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "~/.rbenv/shims/bundler",
            "~/.rvm/wrappers/default/bundler",
            "~/.asdf/shims/bundler",
        ]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean bundler's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "Bundler version 2.2.3" -> "2.2.3"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Categorizable for Bundler {
    fn category(&self) -> Category {
        Category::Ruby
    }
}
