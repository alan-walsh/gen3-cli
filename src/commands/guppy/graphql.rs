use crate::auth::token::get_access_token;
use crate::config::Config;
use anyhow::{Context, Result};

fn graphql_url(api_endpoint: &str) -> String {
    format!("{}/guppy/graphql", api_endpoint)
}

pub(super) async fn run_graphql(
    query_str: &str,
    vars: Option<serde_json::Value>,
) -> Result<serde_json::Value> {
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;

    let client = reqwest::Client::new();
    let token = get_access_token(&client, profile).await?;

    let mut body = serde_json::json!({ "query": query_str });
    if let Some(v) = vars {
        body["variables"] = v;
    }

    let url = graphql_url(&profile.api_endpoint);
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
        anyhow::bail!("Guppy request failed ({}): {}", status, text);
    }

    let result: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse GraphQL response")?;

    if let Some(errors) = result.get("errors") {
        if !errors.as_array().map(|a| a.is_empty()).unwrap_or(true) {
            eprintln!("GraphQL errors: {}", serde_json::to_string_pretty(errors)?);
        }
    }

    Ok(result)
}

pub async fn query(query_str: &str, vars: Option<&str>) -> Result<()> {
    let vars_val = vars
        .map(|v| serde_json::from_str(v).context("--vars must be valid JSON"))
        .transpose()?;
    let result = run_graphql(query_str, vars_val).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn introspect() -> Result<()> {
    let introspection_query = r#"
{
  __schema {
    queryType { name }
    types {
      name
      kind
      fields(includeDeprecated: false) {
        name
        type { name kind ofType { name kind } }
      }
    }
  }
}
"#;

    let result = run_graphql(introspection_query, None).await?;

    if let Some(schema) = result.get("data").and_then(|d| d.get("__schema")) {
        if let Some(query_type) = schema.get("queryType").and_then(|q| q.get("name")) {
            println!("Root query type: {}", query_type);
        }
        if let Some(types) = schema.get("types").and_then(|t| t.as_array()) {
            let skip: &[&str] = &["String", "Int", "Float", "Boolean", "JSON", "__Schema",
                "__Type", "__Field", "__InputValue", "__EnumValue", "__Directive", "__DirectiveLocation"];
            let user_types: Vec<&serde_json::Value> = types
                .iter()
                .filter(|t| {
                    t.get("name")
                        .and_then(|n| n.as_str())
                        .map(|n| !n.starts_with("__") && !skip.contains(&n))
                        .unwrap_or(false)
                })
                .collect();

            println!("\nAvailable types ({}):", user_types.len());
            for t in &user_types {
                let name = t.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                let kind = t.get("kind").and_then(|k| k.as_str()).unwrap_or("?");
                if kind == "OBJECT" {
                    let field_count = t
                        .get("fields")
                        .and_then(|f| f.as_array())
                        .map(|f| f.len())
                        .unwrap_or(0);
                    println!("  {} ({} fields)", name, field_count);
                }
            }
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }
    Ok(())
}
