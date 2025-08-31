use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Yarn;

impl Yarn {
    const NAME: &'static str = "yarn";
}

impl Find for Yarn {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }
}
