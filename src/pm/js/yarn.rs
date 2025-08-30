use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Yarn;
impl Find for Yarn {
    type Output = PmInfo;
    fn name(&self) -> &'static str {
        "yarn"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(self.name())
    }
}
