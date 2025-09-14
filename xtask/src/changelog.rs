use std::{collections::HashMap, fs, process::Command};

use anyhow::{Context, Result, bail};
use regex::Regex;

pub fn run() -> Result<()> {
    println!("[info] Generating changelog preview...");
    generate_changelog()
}

fn generate_changelog() -> Result<()> {
    let repo_slug = resolve_repo_slug()?;

    let current_tag = get_current_tag()?;
    let prev_tag = get_previous_tag()?;
    let range = if prev_tag.is_empty() {
        "HEAD".to_string()
    } else {
        format!("{}..HEAD", prev_tag)
    };

    println!("[info]  Analyzing commits in range: {}", range);

    let categories = process_commits(&range)?;
    emit_changelog_sections(&current_tag, &prev_tag, &repo_slug, &categories)?;

    println!("[info]  Generated changelog.md");
    Ok(())
}

fn resolve_repo_slug() -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("Failed to get remote URL")?;

    if !output.status.success() {
        return Ok("local/repo".to_string());
    }

    let url_binding = String::from_utf8_lossy(&output.stdout);
    let url = url_binding.trim();

    if let Some(captures) = extract_github_slug(url) {
        Ok(captures)
    } else {
        Ok("local/repo".to_string())
    }
}

fn extract_github_slug(url: &str) -> Option<String> {
    if url.contains("github.com") {
        // Extract from formats like:
        // git@github.com:user/repo.git
        // https://github.com/user/repo.git
        let parts: Vec<&str> = if url.contains('@') {
            url.split(':').collect()
        } else {
            url.split('/').collect()
        };

        if parts.len() >= 2 {
            let last_parts: Vec<&str> = parts.last()?.split('/').collect();
            if last_parts.len() >= 2 {
                let user = last_parts[last_parts.len() - 2];
                let repo = last_parts[last_parts.len() - 1].trim_end_matches(".git");
                return Some(format!("{}/{}", user, repo));
            }
        }
    }
    None
}

fn get_current_tag() -> Result<String> {
    if let Ok(tag) = std::env::var("GITHUB_REF_NAME")
        && !tag.is_empty()
    {
        return Ok(tag);
    }

    let output = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()
        .context("Failed to get current tag")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Ok("v0.0.0-local".to_string())
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

fn process_commits(range: &str) -> Result<HashMap<String, Vec<String>>> {
    let output = Command::new("git")
        .args([
            "log",
            "-z",
            "--no-merges",
            "--format=%H%x1f%s%x1f%b%x00",
            range,
        ])
        .output()
        .context("Failed to get git log")?;

    if !output.status.success() {
        bail!("Git log command failed");
    }

    let log_output = String::from_utf8_lossy(&output.stdout);
    let mut categories: HashMap<String, Vec<String>> = HashMap::new();

    categories.insert("feat".to_string(), Vec::new());
    categories.insert("fix".to_string(), Vec::new());
    categories.insert("perf".to_string(), Vec::new());
    categories.insert("revert".to_string(), Vec::new());
    categories.insert("docs".to_string(), Vec::new());
    categories.insert("breaking".to_string(), Vec::new());

    for record in log_output.split('\0') {
        if record.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = record.split('\x1f').collect();
        if parts.len() < 3 {
            continue;
        }

        let hash = parts[0];
        let subject = parts[1];
        let body = parts.get(2).unwrap_or(&"");
        let header = if let Some(colon_pos) = subject.find(':') {
            &subject[..colon_pos]
        } else {
            subject
        };

        let is_breaking = header.contains('!')
            || body.lines().any(|line| {
                line.to_lowercase().starts_with("breaking change")
                    || line.to_lowercase().starts_with("breaking:")
            });

        let base_type = if let Some(paren_pos) = header.find('(') {
            &header[..paren_pos]
        } else {
            header
        };
        let base_type = base_type.trim_end_matches('!');
        let commit_type = base_type.to_lowercase();

        let line = format!("- {}", subject);

        match commit_type.as_str() {
            "feat" => categories.get_mut("feat").unwrap().push(line.clone()),
            "fix" => categories.get_mut("fix").unwrap().push(line.clone()),
            "perf" => categories.get_mut("perf").unwrap().push(line.clone()),
            "revert" => categories.get_mut("revert").unwrap().push(line.clone()),
            "docs" => {
                if include_docs_commit(hash)? {
                    categories.get_mut("docs").unwrap().push(line.clone());
                }
            },
            _ => {}, // Ignore other types
        }

        if is_breaking {
            categories.get_mut("breaking").unwrap().push(line);
        }
    }

    Ok(categories)
}

fn emit_changelog_sections(
    current_tag: &str,
    prev_tag: &str,
    repo_slug: &str,
    categories: &HashMap<String, Vec<String>>,
) -> Result<()> {
    let output_file = "changelog.md";
    let date_utc = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut content = format!("# Changelog {} ({})\n\n", current_tag, date_utc);

    if !prev_tag.is_empty() {
        content.push_str(&format!(
            "Compare {}...{} (https://github.com/{}/compare/{}...{})\n\n",
            prev_tag, current_tag, repo_slug, prev_tag, current_tag
        ));

        let count_output = Command::new("git")
            .args(["rev-list", "--count", &format!("{}..HEAD", prev_tag)])
            .output()
            .context("Failed to count commits")?;

        if count_output.status.success() {
            let commit_count_str = String::from_utf8_lossy(&count_output.stdout);
            let commit_count = commit_count_str.trim();
            content.push_str(&format!(
                "This release includes {} commits.\n\n",
                commit_count
            ));
        }
    } else {
        let count_output = Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .output()
            .context("Failed to count commits")?;

        if count_output.status.success() {
            let commit_count_str = String::from_utf8_lossy(&count_output.stdout);
            let commit_count = commit_count_str.trim();
            content.push_str(&format!(
                "This release includes {} commits.\n\n",
                commit_count
            ));
        }
    }

    let sections = [
        ("Breaking Changes", "breaking"),
        ("New Features", "feat"),
        ("Bug Fixes", "fix"),
        ("Performance", "perf"),
        ("Documentation", "docs"),
        ("Reverts", "revert"),
    ];

    let mut has_sections = false;
    for (title, key) in &sections {
        if let Some(commits) = categories.get(*key)
            && !commits.is_empty()
        {
            content.push_str(&format!("## {}\n\n", title));
            for commit in commits {
                content.push_str(&format!("{}\n", commit));
            }
            content.push('\n');
            has_sections = true;
        }
    }

    if !has_sections {
        content.push_str("No user-facing changes in this release.\n");
    }

    fs::write(output_file, content).context("Failed to write changelog")?;
    println!("Generated {}", output_file);

    Ok(())
}

fn include_docs_commit(hash: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["diff-tree", "--no-commit-id", "--name-only", "-r", hash])
        .output()
        .context("Failed to get commit file changes")?;

    if !output.status.success() {
        return Ok(false);
    }

    let files = String::from_utf8_lossy(&output.stdout);
    let doc_pattern = Regex::new(
        r"\.(md|rst|txt|adoc)$|^docs/|^documentation/|README|CHANGELOG|CONTRIBUTING|LICENSE",
    )?;

    Ok(files.lines().any(|file| doc_pattern.is_match(file)))
}
