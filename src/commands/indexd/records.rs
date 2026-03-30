use crate::auth::token::get_access_token;
use crate::config::Config;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct Record {
    did: String,
    rev: String,
    baseid: String,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    file_name: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    uploader: Option<String>,
    #[serde(default)]
    urls: Vec<String>,
    #[serde(default)]
    hashes: HashMap<String, String>,
    #[serde(default)]
    acl: Vec<String>,
    #[serde(default)]
    authz: Vec<String>,
    #[serde(default)]
    created_date: Option<String>,
    #[serde(default)]
    updated_date: Option<String>,
}

#[derive(Deserialize)]
struct OutputRef {
    did: String,
    rev: String,
    baseid: String,
}

#[derive(Deserialize)]
struct ListResponse {
    records: Vec<Record>,
}

fn print_record(r: &Record) {
    println!("did:          {}", r.did);
    println!("rev:          {}", r.rev);
    println!("baseid:       {}", r.baseid);
    if let Some(s) = r.size {
        println!("size:         {}", s);
    }
    if let Some(f) = &r.file_name {
        println!("file_name:    {}", f);
    }
    if let Some(v) = &r.version {
        println!("version:      {}", v);
    }
    if let Some(u) = &r.uploader {
        println!("uploader:     {}", u);
    }
    if !r.urls.is_empty() {
        println!("urls:         {}", r.urls.join(", "));
    }
    if !r.hashes.is_empty() {
        let mut pairs: Vec<String> = r
            .hashes
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect();
        pairs.sort();
        println!("hashes:       {}", pairs.join(", "));
    }
    if !r.acl.is_empty() {
        println!("acl:          {}", r.acl.join(", "));
    }
    if !r.authz.is_empty() {
        println!("authz:        {}", r.authz.join(", "));
    }
    if let Some(c) = &r.created_date {
        println!("created:      {}", c);
    }
    if let Some(u) = &r.updated_date {
        println!("updated:      {}", u);
    }
}

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

pub async fn get(guid: &str, expand: bool) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let mut req = client.get(format!("{}/index/{}", profile.api_endpoint, guid));
    if expand {
        req = req.query(&[("expand", "true")]);
    }

    let response = req.send().await.context("Failed to connect to Indexd")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Indexd request failed ({}): {}", status, text);
    }

    let record: Record = response
        .json()
        .await
        .context("Failed to parse record response")?;
    print_record(&record);
    Ok(())
}

pub async fn list(
    limit: Option<u32>,
    page: Option<u32>,
    hashes: Vec<String>,
    urls: Vec<String>,
    acl: Option<String>,
    authz: Option<String>,
    uploader: Option<String>,
) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let mut req = client.get(format!("{}/index", profile.api_endpoint));

    if let Some(l) = limit {
        req = req.query(&[("limit", l.to_string())]);
    }
    if let Some(p) = page {
        req = req.query(&[("page", p.to_string())]);
    }
    for h in &hashes {
        req = req.query(&[("hash", h)]);
    }
    for u in &urls {
        req = req.query(&[("url", u)]);
    }
    if let Some(a) = &acl {
        req = req.query(&[("acl", a)]);
    }
    if let Some(a) = &authz {
        req = req.query(&[("authz", a)]);
    }
    if let Some(u) = &uploader {
        req = req.query(&[("uploader", u)]);
    }

    let response = req.send().await.context("Failed to connect to Indexd")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Indexd request failed ({}): {}", status, text);
    }

    let list_resp: ListResponse = response
        .json()
        .await
        .context("Failed to parse list response")?;

    if list_resp.records.is_empty() {
        println!("No records found.");
    } else {
        println!("Found {} record(s):", list_resp.records.len());
        for r in &list_resp.records {
            println!("---");
            print_record(r);
        }
    }
    Ok(())
}

