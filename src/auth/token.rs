use crate::config::Profile;
use anyhow::{Context, Result};
use secrecy::ExposeSecret;
use serde::Deserialize;

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

pub async fn get_access_token(client: &reqwest::Client, profile: &Profile) -> Result<String> {
    let url = format!("{}/user/credentials/api/access_token", profile.api_endpoint);
    let body = serde_json::json!({ "api_key": profile.api_key.expose_secret() });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("Failed to connect to Fence")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Token exchange failed ({}): {}", status, text);
    }

    let token_resp: TokenResponse = response
        .json()
        .await
        .context("Failed to parse token response")?;

    Ok(token_resp.access_token)
}
