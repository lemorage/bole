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

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::{TcpListener, TcpStream},
        sync::Mutex,
        thread,
    };

    use super::*;

    /// Mock HTTP server for testing network connectivity checks.
    struct MockHttpServer {
        listener: TcpListener,
        port: u16,
        response_status: u16,
        response_delay_ms: u64,
    }

    impl MockHttpServer {
        fn new(status: u16, delay_ms: u64) -> Self {
            // Bind to random available port
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();

            MockHttpServer {
                listener,
                port,
                response_status: status,
                response_delay_ms: delay_ms,
            }
        }

        fn spawn(self) -> thread::JoinHandle<()> {
            thread::spawn(move || {
                for stream in self.listener.incoming() {
                    if let Ok(mut stream) = stream {
                        // Simulate network delay
                        if self.response_delay_ms > 0 {
                            thread::sleep(Duration::from_millis(self.response_delay_ms));
                        }

                        // Read request
                        let mut reader = BufReader::new(&stream);
                        let mut _request_line = String::new();
                        let _ = reader.read_line(&mut _request_line);

                        // Send HTTP response
                        let response = match self.response_status {
                            204 => "HTTP/1.1 204 No Content\r\n\r\n",
                            200 => "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK",
                            404 => "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found",
                            500 => {
                                "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 5\r\n\r\nError"
                            },
                            _ => "HTTP/1.1 503 Service Unavailable\r\n\r\n",
                        };

                        let _ = stream.write_all(response.as_bytes());
                        break; // Handle only one request per test
                    }
                }
            })
        }
    }

    #[test]
    fn test_network_status_enum_variants() {
        let good = NetworkStatus::Good;
        let slow = NetworkStatus::Slow(Duration::from_secs(3));
        let offline = NetworkStatus::Offline;

        assert_eq!(format!("{:?}", good), "Good");
        assert!(format!("{:?}", slow).contains("Slow"));
        assert_eq!(format!("{:?}", offline), "Offline");
    }

    #[test]
    fn test_network_status_slow_contains_duration() {
        // Arrange
        let test_duration = Duration::from_millis(2500);

        // Act
        let status = NetworkStatus::Slow(test_duration);

        // Assert
        match status {
            NetworkStatus::Slow(d) => assert_eq!(d, test_duration),
            _ => panic!("Expected Slow variant"),
        }
    }

    #[test]
    #[ignore = "Requires internet connection"]
    fn test_real_network_connectivity_check() {
        // Act
        let status = check_network_status();

        // Assert - should be either Good or Slow (not Offline) if internet is available
        match status {
            NetworkStatus::Good | NetworkStatus::Slow(_) => {},
            NetworkStatus::Offline => {
                panic!("Expected network connectivity for this test");
            },
        }
    }

    // Test with localhost mock server
    #[test]
    fn test_http_status_code_handling() {
        // Arrange
        let server = MockHttpServer::new(200, 0);
        let port = server.port;
        let _handle = server.spawn();

        // Act
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(1)))
            .build();
        let agent: ureq::Agent = config.into();

        let url = format!("http://127.0.0.1:{}", port);
        let response = agent.get(&url).call();

        // Assert
        assert!(response.is_ok());
        assert_eq!(response.unwrap().status(), 200);
    }

    #[test]
    fn test_http_no_content_status_handling() {
        // Arrange
        let server = MockHttpServer::new(204, 0);
        let port = server.port;
        let _handle = server.spawn();

        // Act
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(1)))
            .build();
        let agent: ureq::Agent = config.into();

        let url = format!("http://127.0.0.1:{}", port);
        let response = agent.get(&url).call();

        // Assert
        assert!(response.is_ok());
        assert_eq!(response.unwrap().status(), 204);
    }

    #[test]
    fn test_timeout_behavior() {
        // Arrange - Create slow mock server that delays response
        let server = MockHttpServer::new(200, 2000); // 2 second delay
        let port = server.port;
        let _handle = server.spawn();

        // Act
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(1)))
            .build();
        let agent: ureq::Agent = config.into();

        let url = format!("http://127.0.0.1:{}", port);
        let start = std::time::Instant::now();
        let response = agent.get(&url).call();
        let elapsed = start.elapsed();

        // Assert
        assert!(response.is_err());
        assert!(elapsed.as_secs() <= 2); // Should timeout at ~1 second, not wait full 2
    }

    #[test]
    fn test_latency_measurement_accuracy() {
        // Arrange
        let start = std::time::Instant::now();

        // Act
        thread::sleep(Duration::from_millis(100));
        let elapsed = start.elapsed();

        // Assert
        assert!(elapsed.as_millis() >= 100); // Elapsed time should be at least 100ms
        assert!(elapsed.as_millis() < 200); // But not too much more
    }

    #[test]
    fn test_concurrent_status_checks_safety() {
        // Arrange - Shared counter for thread safety verification
        let counter = std::sync::Arc::new(Mutex::new(0));
        let mut handles = vec![];

        // Act - Spawn multiple threads checking network status
        for _ in 0..3 {
            let counter_clone = counter.clone();
            let handle = thread::spawn(move || {
                let _status = check_network_status();
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Assert
        assert_eq!(*counter.lock().unwrap(), 3);
    }

    #[test]
    fn test_error_status_codes_not_accepted() {
        // Arrange
        let server = MockHttpServer::new(500, 0);
        let port = server.port;
        let _handle = server.spawn();

        // Act
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(1)))
            .build();
        let agent: ureq::Agent = config.into();

        let url = format!("http://127.0.0.1:{}", port);
        let response = agent.get(&url).call();

        // Assert
        if let Ok(resp) = response {
            assert_ne!(resp.status(), HTTP_OK);
            assert_ne!(resp.status(), HTTP_NO_CONTENT);
        }
    }
}
