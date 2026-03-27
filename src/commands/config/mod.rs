use clap::Subcommand;
use anyhow::{bail, Result};
use crate::config::Config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Set a configuration value
    Set {
        #[command(subcommand)]
        target: ConfigSetTarget,
    },
}

#[derive(Subcommand)]
pub enum ConfigSetTarget {
    /// Set the active profile
    Profile {
        /// Name of the profile to activate
        account: String,
    },
}

pub fn run(cmd: ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Set { target } => match target {
            ConfigSetTarget::Profile { account } => set_profile(account),
        },
    }
}

fn set_profile(account: String) -> Result<()> {
    let mut config = Config::load()?;

    if config.profiles.is_empty() {
        bail!("no profiles configured. Run `gen3 auth setup` to create one.");
    }

    if !config.profiles.contains_key(&account) {
        let available: Vec<&str> = config.profiles.keys().map(|s| s.as_str()).collect();
        bail!(
            "profile '{}' not found. Available profiles: {}",
            account,
            available.join(", ")
        );
    }

    config.active_profile = Some(account.clone());
    config.save()?;

    println!("Active profile set to '{}'.", account);
    Ok(())
}
