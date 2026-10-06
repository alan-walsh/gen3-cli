mod logs;
mod system;

use clap::{Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum AuditCategory {
    Login,
    #[value(name = "presigned_url")]
    PresignedUrl,
}

impl AuditCategory {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::PresignedUrl => "presigned_url",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Jsonl,
    Csv,
}

pub(super) struct QueryOptions {
    category: AuditCategory,
    start: Option<i64>,
    stop: Option<i64>,
    filters: Vec<String>,
    group_by: Vec<String>,
    count: bool,
    all_pages: bool,
    output: OutputFormat,
}

#[derive(Subcommand)]
pub enum AuditServiceResource {
    /// Query login or presigned-URL audit logs
    Logs {
        #[command(subcommand)]
        method: LogsMethod,
    },
    /// Inspect Audit Service health, version, and schema
    System {
        #[command(subcommand)]
        method: SystemMethod,
    },
}

#[derive(Subcommand)]
pub enum LogsMethod {
    /// Query an audit log category
    Query {
        /// Audit category to query
        #[arg(long, value_enum)]
        category: AuditCategory,
        /// Inclusive lower timestamp bound in Unix epoch seconds
        #[arg(long)]
        start: Option<i64>,
        /// Exclusive upper timestamp bound in Unix epoch seconds
        #[arg(long)]
        stop: Option<i64>,
        /// Field filter in FIELD=VALUE form. Repeatable.
        #[arg(long = "filter")]
        filters: Vec<String>,
        /// Group results by a category field. Repeatable.
        #[arg(long = "group-by")]
        group_by: Vec<String>,
        /// Return the number of matching rows instead of rows
        #[arg(long)]
        count: bool,
        /// Follow pagination cursors until all rows have been fetched
        #[arg(long, conflicts_with_all = ["count", "group_by"])]
        all_pages: bool,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        output: OutputFormat,
    },
}

#[derive(Subcommand)]
pub enum SystemMethod {
    /// Check Audit Service and its database connection
    Status,
    /// Get the deployed Audit Service version
    Version,
    /// Get category schema versions and fields
    Schema,
}

pub async fn run(resource: AuditServiceResource) -> anyhow::Result<()> {
    match resource {
        AuditServiceResource::Logs { method } => match method {
            LogsMethod::Query {
                category,
                start,
                stop,
                filters,
                group_by,
                count,
                all_pages,
                output,
            } => {
                logs::query(QueryOptions {
                    category,
                    start,
                    stop,
                    filters,
                    group_by,
                    count,
                    all_pages,
                    output,
                })
                .await
            }
        },
        AuditServiceResource::System { method } => match method {
            SystemMethod::Status => system::status().await,
            SystemMethod::Version => system::version().await,
            SystemMethod::Schema => system::schema().await,
        },
    }
}

pub(super) fn audit_base_url(api_endpoint: &str) -> String {
    let endpoint = api_endpoint.trim_end_matches('/');
    if endpoint.ends_with("/audit") {
        endpoint.to_owned()
    } else {
        format!("{endpoint}/audit")
    }
}

#[cfg(test)]
mod tests {
    use super::audit_base_url;

    #[test]
    fn audit_base_url_adds_prefix_once() {
        assert_eq!(
            audit_base_url("https://commons.example.org"),
            "https://commons.example.org/audit"
        );
        assert_eq!(
            audit_base_url("https://commons.example.org/audit/"),
            "https://commons.example.org/audit"
        );
    }
}
