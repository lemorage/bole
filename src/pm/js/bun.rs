use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// bun - All-in-one JavaScript runtime and toolkit
pub struct Bun;

impl Bun {
    const NAME: &'static str = "bun";
}

impl Find for Bun {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }
}

impl Categorizable for Bun {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}
