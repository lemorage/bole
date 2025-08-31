use serde::Serialize;
use tabled::Tabled;

/// How a package manager was installed on the system
#[derive(Debug, Serialize, Tabled, Clone)]
pub enum InstallMethod {
    OfficialInstaller(&'static str),
    SystemPackageManager(&'static str),
    LanguageToolchain(&'static str),
    SystemProvided,
    Unknown,
}

impl std::fmt::Display for InstallMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallMethod::OfficialInstaller(via) => write!(f, "Official Installer ({})", via),
            InstallMethod::SystemPackageManager(pm) => write!(f, "{} Package", pm),
            InstallMethod::LanguageToolchain(tool) => write!(f, "{}", tool),
            InstallMethod::SystemProvided => write!(f, "System Provided"),
            InstallMethod::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Information about a discovered package manager instance
#[derive(Debug, Serialize, Tabled)]
pub struct PmInfo {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Version")]
    pub version: String,
    #[tabled(rename = "Installed Via")]
    pub install_method: InstallMethod,
    #[tabled(rename = "Path")]
    pub path: String,
}

/// Grouped package manager information for clean display
#[derive(Debug, Serialize, Tabled)]
pub struct GroupedPmInfo {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Version")]
    pub version: String,
    #[tabled(rename = "Primary Path")]
    pub primary_path: String,
    #[tabled(rename = "Source")]
    pub install_method: InstallMethod,
    #[tabled(rename = "Alternatives")]
    pub alternatives: String,
}

impl GroupedPmInfo {
    pub fn from_instances(name: String, instances: Vec<PmInfo>) -> Self {
        if instances.is_empty() {
            panic!("Cannot create GroupedPmInfo from empty instances");
        }

        // Find the "primary" instance - prioritize PATH order (first in vec)
        let primary = &instances[0];
        let alternatives_count = instances.len().saturating_sub(1);

        let alternatives = if alternatives_count == 0 {
            "-".to_string()
        } else if alternatives_count == 1 {
            "1 other location".to_string()
        } else {
            format!("{} other locations", alternatives_count)
        };

        Self {
            name,
            version: primary.version.clone(),
            primary_path: primary.path.clone(),
            install_method: primary.install_method.clone(),
            alternatives,
        }
    }
}
