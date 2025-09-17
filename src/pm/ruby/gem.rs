use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// gem - Ruby package manager
pub struct Gem;

impl Gem {
    const NAME: &'static str = "gem";
}

impl Find for Gem {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "/usr/bin/gem",
            "/opt/homebrew/bin/gem",
            "/usr/local/bin/gem",
            "~/.rbenv/shims/gem",
            "~/.rvm/rubies/default/bin/gem",
        ]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }
}

impl Categorizable for Gem {
    fn category(&self) -> Category {
        Category::Ruby
    }
}
