mod program;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum SheepDogResource {
    /// Program operations
    Program {
        #[command(subcommand)]
        method: ProgramMethod,
    },
}

#[derive(Subcommand)]
pub enum ProgramMethod {
    /// List all programs
    List,
}

pub async fn run(resource: SheepDogResource) -> anyhow::Result<()> {
    match resource {
        SheepDogResource::Program { method } => match method {
            ProgramMethod::List => program::list().await,
        },
    }
}
