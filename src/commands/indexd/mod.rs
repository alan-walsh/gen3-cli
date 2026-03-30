mod aliases;
mod blank;
mod bulk;
mod bundles;
mod records;
mod system;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum IndexdResource {
    /// Index record operations (get, list, create, update, delete, versions, latest)
    Records {
        #[command(subcommand)]
        method: RecordsMethod,
    },
    /// Blank record operations — reserve a GUID before upload, then fill it in
    Blank {
        #[command(subcommand)]
        method: BlankMethod,
    },
    /// Alias operations for a GUID
    Aliases {
        #[command(subcommand)]
        method: AliasesMethod,
    },
    /// Bulk document retrieval by list of IDs
    Bulk {
        #[command(subcommand)]
        method: BulkMethod,
    },
    /// Bundle operations
    Bundles {
        #[command(subcommand)]
        method: BundlesMethod,
    },
    /// System health, version, and stats
    System {
        #[command(subcommand)]
        method: SystemMethod,
    },
}

#[derive(Subcommand)]
pub enum RecordsMethod {
    /// Get a record by GUID
    Get {
        /// The GUID of the record
        #[arg(long)]
        guid: String,
        /// Recursively expand bundle contents
        #[arg(long)]
        expand: bool,
    },
    /// List index records with optional filters
    List {
        /// Maximum number of records to return (default 100)
        #[arg(long)]
        limit: Option<u32>,
        /// Page number (0-based)
        #[arg(long)]
        page: Option<u32>,
        /// Filter by hash in format algorithm:value (e.g. md5:abc123). Repeatable.
        #[arg(long = "hash")]
        hashes: Vec<String>,
        /// Filter by storage URL. Repeatable (AND logic).
        #[arg(long = "url")]
        urls: Vec<String>,
        /// Filter by ACL (comma-delimited ACEs)
        #[arg(long)]
        acl: Option<String>,
        /// Filter by authz resource (comma-delimited)
        #[arg(long)]
        authz: Option<String>,
        /// Filter by uploader ID
        #[arg(long)]
        uploader: Option<String>,
    },
    /// Create a new index record
    Create {
        /// Hash in format algorithm:value (e.g. md5:abc123). Repeatable. At least one required.
        #[arg(long = "hash", required = true)]
        hashes: Vec<String>,
        /// File size in bytes
        #[arg(long, required = true)]
        size: u64,
        /// Storage URL. Repeatable.
        #[arg(long = "url")]
        urls: Vec<String>,
        /// ACL entry. Repeatable.
        #[arg(long = "acl")]
        acl: Vec<String>,
        /// Authz resource path. Repeatable.
        #[arg(long = "authz")]
        authz: Vec<String>,
        /// File name
        #[arg(long)]
        file_name: Option<String>,
    },
    /// Update an existing index record
    Update {
        /// The GUID of the record
        #[arg(long)]
        guid: String,
        /// Current revision (required for optimistic locking)
        #[arg(long)]
        rev: String,
        /// Storage URL to set. Repeatable.
        #[arg(long = "url")]
        urls: Vec<String>,
        /// ACL entry to set. Repeatable.
        #[arg(long = "acl")]
        acl: Vec<String>,
        /// Authz resource to set. Repeatable.
        #[arg(long = "authz")]
        authz: Vec<String>,
        /// File name
        #[arg(long)]
        file_name: Option<String>,
        /// Version string
        #[arg(long)]
        version: Option<String>,
    },
    /// Delete an index record
    Delete {
        /// The GUID of the record
        #[arg(long)]
        guid: String,
        /// Current revision (required)
        #[arg(long)]
        rev: String,
    },
    /// List all versions of a record
    Versions {
        /// GUID of any version, or the baseid common to all versions
        #[arg(long)]
        guid: String,
    },
    /// Get the latest version of a record
    Latest {
        /// The GUID of the record
        #[arg(long)]
        guid: String,
        /// Only return the latest record that has a version field set
        #[arg(long)]
        has_version: bool,
    },
}

