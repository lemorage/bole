use serde::Serialize;
use tabled::Tabled;

/// How a package manager was installed on the system
#[derive(Debug, Serialize, Tabled, Clone)]
pub enum InstallMethod {
    Chain(Vec<Origin>),
    System,
    Unknown,
}

/// Origin in an installation chain
#[derive(Debug, Serialize, Clone)]
pub enum Origin {
    PackageManager(&'static str),
    Toolchain(&'static str),
    Direct(Option<&'static str>),
    Wrapper(&'static str),
}

/*
TODO: Comprehensive support for all package managers and toolchains
When adding new PM support, implement AsOrigin trait with appropriate category:

Package Managers to support:
- Homebrew, MacPorts, Nix (macOS/Linux system)
- Apt, Dnf, Pacman (Linux system)
- Scoop, Winget, Chocolatey (Windows)
- Snap, Flatpak (Universal Linux)
- Pipx, Conda (Python ecosystem)

Toolchains to support:
- Rustup (Rust), Nvm (Node.js), Ghcup (Haskell)
- Asdf, Sdkman (Multi-language)
- Pyenv (Python), Go (Go toolchain)

Implementation pattern:
```rust
impl AsOrigin for NewPM {
    fn as_origin() -> Origin {
        Origin::PackageManager("NewPM")  // or Toolchain/Wrapper
    }
}
```
*/

/// Trait for types that can represent themselves in an installation chain
pub trait AsOrigin {
    fn as_origin() -> Origin;
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Origin::PackageManager(name) => write!(f, "{}", name),
            Origin::Toolchain(name) => write!(f, "{}", name),
            Origin::Direct(Some(name)) => write!(f, "Official Installer ({})", name),
            Origin::Direct(None) => write!(f, "Official Installer"),
            Origin::Wrapper(name) => write!(f, "{}", name),
        }
    }
}

impl std::fmt::Display for InstallMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallMethod::Chain(layers) => {
                let parts: Vec<String> = layers.iter().map(|l| l.to_string()).collect();
                write!(f, "{}", parts.join(" → "))
            },
            InstallMethod::System => write!(f, "System Provided"),
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

        // Find the "primary" instance - prioritize PATH order
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
