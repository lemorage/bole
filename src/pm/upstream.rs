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

/// Returns user agent string for HTTP requests.
#[inline]
fn user_agent() -> &'static str {
    concat!("bole/", env!("CARGO_PKG_VERSION"))
}
