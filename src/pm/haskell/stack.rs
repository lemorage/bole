use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Stack;

impl Find for Stack {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "stack"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(self.name())
            .into_iter()
            .map(|mut pm_info| {
                // Clean stack's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "Version 3.5.1, Git revision SHA..." -> "3.5.1"
                    let v = version.trim_end_matches(|c: char| c == ',' || c.is_whitespace());
                    pm_info.version = v.to_string();
                }
                pm_info
            })
            .collect()
    }
}
