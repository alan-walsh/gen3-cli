use crate::auth::token::get_access_token;
use crate::config::Config;
use anyhow::{Context, Result};

pub async fn get(id: &str, format: &str) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = crate::http::create_http_client();
    let token = get_access_token(&client, profile).await?;

    let accept = match format {
        "schema-org" => "application/vnd.schemaorg.ld+json",
        "bibtex" => "x-bibtex",
        _ => "application/json",
    };

    let url = format!("{}/api/{}", profile.api_endpoint, id);
    let response = client
        .get(&url)
        .bearer_auth(&token)
        .header("Accept", accept)
        .send()
        .await
        .context("Failed to connect to Peregrine")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Peregrine request failed ({}): {}", status, text);
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if content_type.contains("application/json") || content_type.contains("ld+json") {
        let raw: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse metadata response")?;
        println!("{}", serde_json::to_string_pretty(&raw)?);
    } else {
        let text = response
            .text()
            .await
            .context("Failed to read metadata response")?;
        println!("{}", text);
    }
    Ok(())
}
