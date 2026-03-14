//! Config loading and persistence for Morpheum CLI.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// CLI configuration (stored in ~/.morpheum/config.toml).
///
/// **Security**: Mnemonic is NEVER stored in config. Use MORPHEUM_MNEMONIC env only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// RPC endpoint URL
    #[serde(default = "default_rpc")]
    pub rpc_endpoint: String,

    /// Chain ID
    #[serde(default = "default_chain_id")]
    pub chain_id: String,

    /// Legacy: ignore mnemonic if present in old config files (never write it)
    #[serde(skip_serializing, default)]
    #[allow(dead_code)]
    mnemonic: Option<String>,
}

fn default_rpc() -> String {
    "https://rpc.morpheum.xyz".to_string()
}

fn default_chain_id() -> String {
    "morpheum-1".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_endpoint: default_rpc(),
            chain_id: default_chain_id(),
            mnemonic: None,
        }
    }
}

impl Config {
    /// Load config from file, merging with defaults.
    /// Returns default config if file doesn't exist.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let s = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;
        let cfg: Self = toml::from_str(&s)
            .with_context(|| format!("Failed to parse config: {}", path.display()))?;
        Ok(cfg)
    }

    /// Save config to file, creating parent dirs if needed.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config dir: {}", parent.display()))?;
        }
        let s = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        std::fs::write(path, s)
            .with_context(|| format!("Failed to write config: {}", path.display()))?;
        Ok(())
    }

    /// Get mnemonic from env only. Never from file (security).
    pub fn mnemonic(&self) -> Option<String> {
        std::env::var("MORPHEUM_MNEMONIC").ok()
    }
}

/// Default config directory (~/.morpheum).
pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".morpheum")
}

/// Default config file path.
pub fn default_config_path() -> PathBuf {
    config_dir().join("config.toml")
}
