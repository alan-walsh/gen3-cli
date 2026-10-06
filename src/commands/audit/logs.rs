use super::{audit_base_url, AuditCategory, OutputFormat, QueryOptions};
use crate::auth::token::get_access_token;
use crate::config::{Config, Profile};
use anyhow::{Context, Result};
use reqwest::{Client, StatusCode, Url};
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};

struct QuerySpec<'a> {
    category: AuditCategory,
    start: Option<i64>,
    stop: Option<i64>,
    filters: &'a [String],
    group_by: &'a [String],
    count: bool,
}

pub async fn query(options: QueryOptions) -> Result<()> {
    validate_query_options(&options)?;
    let QueryOptions {
        category,
        start,
        stop,
        filters,
        group_by,
        count,
        all_pages,
        output,
    } = options;
    let config = Config::load().context("Failed to load config")?;
    let profile = config
        .active_profile()
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run `gen3 auth setup` first."))?;
    let client = crate::http::create_http_client();
    let mut token = get_access_token(&client, profile).await?;
    let spec = QuerySpec {
        category,
        start,
        stop,
        filters: &filters,
        group_by: &group_by,
        count,
    };

    let first_page = fetch_page(&client, profile, &spec, None, &mut token).await?;
    if all_pages && output != OutputFormat::Json {
        return stream_pages(&client, profile, &spec, first_page, &mut token, output).await;
    }

    let result = if all_pages {
        fetch_remaining_pages(&client, profile, &spec, first_page, &mut token).await?
    } else {
        first_page
    };

    let rendered = render_output(&result, output)?;
    if !rendered.is_empty() {
        println!("{rendered}");
    }
    Ok(())
}

fn validate_query_options(options: &QueryOptions) -> Result<()> {
    if matches!((options.start, options.stop), (Some(start), Some(stop)) if start > stop) {
        anyhow::bail!("--start must be less than or equal to --stop");
    }
    if options.all_pages && (options.count || !options.group_by.is_empty()) {
        anyhow::bail!("--all-pages cannot be combined with --count or --group-by");
    }
    if options.all_pages && (options.start.is_none() || options.stop.is_none()) {
        anyhow::bail!("--all-pages requires both --start and --stop to bound the audit query");
    }
    Ok(())
}

async fn stream_pages(
    client: &Client,
    profile: &Profile,
    spec: &QuerySpec<'_>,
    first_page: Value,
    token: &mut String,
    output: OutputFormat,
) -> Result<()> {
    let mut page = first_page;
    let mut seen = HashSet::new();
    let mut csv_columns = None;
    let mut wrote_output = false;

    loop {
        let chunk = match output {
            OutputFormat::Jsonl => render_jsonl(&page)?,
            OutputFormat::Csv => render_csv_page(&page, &mut csv_columns)?,
            OutputFormat::Json => unreachable!("JSON pagination is collected before rendering"),
        };
        if !chunk.is_empty() {
            if wrote_output {
                println!();
            }
            print!("{chunk}");
            wrote_output = true;
        }

        let Some(timestamp) = page_cursor(&page)? else {
            break;
        };
        if !seen.insert(timestamp) {
            anyhow::bail!(
                "Audit Service returned the pagination cursor {timestamp} more than once"
            );
        }
        page = fetch_page(client, profile, spec, Some(timestamp), token).await?;
    }

    if wrote_output {
        println!();
    }
    Ok(())
}

async fn fetch_remaining_pages(
    client: &Client,
    profile: &Profile,
    spec: &QuerySpec<'_>,
    first_page: Value,
    token: &mut String,
) -> Result<Value> {
    let mut data = Vec::new();
    append_page_data(&mut data, &first_page)?;

    let mut cursor = page_cursor(&first_page)?;
    let mut seen = HashSet::new();
    while let Some(timestamp) = cursor {
        if !seen.insert(timestamp) {
            anyhow::bail!(
                "Audit Service returned the pagination cursor {timestamp} more than once"
            );
        }
        let page = fetch_page(client, profile, spec, Some(timestamp), token).await?;
        append_page_data(&mut data, &page)?;
        cursor = page_cursor(&page)?;
    }

    Ok(serde_json::json!({
        "nextTimeStamp": null,
        "data": data,
    }))
}

