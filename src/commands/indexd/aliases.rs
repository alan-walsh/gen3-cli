use crate::auth::token::get_access_token;
use crate::config::Config;
use anyhow::{Context, Result};

pub async fn list(guid: &str) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let url = format!("{}/index/{}/aliases", profile.api_endpoint, guid);
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
        .context("Failed to parse aliases response")?;

    let aliases = raw
        .get("aliases")
        .and_then(|a| a.as_array())
        .cloned()
        .unwrap_or_default();

    if aliases.is_empty() {
        println!("No aliases for {}.", guid);
    } else {
        println!("Aliases for {} ({} total):", guid, aliases.len());
        for a in &aliases {
            if let Some(s) = a.as_str() {
                println!("  {}", s);
            }
        }
    }
    Ok(())
}

pub async fn add(guid: &str, aliases: Vec<String>) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let token = get_access_token(&client, profile).await?;

    let url = format!("{}/index/{}/aliases", profile.api_endpoint, guid);
    let body = serde_json::json!({ "aliases": aliases });
    let response = client
        .post(&url)
        .bearer_auth(&token)
        .json(&body)
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
        .context("Failed to parse response")?;
    let current = raw
        .get("aliases")
        .and_then(|a| a.as_array())
        .cloned()
        .unwrap_or_default();
    println!("Aliases for {} ({} total):", guid, current.len());
    for a in &current {
        if let Some(s) = a.as_str() {
            println!("  {}", s);
        }
    }
    Ok(())
}

pub async fn replace(guid: &str, aliases: Vec<String>) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let token = get_access_token(&client, profile).await?;

    let url = format!("{}/index/{}/aliases", profile.api_endpoint, guid);
    let body = serde_json::json!({ "aliases": aliases });
    let response = client
        .put(&url)
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .context("Failed to connect to Indexd")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Indexd request failed ({}): {}", status, text);
    }

    println!("Aliases for {} replaced with {} alias(es).", guid, aliases.len());
    Ok(())
}

pub async fn delete_all(guid: &str) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let token = get_access_token(&client, profile).await?;

    let url = format!("{}/index/{}/aliases", profile.api_endpoint, guid);
    let response = client
        .delete(&url)
        .bearer_auth(&token)
        .send()
        .await
        .context("Failed to connect to Indexd")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Indexd request failed ({}): {}", status, text);
    }

    println!("All aliases for {} deleted.", guid);
    Ok(())
}

pub async fn delete_one(guid: &str, alias: &str) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let token = get_access_token(&client, profile).await?;

    let url = format!(
        "{}/index/{}/aliases/{}",
        profile.api_endpoint,
        guid,
        urlencoding::encode(alias)
    );
    let response = client
        .delete(&url)
        .bearer_auth(&token)
        .send()
        .await
        .context("Failed to connect to Indexd")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Indexd request failed ({}): {}", status, text);
    }

    println!("Alias '{}' deleted from {}.", alias, guid);
    Ok(())
}
