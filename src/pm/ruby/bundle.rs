use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// bundle - Ruby dependency manager (bundler CLI)
pub struct Bundle;

impl Bundle {
    const NAME: &'static str = "bundle";
}

impl Find for Bundle {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "~/.rbenv/shims/bundle",
            "~/.rvm/wrappers/default/bundle",
            "~/.asdf/shims/bundle",
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

impl Categorizable for Bundle {
    fn category(&self) -> Category {
        Category::Ruby
    }
}
