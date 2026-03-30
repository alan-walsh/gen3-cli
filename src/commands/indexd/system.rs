use crate::config::Config;
use anyhow::{Context, Result};

pub async fn status() -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let url = format!("{}/_status", profile.api_endpoint);
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Indexd")?;

    if response.status().is_success() {
        println!("Indexd is healthy ({}).", response.status());
    } else {
        println!("Indexd is unhealthy ({}).", response.status());
    }
    Ok(())
}

pub async fn version() -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let url = format!("{}/_version", profile.api_endpoint);
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Indexd")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Indexd request failed ({}): {}", status, text);
    }

    let raw: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse version response")?;
    println!("{}", serde_json::to_string_pretty(&raw)?);
    Ok(())
}

pub async fn stats() -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let url = format!("{}/_stats", profile.api_endpoint);
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Indexd")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Indexd request failed ({}): {}", status, text);
    }

    let raw: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse stats response")?;
    println!("{}", serde_json::to_string_pretty(&raw)?);
    Ok(())
}
