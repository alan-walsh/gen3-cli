use super::graphql::run_graphql;
use anyhow::Result;

pub async fn list(type_name: Option<&str>, search: Option<&str>) -> Result<()> {
    // Discover available mapping types
    let types_query = r#"{ __type(name: "Mapping") { fields { name } } }"#;
    let types_result = run_graphql(types_query, None).await?;

    let all_types: Vec<String> = types_result
        .pointer("/data/__type/fields")
        .and_then(|f| f.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|f| f.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let target_types: Vec<&str> = if let Some(t) = type_name {
        if !all_types.iter().any(|s| s == t) {
            anyhow::bail!(
                "Unknown type '{}'. Available types: {}",
                t,
                all_types.join(", ")
            );
        }
        vec![t]
    } else {
        all_types.iter().map(|s| s.as_str()).collect()
    };

    if target_types.is_empty() {
        println!("No index types found.");
        return Ok(());
    }

    let search_clause = search
        .map(|s| format!(r#"(searchInput: "{}")"#, s))
        .unwrap_or_default();

    let type_fields: String = target_types
        .iter()
        .map(|t| format!("  {}{}", t, search_clause))
        .collect::<Vec<_>>()
        .join("\n");

    let query = format!("{{ _mapping {{\n{}\n}} }}", type_fields);
    let result = run_graphql(&query, None).await?;

    if let Some(mapping) = result.pointer("/data/_mapping").and_then(|v| v.as_object()) {
        for t in &target_types {
            if let Some(fields) = mapping.get(*t).and_then(|v| v.as_array()) {
                println!("{}:", t);
                for f in fields {
                    if let Some(name) = f.as_str() {
                        println!("  {}", name);
                    }
                }
                println!();
            }
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}
