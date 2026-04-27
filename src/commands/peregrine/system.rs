use crate::config::Config;
use anyhow::{Context, Result};

pub async fn status() -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = crate::http::create_http_client();
    let url = format!("{}/api/_status", profile.api_endpoint);
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Peregrine")?;

    if response.status().is_success() {
        println!("Peregrine is healthy ({}).", response.status());
    } else {
        println!("Peregrine is unhealthy ({}).", response.status());
    }
    Ok(())
}

pub async fn version() -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = crate::http::create_http_client();
    let url = format!("{}/api/_version", profile.api_endpoint);
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Peregrine")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Peregrine request failed ({}): {}", status, text);
    }

    let raw: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse version response")?;
    println!("{}", serde_json::to_string_pretty(&raw)?);
    Ok(())
}
