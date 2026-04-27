use std::time::Duration;
use reqwest::redirect::Policy;

/// Maximum time to wait for a TCP connection to be established.
const CONNECT_TIMEOUT_SECS: u64 = 10;

/// Maximum total time for a single HTTP request (including body transfer).
const REQUEST_TIMEOUT_SECS: u64 = 60;

/// Creates an HTTP client with connection and request timeouts and redirect following disabled.
///
/// - connect timeout: [`CONNECT_TIMEOUT_SECS`] seconds
/// - overall request timeout: [`REQUEST_TIMEOUT_SECS`] seconds
/// - redirects: disabled — `reqwest` forwards all headers (including `Authorization`) to redirect
///   targets by default; disabling redirect-following prevents access tokens from being leaked
///   to a third-party host if the server issues a cross-domain redirect.
pub fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .redirect(Policy::none())
        .build()
        .unwrap_or_else(|e| panic!("Failed to build HTTP client: {e}"))
}
