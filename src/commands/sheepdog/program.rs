use crate::auth::token::get_access_token;
use crate::config::Config;
use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct ProgramsResponse {
    links: Vec<String>,
}

pub async fn list() -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let token = get_access_token(&client, profile).await?;

    let url = format!("{}/api/v0/submission/", profile.api_endpoint);
    let response = client
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await
        .context("Failed to connect to Sheepdog")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Sheepdog request failed ({}): {}", status, text);
    }

    let programs: ProgramsResponse = response
        .json()
        .await
        .context("Failed to parse programs response")?;

    let names: Vec<&str> = programs
        .links
        .iter()
        .filter_map(|l| l.split('/').last())
        .collect();

    if names.is_empty() {
        println!("No programs found.");
    } else {
        println!("Programs: {}", names.join(", "));
    }

    Ok(())
}
