use crate::auth::token::get_access_token;
use crate::config::Config;
use anyhow::{Context, Result};

fn graphql_url(api_endpoint: &str) -> String {
    format!("{}/api/v0/submission/graphql", api_endpoint)
}

async fn run_graphql(
    query_str: &str,
    vars: Option<&str>,
    operation_name: Option<&str>,
) -> Result<serde_json::Value> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let token = get_access_token(&client, profile).await?;

    let mut body = serde_json::json!({ "query": query_str });
    if let Some(v) = vars {
        let parsed: serde_json::Value =
            serde_json::from_str(v).context("--vars must be valid JSON")?;
        body["variables"] = parsed;
    }
    if let Some(op) = operation_name {
        body["operationName"] = serde_json::Value::String(op.to_string());
    }

    let url = graphql_url(&profile.api_endpoint);
    let response = client
        .post(&url)
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .context("Failed to connect to Peregrine")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Peregrine request failed ({}): {}", status, text);
    }

    let result: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse GraphQL response")?;

    // Surface GraphQL-level errors even on HTTP 200
    if let Some(errors) = result.get("errors") {
        if !errors.as_array().map(|a| a.is_empty()).unwrap_or(true) {
            eprintln!("GraphQL errors: {}", serde_json::to_string_pretty(errors)?);
        }
    }

    Ok(result)
}

pub async fn query(
    query_str: &str,
    vars: Option<&str>,
    operation_name: Option<&str>,
) -> Result<()> {
    let result = run_graphql(query_str, vars, operation_name).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn introspect() -> Result<()> {
    // Compact introspection: list all named types and their fields
    let introspection_query = r#"
{
  __schema {
    queryType { name }
    types {
      name
      kind
      description
      fields(includeDeprecated: false) {
        name
        description
        type {
          name
          kind
          ofType { name kind }
        }
      }
    }
  }
}
"#;

    let result = run_graphql(introspection_query, None, None).await?;

    // Print a readable summary rather than raw JSON
    if let Some(schema) = result
        .get("data")
        .and_then(|d| d.get("__schema"))
    {
        if let Some(query_type) = schema.get("queryType").and_then(|q| q.get("name")) {
            println!("Root query type: {}", query_type);
        }
        if let Some(types) = schema.get("types").and_then(|t| t.as_array()) {
            let user_types: Vec<&serde_json::Value> = types
                .iter()
                .filter(|t| {
                    t.get("name")
                        .and_then(|n| n.as_str())
                        .map(|n| !n.starts_with("__"))
                        .unwrap_or(false)
                })
                .collect();

            println!("\nAvailable types ({} total):", user_types.len());
            for t in &user_types {
                let name = t.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                let kind = t.get("kind").and_then(|k| k.as_str()).unwrap_or("?");
                let desc = t
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");
                if kind == "OBJECT" {
                    let field_count = t
                        .get("fields")
                        .and_then(|f| f.as_array())
                        .map(|f| f.len())
                        .unwrap_or(0);
                    println!(
                        "  {} ({} fields){}",
                        name,
                        field_count,
                        if desc.is_empty() {
                            String::new()
                        } else {
                            format!(" — {}", desc)
                        }
                    );
                }
            }
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }
    Ok(())
}

pub async fn schema() -> Result<()> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let url = format!("{}/api/v0/submission/getschema", profile.api_endpoint);
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Peregrine")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Peregrine request failed ({}): {}", status, text);
    }

    let raw: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse schema response")?;
    println!("{}", serde_json::to_string_pretty(&raw)?);
    Ok(())
}
