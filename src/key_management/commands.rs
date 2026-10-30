use clap::{Args, Subcommand};
use secrecy::SecretString;
use serde::Serialize;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::keyring::KeyringManager;

/// Secure key management commands.
///
/// Stores BIP-39 mnemonics in the OS-native keyring (macOS Keychain,
/// Windows Credential Manager, Linux Secret Service) with a plaintext
/// file fallback. Agent delegation keys (`AgentSigner` + `TradingKeyClaim`)
/// are derived at command execution time from stored native keys.
#[derive(Subcommand)]
pub enum KeysCommands {
    /// Add a new native wallet key from a BIP-39 mnemonic
    Add(AddArgs),

    /// Import a raw EVM private key (hex)
    ImportEvm(ImportEvmArgs),

    /// Show derived addresses for a stored key
    Show(ShowArgs),

    /// List all stored keys
    List,

    /// Delete a key
    Delete(DeleteArgs),

    /// Export key information (public key only — mnemonic export disabled)
    Export(ExportArgs),
}

#[derive(Args)]
pub struct AddArgs {
    /// Unique name for the key
    #[arg(required = true)]
    pub name: String,

    /// BIP-39 mnemonic phrase
    #[arg(long, required = true)]
    pub mnemonic: String,
}

#[derive(Args)]
pub struct ImportEvmArgs {
    /// Unique name for the key
    #[arg(required = true)]
    pub name: String,

    /// Raw EVM private key (hex, with or without 0x prefix)
    #[arg(long, required = true)]
    pub private_key: String,
}

#[derive(Args)]
pub struct ShowArgs {
    /// Name of the key to display
    #[arg(required = true)]
    pub name: String,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// Name of the key to delete
    #[arg(required = true)]
    pub name: String,
}

#[derive(Args)]
pub struct ExportArgs {
    /// Name of the key to export
    #[arg(required = true)]
    pub name: String,
}

#[allow(clippy::unused_async)]
pub async fn execute(cmd: KeysCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        KeysCommands::Add(args) => add_key(args, &dispatcher),
        KeysCommands::ImportEvm(args) => import_evm_key(&args, &dispatcher),
        KeysCommands::Show(args) => show_key(&args, &dispatcher),
        KeysCommands::List => list_keys(&dispatcher),
        KeysCommands::Delete(args) => delete_key(args, &dispatcher),
        KeysCommands::Export(args) => export_key(args, &dispatcher),
    }
}

fn add_key(args: AddArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    output.info(format!("Adding native key: {}", args.name));
    dispatcher
        .keyring
        .add_native(&args.name, &SecretString::new(args.mnemonic))?;
    output.success(format!("Key '{}' added successfully", args.name));

    Ok(())
}

fn import_evm_key(args: &ImportEvmArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    let hex_str = args
        .private_key
        .strip_prefix("0x")
        .unwrap_or(&args.private_key);
    if hex_str.len() != 64 || !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(CliError::invalid_input(
            "private key must be 32 bytes (64 hex characters)",
        ));
    }

    let normalized = format!("0x{hex_str}");
    dispatcher
        .keyring
        .add_native(&args.name, &SecretString::new(normalized))?;
    output.success(format!("EVM key '{}' imported successfully", args.name));

    Ok(())
}

fn show_key(args: &ShowArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let info = KeyAddresses::for_stored_key(&dispatcher.keyring, &args.name)?;

    dispatcher
        .output
        .print_json(&info)
        .map_err(|e| CliError::invalid_input(format!("JSON output: {e}")))?;

    Ok(())
}

fn list_keys(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;
    let keys = dispatcher.keyring.list_keys();

    if keys.is_empty() {
        output.info("No keys stored yet. Add one with `morpheum keys add`");
        return Ok(());
    }

    let list: Vec<KeyInfo> = keys.into_iter().map(|name| KeyInfo { name }).collect();
    output.print_list(&list)?;

    Ok(())
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn delete_key(args: DeleteArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    dispatcher.keyring.delete_key(&args.name);
    dispatcher
        .output
        .success(format!("Key '{}' deleted successfully", args.name));

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn export_key(args: ExportArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;
    output.warn("Exporting key information — use only in a secure environment");

    let _signer = dispatcher.keyring.get_native_signer(&args.name)?;
    output.success(format!("Public key for '{}' exported", args.name));

    Ok(())
}

#[derive(Serialize)]
struct KeyAddresses {
    name: String,
    #[serde(rename = "morpheum_address", skip_serializing_if = "Option::is_none")]
    canonical_morpheum_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    solana_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    evm_address: Option<String>,
}

impl KeyAddresses {
    fn for_stored_key(keyring: &KeyringManager, name: &str) -> Result<Self, CliError> {
        if keyring.is_hex_key(name) {
            return Ok(Self {
                name: name.to_owned(),
                canonical_morpheum_address: None,
                solana_address: None,
                evm_address: Some(keyring.evm_address(name)?),
            });
        }

        Ok(Self {
            name: name.to_owned(),
            canonical_morpheum_address: Some(keyring.morpheum_address(name)?),
            solana_address: Some(keyring.solana_address(name)?),
            evm_address: keyring.evm_address(name).ok(),
        })
    }
}

#[derive(tabled::Tabled, Serialize)]
struct KeyInfo {
    #[tabled(rename = "Key Name")]
    name: String,
}

#[cfg(test)]
mod tests {
    use super::KeyAddresses;

    #[test]
    fn shown_addresses_keep_the_morpheum_address_json_field() {
        let shown = KeyAddresses {
            name: "trader".to_owned(),
            canonical_morpheum_address: Some("morm1example".to_owned()),
            solana_address: Some("So11111111111111111111111111111111111111112".to_owned()),
            evm_address: Some("0xabc".to_owned()),
        };
        let value = serde_json::to_value(&shown).expect("serialize shown addresses");

        assert_eq!(value["name"], "trader");
        assert_eq!(value["morpheum_address"], "morm1example");
        assert_eq!(
            value["solana_address"],
            "So11111111111111111111111111111111111111112"
        );
        assert_eq!(value["evm_address"], "0xabc");
        assert!(value.get("canonical_morpheum_address").is_none());
    }

    #[test]
    fn a_hex_key_omits_morpheum_and_solana_addresses() {
        let shown = KeyAddresses {
            name: "evm".to_owned(),
            canonical_morpheum_address: None,
            solana_address: None,
            evm_address: Some("0xabc".to_owned()),
        };
        let value = serde_json::to_value(&shown).expect("serialize hex-key addresses");

        assert_eq!(value["name"], "evm");
        assert_eq!(value["evm_address"], "0xabc");
        assert!(value.get("morpheum_address").is_none());
        assert!(value.get("solana_address").is_none());
    }
}