async fn fetch_page(
    client: &Client,
    profile: &Profile,
    spec: &QuerySpec<'_>,
    cursor: Option<i64>,
    token: &mut String,
) -> Result<Value> {
    for attempt in 0..2 {
        let url = query_url(&profile.api_endpoint, spec, cursor)?;
        let request = build_query_request(client, url, token)?;
        let response = client
            .execute(request)
            .await
            .context("Failed to connect to Audit Service")?;

        if response.status() == StatusCode::UNAUTHORIZED && attempt == 0 {
            *token = get_access_token(client, profile).await?;
            continue;
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            if status == StatusCode::FORBIDDEN {
                anyhow::bail!(
                    "Audit Service request failed ({}): read access to \
                     /services/audit/{} is required. {}",
                    status,
                    spec.category.as_str(),
                    text
                );
            }
            anyhow::bail!("Audit Service request failed ({}): {}", status, text);
        }

        return response
            .json()
            .await
            .context("Failed to parse Audit Service query response");
    }

    unreachable!("query attempts always return or fail")
}

fn query_url(api_endpoint: &str, spec: &QuerySpec<'_>, cursor: Option<i64>) -> Result<Url> {
    let base = audit_base_url(api_endpoint);
    let mut url = Url::parse(&format!("{base}/log/{}", spec.category.as_str()))
        .context("Failed to build Audit Service query URL")?;

    {
        let mut pairs = url.query_pairs_mut();
        if let Some(start) = cursor.or(spec.start) {
            pairs.append_pair("start", &start.to_string());
        }
        if let Some(stop) = spec.stop {
            pairs.append_pair("stop", &stop.to_string());
        }
        for filter in spec.filters {
            let (field, value) = filter
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("--filter must use FIELD=VALUE: {filter}"))?;
            if field.is_empty() {
                anyhow::bail!("--filter field must not be empty: {filter}");
            }
            pairs.append_pair(field, value);
        }
        for field in spec.group_by {
            if field.is_empty() {
                anyhow::bail!("--group-by field must not be empty");
            }
            pairs.append_pair("groupby", field);
        }
        if spec.count {
            pairs.append_pair("count", "");
        }
    }

    Ok(url)
}

fn build_query_request(client: &Client, url: Url, token: &str) -> Result<reqwest::Request> {
    client
        .get(url)
        .bearer_auth(token)
        .build()
        .context("Failed to build Audit Service query request")
}

fn page_cursor(response: &Value) -> Result<Option<i64>> {
    match response.get("nextTimeStamp") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(number)) => number
            .as_i64()
            .map(Some)
            .context("Audit Service returned an invalid nextTimeStamp"),
        Some(_) => anyhow::bail!("Audit Service returned a non-numeric nextTimeStamp"),
    }
}

fn append_page_data(target: &mut Vec<Value>, response: &Value) -> Result<()> {
    let rows = response
        .get("data")
        .and_then(Value::as_array)
        .context("Audit Service returned non-array data for a paginated query")?;
    target.extend(rows.iter().cloned());
    Ok(())
}

fn render_output(response: &Value, output: OutputFormat) -> Result<String> {
    match output {
        OutputFormat::Json => {
            serde_json::to_string_pretty(response).context("Failed to serialize JSON output")
        }
        OutputFormat::Jsonl => render_jsonl(response),
        OutputFormat::Csv => render_csv(response),
    }
}

fn render_jsonl(response: &Value) -> Result<String> {
    let data = response
        .get("data")
        .context("Audit Service response is missing data")?;
    match data {
        Value::Array(rows) => rows
            .iter()
            .map(serde_json::to_string)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map(|lines| lines.join("\n"))
            .context("Failed to serialize JSONL output"),
        value => serde_json::to_string(value).context("Failed to serialize JSONL output"),
    }
}

fn render_csv(response: &Value) -> Result<String> {
    let mut columns = None;
    render_csv_page(response, &mut columns)
}

