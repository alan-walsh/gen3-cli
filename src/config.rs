use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};

/// A named profile storing all credentials and endpoint info for one Gen3 commons.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub api_endpoint: String,
    pub api_key: String,
    pub key_id: String,
}

/// Root config structure stored in ~/.gen3/config (TOML format).
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
    pub active_profile: Option<String>,
}

/// The format of the credentials JSON file downloaded from the Gen3 Fence UI.
#[derive(Debug, Deserialize)]
pub struct CredentialsFile {
    pub api_key: String,
    pub key_id: String,
}

impl Config {
    /// Returns the path to the config directory (~/.gen3/).
    pub fn config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(".gen3"))
    }

    /// Returns the path to the config file (~/.gen3/config).
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config"))
    }

    /// Load the config from disk, or return an empty Config if the file does not exist.
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        Ok(config)
    }

    /// Save the config to disk, creating the ~/.gen3/ directory if necessary.
    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;
        }
        let path = Self::config_path()?;
        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&path, contents)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        Ok(())
    }

    /// Add or overwrite a profile and persist the config.
    pub fn add_profile(&mut self, name: String, profile: Profile) -> Result<()> {
        if self.active_profile.is_none() {
            self.active_profile = Some(name.clone());
        }
        self.profiles.insert(name, profile);
        self.save()
    }

    /// Return the currently active profile, if one is set and exists.
    pub fn active_profile(&self) -> Option<&Profile> {
        self.active_profile
            .as_ref()
            .and_then(|name| self.profiles.get(name))
    }
}

impl CredentialsFile {
    /// Read and parse a Fence API credentials JSON file from the given path.
    pub fn load_from_path(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read credentials file: {path}"))?;
        let creds: CredentialsFile = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse credentials JSON file: {path}"))?;
        Ok(creds)
    }
}
