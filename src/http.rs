use std::time::Duration;

/// Creates an HTTP client with connection and read timeouts.
///
/// - connect timeout: 10 seconds
/// - overall request timeout: 60 seconds
pub fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(60))
        .build()
        .expect("Failed to build HTTP client")
}
