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
    /// Packagist (PHP package repository).
    Packagist {
        vendor: &'static str,
        package: &'static str,
    },
    /// PEAR (PHP Extension and Application Repository).
    Pear(&'static str),
    /// Python Package Index (uses Simple API with JSON accept header).
    PyPI(&'static str),
    /// RubyGems repository.
    RubyGems(&'static str),
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

                // Strip various prefixes:
                // - "cabal-install-v" for cabal
                // - "{repo}-v" for bun-like repos
                // - "v" for standard repos
                let version = if *repo == "cabal" {
                    tag.strip_prefix("cabal-install-v")
                        .or_else(|| tag.strip_prefix('v'))
                        .unwrap_or(tag)
                } else {
                    tag.strip_prefix(&format!("{}-v", repo))
                        .or_else(|| tag.strip_prefix('v'))
                        .unwrap_or(tag)
                };

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
            Upstream::Packagist { vendor, package } => {
                let url = format!("https://packagist.org/p2/{}/{}.json", vendor, package);
                let json = fetch_json(http, &url, None)?;

                // Packagist returns nested structure: packages -> vendor/package -> array of
                // versions We want the first element which is the latest stable
                // version
                json["packages"]
                    .get(format!("{}/{}", vendor, package))
                    .and_then(|versions| versions.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v["version"].as_str())
                    .ok_or(FetchErr::Missing)
                    .map(String::from)
            },
            Upstream::Pear(package) => {
                // PEAR has a simple text API that returns just the version
                // We use fetch_json but expect a plain string wrapped in JSON
                let url = format!("https://pear.php.net/rest/r/{}/latest.txt", package);
                let json = fetch_json(http, &url, None)?;

                json.as_str()
                    .ok_or(FetchErr::Missing)
                    .map(|s| s.trim().to_string())
            },
            Upstream::RubyGems(gem) => {
                let url = format!("https://rubygems.org/api/v1/versions/{}/latest.json", gem);
                let json = fetch_json(http, &url, None)?;
                json["version"]
                    .as_str()
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

    // Special case for PEAR plain text responses
    if url.contains("pear.php.net") && url.ends_with(".txt") {
        let mut body = response.into_body();
        let text = body.read_to_string()?;
        return Ok(Value::String(text.trim().to_string()));
    }

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
        if url.contains("cabal") {
            return Ok(serde_json::from_str(
                r#"{"tag_name": "cabal-install-v3.12.1.0"}"#,
            )?);
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

    // Packagist mocks
    if url.contains("packagist.org") {
        if url.contains("composer/composer") {
            return Ok(serde_json::from_str(
                r#"{"packages": {"composer/composer": [{"version": "2.8.1"}]}}"#,
            )?);
        }
        if url.contains("missing-package") {
            return Ok(serde_json::from_str(r#"{"packages": {}}"#)?);
        }
    }

    // PEAR mocks (returns plain text wrapped in JSON string)
    if url.contains("pear.php.net") {
        if url.contains("/pear/") {
            return Ok(Value::String("1.10.16".to_string()));
        }
        if url.contains("/missing/") {
            return Err(FetchErr::Missing);
        }
    }

    // RubyGems mocks
    if url.contains("rubygems.org") {
        if url.contains("rubygems-update") {
            return Ok(serde_json::from_str(r#"{"version": "3.5.0"}"#)?);
        }
        if url.contains("bundler") {
            return Ok(serde_json::from_str(r#"{"version": "2.5.0"}"#)?);
        }
        if url.contains("missing-gem") {
            return Ok(serde_json::from_str(r#"{"name": "missing"}"#)?);
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

        // Act & Assert (cabal-install-v prefix)
        let upstream = Upstream::GitHub {
            owner: "haskell",
            repo: "cabal",
        };
        assert_eq!(upstream.latest(&agent).unwrap(), "3.12.1.0");
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
    fn test_packagist_latest() {
        // Arrange
        let agent = ureq::agent();

        // Act & Assert (success)
        let upstream = Upstream::Packagist {
            vendor: "composer",
            package: "composer",
        };
        assert_eq!(upstream.latest(&agent).unwrap(), "2.8.1");

        // Act & Assert (missing package)
        let upstream = Upstream::Packagist {
            vendor: "missing",
            package: "missing-package",
        };
        assert!(matches!(upstream.latest(&agent), Err(FetchErr::Missing)));
    }

    #[test]
    fn test_pear_latest() {
        // Arrange
        let agent = ureq::agent();

        // Act & Assert (success)
        let upstream = Upstream::Pear("pear");
        assert_eq!(upstream.latest(&agent).unwrap(), "1.10.16");

        // Act & Assert (missing package)
        let upstream = Upstream::Pear("missing");
        assert!(matches!(upstream.latest(&agent), Err(FetchErr::Missing)));
    }

    #[test]
    fn test_rubygems_latest() {
        // Arrange
        let agent = ureq::agent();

        // Act & Assert (success)
        let upstream = Upstream::RubyGems("rubygems-update");
        assert_eq!(upstream.latest(&agent).unwrap(), "3.5.0");

        let upstream = Upstream::RubyGems("bundler");
        assert_eq!(upstream.latest(&agent).unwrap(), "2.5.0");

        // Act & Assert (missing field)
        let upstream = Upstream::RubyGems("missing-gem");
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
        let packagist = Upstream::Packagist {
            vendor: "v",
            package: "p",
        };
        let pear = Upstream::Pear("test");
        let rubygems = Upstream::RubyGems("test");

        // Act & Assert
        let npm2 = npm.clone();
        match (npm, npm2) {
            (Upstream::Npm(a), Upstream::Npm(b)) => assert_eq!(a, b),
            _ => panic!("Clone failed"),
        }

        let packagist2 = packagist.clone();
        match (packagist, packagist2) {
            (
                Upstream::Packagist {
                    vendor: v1,
                    package: p1,
                },
                Upstream::Packagist {
                    vendor: v2,
                    package: p2,
                },
            ) => {
                assert_eq!(v1, v2);
                assert_eq!(p1, p2);
            },
            _ => panic!("Clone failed"),
        }

        let pear2 = pear.clone();
        match (pear, pear2) {
            (Upstream::Pear(a), Upstream::Pear(b)) => assert_eq!(a, b),
            _ => panic!("Clone failed"),
        }

        let rubygems2 = rubygems.clone();
        match (rubygems, rubygems2) {
            (Upstream::RubyGems(a), Upstream::RubyGems(b)) => assert_eq!(a, b),
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
