use bole::pm::{self, Category};
use tabled::{Table, settings::Style};

use crate::display::{display_grouped_tree, display_tree, group_pm_instances};

pub(super) fn handle_show_command(category: Option<String>, all: bool, tree: bool) {
    // Filter by category if specified
    let filtered_pms = if let Some(ref category_str) = category {
        let target_category = match parse_category(category_str) {
            Some(cat) => cat,
            None => {
                println!("Unknown category '{}'. Available categories:", category_str);
                println!("  system      - System package managers (brew, nix, macports)");
                println!(
                    "  javascript  - JavaScript package managers (npm, yarn, pnpm, bun, deno)"
                );
                println!(
                    "  python      - Python package managers (pip, poetry, uv, conda, pdm, pipx)"
                );
                println!("  rust        - Rust package managers (cargo)");
                println!("  go          - Go package managers (go)");
                println!("  haskell     - Haskell package managers (cabal, stack)");
                println!("  gleam       - Gleam package managers (gleam)");
                println!("  tools       - Version managers (asdf, volta, mise, corepack)");
                return;
            },
        };

        filter_by_category(target_category)
    // No category filter, return all found package managers
    } else {
        let mut found_pms = Vec::new();

        for detector in pm::all_package_managers() {
            let instances = detector.find();
            found_pms.extend(instances);
        }

        found_pms
    };

    if filtered_pms.is_empty() {
        if let Some(cat_str) = category {
            println!("No {} package managers found.", cat_str);
        }
        return;
    }

    match (all, tree) {
        (false, false) => {
            // Default: grouped table
            let mut table = Table::new(group_pm_instances(filtered_pms));
            println!("{}", table.with(Style::modern()));
            if !all {
                println!("\nTip: Use --all to see all the other locations");
            }
        },
        (true, false) => {
            // All instances table
            let mut table = Table::new(filtered_pms);
            println!("{}", table.with(Style::modern()));
        },
        (false, true) => {
            // Grouped tree
            display_grouped_tree(group_pm_instances(filtered_pms));
            if !all {
                println!("\nTip: Use --all to see all the other locations");
            }
        },
        (true, true) => {
            // All instances tree
            display_tree(filtered_pms);
        },
    }
}

fn parse_category(category_str: &str) -> Option<Category> {
    match category_str.to_lowercase().as_str() {
        "system" | "sys" => Some(Category::System),
        "javascript" | "js" | "typescript" | "ts" | "node.js" | "node" => {
            Some(Category::JavaScript)
        },
        "python" | "py" => Some(Category::Python),
        "rust" | "rs" => Some(Category::Rust),
        "go" => Some(Category::Go),
        "haskell" => Some(Category::Haskell),
        "gleam" => Some(Category::Gleam),
        "tools" => Some(Category::Tools),
        _ => None,
    }
}

fn filter_by_category(target_category: Category) -> Vec<pm::PmInfo> {
    let mut filtered = Vec::new();

    for detector in pm::all_package_managers() {
        if detector.category() == target_category {
            let instances = detector.find();
            filtered.extend(instances);
        }
    }

    filtered
}
