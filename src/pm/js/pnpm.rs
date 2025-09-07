use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

pub struct Pnpm;

impl Pnpm {
    const NAME: &'static str = "pnpm";
}

impl Find for Pnpm {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }
}

impl Categorizable for Pnpm {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}
