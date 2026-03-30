mod aggregation;
mod download;
mod graphql;
mod mapping;
mod system;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum GuppyResource {
    /// Raw GraphQL queries and schema introspection
    Graphql {
        #[command(subcommand)]
        method: GraphqlMethod,
    },
    /// Aggregation counts and field histograms
    Aggregation {
        #[command(subcommand)]
        method: AggregationMethod,
    },
    /// Field mapping discovery
    Mapping {
        #[command(subcommand)]
        method: MappingMethod,
    },
    /// Download raw records from an index
    Download {
        #[command(subcommand)]
        method: DownloadMethod,
    },
    /// System health and version
    System {
        #[command(subcommand)]
        method: SystemMethod,
    },
}

#[derive(Subcommand)]
pub enum GraphqlMethod {
    /// Run a raw GraphQL query
    Query {
        /// GraphQL query string
        #[arg(long)]
        query: String,
        /// JSON-encoded variables object
        #[arg(long)]
        vars: Option<String>,
    },
    /// Introspect the schema to discover available types and fields
    Introspect,
}

#[derive(Subcommand)]
pub enum AggregationMethod {
    /// Get total record counts for each index type (or a specific type)
    Counts {
        /// Index type (e.g. case, follow_up). Omit for all types.
        #[arg(long)]
        r#type: Option<String>,
        /// JSON filter expression
        #[arg(long)]
        filter: Option<String>,
        /// Accessibility: all (default), accessible, unaccessible
        #[arg(long, default_value = "all")]
        accessibility: String,
    },
    /// Get a value histogram for a specific field in an index type
    Histogram {
        /// Index type (e.g. case)
        #[arg(long)]
        r#type: String,
        /// Field name (e.g. gender, age_at_index)
        #[arg(long)]
        field: String,
        /// JSON filter expression
        #[arg(long)]
        filter: Option<String>,
        /// Accessibility: all (default), accessible, unaccessible
        #[arg(long, default_value = "all")]
        accessibility: String,
    },
}

#[derive(Subcommand)]
pub enum MappingMethod {
    /// List fields available in one or all index types
    List {
        /// Index type (e.g. case). Omit to list all types.
        #[arg(long)]
        r#type: Option<String>,
        /// Filter fields by a search string
        #[arg(long)]
        search: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum DownloadMethod {
    /// Download records from an index as JSON
    Records {
        /// Index type (e.g. case, follow_up)
        #[arg(long)]
        r#type: String,
        /// Comma-separated fields to include (omit for all)
        #[arg(long)]
        fields: Option<String>,
        /// JSON filter expression
        #[arg(long)]
        filter: Option<String>,
        /// JSON sort array (e.g. '[{"field":"age_at_index","order":"asc"}]')
        #[arg(long)]
        sort: Option<String>,
        /// Accessibility: accessible (default), all, unaccessible
        #[arg(long, default_value = "accessible")]
        accessibility: String,
    },
}

#[derive(Subcommand)]
pub enum SystemMethod {
    /// Check if Guppy is healthy
    Status,
    /// Get the Guppy version and commit
    Version,
    /// List all configured Elasticsearch indices and their aliases
    Indices,
}

pub async fn run(resource: GuppyResource) -> anyhow::Result<()> {
    match resource {
        GuppyResource::Graphql { method } => match method {
            GraphqlMethod::Query { query, vars } => graphql::query(&query, vars.as_deref()).await,
            GraphqlMethod::Introspect => graphql::introspect().await,
        },
        GuppyResource::Aggregation { method } => match method {
            AggregationMethod::Counts {
                r#type,
                filter,
                accessibility,
            } => aggregation::counts(r#type.as_deref(), filter.as_deref(), &accessibility).await,
            AggregationMethod::Histogram {
                r#type,
                field,
                filter,
                accessibility,
            } => aggregation::histogram(&r#type, &field, filter.as_deref(), &accessibility).await,
        },
        GuppyResource::Mapping { method } => match method {
            MappingMethod::List { r#type, search } => {
                mapping::list(r#type.as_deref(), search.as_deref()).await
            }
        },
        GuppyResource::Download { method } => match method {
            DownloadMethod::Records {
                r#type,
                fields,
                filter,
                sort,
                accessibility,
            } => {
                download::records(
                    &r#type,
                    fields.as_deref(),
                    filter.as_deref(),
                    sort.as_deref(),
                    &accessibility,
                )
                .await
            }
        },
        GuppyResource::System { method } => match method {
            SystemMethod::Status => system::status().await,
            SystemMethod::Version => system::version().await,
            SystemMethod::Indices => system::indices().await,
        },
    }
}
