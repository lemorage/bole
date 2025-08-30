use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Bun;

impl Find for Bun {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "bun"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(self.name())
    }
}