fn render_csv_page(response: &Value, columns: &mut Option<Vec<String>>) -> Result<String> {
    let data = response
        .get("data")
        .context("Audit Service response is missing data")?;

    if !data.is_array() {
        return Ok(format!("count\n{}", csv_cell(data)));
    }

    let rows = data.as_array().expect("array checked above");
    if rows.is_empty() {
        return Ok(String::new());
    }

    let include_header = columns.is_none();
    if include_header {
        let mut discovered = BTreeSet::new();
        for row in rows {
            let object = row
                .as_object()
                .context("CSV output requires audit rows to be JSON objects")?;
            discovered.extend(object.keys().cloned());
        }
        *columns = Some(discovered.into_iter().collect());
    }
    let columns = columns.as_ref().expect("columns initialized above");

    let mut lines = Vec::with_capacity(rows.len() + usize::from(include_header));
    if include_header {
        lines.push(
            columns
                .iter()
                .map(|name| csv_escape(name))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    for row in rows {
        let object = row
            .as_object()
            .context("CSV output requires audit rows to be JSON objects")?;
        if object.keys().any(|key| !columns.contains(key)) {
            anyhow::bail!("Audit Service returned inconsistent fields across CSV pages");
        }
        lines.push(
            columns
                .iter()
                .map(|column| csv_cell(object.get(column).unwrap_or(&Value::Null)))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    Ok(lines.join("\n"))
}

fn csv_cell(value: &Value) -> String {
    let raw = match value {
        Value::Null => String::new(),
        Value::String(value) => spreadsheet_safe(value),
        value => serde_json::to_string(value).unwrap_or_default(),
    };
    csv_escape(&raw)
}

fn spreadsheet_safe(value: &str) -> String {
    if value.starts_with(['=', '+', '-', '@']) {
        format!("'{value}")
    } else {
        value.to_owned()
    }
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::AUTHORIZATION;
    use secrecy::SecretString;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn spec<'a>(filters: &'a [String], group_by: &'a [String], count: bool) -> QuerySpec<'a> {
        QuerySpec {
            category: AuditCategory::PresignedUrl,
            start: Some(10),
            stop: Some(20),
            filters,
            group_by,
            count,
        }
    }

    fn query_options() -> QueryOptions {
        QueryOptions {
            category: AuditCategory::PresignedUrl,
            start: Some(10),
            stop: Some(20),
            filters: Vec::new(),
            group_by: Vec::new(),
            count: false,
            all_pages: false,
            output: OutputFormat::Json,
        }
    }

    async fn spawn_http_sequence(
        responses: Vec<(u16, &'static str)>,
    ) -> (String, tokio::task::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            let mut requests = Vec::new();
            for (status, body) in responses {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                let mut buffer = [0_u8; 4096];
                loop {
                    let read = socket.read(&mut buffer).await.unwrap();
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..read]);
                    if request_is_complete(&request) {
                        break;
                    }
                }
                requests.push(String::from_utf8(request).unwrap());

                let reason = if status == 200 { "OK" } else { "Unauthorized" };
                let response = format!(
                    "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                socket.write_all(response.as_bytes()).await.unwrap();
            }
            requests
        });
        (format!("http://{address}"), handle)
    }

    fn request_is_complete(request: &[u8]) -> bool {
        let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
            return false;
        };
        let headers = String::from_utf8_lossy(&request[..header_end]);
        let content_length = headers
            .lines()
            .filter_map(|line| line.split_once(':'))
            .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
            .and_then(|(_, value)| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        request.len() >= header_end + 4 + content_length
    }

    fn mock_profile(api_endpoint: String) -> Profile {
        Profile {
            api_endpoint,
            api_key: SecretString::from("api-key".to_string()),
            key_id: "key-id".to_string(),
        }
    }

    #[test]
    fn all_pages_requires_a_bounded_time_window() {
        let mut options = query_options();
        options.all_pages = true;
        options.stop = None;

        let error = validate_query_options(&options).unwrap_err();
        assert!(error
            .to_string()
            .contains("requires both --start and --stop"));
    }

    #[test]
    fn query_url_preserves_repeated_filters_and_grouping() {
        let filters = vec![
            "guid=dg.XXXX/one".to_string(),
            "guid=dg.XXXX/two".to_string(),
            "status_code=200".to_string(),
        ];
        let group_by = vec!["protocol".to_string()];
        let url = query_url(
            "https://commons.example.org",
            &spec(&filters, &group_by, true),
            Some(15),
        )
        .unwrap();
        let pairs: Vec<(String, String)> = url.query_pairs().into_owned().collect();

        assert_eq!(url.path(), "/audit/log/presigned_url");
        assert_eq!(
            pairs,
            vec![
                ("start".into(), "15".into()),
                ("stop".into(), "20".into()),
                ("guid".into(), "dg.XXXX/one".into()),
                ("guid".into(), "dg.XXXX/two".into()),
                ("status_code".into(), "200".into()),
                ("groupby".into(), "protocol".into()),
                ("count".into(), String::new()),
            ]
        );
    }

    #[test]
    fn query_url_rejects_malformed_filter() {
        let filters = vec!["guid".to_string()];
        let error = query_url(
            "https://commons.example.org",
            &spec(&filters, &[], false),
            None,
        )
        .unwrap_err();
        assert!(error.to_string().contains("FIELD=VALUE"));
    }

    #[test]
    fn query_request_adds_bearer_token() {
        let client = crate::http::create_http_client();
        let url = Url::parse("https://commons.example.org/audit/log/login").unwrap();
        let request = build_query_request(&client, url, "test-token").unwrap();

        assert_eq!(
            request.headers().get(AUTHORIZATION).unwrap(),
            "Bearer test-token"
        );
    }

    #[tokio::test]
    async fn unauthorized_query_refreshes_token_once() {
        let (endpoint, server) = spawn_http_sequence(vec![
            (401, ""),
            (200, r#"{"access_token":"new-token"}"#),
            (200, r#"{"nextTimeStamp":null,"data":[]}"#),
        ])
        .await;
        let profile = mock_profile(endpoint);
        let client = crate::http::create_http_client();
        let mut token = "old-token".to_string();

        let response = fetch_page(&client, &profile, &spec(&[], &[], false), None, &mut token)
            .await
            .unwrap();
        let requests = server.await.unwrap();

        assert_eq!(response["data"], serde_json::json!([]));
        assert_eq!(token, "new-token");
        assert!(requests[0].contains("authorization: Bearer old-token"));
        assert!(requests[1].starts_with("POST /user/credentials/api/access_token "));
        assert!(requests[2].contains("authorization: Bearer new-token"));
    }

    #[tokio::test]
    async fn second_unauthorized_response_is_not_retried() {
        let (endpoint, server) = spawn_http_sequence(vec![
            (401, ""),
            (200, r#"{"access_token":"new-token"}"#),
            (401, ""),
        ])
        .await;
        let profile = mock_profile(endpoint);
        let client = crate::http::create_http_client();
        let mut token = "old-token".to_string();

        let error = fetch_page(&client, &profile, &spec(&[], &[], false), None, &mut token)
            .await
            .unwrap_err();
        let requests = server.await.unwrap();

        assert!(error.to_string().contains("401"));
        assert_eq!(requests.len(), 3);
        assert_eq!(
            requests
                .iter()
                .filter(|request| request.starts_with("POST "))
                .count(),
            1
        );
    }

    #[test]
    fn pagination_cursor_and_rows_are_combined() {
        let first = serde_json::json!({
            "nextTimeStamp": 20,
            "data": [{"id": 1}]
        });
        let second = serde_json::json!({
            "nextTimeStamp": null,
            "data": [{"id": 2}]
        });
        let mut rows = Vec::new();

        append_page_data(&mut rows, &first).unwrap();
        append_page_data(&mut rows, &second).unwrap();

        assert_eq!(page_cursor(&first).unwrap(), Some(20));
        assert_eq!(page_cursor(&second).unwrap(), None);
        assert_eq!(
            rows,
            vec![serde_json::json!({"id": 1}), serde_json::json!({"id": 2})]
        );
    }

    #[test]
    fn output_renderers_handle_rows_and_counts() {
        let rows = serde_json::json!({
            "nextTimeStamp": null,
            "data": [
                {"action": "download", "guid": "a,b"},
                {"action": "upload", "guid": "c"}
            ]
        });
        let count = serde_json::json!({"nextTimeStamp": null, "data": 2});

        assert_eq!(
            render_jsonl(&rows).unwrap(),
            "{\"action\":\"download\",\"guid\":\"a,b\"}\n{\"action\":\"upload\",\"guid\":\"c\"}"
        );
        assert_eq!(
            render_csv(&rows).unwrap(),
            "action,guid\ndownload,\"a,b\"\nupload,c"
        );
        assert_eq!(render_csv(&count).unwrap(), "count\n2");
    }

    #[test]
    fn csv_output_neutralizes_formulas_and_streams_one_header() {
        let first = serde_json::json!({
            "data": [{"username": "=HYPERLINK(\"https://example.org\")"}]
        });
        let second = serde_json::json!({
            "data": [
                {"username": "+cmd"},
                {"username": "-1+1"},
                {"username": "@SUM(A1:A2)"}
            ]
        });
        let mut columns = None;

        assert_eq!(
            render_csv_page(&first, &mut columns).unwrap(),
            "username\n\"'=HYPERLINK(\"\"https://example.org\"\")\""
        );
        assert_eq!(
            render_csv_page(&second, &mut columns).unwrap(),
            "'+cmd\n'-1+1\n'@SUM(A1:A2)"
        );
    }
}
