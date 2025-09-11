use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

pub struct Gem;

impl Gem {
    const NAME: &'static str = "gem";
}

impl Find for Gem {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
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
