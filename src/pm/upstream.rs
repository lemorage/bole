//! Version checking via upstream APIs.
//!
//! Provides unified interface for checking latest package versions from
//! various registries.

use std::fmt;

use serde_json::Value;

/// Error type for version fetching.
#[derive(Debug)]
pub(crate) enum FetchErr {
    /// HTTP request failed
    Http(ureq::Error),
    /// JSON parsing failed
    Json(serde_json::Error),
    /// Expected field missing in response
    Missing,
}

impl fmt::Display for FetchErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchErr::Http(e) => write!(f, "HTTP request failed: {}", e),
            FetchErr::Json(e) => write!(f, "JSON parsing failed: {}", e),
            FetchErr::Missing => write!(f, "Expected field missing in response"),
        }
    }
}

impl std::error::Error for FetchErr {}

impl From<ureq::Error> for FetchErr {
    fn from(e: ureq::Error) -> Self {
        FetchErr::Http(e)
    }
}

impl From<serde_json::Error> for FetchErr {
    fn from(e: serde_json::Error) -> Self {
        FetchErr::Json(e)
    }
}

/// Supported upstream package registries.
#[derive(Debug, Clone)]
pub(crate) enum Upstream {
    /// npm registry (uses dist-tags endpoint for minimal response).
    Npm(&'static str),
    /// GitHub releases API (extracts tag_name from latest release).
    GitHub {
        owner: &'static str,
        repo: &'static str,
    },
    /// Python Package Index (uses Simple API with JSON accept header).
    PyPI(&'static str),
}

impl Upstream {
    /// Fetch latest version from upstream registry.
    pub(crate) fn latest(&self, http: &ureq::Agent) -> Result<String, FetchErr> {
        match self {
            Upstream::Npm(pkg) => {
                let url = format!("https://registry.npmjs.org/-/package/{}/dist-tags", pkg);
                let json = fetch_json(http, &url, None)?;
                json["latest"]
                    .as_str()
                    .ok_or(FetchErr::Missing)
                    .map(String::from)
            },
            Upstream::GitHub { owner, repo } => {
                let url = format!(
                    "https://api.github.com/repos/{}/{}/releases/latest",
                    owner, repo
                );
                let headers = vec![
                    ("User-Agent", user_agent()),
                    ("Accept", "application/vnd.github+json"),
                ];
                let json = fetch_json(http, &url, Some(&headers))?;
                let tag = json["tag_name"].as_str().ok_or(FetchErr::Missing)?;

                // Strip "{repo}-v" prefix (e.g., "bun-v1.0" -> "1.0")
                let version = tag
                    .strip_prefix(&format!("{}-v", repo))
                    .or_else(|| tag.strip_prefix('v'))
                    .unwrap_or(tag);

                Ok(version.to_string())
            },
            Upstream::PyPI(pkg) => {
                let url = format!("https://pypi.org/simple/{}/", pkg);
                let headers = vec![
                    ("User-Agent", user_agent()),
                    ("Accept", "application/vnd.pypi.simple.v1+json"),
                ];
                let json = fetch_json(http, &url, Some(&headers))?;
                json["versions"]
                    .as_array()
                    .and_then(|arr| arr.last())
                    .and_then(|v| v.as_str())
                    .ok_or(FetchErr::Missing)
                    .map(String::from)
            },
        }
    }
}

/// Fetch JSON from a URL with optional headers.
#[cfg(not(test))]
fn fetch_json(
    http: &ureq::Agent,
    url: &str,
    headers: Option<&[(&str, &str)]>,
) -> Result<Value, FetchErr> {
    let mut req = http.get(url);
    if let Some(headers) = headers {
        for (key, value) in headers {
            req = req.header(*key, *value);
        }
    }
    let response = req.call()?;
    let reader = response
        .into_body()
        .into_with_config()
        .limit(10 * 1024 * 1024)
        .reader();
    Ok(serde_json::from_reader(reader)?)
}

// Test mock for fetch_json
#[cfg(test)]
fn fetch_json(
    _http: &ureq::Agent,
    url: &str,
    _headers: Option<&[(&str, &str)]>,
) -> Result<Value, FetchErr> {
    // npm registry mocks
    if url.contains("test-npm") {
        return Ok(serde_json::from_str(r#"{"latest": "10.5.0"}"#)?);
    }
    if url.contains("npm-missing") {
        return Ok(serde_json::from_str(r#"{"next": "11.0.0"}"#)?);
    }

    // GitHub API mocks
    if url.contains("github.com") {
        if url.contains("test-repo") {
            return Ok(serde_json::from_str(r#"{"tag_name": "v1.2.3"}"#)?);
        }
        if url.contains("bun") {
            return Ok(serde_json::from_str(r#"{"tag_name": "bun-v1.0.0"}"#)?);
        }
        if url.contains("no-prefix") {
            return Ok(serde_json::from_str(r#"{"tag_name": "2.0.0"}"#)?);
        }
        if url.contains("missing-tag") {
            return Ok(serde_json::from_str(r#"{"name": "Release"}"#)?);
        }
    }

    // PyPI mocks
    if url.contains("pypi.org") {
        if url.contains("test-pypi") {
            return Ok(serde_json::from_str(r#"{"versions": ["0.1.0", "1.0.0"]}"#)?);
        }
        if url.contains("empty-versions") {
            return Ok(serde_json::from_str(r#"{"versions": []}"#)?);
        }
        if url.contains("no-versions") {
            return Ok(serde_json::from_str(r#"{"name": "package"}"#)?);
        }
        if url.contains("invalid-versions") {
            return Ok(serde_json::from_str(r#"{"versions": [123, 456]}"#)?);
        }
    }

    // Invalid JSON mock
    if url.contains("bad-json") {
        return Err(FetchErr::Json(
            serde_json::from_str::<Value>("invalid").unwrap_err(),
        ));
    }

    Err(FetchErr::Missing)
}

/// Returns user agent string for HTTP requests.
#[inline]
fn user_agent() -> &'static str {
    concat!("bole/", env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npm_latest() {
        // Arrange
        let agent = ureq::agent();

        // Act & Assert
        let upstream = Upstream::Npm("test-npm");
        assert_eq!(upstream.latest(&agent).unwrap(), "10.5.0");

        // Act & Assert
        let upstream = Upstream::Npm("npm-missing");
        assert!(matches!(upstream.latest(&agent), Err(FetchErr::Missing)));
    }

    #[test]
    fn test_github_latest() {
        // Arrange
        let agent = ureq::agent();

        // Act & Assert (v prefix)
        let upstream = Upstream::GitHub {
            owner: "owner",
            repo: "test-repo",
        };
        assert_eq!(upstream.latest(&agent).unwrap(), "1.2.3");

        // Act & Assert (repo-v prefix)
        let upstream = Upstream::GitHub {
            owner: "owner",
            repo: "bun",
        };
        assert_eq!(upstream.latest(&agent).unwrap(), "1.0.0");

        // Act & Assert (no prefix)
        let upstream = Upstream::GitHub {
            owner: "owner",
            repo: "no-prefix",
        };
        assert_eq!(upstream.latest(&agent).unwrap(), "2.0.0");

        // Act & Assert (missing tag)
        let upstream = Upstream::GitHub {
            owner: "owner",
            repo: "missing-tag",
        };
        assert!(matches!(upstream.latest(&agent), Err(FetchErr::Missing)));
    }

    #[test]
    fn test_pypi_latest() {
        // Arrange
        let agent = ureq::agent();

        // Act & Assert (success)
        let upstream = Upstream::PyPI("test-pypi");
        assert_eq!(upstream.latest(&agent).unwrap(), "1.0.0");

        // Act & Assert (empty versions)
        let upstream = Upstream::PyPI("empty-versions");
        assert!(matches!(upstream.latest(&agent), Err(FetchErr::Missing)));

        // Act & Assert (missing field)
        let upstream = Upstream::PyPI("no-versions");
        assert!(matches!(upstream.latest(&agent), Err(FetchErr::Missing)));

        // Act & Assert (invalid versions)
        let upstream = Upstream::PyPI("invalid-versions");
        assert!(matches!(upstream.latest(&agent), Err(FetchErr::Missing)));
    }

    #[test]
    fn test_json_error() {
        // Arrange
        let agent = ureq::agent();
        let upstream = Upstream::Npm("bad-json");

        // Acts
        let result = upstream.latest(&agent);

        // Assert
        assert!(matches!(result, Err(FetchErr::Json(_))));
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            FetchErr::Missing.to_string(),
            "Expected field missing in response"
        );

        let json_err = serde_json::from_str::<Value>("bad").unwrap_err();
        assert!(
            FetchErr::Json(json_err)
                .to_string()
                .starts_with("JSON parsing failed:")
        );
    }

    #[test]
    fn test_error_trait() {
        // Arrange
        let err = FetchErr::Missing;

        // Act & Assert
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_from_impls() {
        // Arrange & Act & Assert
        let json_err = serde_json::from_str::<Value>("bad").unwrap_err();
        let _: FetchErr = json_err.into();

        let ureq_err = ureq::get("http://[invalid").call().unwrap_err();
        let _: FetchErr = ureq_err.into();
    }

    #[test]
    fn test_upstream_traits() {
        // Arrange
        let npm = Upstream::Npm("test");
        let github = Upstream::GitHub {
            owner: "o",
            repo: "r",
        };

        // Act & Assert
        let npm2 = npm.clone();
        match (npm, npm2) {
            (Upstream::Npm(a), Upstream::Npm(b)) => assert_eq!(a, b),
            _ => panic!("Clone failed"),
        }

        // Act & Assert
        assert!(format!("{:?}", github).contains("GitHub"));
    }

    #[test]
    fn test_user_agent() {
        // Arrange & Act
        let agent = user_agent();

        // Assert
        assert!(agent.starts_with("bole/"));
        assert!(!agent.contains(' '));
    }
}
