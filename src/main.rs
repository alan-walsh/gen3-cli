mod config;
mod auth;
mod ui;
mod commands;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "gen3", about = "Gen3 platform CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication and profile management
    Auth {
        #[command(subcommand)]
        subcommand: AuthCommands,
    },
    /// CLI configuration (profiles, settings)
    Config {
        #[command(subcommand)]
        subcommand: commands::ConfigCommands,
    },
    /// Sheepdog data submission operations
    Sheepdog {
        #[command(subcommand)]
        resource: commands::SheepDogResource,
    },
    /// Indexd GUID, record, bundle, and alias operations
    Indexd {
        #[command(subcommand)]
        resource: commands::IndexdResource,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Configure a Gen3 profile (API endpoint and credentials)
    Setup,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            use clap::CommandFactory;
            Cli::command().print_help()?;
            println!();
        }
        Some(Commands::Auth { subcommand }) => match subcommand {
            AuthCommands::Setup => auth::setup::run()?,
        },
        Some(Commands::Config { subcommand }) => {
            commands::config::run(subcommand)?;
        }
        Some(Commands::Sheepdog { resource }) => {
            commands::sheepdog::run(resource).await?;
        }
        Some(Commands::Indexd { resource }) => {
            commands::indexd::run(resource).await?;
        }
    }

    Ok(())
}
