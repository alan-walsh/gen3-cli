use crate::auth::token::get_access_token;
use crate::config::Config;
use anyhow::{Context, Result};

pub async fn records(
    type_name: &str,
    fields: Option<&str>,
    filter: Option<&str>,
    sort: Option<&str>,
    accessibility: &str,
) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let token = get_access_token(&client, profile).await?;

    let mut body = serde_json::json!({
        "type": type_name,
        "accessibility": accessibility,
    });

    if let Some(f) = fields {
        let field_list: Vec<&str> = f.split(',').map(|s| s.trim()).collect();
        body["fields"] = serde_json::json!(field_list);
    }

    if let Some(f) = filter {
        let parsed: serde_json::Value =
            serde_json::from_str(f).context("--filter must be valid JSON")?;
        body["filter"] = parsed;
    }

    if let Some(s) = sort {
        let parsed: serde_json::Value =
            serde_json::from_str(s).context("--sort must be valid JSON")?;
        body["sort"] = parsed;
    }

    let url = format!("{}/guppy/download", profile.api_endpoint);
    let response = client
        .post(&url)
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .context("Failed to connect to Guppy")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Guppy download failed ({}): {}", status, text);
    }

    let raw: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse download response")?;
    println!("{}", serde_json::to_string_pretty(&raw)?);
    Ok(())
}
