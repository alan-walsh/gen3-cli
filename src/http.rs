use std::time::Duration;

/// Maximum time to wait for a TCP connection to be established.
const CONNECT_TIMEOUT_SECS: u64 = 10;

/// Maximum total time for a single HTTP request (including body transfer).
const REQUEST_TIMEOUT_SECS: u64 = 60;

/// Creates an HTTP client with connection and request timeouts.
///
/// - connect timeout: [`CONNECT_TIMEOUT_SECS`] seconds
/// - overall request timeout: [`REQUEST_TIMEOUT_SECS`] seconds
pub fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .unwrap_or_else(|e| panic!("Failed to build HTTP client: {e}"))
}
