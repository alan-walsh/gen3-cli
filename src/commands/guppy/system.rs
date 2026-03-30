use crate::config::Config;
use anyhow::{Context, Result};

pub async fn status() -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let url = format!("{}/guppy/_status", profile.api_endpoint);
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Guppy")?;

    if response.status().is_success() {
        println!("Guppy is healthy ({}).", response.status());
    } else {
        println!("Guppy is unhealthy ({}).", response.status());
    }
    Ok(())
}

pub async fn version() -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let url = format!("{}/guppy/_version", profile.api_endpoint);
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Guppy")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Guppy request failed ({}): {}", status, text);
    }

    let raw: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse version response")?;
    println!("{}", serde_json::to_string_pretty(&raw)?);
    Ok(())
}

pub async fn indices() -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let url = format!("{}/guppy/_status", profile.api_endpoint);
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Guppy")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Guppy request failed ({}): {}", status, text);
    }

    let raw: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse status response")?;
    println!("{}", serde_json::to_string_pretty(&raw)?);
    Ok(())
}
