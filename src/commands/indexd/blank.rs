use crate::auth::token::get_access_token;
use crate::config::Config;
use anyhow::{Context, Result};
use std::collections::HashMap;

fn parse_hashes(hashes: Vec<String>) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for h in hashes {
        let parts: Vec<&str> = h.splitn(2, ':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Hash must be in format algorithm:value, got: {}", h);
        }
        map.insert(parts[0].to_string(), parts[1].to_string());
    }
    Ok(map)
}

pub async fn create(uploader: Option<String>, authz: Vec<String>) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let token = get_access_token(&client, profile).await?;

    let mut body = serde_json::Map::new();
    if let Some(u) = uploader {
        body.insert("uploader".to_string(), serde_json::json!(u));
    }
    if !authz.is_empty() {
        body.insert("authz".to_string(), serde_json::json!(authz));
    }

    let url = format!("{}/index/blank", profile.api_endpoint);
    let response = client
        .post(&url)
        .bearer_auth(&token)
        .json(&serde_json::Value::Object(body))
        .send()
        .await
        .context("Failed to connect to Indexd")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Indexd request failed ({}): {}", status, text);
    }

    let out: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse create response")?;
    println!("Created blank record:");
    println!("  did:    {}", out["did"].as_str().unwrap_or("?"));
    println!("  rev:    {}", out["rev"].as_str().unwrap_or("?"));
    println!("  baseid: {}", out["baseid"].as_str().unwrap_or("?"));
    Ok(())
}

pub async fn update(
    guid: &str,
    rev: &str,
    hashes: Vec<String>,
    size: u64,
    urls: Vec<String>,
    authz: Vec<String>,
) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let token = get_access_token(&client, profile).await?;
    let hash_map = parse_hashes(hashes)?;

    let mut body = serde_json::json!({
        "hashes": hash_map,
        "size": size,
    });
    if !urls.is_empty() {
        body["urls"] = serde_json::json!(urls);
    }
    if !authz.is_empty() {
        body["authz"] = serde_json::json!(authz);
    }

    let url = format!("{}/index/blank/{}", profile.api_endpoint, guid);
    let response = client
        .put(&url)
        .query(&[("rev", rev)])
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

    let out: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse update response")?;
    println!("Updated blank record:");
    println!("  did:    {}", out["did"].as_str().unwrap_or("?"));
    println!("  rev:    {}", out["rev"].as_str().unwrap_or("?"));
    println!("  baseid: {}", out["baseid"].as_str().unwrap_or("?"));
    Ok(())
}
