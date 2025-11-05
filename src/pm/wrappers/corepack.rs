use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// Corepack - Node.js package manager wrapper
///
/// Corepack is a zero-runtime-dependency Node.js script that acts as a bridge
/// between Node.js projects and package managers (npm, pnpm, yarn).
pub struct Corepack;

impl Corepack {
    const NAME: &'static str = "corepack";
}

impl Find for Corepack {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.volta/bin/corepack", "~/.asdf/shims/corepack"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }
}

impl Categorizable for Corepack {
    fn category(&self) -> Category {
        Category::Tools
    }
}
