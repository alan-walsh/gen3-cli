use super::graphql::run_graphql;
use anyhow::{Context, Result};

/// Discover the index types available under the Aggregation type.
async fn available_types() -> Result<Vec<String>> {
    let q = r#"{ __type(name: "Aggregation") { fields { name } } }"#;
    let result = run_graphql(q, None).await?;
    let types = result
        .pointer("/data/__type/fields")
        .and_then(|f| f.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|f| f.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();
    Ok(types)
}

pub async fn counts(
    type_name: Option<&str>,
    filter: Option<&str>,
    accessibility: &str,
) -> Result<()> {
    let all_types = available_types().await?;

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

    // Use variables to pass the filter safely as a JSON scalar.
    let (filter_arg, vars) = if let Some(f) = filter {
        let parsed: serde_json::Value =
            serde_json::from_str(f).context("--filter must be valid JSON")?;
        (
            ", filter: $filter".to_string(),
            Some(serde_json::json!({ "filter": parsed })),
        )
    } else {
        (String::new(), None)
    };

    let type_fields: String = target_types
        .iter()
        .map(|t| {
            format!(
                "  {}(accessibility: {}{}) {{ _totalCount }}",
                t, accessibility, filter_arg
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let query = if vars.is_some() {
        format!(
            "query($filter: JSON) {{ _aggregation {{\n{}\n}} }}",
            type_fields
        )
    } else {
        format!("{{ _aggregation {{\n{}\n}} }}", type_fields)
    };

    let result = run_graphql(&query, vars).await?;

    if let Some(agg) = result.pointer("/data/_aggregation").and_then(|v| v.as_object()) {
        let mut total_width = 0usize;
        let rows: Vec<(String, i64)> = target_types
            .iter()
            .map(|t| {
                let count = agg
                    .get(*t)
                    .and_then(|v| v.get("_totalCount"))
                    .and_then(|c| c.as_i64())
                    .unwrap_or(0);
                total_width = total_width.max(t.len());
                (t.to_string(), count)
            })
            .collect();
        for (t, count) in &rows {
            println!("{:<width$}  {}", t, count, width = total_width);
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

pub async fn histogram(
    type_name: &str,
    field: &str,
    filter: Option<&str>,
    accessibility: &str,
) -> Result<()> {
    // We use asTextHistogram which is defined on both HistogramForString and
    // HistogramForNumber, so this works regardless of the field's ES type.
    let (filter_arg, vars) = if let Some(f) = filter {
        let parsed: serde_json::Value =
            serde_json::from_str(f).context("--filter must be valid JSON")?;
        (
            ", filter: $filter".to_string(),
            Some(serde_json::json!({ "filter": parsed })),
        )
    } else {
        (String::new(), None)
    };

    let query = format!(
        r#"{query_decl} {{
  _aggregation {{
    {type_name}(accessibility: {accessibility}{filter_arg}) {{
      _totalCount
      {field} {{
        _totalCount
        _cardinalityCount
        asTextHistogram {{ key count }}
      }}
    }}
  }}
}}"#,
        query_decl = if vars.is_some() { "query($filter: JSON)" } else { "" },
        type_name = type_name,
        accessibility = accessibility,
        filter_arg = filter_arg,
        field = field,
    );

    let result = run_graphql(&query, vars).await?;

    // Check for GraphQL errors (e.g. unknown field name)
    if let Some(errors) = result.get("errors") {
        if !errors.as_array().map(|a| a.is_empty()).unwrap_or(true) {
            anyhow::bail!("GraphQL error: {}", serde_json::to_string_pretty(errors)?);
        }
    }

    let agg_path = format!("/data/_aggregation/{}", type_name);
    if let Some(type_agg) = result.pointer(&agg_path) {
        let total = type_agg
            .get("_totalCount")
            .and_then(|c| c.as_i64())
            .unwrap_or(0);
        let field_agg = type_agg.get(field);
        let field_total = field_agg
            .and_then(|f| f.get("_totalCount"))
            .and_then(|c| c.as_i64())
            .unwrap_or(0);
        let cardinality = field_agg
            .and_then(|f| f.get("_cardinalityCount"))
            .and_then(|c| c.as_i64())
            .unwrap_or(0);

        println!(
            "{}.{} — {} records, {} with values, {} unique values",
            type_name, field, total, field_total, cardinality
        );

        if let Some(buckets) = field_agg
            .and_then(|f| f.get("asTextHistogram"))
            .and_then(|h| h.as_array())
        {
            if buckets.is_empty() {
                println!("  (no data)");
            } else {
                let max_count = buckets
                    .iter()
                    .filter_map(|b| b.get("count").and_then(|c| c.as_i64()))
                    .max()
                    .unwrap_or(1);
                let bar_width = 30usize;
                let key_width = buckets
                    .iter()
                    .filter_map(|b| b.get("key").and_then(|k| k.as_str()).map(|k| k.len()))
                    .max()
                    .unwrap_or(10)
                    .max(10);

                println!();
                for bucket in buckets {
                    let key = bucket
                        .get("key")
                        .and_then(|k| k.as_str())
                        .unwrap_or("(null)");
                    let count = bucket
                        .get("count")
                        .and_then(|c| c.as_i64())
                        .unwrap_or(0);
                    let pct = if field_total > 0 {
                        count as f64 / field_total as f64 * 100.0
                    } else {
                        0.0
                    };
                    let bar_len = if max_count > 0 {
                        (count as f64 / max_count as f64 * bar_width as f64) as usize
                    } else {
                        0
                    };
                    let bar: String = "█".repeat(bar_len);
                    println!(
                        "  {:<key_width$}  {:>8}  {:>5.1}%  {}",
                        key,
                        count,
                        pct,
                        bar,
                        key_width = key_width
                    );
                }
            }
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}
