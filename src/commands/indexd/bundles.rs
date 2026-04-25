use crate::auth::token::get_access_token;
use crate::config::Config;
use anyhow::{Context, Result};

pub async fn list(limit: Option<u32>) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = crate::http::create_http_client();
    let mut req = client
        .get(format!("{}/bundle", profile.api_endpoint))
        .query(&[("form", "bundle")]);
    if let Some(l) = limit {
        req = req.query(&[("limit", l.to_string())]);
    }

    let response = req.send().await.context("Failed to connect to Indexd")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Indexd request failed ({}): {}", status, text);
    }

    let raw: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse bundle list response")?;
    println!("{}", serde_json::to_string_pretty(&raw)?);
    Ok(())
}

pub async fn get(guid: &str, expand: bool) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = crate::http::create_http_client();
    let mut req = client.get(format!("{}/bundle/{}", profile.api_endpoint, guid));
    if expand {
        req = req.query(&[("expand", "true")]);
    }

    let response = req.send().await.context("Failed to connect to Indexd")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Indexd request failed ({}): {}", status, text);
    }

    let raw: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse bundle response")?;
    println!("{}", serde_json::to_string_pretty(&raw)?);
    Ok(())
}

pub async fn create(bundles: Vec<String>, name: Option<String>) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = crate::http::create_http_client();
    let token = get_access_token(&client, profile).await?;

    let mut body = serde_json::json!({ "bundles": bundles });
    if let Some(n) = name {
        body["name"] = serde_json::Value::String(n);
    }

    let url = format!("{}/bundle", profile.api_endpoint);
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
        .context("Failed to parse create bundle response")?;
    println!("Created bundle:");
    println!("{}", serde_json::to_string_pretty(&raw)?);
    Ok(())
}

pub async fn delete(guid: &str) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = crate::http::create_http_client();
    let token = get_access_token(&client, profile).await?;

    let url = format!("{}/bundle/{}", profile.api_endpoint, guid);
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

    println!("Bundle {} deleted.", guid);
    Ok(())
}
