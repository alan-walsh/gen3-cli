use crate::auth::token::get_access_token;
use crate::config::Config;
use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct Project {
    name: Option<String>,
    code: Option<String>,
    dbgap_accession_number: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct ProjectsResponse {
    projects: Vec<Project>,
}

pub async fn list() -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = crate::http::create_http_client();
    let token = get_access_token(&client, profile).await?;

    let url = format!("{}/api/datasets/projects", profile.api_endpoint);
    let response = client
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await
        .context("Failed to connect to Peregrine")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Peregrine request failed ({}): {}", status, text);
    }

    let resp: ProjectsResponse = response
        .json()
        .await
        .context("Failed to parse projects response")?;

    if resp.projects.is_empty() {
        println!("No projects found.");
    } else {
        println!("Projects ({} total):", resp.projects.len());
        for p in &resp.projects {
            let name = p.name.as_deref().unwrap_or("?");
            let code = p.code.as_deref().unwrap_or("?");
            let accession = p.dbgap_accession_number.as_deref().unwrap_or("—");
            let desc = p.description.as_deref().unwrap_or("");
            print!("  {} (code: {}, dbgap: {})", name, code, accession);
            if !desc.is_empty() {
                print!(" — {}", desc);
            }
            println!();
        }
    }
    Ok(())
}

pub async fn counts(nodes: Option<&str>) -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = crate::http::create_http_client();
    let token = get_access_token(&client, profile).await?;

    let mut req = client
        .get(format!("{}/api/datasets", profile.api_endpoint))
        .bearer_auth(&token);
    if let Some(n) = nodes {
        req = req.query(&[("nodes", n)]);
    }

    let response = req.send().await.context("Failed to connect to Peregrine")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Peregrine request failed ({}): {}", status, text);
    }

    let raw: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse counts response")?;

    if let Some(obj) = raw.as_object() {
        if obj.is_empty() {
            println!("No data found.");
        } else {
            println!("Node counts per project:");
            let mut projects: Vec<(&String, &serde_json::Value)> = obj.iter().collect();
            projects.sort_by_key(|(k, _)| k.as_str());
            for (project, counts) in projects {
                println!("  {}:", project);
                if let Some(count_map) = counts.as_object() {
                    let mut nodes: Vec<(&String, &serde_json::Value)> =
                        count_map.iter().collect();
                    nodes.sort_by_key(|(k, _)| k.as_str());
                    for (node, count) in nodes {
                        println!("    {}: {}", node, count);
                    }
                }
            }
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&raw)?);
    }
    Ok(())
}
