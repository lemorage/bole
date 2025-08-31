use std::{collections::HashMap, path::Path};

use crate::pm::{
    python::{Pip, Pipx},
    rust::Cargo,
    types::{AsOrigin, InstallMethod},
};

/// Trait for package manager queriers that can authoritatively identify
/// which tools they manage
pub trait Querier {
    /// Name of the package manager this querier can query (e.g., "pipx", "rye")
    fn name(&self) -> &'static str;

    /// Check if this package manager is available on the system
    fn is_available(&self) -> bool;

    /// Get all tools managed by this package manager
    fn list(&self) -> Vec<Tool>;

    /// Check if this package manager owns a specific tool
    fn owns(&self, tool_name: &str) -> Option<Tool>;
}

/// Information about a tool managed by a package manager
#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub version: String,
    pub path: Option<String>,
    pub manager: String,
}

/// Resolver that queries package managers to determine
/// the authoritative source for each detected binary
pub struct Resolver {
    queriers: Vec<Box<dyn Querier>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            queriers: vec![Box::new(Pipx), Box::new(Pip), Box::new(Cargo)],
        }
    }

    /// Find which package manager owns a specific tool at a given path
    pub fn resolve(&self, tool_name: &str, _path: &Path) -> Option<String> {
        for querier in &self.queriers {
            if querier.is_available() {
                if let Some(_tool) = querier.owns(tool_name) {
                    return Some(querier.name().to_string());
                }
            }
        }
        None
    }

    /// Get all tools managed by all available package managers
    pub fn scan(&self) -> HashMap<String, Vec<Tool>> {
        let mut all_tools = HashMap::new();

        for querier in &self.queriers {
            if querier.is_available() {
                let tools = querier.list();
                if !tools.is_empty() {
                    all_tools.insert(querier.name().to_string(), tools);
                }
            }
        }

        all_tools
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Main entry point for query-based attribution
pub fn resolve(tool_name: &str, path: &Path) -> Option<InstallMethod> {
    let resolver = Resolver::new();
    if let Some(manager) = resolver.resolve(tool_name, path) {
        // Use the AsOrigin trait to get proper origin representation
        let origin = match manager.as_str() {
            "pipx" => Pipx::as_origin(),
            "pip" => Pipx::as_origin(), // pip installs often via pipx
            "cargo" => Cargo::as_origin(),
            _ => return Some(InstallMethod::Unknown),
        };
        return Some(InstallMethod::Chain(vec![origin]));
    }
    None
}
