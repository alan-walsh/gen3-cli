use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use url::{Host, Url};
use secrecy::{ExposeSecret, SecretString};

fn default_secret_string() -> SecretString {
    SecretString::from(String::new())
}

/// A named profile storing all credentials and endpoint info for one Gen3 commons.
#[derive(Clone, Serialize, Deserialize)]
pub struct Profile {
    pub api_endpoint: String,
    /// Held in memory only — never written to the config file.
    /// Stored in and retrieved from the OS keychain.
    /// Wrapped in `SecretString` so the heap bytes are zeroed on drop.
    #[serde(skip_serializing, default = "default_secret_string")]
    pub api_key: SecretString,
    pub key_id: String,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            api_endpoint: String::new(),
            api_key: default_secret_string(),
            key_id: String::new(),
        }
    }
}

impl std::fmt::Debug for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Profile")
            .field("api_endpoint", &self.api_endpoint)
            .field("api_key", &"[REDACTED]")
            .field("key_id", &self.key_id)
            .finish()
    }
}

/// Root config structure stored in ~/.gen3/config (TOML format).
#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
    pub active_profile: Option<String>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("profiles", &self.profiles)
            .field("active_profile", &self.active_profile)
            .finish()
    }
}

/// The format of the credentials JSON file downloaded from the Gen3 Fence UI.
#[derive(Deserialize)]
pub struct CredentialsFile {
    /// Wrapped in `SecretString` so the heap bytes are zeroed on drop.
    pub api_key: SecretString,
    pub key_id: String,
}

impl std::fmt::Debug for CredentialsFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialsFile")
            .field("api_key", &"[REDACTED]")
            .field("key_id", &self.key_id)
            .finish()
    }
}

/// Validates that `endpoint` is a safe, public HTTPS URL.
///
/// Rejects:
/// - Non-HTTPS schemes (prevents credential theft via HTTP downgrade or exotic schemes)
/// - URLs with embedded userinfo (prevents accidentally persisting credentials in config/logs)
/// - Private / loopback / link-local hosts (prevents SSRF against internal services
///   such as cloud metadata endpoints)
pub fn validate_endpoint(endpoint: &str) -> Result<()> {
    let url = Url::parse(endpoint).context("api_endpoint is not a valid URL")?;

    if !url.username().is_empty() || url.password().is_some() {
        anyhow::bail!(
            "api_endpoint must not include embedded credentials in the URL. \
             Provide only the HTTPS endpoint host/path."
        );
    }
    if url.scheme() != "https" {
        anyhow::bail!(
            "api_endpoint must use HTTPS (got '{}'). \
             Plain-text or non-HTTPS schemes expose credentials in transit.",
            url.scheme()
        );
    }

    match url.host() {
        None => anyhow::bail!("api_endpoint must include a host name"),
        Some(Host::Domain(host)) => {
            // Block localhost variants
            let is_localhost = host == "localhost" || host.ends_with(".localhost");
            // Block well-known cloud metadata service hostnames
            let is_metadata_service = host == "metadata.google.internal"
                || host == "metadata.goog"
                || host == "instance-data"; // DigitalOcean
            if is_localhost || is_metadata_service {
                anyhow::bail!(
                    "api_endpoint host '{}' is not allowed. Use a public HTTPS endpoint.",
                    host
                );
            }
        }
        Some(Host::Ipv4(addr)) => {
            if addr.is_loopback() || addr.is_private() || addr.is_link_local() {
                anyhow::bail!(
                    "api_endpoint IP address '{}' is loopback, private, or link-local. \
                     Use a public HTTPS endpoint.",
                    addr
                );
            }
        }
        Some(Host::Ipv6(addr)) => {
            let o = addr.octets();
            // ::1 loopback, fc00::/7 unique-local, fe80::/10 link-local
            let is_unique_local = o[0] & 0xfe == 0xfc;
            let is_link_local = o[0] == 0xfe && o[1] & 0xc0 == 0x80;
            if addr.is_loopback() || is_unique_local || is_link_local {
                anyhow::bail!(
                    "api_endpoint IPv6 address '{}' is loopback or private. \
                     Use a public HTTPS endpoint.",
                    addr
                );
            }
            // Block IPv4-mapped (::ffff:x.x.x.x) and IPv4-compatible (::x.x.x.x)
            // addresses to prevent bypassing IPv4 restrictions via IPv6 notation.
            if let Some(ipv4) = addr.to_ipv4_mapped().or_else(|| addr.to_ipv4()) {
                if ipv4.is_loopback() || ipv4.is_private() || ipv4.is_link_local() {
                    anyhow::bail!(
                        "api_endpoint IPv6 address '{}' maps to a loopback, private, or \
                         link-local IPv4 address. Use a public HTTPS endpoint.",
                        addr
                    );
                }
            }
        }
    }

    Ok(())
}

/// Returns a keychain entry handle for the given profile name.
fn keychain_entry(profile_name: &str) -> Result<Entry> {
    Entry::new("gen3-cli", &format!("api_key:{}", profile_name))
        .context("Failed to access OS keychain")
}

