use crate::config::MorpheumConfig;
use crate::error::CliError;
use morpheum_signing_native::NativeSigner;
use secrecy::{ExposeSecret, SecretString};
use std::fs;
use std::path::PathBuf;

/// Secure key storage for the Morpheum CLI.
///
/// Stores BIP-39 mnemonics in the OS-native keyring (macOS Keychain, Windows
/// Credential Manager, Linux Secret Service) with a plaintext file fallback
/// in `~/.config/morpheum/keys/` when the OS keyring is unavailable.
///
/// All sensitive material is wrapped in `SecretString` to prevent accidental
/// logging. Production deployments should always use `keyring_backend = "os"`.
#[derive(Debug)]
pub struct KeyringManager {
    config: MorpheumConfig,
    key_dir: PathBuf,
}

impl KeyringManager {
    pub fn new(config: MorpheumConfig) -> Self {
        let key_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("morpheum")
            .join("keys");

        let _ = fs::create_dir_all(&key_dir);

        Self { config, key_dir }
    }

    /// Stores a native wallet key from a BIP-39 mnemonic.
    pub fn add_native(&self, name: &str, mnemonic: &SecretString) -> Result<(), CliError> {
        self.store_secret(name, mnemonic)
    }

    /// Retrieves a `NativeSigner` by key name.
    pub fn get_native_signer(&self, name: &str) -> Result<NativeSigner, CliError> {
        let mnemonic = self.load_secret(name)?;
        NativeSigner::from_mnemonic(mnemonic.expose_secret(), "")
            .map_err(CliError::Signing)
    }

    /// Lists all stored key names.
    pub fn list_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();

        if let Ok(entry) = keyring::Entry::new("morpheum", "keys-index") {
            if let Ok(data) = entry.get_password() {
                if let Ok(list) = serde_json::from_str::<Vec<String>>(&data) {
                    keys.extend(list);
                }
            }
        }

        if let Ok(entries) = fs::read_dir(&self.key_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    if !keys.contains(&name.to_string()) {
                        keys.push(name.to_string());
                    }
                }
            }
        }

        keys.sort();
        keys.dedup();
        keys
    }

    /// Deletes a key by name from both OS keyring and file fallback.
    pub fn delete_key(&self, name: &str) {
        let _ = keyring::Entry::new("morpheum", name)
            .and_then(|e| e.delete_credential());

        let _ = fs::remove_file(self.key_dir.join(name));
    }

    fn store_secret(&self, name: &str, secret: &SecretString) -> Result<(), CliError> {
        if self.config.keyring_backend == "os" {
            if let Ok(entry) = keyring::Entry::new("morpheum", name) {
                if entry.set_password(secret.expose_secret()).is_ok() {
                    return Ok(());
                }
            }
        }

        let path = self.key_dir.join(name);
        fs::write(&path, secret.expose_secret()).map_err(|e| CliError::Io {
            context: format!("Failed to write key file for {name}"),
            source: e,
        })
    }

    fn load_secret(&self, name: &str) -> Result<SecretString, CliError> {
        if self.config.keyring_backend == "os" {
            if let Ok(entry) = keyring::Entry::new("morpheum", name) {
                if let Ok(password) = entry.get_password() {
                    return Ok(SecretString::new(password));
                }
            }
        }

        let path = self.key_dir.join(name);
        let content = fs::read_to_string(&path).map_err(|e| CliError::Io {
            context: format!("Key '{name}' not found in keyring or file storage"),
            source: e,
        })?;

        Ok(SecretString::new(content))
    }
}
