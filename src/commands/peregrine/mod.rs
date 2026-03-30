mod datasets;
mod graphql;
mod metadata;
mod system;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum PeregrineResource {
    /// GraphQL queries and schema introspection
    Graphql {
        #[command(subcommand)]
        method: GraphqlMethod,
    },
    /// Project listing and per-project node counts
    Datasets {
        #[command(subcommand)]
        method: DatasetsMethod,
    },
    /// Core metadata lookup for a data object
    Metadata {
        #[command(subcommand)]
        method: MetadataMethod,
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
        /// GraphQL query string (e.g. '{ project { project_id } }')
        #[arg(long)]
        query: String,
        /// JSON-encoded variables object
        #[arg(long)]
        vars: Option<String>,
        /// GraphQL operation name
        #[arg(long)]
        operation_name: Option<String>,
    },
    /// Run a GraphQL introspection query to discover all available types and fields
    Introspect,
    /// Get the full data dictionary schema (JSON)
    Schema,
}

#[derive(Subcommand)]
pub enum DatasetsMethod {
    /// List all projects with high-level information
    List,
    /// Get node record counts for each project
    Counts {
        /// Comma-delimited node types to count (e.g. case,aliquot). Omit for all.
        #[arg(long)]
        nodes: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum MetadataMethod {
    /// Get core metadata for a data object by its ID
    Get {
        /// The object ID (GUID) to look up
        #[arg(long)]
        id: String,
        /// Response format: json (default), schema-org, bibtex
        #[arg(long, default_value = "json")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum SystemMethod {
    /// Check if Peregrine is healthy
    Status,
    /// Get the Peregrine version and data dictionary version
    Version,
}

pub async fn run(resource: PeregrineResource) -> anyhow::Result<()> {
    match resource {
        PeregrineResource::Graphql { method } => match method {
            GraphqlMethod::Query {
                query,
                vars,
                operation_name,
            } => graphql::query(&query, vars.as_deref(), operation_name.as_deref()).await,
            GraphqlMethod::Introspect => graphql::introspect().await,
            GraphqlMethod::Schema => graphql::schema().await,
        },
        PeregrineResource::Datasets { method } => match method {
            DatasetsMethod::List => datasets::list().await,
            DatasetsMethod::Counts { nodes } => datasets::counts(nodes.as_deref()).await,
        },
        PeregrineResource::Metadata { method } => match method {
            MetadataMethod::Get { id, format } => metadata::get(&id, &format).await,
        },
        PeregrineResource::System { method } => match method {
            SystemMethod::Status => system::status().await,
            SystemMethod::Version => system::version().await,
        },
    }
}
