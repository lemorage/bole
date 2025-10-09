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
            "~/.rbenv/shims/gem",
            "~/.rvm/wrappers/default/gem",
            "~/.asdf/shims/gem",
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
