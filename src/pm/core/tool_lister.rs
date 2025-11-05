use std::collections::HashMap;

use serde::Serialize;

use crate::pm::{python::Pipx, rust::Cargo};

/// Package managers that can list their installed tools.
pub trait ToolLister {
    /// Package manager name (e.g., "pipx", "cargo").
    fn name(&self) -> &'static str;

    /// Check if available on the system.
    fn is_available(&self) -> bool;

    /// List all installed tools.
    fn list(&self) -> Vec<Tool>;

    /// Check if owns a specific tool.
    fn owns(&self, tool_name: &str) -> Option<Tool>;
}

/// Information about a tool managed by a package manager
#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    pub name: String,
    pub version: String,
    pub path: Option<String>,
    pub manager: String,
}

/// Resolver for tool ownership queries.
pub struct Resolver {
    listers: Vec<Box<dyn ToolLister>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            listers: vec![Box::new(Pipx), Box::new(Cargo)],
        }
    }

    /// Find which package manager owns a specific tool.
    pub fn find_owner(&self, tool_name: &str) -> Option<String> {
        for lister in &self.listers {
            if lister.is_available()
                && let Some(_tool) = lister.owns(tool_name)
            {
                return Some(lister.name().to_string());
            }
        }
        None
    }

    /// Get all tools from all available package managers.
    pub fn scan(&self) -> HashMap<String, Vec<Tool>> {
        let mut all_tools = HashMap::new();

        for lister in &self.listers {
            if lister.is_available() {
                let tools = lister.list();
                if !tools.is_empty() {
                    all_tools.insert(lister.name().to_string(), tools);
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
