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

/// The 32-byte genesis hash of the chain this CLI signs for.
///
/// Bound into every signing preimage (Phase M3) so a signature valid on one
/// chain cannot be replayed onto another that happens to share its `chain_id`.
///
/// # Trust model
///
/// This is **chain identity**, and it deliberately comes from the same trust
/// root as [`MorpheumConfig::chain_id`]: operator configuration. It is
/// **never** fetched from the node being submitted to.
///
/// That distinction is the whole point of the binding. A client that asked its
/// RPC endpoint for the hash and then signed against the answer would let
/// whoever controls that endpoint choose which chain the signature authorises
/// — hand it a second chain's hash, and the resulting transaction is replayable
/// there. Fetching the value would re-create precisely the attack the binding
/// exists to prevent.
///
/// Obtain it the same way you obtain `chain_id`: from the chain's published
/// parameters or its genesis document. A node may expose its own hash for
/// *verification* — comparing it against this value turns a misconfiguration
/// into a loud error — but that is a cross-check, not a source.
///
/// Parsed and length-checked at config load, so a malformed value is reported
/// when the operator sets it rather than silently producing a preimage no
/// validator accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenesisHash([u8; Self::LEN]);

impl GenesisHash {
    /// Length of a `blake3` genesis digest.
    pub const LEN: usize = 32;

    /// The raw digest, ready to bind into a signing preimage.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; Self::LEN] {
        &self.0
    }
}

impl std::str::FromStr for GenesisHash {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed = value.strip_prefix("0x").unwrap_or(value);
        let raw = hex::decode(trimmed).map_err(|err| format!("not valid hex: {err}"))?;
        let len = raw.len();
        let bytes: [u8; Self::LEN] = raw.try_into().map_err(|_| {
            format!("expected {} bytes ({} hex chars), got {len}", Self::LEN, Self::LEN * 2)
        })?;
        Ok(Self(bytes))
    }
}

impl std::fmt::Display for GenesisHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl Serialize for GenesisHash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for GenesisHash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(serde::de::Error::custom)
    }
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

    /// Genesis hash of the chain identified by [`Self::chain_id`], bound into
    /// every signing preimage. See [`GenesisHash`] for why this is configured
    /// rather than fetched.
    ///
    /// `None` leaves signatures unbound to a chain instance, which validators
    /// still accept while the strict genesis fork is advisory. Signing warns in
    /// that case: an unbound signature authorises the transaction on any chain
    /// sharing this `chain_id`.
    #[serde(default)]
    pub genesis_hash: Option<GenesisHash>,

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
            // No default: a wrong genesis hash silently produces signatures no
            // validator accepts, and there is no value that is correct for
            // every chain. Absent is honest; a placeholder would not be.
            genesis_hash: None,
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
        let config: MorpheumConfig = confy::load("morpheum", None).map_err(CliError::Config)?;
        Ok(config)
    }

    /// Saves the current configuration back to disk.
    pub fn save(&self) -> Result<(), CliError> {
        confy::store("morpheum", None, self).map_err(CliError::Config)?;
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
    /// Configuration key (e.g. `chain_id`, `genesis_hash`, `rpc_url`, `keyring_backend`)
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
            output.info(format!(
                "genesis_hash:    {}",
                dispatcher.config.genesis_hash.map_or_else(
                    || "<unset — signatures are not bound to a chain instance>".to_string(),
                    |hash| hash.to_string(),
                )
            ));
            output.info(format!("rpc_url:         {}", dispatcher.config.rpc_url));
            output.info(format!(
                "timeout_secs:    {}",
                dispatcher.config.timeout_secs
            ));
            output.info(format!(
                "keyring_backend: {}",
                dispatcher.config.keyring_backend
            ));
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
                "genesis_hash" => {
                    config.genesis_hash = Some(value.parse().map_err(|err| {
                        CliError::invalid_input(format!("genesis_hash {err}"))
                    })?);
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    const HASH_HEX: &str = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a";

    #[test]
    fn genesis_hash_round_trips_through_hex() {
        let parsed: GenesisHash = HASH_HEX.parse().expect("32 bytes of hex parses");
        assert_eq!(parsed.as_bytes(), &[0x5a; GenesisHash::LEN]);
        assert_eq!(parsed.to_string(), HASH_HEX);
    }

    #[test]
    fn genesis_hash_accepts_an_0x_prefix() {
        let with_prefix: GenesisHash = format!("0x{HASH_HEX}").parse().expect("0x prefix parses");
        let without: GenesisHash = HASH_HEX.parse().expect("bare hex parses");
        assert_eq!(with_prefix, without);
    }

    /// A wrong-length hash is not a cosmetic error: it produces a signing
    /// preimage no validator accepts, so every transaction would be rejected
    /// with a signature failure that points nowhere near the real cause.
    /// Rejecting it at parse time reports it when the operator sets the value.
    #[test]
    fn genesis_hash_rejects_a_wrong_length_digest() {
        let short = "5a5a5a";
        let err = short.parse::<GenesisHash>().expect_err("3 bytes must not parse");
        assert!(err.contains("expected 32 bytes"), "unhelpful error: {err}");

        let long = format!("{HASH_HEX}5a");
        let err = long.parse::<GenesisHash>().expect_err("33 bytes must not parse");
        assert!(err.contains("expected 32 bytes"), "unhelpful error: {err}");
    }

    #[test]
    fn genesis_hash_rejects_non_hex() {
        let err = "zz".repeat(32).parse::<GenesisHash>().expect_err("non-hex must not parse");
        assert!(err.contains("not valid hex"), "unhelpful error: {err}");
    }

    /// Absent by default. There is no genesis hash that is correct for every
    /// chain, so a placeholder would be a value that is always wrong.
    #[test]
    fn genesis_hash_defaults_to_unset() {
        assert_eq!(MorpheumConfig::default().genesis_hash, None);
    }

    /// The config file is the trust root, so the value must survive a
    /// serialize/deserialize round trip through it unchanged.
    #[test]
    fn genesis_hash_survives_a_config_round_trip() {
        let config = MorpheumConfig {
            genesis_hash: Some(HASH_HEX.parse().expect("parses")),
            ..Default::default()
        };

        let encoded = toml::to_string(&config).expect("config serialises");
        let decoded: MorpheumConfig = toml::from_str(&encoded).expect("config deserialises");

        assert_eq!(decoded.genesis_hash, config.genesis_hash);
    }
}
