use crate::config::Config;
use anyhow::{Context, Result};

pub async fn get(ids: Vec<String>) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let body = serde_json::json!({ "ids": ids });

    let url = format!("{}/bulk/documents", profile.api_endpoint);
    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("Failed to connect to Indexd")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Indexd request failed ({}): {}", status, text);
    }

    let docs: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse bulk response")?;

    if let Some(arr) = docs.as_array() {
        println!("Found {} document(s):", arr.len());
        for doc in arr {
            println!("---");
            println!("{}", serde_json::to_string_pretty(doc)?);
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&docs)?);
    }
    Ok(())
}
