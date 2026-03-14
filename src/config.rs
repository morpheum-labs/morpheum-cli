use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Output format preference for CLI commands (table vs JSON).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

/// Central configuration for the Morpheum CLI.
///
/// Loaded from `~/.config/morpheum/config.toml` via `confy`.
/// Sensible production defaults are provided. Environment variables can override
/// specific fields via the `clap` `env` attribute in `cli.rs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorpheumConfig {
    #[serde(default = "default_chain_id")]
    pub chain_id: String,

    #[serde(default = "default_rpc_url")]
    pub rpc_url: String,

    #[serde(default)]
    pub default_output: OutputFormat,

    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    #[serde(default = "default_keyring_backend")]
    pub keyring_backend: String,
}

impl Default for MorpheumConfig {
    fn default() -> Self {
        Self {
            chain_id: default_chain_id(),
            rpc_url: default_rpc_url(),
            default_output: OutputFormat::Table,
            timeout_secs: default_timeout_secs(),
            keyring_backend: default_keyring_backend(),
        }
    }
}

impl MorpheumConfig {
    /// Loads configuration from the standard location.
    /// If the file does not exist, returns `Default` values.
    pub fn load() -> Result<Self, CliError> {
        let config: MorpheumConfig = confy::load("morpheum", None)
            .map_err(CliError::Config)?;
        Ok(config)
    }

    /// Saves the current configuration back to disk.
    pub fn save(&self) -> Result<(), CliError> {
        confy::store("morpheum", None, self)
            .map_err(CliError::Config)?;
        Ok(())
    }

    /// Returns the full path to the config file (for user messaging).
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("morpheum")
            .join("config.toml")
    }
}

// ── Default helpers ─────────────────────────────────────────────────────────

fn default_chain_id() -> String {
    "morpheum-test-1".to_string()
}

fn default_rpc_url() -> String {
    "https://sentry.morpheum.xyz".to_string()
}

fn default_timeout_secs() -> u64 {
    30
}

fn default_keyring_backend() -> String {
    "os".to_string()
}

// ── Config subcommands (`morpheum config show`, `morpheum config path`, etc.) ──

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Display the current configuration
    Show,

    /// Print the path to the config file
    Path,

    /// Update a configuration value
    Set(SetConfigArgs),

    /// Reset configuration to defaults
    Reset,
}

#[derive(Args)]
pub struct SetConfigArgs {
    /// Configuration key (e.g. `chain_id`, `rpc_url`, `keyring_backend`)
    #[arg(required = true)]
    pub key: String,

    /// New value
    #[arg(required = true)]
    pub value: String,
}

#[allow(clippy::unused_async)]
pub async fn execute(cmd: ConfigCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    match cmd {
        ConfigCommands::Show => {
            output.info(format!("chain_id:        {}", dispatcher.config.chain_id));
            output.info(format!("rpc_url:         {}", dispatcher.config.rpc_url));
            output.info(format!("timeout_secs:    {}", dispatcher.config.timeout_secs));
            output.info(format!("keyring_backend: {}", dispatcher.config.keyring_backend));
            output.info(format!(
                "default_output:  {:?}",
                dispatcher.config.default_output
            ));
        }
        ConfigCommands::Path => {
            println!("{}", MorpheumConfig::config_path().display());
        }
        ConfigCommands::Set(args) => {
            let mut config = dispatcher.config.clone();
            let SetConfigArgs { key, value } = args;
            match key.as_str() {
                "chain_id" => config.chain_id.clone_from(&value),
                "rpc_url" => config.rpc_url.clone_from(&value),
                "timeout_secs" => {
                    config.timeout_secs = value.parse().map_err(|_| {
                        CliError::invalid_input("timeout_secs must be a positive integer")
                    })?;
                }
                "keyring_backend" => config.keyring_backend.clone_from(&value),
                _ => {
                    return Err(CliError::invalid_input(format!(
                        "Unknown config key: {key}"
                    )));
                }
            }
            config.save()?;
            output.success(format!("Configuration updated: {key} = {value}"));
        }
        ConfigCommands::Reset => {
            MorpheumConfig::default().save()?;
            output.success("Configuration reset to defaults");
        }
    }

    Ok(())
}