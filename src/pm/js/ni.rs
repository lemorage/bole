use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

pub struct Ni;

impl Ni {
    const NAME: &'static str = "ni";
}

impl Find for Ni {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }
}

impl Categorizable for Ni {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}