pub async fn create(
    hashes: Vec<String>,
    size: u64,
    urls: Vec<String>,
    acl: Vec<String>,
    authz: Vec<String>,
    file_name: Option<String>,
) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let token = get_access_token(&client, profile).await?;
    let hash_map = parse_hashes(hashes)?;

    let mut body = serde_json::json!({
        "form": "object",
        "hashes": hash_map,
        "size": size,
        "urls": urls,
        "acl": acl,
        "authz": authz,
    });
    if let Some(f) = file_name {
        body["file_name"] = serde_json::Value::String(f);
    }

    let url = format!("{}/index", profile.api_endpoint);
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

    let out: OutputRef = response
        .json()
        .await
        .context("Failed to parse create response")?;
    println!("Created record:");
    println!("  did:    {}", out.did);
    println!("  rev:    {}", out.rev);
    println!("  baseid: {}", out.baseid);
    Ok(())
}

pub async fn update(
    guid: &str,
    rev: &str,
    urls: Vec<String>,
    acl: Vec<String>,
    authz: Vec<String>,
    file_name: Option<String>,
    version: Option<String>,
) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let token = get_access_token(&client, profile).await?;

    let mut body = serde_json::Map::new();
    if !urls.is_empty() {
        body.insert("urls".to_string(), serde_json::json!(urls));
    }
    if !acl.is_empty() {
        body.insert("acl".to_string(), serde_json::json!(acl));
    }
    if !authz.is_empty() {
        body.insert("authz".to_string(), serde_json::json!(authz));
    }
    if let Some(f) = file_name {
        body.insert("file_name".to_string(), serde_json::json!(f));
    }
    if let Some(v) = version {
        body.insert("version".to_string(), serde_json::json!(v));
    }

    let url = format!("{}/index/{}", profile.api_endpoint, guid);
    let response = client
        .put(&url)
        .query(&[("rev", rev)])
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

    let out: OutputRef = response
        .json()
        .await
        .context("Failed to parse update response")?;
    println!("Updated record:");
    println!("  did:    {}", out.did);
    println!("  rev:    {}", out.rev);
    println!("  baseid: {}", out.baseid);
    Ok(())
}

pub async fn delete(guid: &str, rev: &str) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let token = get_access_token(&client, profile).await?;

    let url = format!("{}/index/{}", profile.api_endpoint, guid);
    let response = client
        .delete(&url)
        .query(&[("rev", rev)])
        .bearer_auth(&token)
        .send()
        .await
        .context("Failed to connect to Indexd")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Indexd request failed ({}): {}", status, text);
    }

    println!("Deleted record {}.", guid);
    Ok(())
}

pub async fn versions(guid: &str) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let url = format!("{}/index/{}/versions", profile.api_endpoint, guid);
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
        .context("Failed to parse versions response")?;

    // Response may be an array or an object keyed by version number
    let records: Vec<serde_json::Value> = if let Some(arr) = raw.as_array() {
        arr.clone()
    } else if let Some(obj) = raw.as_object() {
        obj.values().cloned().collect()
    } else {
        vec![raw]
    };

    if records.is_empty() {
        println!("No versions found.");
    } else {
        println!("Versions for {} ({} total):", guid, records.len());
        for v in &records {
            println!("---");
            if let Ok(r) = serde_json::from_value::<Record>(v.clone()) {
                print_record(&r);
            } else {
                println!("{}", serde_json::to_string_pretty(v)?);
            }
        }
    }
    Ok(())
}

pub async fn latest(guid: &str, has_version: bool) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let mut req = client.get(format!("{}/index/{}/latest", profile.api_endpoint, guid));
    if has_version {
        req = req.query(&[("has_version", "true")]);
    }

    let response = req.send().await.context("Failed to connect to Indexd")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Indexd request failed ({}): {}", status, text);
    }

    let record: Record = response
        .json()
        .await
        .context("Failed to parse latest response")?;
    println!("Latest version:");
    print_record(&record);
    Ok(())
}