/// Stores an API key for the given profile in the OS keychain.
fn store_api_key(profile_name: &str, api_key: &str) -> Result<()> {
    keychain_entry(profile_name)?
        .set_password(api_key)
        .context("Failed to store API key in OS keychain")
}

/// Retrieves the API key for the given profile from the OS keychain.
fn load_api_key(profile_name: &str) -> Result<String> {
    keychain_entry(profile_name)?
        .get_password()
        .with_context(|| {
            format!(
                "API key for profile '{}' not found in the OS keychain. \
                 Run `gen3 auth setup` to re-configure this profile.",
                profile_name
            )
        })
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
    ///
    /// API keys are never stored in the config file; they are loaded from the OS keychain.
    /// If an existing config file contains a plaintext `api_key` (legacy format), it is
    /// automatically migrated into the keychain and the file is rewritten without it.
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let mut config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        let mut needs_migration = false;
        for (name, profile) in &mut config.profiles {
            if !profile.api_key.expose_secret().is_empty() {
                // Legacy plaintext api_key found in config file — migrate to OS keychain.
                store_api_key(name, profile.api_key.expose_secret())
                    .context("Failed to migrate API key to OS keychain")?;
                needs_migration = true;
            } else {
                // Load api_key from OS keychain.
                profile.api_key = SecretString::from(load_api_key(name)?);
            }
        }

        if needs_migration {
            // Rewrite the config file without the plaintext api_key fields.
            config.save()?;
        }

        // Validate all profile endpoints to catch manually-edited unsafe configs.
        for (name, profile) in &config.profiles {
            validate_endpoint(&profile.api_endpoint)
                .with_context(|| {
                    format!(
                        "Profile '{}' has an invalid api_endpoint: '{}'",
                        name, profile.api_endpoint
                    )
                })?;
        }

        Ok(config)
    }

    /// Save the config to disk, creating the ~/.gen3/ directory if necessary.
    ///
    /// On Unix, the directory is created/set to 0o700 and the file to 0o600 so that
    /// only the owning user can read or write it. API keys are never included in the
    /// serialized output regardless of platform.
    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
            if !dir.exists() {
                fs::DirBuilder::new()
                    .recursive(true)
                    .mode(0o700)
                    .create(&dir)
                    .with_context(|| {
                        format!("Failed to create config directory: {}", dir.display())
                    })?;
            } else {
                fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))
                    .with_context(|| {
                        format!(
                            "Failed to set permissions on config directory: {}",
                            dir.display()
                        )
                    })?;
            }
        }

        #[cfg(not(unix))]
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;
        }

        let path = Self::config_path()?;
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;

        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)
                .with_context(|| {
                    format!("Failed to open config file for writing: {}", path.display())
                })?;
            file.write_all(contents.as_bytes())
                .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        }

        #[cfg(not(unix))]
        fs::write(&path, &contents)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Add or overwrite a profile, store its API key in the OS keychain, and persist the config.
    ///
    /// Returns an error if `profile.api_endpoint` fails HTTPS / SSRF validation.
    pub fn add_profile(&mut self, name: String, profile: Profile) -> Result<()> {
        validate_endpoint(&profile.api_endpoint)
            .context("Invalid api_endpoint")?;
        if self.active_profile.is_none() {
            self.active_profile = Some(name.clone());
        }
        store_api_key(&name, profile.api_key.expose_secret())?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_debug_redacts_api_key() {
        let profile = Profile {
            api_endpoint: "https://example.org".to_string(),
            api_key: SecretString::from("super-secret-key".to_string()),
            key_id: "key-abc123".to_string(),
        };
        let debug_output = format!("{:?}", profile);
        assert!(
            !debug_output.contains("super-secret-key"),
            "Profile Debug output must not expose the api_key: {debug_output}"
        );
        assert!(
            debug_output.contains("[REDACTED]"),
            "Profile Debug output must contain [REDACTED]: {debug_output}"
        );
        assert!(
            debug_output.contains("https://example.org"),
            "Profile Debug output should include api_endpoint: {debug_output}"
        );
        assert!(
            debug_output.contains("key-abc123"),
            "Profile Debug output should include key_id: {debug_output}"
        );
    }

    #[test]
    fn credentials_file_debug_redacts_api_key() {
        let creds = CredentialsFile {
            api_key: SecretString::from("top-secret-api-key".to_string()),
            key_id: "kid-xyz".to_string(),
        };
        let debug_output = format!("{:?}", creds);
        assert!(
            !debug_output.contains("top-secret-api-key"),
            "CredentialsFile Debug output must not expose the api_key: {debug_output}"
        );
        assert!(
            debug_output.contains("[REDACTED]"),
            "CredentialsFile Debug output must contain [REDACTED]: {debug_output}"
        );
        assert!(
            debug_output.contains("kid-xyz"),
            "CredentialsFile Debug output should include key_id: {debug_output}"
        );
    }

    #[test]
    fn config_debug_redacts_profile_api_keys() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "prod".to_string(),
            Profile {
                api_endpoint: "https://prod.example.org".to_string(),
                api_key: SecretString::from("prod-secret-key".to_string()),
                key_id: "prod-key-id".to_string(),
            },
        );
        let config = Config {
            profiles,
            active_profile: Some("prod".to_string()),
        };
        let debug_output = format!("{:?}", config);
        assert!(
            !debug_output.contains("prod-secret-key"),
            "Config Debug output must not expose profile api_key: {debug_output}"
        );
        assert!(
            debug_output.contains("[REDACTED]"),
            "Config Debug output must contain [REDACTED]: {debug_output}"
        );
    }

    #[test]
    fn valid_https_endpoint_is_accepted() {
        assert!(validate_endpoint("https://commons.example.org").is_ok());
        assert!(validate_endpoint("https://gen3.datacommons.io").is_ok());
    }

    #[test]
    fn http_scheme_is_rejected() {
        let err = validate_endpoint("http://commons.example.org").unwrap_err();
        assert!(err.to_string().contains("HTTPS"), "expected HTTPS error, got: {err}");
    }

    #[test]
    fn non_http_schemes_are_rejected() {
        for scheme in &["file:///etc/passwd", "gopher://example.org", "ftp://example.org"] {
            let err = validate_endpoint(scheme).unwrap_err();
            assert!(
                err.to_string().contains("HTTPS"),
                "expected HTTPS error for {scheme}, got: {err}"
            );
        }
    }

    #[test]
    fn localhost_is_rejected() {
        let err = validate_endpoint("https://localhost/api").unwrap_err();
        assert!(err.to_string().contains("not allowed"), "got: {err}");
    }

    #[test]
    fn loopback_ipv4_is_rejected() {
        let err = validate_endpoint("https://127.0.0.1/api").unwrap_err();
        assert!(err.to_string().contains("loopback, private, or link-local"), "got: {err}");
    }

    #[test]
    fn private_ipv4_ranges_are_rejected() {
        for addr in &["10.0.0.1", "192.168.1.1", "172.16.0.1"] {
            let endpoint = format!("https://{addr}/api");
            let err = validate_endpoint(&endpoint).unwrap_err();
            assert!(
                err.to_string().contains("loopback, private, or link-local"),
                "expected loopback/private/link-local error for {addr}, got: {err}"
            );
        }
    }

    #[test]
    fn link_local_ipv4_is_rejected() {
        let err = validate_endpoint("https://169.254.169.254/latest/meta-data").unwrap_err();
        assert!(err.to_string().contains("loopback, private, or link-local"), "got: {err}");
    }

    #[test]
    fn loopback_ipv6_is_rejected() {
        let err = validate_endpoint("https://[::1]/api").unwrap_err();
        assert!(err.to_string().contains("loopback or private"), "got: {err}");
    }

    #[test]
    fn unique_local_ipv6_is_rejected() {
        let err = validate_endpoint("https://[fc00::1]/api").unwrap_err();
        assert!(err.to_string().contains("loopback or private"), "got: {err}");
    }

    #[test]
    fn link_local_ipv6_is_rejected() {
        let err = validate_endpoint("https://[fe80::1]/api").unwrap_err();
        assert!(err.to_string().contains("loopback or private"), "got: {err}");
    }

    #[test]
    fn ipv4_mapped_ipv6_loopback_is_rejected() {
        // ::ffff:127.0.0.1 — IPv4-mapped IPv6 loopback bypass
        let err = validate_endpoint("https://[::ffff:127.0.0.1]/api").unwrap_err();
        assert!(
            err.to_string().contains("maps to a loopback, private, or link-local IPv4"),
            "got: {err}"
        );
    }

    #[test]
    fn ipv4_mapped_ipv6_link_local_is_rejected() {
        // ::ffff:169.254.169.254 — IPv4-mapped IPv6 AWS metadata bypass
        let err = validate_endpoint("https://[::ffff:169.254.169.254]/latest/meta-data").unwrap_err();
        assert!(
            err.to_string().contains("maps to a loopback, private, or link-local IPv4"),
            "got: {err}"
        );
    }

    #[test]
    fn ipv4_mapped_ipv6_private_is_rejected() {
        // ::ffff:10.0.0.1 — IPv4-mapped IPv6 private RFC-1918 bypass
        let err = validate_endpoint("https://[::ffff:10.0.0.1]/api").unwrap_err();
        assert!(
            err.to_string().contains("maps to a loopback, private, or link-local IPv4"),
            "got: {err}"
        );
    }

    #[test]
    fn metadata_hostname_is_rejected() {
        let err = validate_endpoint("https://metadata.google.internal/computeMetadata/v1").unwrap_err();
        assert!(err.to_string().contains("not allowed"), "got: {err}");

        let err2 = validate_endpoint("https://metadata.goog/").unwrap_err();
        assert!(err2.to_string().contains("not allowed"), "got: {err2}");
    }

    #[test]
    fn invalid_url_is_rejected() {
        let err = validate_endpoint("not a url").unwrap_err();
        assert!(err.to_string().contains("valid URL"), "got: {err}");
    }
}
