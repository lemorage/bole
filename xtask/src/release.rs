use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use colored::*;
use regex::Regex;

use crate::{run_clippy, run_fmt, run_test};

#[derive(Debug)]
pub struct ReleaseArgs {
    pub version: String,
    pub dry_run: bool,
    pub no_push: bool,
    pub skip_checks: bool,
    pub yes: bool,
}

#[derive(Debug)]
pub struct ReleaseConfig {
    pub args: ReleaseArgs,
    pub branch_name: String,
    pub default_branch: String,
    pub curr_ver: String,
    pub new_ver: String,
    pub tag: String,
    pub prev_tag: String,
}

pub fn run(args: ReleaseArgs) -> Result<()> {
    let mut config = ReleaseConfig {
        branch_name: get_current_branch()?,
        default_branch: std::env::var("DEFAULT_BRANCH").unwrap_or_else(|_| "dev".to_string()),
        curr_ver: String::new(),
        new_ver: String::new(),
        tag: String::new(),
        prev_tag: String::new(),
        args,
    };

    main_flow(&mut config)
}

fn main_flow(config: &mut ReleaseConfig) -> Result<()> {
    // Phase 1: Validation & Setup
    validate_environment(config.args.dry_run)?;
    validate_version_input(&config.args.version)?;

    if config.branch_name != config.default_branch {
        eprintln!(
            "{}",
            format!(
                "[warn]  Current branch is '{}' (expected '{}').",
                config.branch_name, config.default_branch
            )
            .yellow()
        );
        eprintln!(
            "{}",
            "[warn]  Proceeding in 5 seconds. Ctrl-C to abort.".yellow()
        );
        if !config.args.dry_run {
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }

    calculate_versions(config)?;
    config.prev_tag = get_previous_tag().unwrap_or_default();

    // Phase 2: Pre-flight Checks
    if !config.args.skip_checks {
        run_quality_checks()?;
    }

    // Phase 3: File Updates
    update_cargo_files(config)?;
    preview_changes(config)?;

    // Phase 4: Git Operations
    create_release_commit(config)?;
    if !config.args.no_push {
        push_release(config)?;
    } else {
        println!("{}", "--no-push set; not pushing to origin.".yellow());
    }

    println!(
        "{}",
        format!(
            "[info]  Done. CI will publish and attach binaries for {}.",
            config.tag
        )
        .cyan()
    );
    Ok(())
}

fn find_workspace_root() -> Result<PathBuf> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    if is_workspace_root(&current_dir)? {
        return Ok(current_dir);
    }

    if let Some(parent) = current_dir.parent() {
        if is_workspace_root(parent)? {
            return Ok(parent.to_path_buf());
        }
    }

    bail!("Could not find workspace root directory");
}

fn is_workspace_root(path: &Path) -> Result<bool> {
    let cargo_toml_path = path.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(&cargo_toml_path).context("Failed to read Cargo.toml")?;

    Ok(content.contains("[workspace]"))
}

fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to get current branch")?;

    if !output.status.success() {
        bail!("Git command failed");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn validate_environment(dry_run: bool) -> Result<()> {
    let workspace_root = find_workspace_root()?;

    std::env::set_current_dir(&workspace_root)
        .context("Failed to change to workspace root directory")?;

    if Command::new("git").arg("--version").output().is_err() {
        bail!("git not found in PATH");
    }

    require_clean_git(dry_run)?;
    Ok(())
}

fn require_clean_git(dry_run: bool) -> Result<()> {
    if dry_run {
        println!(
            "{}",
            "[warn] Skipping clean working tree check due to --dry-run".yellow()
        );
        return Ok(());
    }

    let output = Command::new("git")
        .args(["diff-index", "--quiet", "HEAD", "--"])
        .output()
        .context("Failed to check git status")?;

    if !output.status.success() {
        bail!("Working tree is dirty. Commit or stash changes first.");
    }

    Ok(())
}

fn validate_version_input(version: &str) -> Result<()> {
    let version_pattern = Regex::new(r"^(major|minor|patch|[0-9]+\.[0-9]+\.[0-9]+)$")?;

    if !version_pattern.is_match(version) {
        bail!(
            "Invalid version argument: '{}' (use major|minor|patch or x.y.z)",
            version
        );
    }

    Ok(())
}

fn calculate_versions(config: &mut ReleaseConfig) -> Result<()> {
    // Read current version from Cargo.toml
    let cargo_toml = std::fs::read_to_string("Cargo.toml").context("Failed to read Cargo.toml")?;

    let doc = cargo_toml
        .parse::<toml_edit::DocumentMut>()
        .context("Failed to parse Cargo.toml")?;

    let current_version = doc["package"]["version"]
        .as_str()
        .context("No version found in Cargo.toml")?;

    config.curr_ver = current_version.to_string();

    // Parse current version into parts
    let version_parts: Vec<&str> = current_version.split('.').collect();
    if version_parts.len() != 3 {
        bail!("Invalid version format: {}", current_version);
    }

    let major: u32 = version_parts[0].parse().context("Invalid major version")?;
    let minor: u32 = version_parts[1].parse().context("Invalid minor version")?;
    let patch: u32 = version_parts[2].parse().context("Invalid patch version")?;

    // Calculate new version based on argument
    let (new_major, new_minor, new_patch) = match config.args.version.as_str() {
        "major" => (major + 1, 0, 0),
        "minor" => (major, minor + 1, 0),
        "patch" => (major, minor, patch + 1),
        explicit => {
            // Handle explicit version like "1.2.3"
            let parts: Vec<&str> = explicit.split('.').collect();
            if parts.len() != 3 {
                bail!("Invalid explicit version format: {}", explicit);
            }
            (
                parts[0]
                    .parse()
                    .context("Invalid major in explicit version")?,
                parts[1]
                    .parse()
                    .context("Invalid minor in explicit version")?,
                parts[2]
                    .parse()
                    .context("Invalid patch in explicit version")?,
            )
        },
    };

    config.new_ver = format!("{}.{}.{}", new_major, new_minor, new_patch);
    config.tag = format!("v{}", config.new_ver);

    println!(
        "{}",
        format!("[info] Current version: {}", config.curr_ver).cyan()
    );
    println!(
        "{}",
        format!("[info] Releasing {} (tag {})", config.new_ver, config.tag).cyan()
    );

    Ok(())
}

fn run_quality_checks() -> Result<()> {
    println!("{}", "[info] Running checks: fmt, clippy, test".cyan());
    run_fmt()?;
    run_clippy()?;
    run_test()?;
    Ok(())
}

fn update_cargo_files(config: &ReleaseConfig) -> Result<()> {
    if config.args.dry_run {
        println!(
            "{}",
            format!("[dry-run] Would update Cargo.toml to {}", config.new_ver).yellow()
        );
        println!("{}", "[dry-run] cargo generate-lockfile".yellow());
        return Ok(());
    }

    println!(
        "{}",
        format!("[info] Updating Cargo.toml version -> {}", config.new_ver).cyan()
    );

    let cargo_toml = std::fs::read_to_string("Cargo.toml").context("Failed to read Cargo.toml")?;

    let mut doc = cargo_toml
        .parse::<toml_edit::DocumentMut>()
        .context("Failed to parse Cargo.toml")?;

    doc["package"]["version"] = toml_edit::value(config.new_ver.clone());

    std::fs::write("Cargo.toml", doc.to_string()).context("Failed to write Cargo.toml")?;

    println!("{}", "[info] Regenerating Cargo.lock".cyan());
    let status = Command::new("cargo")
        .args(["generate-lockfile"])
        .output()
        .context("Failed to regenerate Cargo.lock")?;

    if !status.status.success() {
        bail!("cargo generate-lockfile failed");
    }

    Ok(())
}

fn preview_changes(config: &ReleaseConfig) -> Result<()> {
    if config.args.dry_run {
        println!("[dry-run] Would show git diff of changes");
        return Ok(());
    }

    println!("[info] Previewing changes:");
    let status = Command::new("git")
        .args(["diff", "--color=always"])
        .status()
        .context("Failed to run git diff")?;

    if !status.success() {
        bail!("Git diff failed");
    }
    Ok(())
}

fn create_release_commit(config: &ReleaseConfig) -> Result<()> {
    if config.args.dry_run {
        println!("[dry-run] Would create commit and tag {}", config.tag);
        return Ok(());
    }

    println!("[info] Creating release commit and tag {}...", config.tag);

    // Stage files
    Command::new("git")
        .args(["add", "Cargo.toml", "Cargo.lock"])
        .status()
        .context("Failed to stage files")?;

    // Create commit
    let commit_msg = format!("chore(release): bump bole version to {}", config.new_ver);
    let status = Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .status()
        .context("Failed to create commit")?;

    if !status.success() {
        bail!("Git commit failed");
    }

    // Create tag
    let tag_msg = format!("Release {}", config.new_ver);
    let status = Command::new("git")
        .args(["tag", "-a", &config.tag, "-m", &tag_msg])
        .status()
        .context("Failed to create tag")?;

    if !status.success() {
        bail!("Git tag failed");
    }

    Ok(())
}

fn push_release(config: &ReleaseConfig) -> Result<()> {
    if config.args.dry_run {
        println!(
            "{}",
            format!(
                "[dry-run] Would push branch '{}' and tag '{}'",
                config.branch_name, config.tag
            )
            .yellow()
        );
        return Ok(());
    }

    if !config.args.yes {
        show_push_summary(config)?;
        if !confirm_push(config)? {
            return Ok(());
        }
    }

    println!("{}", "[info] Pushing commit and tags to origin".cyan());

    let status = Command::new("git")
        .args(["push", "origin", &config.branch_name])
        .status()
        .context("Failed to push commit")?;

    if !status.success() {
        bail!("Git push failed");
    }

    let status = Command::new("git")
        .args(["push", "origin", &config.tag])
        .status()
        .context("Failed to push tag")?;

    if !status.success() {
        bail!("Git push tag failed");
    }

    println!(
        "{}",
        format!("✓ Successfully pushed {} to origin!", config.tag).green()
    );
    Ok(())
}

fn show_push_summary(config: &ReleaseConfig) -> Result<()> {
    let origin_url = get_origin_url().unwrap_or_else(|_| "(no origin configured)".to_string());

    println!("{}", "[info] Push summary:".cyan());
    println!("  Repo:     {}", origin_url);
    println!("  Branch:   {}", config.branch_name);
    println!("  Version:  {}", config.new_ver);
    println!("  Tag:      {}", config.tag);

    if !config.prev_tag.is_empty() {
        println!("  Prev tag: {}", config.prev_tag);
        let commit_count = get_commit_count(&format!("{}..HEAD", config.prev_tag))?;
        println!("  Commits:  {}", commit_count);
    } else {
        println!("  Prev tag: (none)");
        let commit_count = get_commit_count("HEAD")?;
        println!("  Commits:  {}", commit_count);
    }
    println!();

    println!("Commits included in this release (most recent first):");
    if !config.prev_tag.is_empty() {
        let _ = Command::new("git")
            .args([
                "--no-pager",
                "log",
                "--oneline",
                &format!("{}..HEAD", config.prev_tag),
            ])
            .status();
    } else {
        let _ = Command::new("git")
            .args(["--no-pager", "log", "--oneline"])
            .status();
    }
    println!();

    println!("Release commit diff summary:");
    let _ = Command::new("git")
        .args(["--no-pager", "show", "--stat", "-1", "HEAD"])
        .status();

    Ok(())
}

fn confirm_push(config: &ReleaseConfig) -> Result<bool> {
    print!(
        "Push branch '{}' and tag '{}' to origin? [y/N] ",
        config.branch_name, config.tag
    );
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;

    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => Ok(true),
        _ => {
            println!("{}", "[info] Aborting push.".yellow());
            Ok(false)
        },
    }
}

fn get_origin_url() -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("Failed to get origin URL")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        bail!("No origin configured")
    }
}

fn get_commit_count(range: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-list", "--count", range])
        .output()
        .context("Failed to count commits")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Ok("0".to_string())
    }
}

fn get_previous_tag() -> Result<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0", "HEAD~"])
        .output()
        .context("Failed to get previous tag")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Ok(String::new())
    }
}
