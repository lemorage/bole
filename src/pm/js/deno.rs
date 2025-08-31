use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms_with_args},
};

pub struct Deno;

impl Deno {
    const NAME: &'static str = "deno";
}

impl Find for Deno {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms_with_args(Self::NAME, &["-V"])
            .into_iter()
            .map(|mut pm_info| {
                // Clean deno's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "deno 2.4.3" -> "2.4.3"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}
