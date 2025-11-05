//! Core infrastructure for package manager operations.
//!
//! This module provides the foundational utilities that all package managers
//! rely on, including:
//!
//! - **Install detection**: Determines how package managers were installed
//! - **Tool listing**: Lists tools managed by each package manager
//! - **Version checking**: Unified interface to query upstream registries for
//!   latest versions
//! - **Update commands**: Generates appropriate update commands based on
//!   installation context
//! - **Path discovery**: Utilities for finding package managers in non-standard
//!   locations
//! - **Types**: Core data structures (`PmInfo`, `Category`) used throughout

pub(super) mod install_method;
pub(super) mod search_paths;
pub mod tool_lister;
pub mod types;
pub(super) mod updater;
pub(super) mod upstream;
