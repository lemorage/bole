use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// Volta - Node.js toolchain manager
///
/// Volta is a hassle-free way to manage JavaScript command-line tools.
/// It ensures everyone has the same tools and versions.
pub struct Volta;

impl Volta {
    const NAME: &'static str = "volta";
}

impl Find for Volta {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.volta/bin/volta"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }
}

impl Categorizable for Volta {
    fn category(&self) -> Category {
        Category::Tools
    }
}
