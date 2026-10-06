use super::audit_base_url;
use crate::config::Config;
use anyhow::{Context, Result};
use reqwest::StatusCode;

pub async fn status() -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;
    let client = crate::http::create_http_client();
    let url = format!("{}/_status", audit_base_url(&profile.api_endpoint));
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Audit Service")?;

    if response.status().is_success() {
        println!("Audit Service is healthy ({}).", response.status());
    } else {
        println!("Audit Service is unhealthy ({}).", response.status());
    }
    Ok(())
}

pub async fn version() -> Result<()> {
    print_json_endpoint("_version", "version").await
}

pub async fn schema() -> Result<()> {
    print_json_endpoint("_schema", "schema").await
}

async fn print_json_endpoint(path: &str, label: &str) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;
    let client = crate::http::create_http_client();
    let url = format!("{}/{}", audit_base_url(&profile.api_endpoint), path);
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Audit Service")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        if path == "_schema" && status == StatusCode::NOT_FOUND {
            anyhow::bail!(
                "Audit Service schema endpoint is unavailable (404); \
                 this deployment may use a legacy schema"
            );
        }
        anyhow::bail!(
            "Audit Service {} request failed ({}): {}",
            label,
            status,
            text
        );
    }

    let raw: serde_json::Value = response
        .json()
        .await
        .with_context(|| format!("Failed to parse Audit Service {label} response"))?;
    println!("{}", serde_json::to_string_pretty(&raw)?);
    Ok(())
}
