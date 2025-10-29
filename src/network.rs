//! Network connectivity checking utilities.

use std::time::Duration;

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
    let agent = ureq::agent();

    // Try multiple reliable endpoints to avoid false negatives
    let endpoints = [
        "https://dns.google/generate_204", // Google's connectivity check endpoint (returns 204)
        "https://www.cloudflare.com/cdn-cgi/trace", // Cloudflare's connectivity check
        "https://api.github.com/zen",      // GitHub API zen endpoint
    ];

    for endpoint in endpoints {
        if let Ok(response) = agent.get(endpoint).call() {
            let status = response.status();
            if status == 204 || status == 200 {
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
        if latency > Duration::from_secs(2) {
            NetworkStatus::Slow(latency)
        } else {
            NetworkStatus::Good
        }
    } else {
        NetworkStatus::Offline
    }
}
