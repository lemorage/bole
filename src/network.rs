//! Network connectivity checking utilities.

use std::time::Duration;

/// Timeout for each network connectivity check attempt.
const TIMEOUT_SECS: u64 = 3;

/// Threshold for determining if network is slow vs good.
const SLOW_THRESHOLD_SECS: u64 = 2;

/// Expected HTTP status codes for successful connectivity checks.
const HTTP_OK: u16 = 200;
const HTTP_NO_CONTENT: u16 = 204;

/// Network connectivity status.
#[derive(Debug)]
pub(crate) enum NetworkStatus {
    /// Network is functioning well
    Good,
    /// Network is slow (with response time)
    Slow(Duration),
    /// No network connectivity detected
    Offline,
}

/// Check if the system has internet connectivity.
///
/// Performs a lightweight connectivity check by attempting to reach
/// well-known, highly available endpoints.
fn has_internet_connection() -> bool {
    // Configure agent with reasonable timeout to prevent hanging
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(TIMEOUT_SECS)))
        .build();
    let agent: ureq::Agent = config.into();

    // Try multiple reliable endpoints to avoid false negatives
    let endpoints = [
        "https://dns.google/generate_204", // Google's connectivity check endpoint (returns 204)
        "https://www.cloudflare.com/cdn-cgi/trace", // Cloudflare's connectivity check
        "https://api.github.com/zen",      // GitHub API zen endpoint
    ];

    for endpoint in endpoints {
        if let Ok(response) = agent.get(endpoint).call() {
            let status = response.status();
            if status == HTTP_OK || status == HTTP_NO_CONTENT {
                return true;
            }
        }
    }

    false
}

/// Check network connectivity with a timeout and return detailed status.
pub(crate) fn check_network_status() -> NetworkStatus {
    let start = std::time::Instant::now();

    if has_internet_connection() {
        let latency = start.elapsed();
        if latency > Duration::from_secs(SLOW_THRESHOLD_SECS) {
            NetworkStatus::Slow(latency)
        } else {
            NetworkStatus::Good
        }
    } else {
        NetworkStatus::Offline
    }
}