#[derive(Subcommand)]
pub enum BlankMethod {
    /// Create a blank record to reserve a GUID before upload
    Create {
        /// Uploader identifier
        #[arg(long)]
        uploader: Option<String>,
        /// Authz resource path. Repeatable.
        #[arg(long = "authz")]
        authz: Vec<String>,
    },
    /// Fill in hashes, size, and URLs for a blank record after upload completes
    Update {
        /// The GUID of the blank record
        #[arg(long)]
        guid: String,
        /// Current revision
        #[arg(long)]
        rev: String,
        /// Hash in format algorithm:value (e.g. md5:abc123). Repeatable. At least one required.
        #[arg(long = "hash", required = true)]
        hashes: Vec<String>,
        /// File size in bytes
        #[arg(long, required = true)]
        size: u64,
        /// Storage URL. Repeatable.
        #[arg(long = "url")]
        urls: Vec<String>,
        /// Authz resource. Repeatable.
        #[arg(long = "authz")]
        authz: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum AliasesMethod {
    /// List aliases for a GUID
    List {
        #[arg(long)]
        guid: String,
    },
    /// Append new aliases to a GUID (all must be globally unique)
    Add {
        #[arg(long)]
        guid: String,
        /// Alias to add. Repeatable.
        #[arg(long = "alias", required = true)]
        aliases: Vec<String>,
    },
    /// Replace all aliases for a GUID
    Replace {
        #[arg(long)]
        guid: String,
        /// Alias to set. Repeatable.
        #[arg(long = "alias", required = true)]
        aliases: Vec<String>,
    },
    /// Delete all aliases for a GUID
    DeleteAll {
        #[arg(long)]
        guid: String,
    },
    /// Delete a single alias from a GUID
    Delete {
        #[arg(long)]
        guid: String,
        /// The alias to delete
        #[arg(long)]
        alias: String,
    },
}

#[derive(Subcommand)]
pub enum BulkMethod {
    /// Fetch multiple records by their GUIDs
    Get {
        /// Comma-delimited list of GUIDs
        #[arg(long, value_delimiter = ',', required = true)]
        ids: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum BundlesMethod {
    /// List all bundles
    List {
        /// Maximum number of bundles to return
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Get a bundle by GUID
    Get {
        /// The bundle GUID
        #[arg(long)]
        guid: String,
        /// Recursively expand nested bundles
        #[arg(long)]
        expand: bool,
    },
    /// Create a new bundle
    Create {
        /// GUID of a record or bundle to include. Repeatable. At least one required.
        #[arg(long = "bundle", required = true)]
        bundles: Vec<String>,
        /// Bundle name
        #[arg(long)]
        name: Option<String>,
    },
    /// Delete a bundle record
    Delete {
        /// The bundle GUID
        #[arg(long)]
        guid: String,
    },
}

#[derive(Subcommand)]
pub enum SystemMethod {
    /// Check if Indexd is healthy
    Status,
    /// Get the Indexd version
    Version,
    /// Get basic stats about records in Indexd
    Stats,
}

pub async fn run(resource: IndexdResource) -> anyhow::Result<()> {
    match resource {
        IndexdResource::Records { method } => match method {
            RecordsMethod::Get { guid, expand } => records::get(&guid, expand).await,
            RecordsMethod::List {
                limit,
                page,
                hashes,
                urls,
                acl,
                authz,
                uploader,
            } => records::list(limit, page, hashes, urls, acl, authz, uploader).await,
            RecordsMethod::Create {
                hashes,
                size,
                urls,
                acl,
                authz,
                file_name,
            } => records::create(hashes, size, urls, acl, authz, file_name).await,
            RecordsMethod::Update {
                guid,
                rev,
                urls,
                acl,
                authz,
                file_name,
                version,
            } => records::update(&guid, &rev, urls, acl, authz, file_name, version).await,
            RecordsMethod::Delete { guid, rev } => records::delete(&guid, &rev).await,
            RecordsMethod::Versions { guid } => records::versions(&guid).await,
            RecordsMethod::Latest { guid, has_version } => {
                records::latest(&guid, has_version).await
            }
        },
        IndexdResource::Blank { method } => match method {
            BlankMethod::Create { uploader, authz } => blank::create(uploader, authz).await,
            BlankMethod::Update {
                guid,
                rev,
                hashes,
                size,
                urls,
                authz,
            } => blank::update(&guid, &rev, hashes, size, urls, authz).await,
        },
        IndexdResource::Aliases { method } => match method {
            AliasesMethod::List { guid } => aliases::list(&guid).await,
            AliasesMethod::Add { guid, aliases } => aliases::add(&guid, aliases).await,
            AliasesMethod::Replace { guid, aliases } => aliases::replace(&guid, aliases).await,
            AliasesMethod::DeleteAll { guid } => aliases::delete_all(&guid).await,
            AliasesMethod::Delete { guid, alias } => aliases::delete_one(&guid, &alias).await,
        },
        IndexdResource::Bulk { method } => match method {
            BulkMethod::Get { ids } => bulk::get(ids).await,
        },
        IndexdResource::Bundles { method } => match method {
            BundlesMethod::List { limit } => bundles::list(limit).await,
            BundlesMethod::Get { guid, expand } => bundles::get(&guid, expand).await,
            BundlesMethod::Create { bundles, name } => bundles::create(bundles, name).await,
            BundlesMethod::Delete { guid } => bundles::delete(&guid).await,
        },
        IndexdResource::System { method } => match method {
            SystemMethod::Status => system::status().await,
            SystemMethod::Version => system::version().await,
            SystemMethod::Stats => system::stats().await,
        },
    }
}
