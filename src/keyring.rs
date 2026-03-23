use crate::config::MorpheumConfig;
use crate::error::CliError;
use morpheum_signing_native::{EvmSigner, NativeSigner, SolanaSigner};
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
    pub fn new(mut config: MorpheumConfig) -> Self {
        let key_dir = match std::env::var("MORPHEUM_KEY_DIR") {
            Ok(dir) => {
                config.keyring_backend = "file".to_string();
                PathBuf::from(dir)
            }
            Err(_) => dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("morpheum")
                .join("keys"),
        };

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

    /// Derives an alloy `PrivateKeySigner` from the stored secret.
    ///
    /// Supports two formats:
    /// - **Mnemonic** (BIP-39 words): derives via path `m/44'/60'/0'/0/0`
    /// - **Raw hex key** (with or without `0x` prefix): used directly
    pub fn get_evm_signer(
        &self,
        name: &str,
    ) -> Result<morpheum_sdk_evm::alloy::signers::local::PrivateKeySigner, CliError> {
        use morpheum_sdk_evm::alloy::primitives::FixedBytes;
        use morpheum_sdk_evm::alloy::signers::local::PrivateKeySigner;

        let secret = self.load_secret(name)?;
        let raw = secret.expose_secret().trim();

        if Self::looks_like_hex_key(raw) {
            let hex_str = raw.strip_prefix("0x").unwrap_or(raw);
            let key_bytes: [u8; 32] = hex::decode(hex_str)
                .map_err(|e| CliError::chain("EVM", format!("invalid hex private key: {e}")))?
                .try_into()
                .map_err(|v: Vec<u8>| {
                    CliError::chain("EVM", format!("private key must be 32 bytes, got {}", v.len()))
                })?;
            PrivateKeySigner::from_bytes(&FixedBytes::from(key_bytes))
                .map_err(|e| CliError::chain("EVM", format!("failed to create EVM signer: {e}")))
        } else {
            let evm = EvmSigner::from_mnemonic(raw, "")
                .map_err(CliError::Signing)?;
            let key_bytes = evm.private_key_bytes();
            PrivateKeySigner::from_bytes(&FixedBytes::from(key_bytes))
                .map_err(|e| CliError::chain("EVM", format!("failed to create EVM signer: {e}")))
        }
    }

    fn looks_like_hex_key(s: &str) -> bool {
        let hex_str = s.strip_prefix("0x").unwrap_or(s);
        hex_str.len() == 64 && hex_str.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Returns the Morpheum bech32 (`morm1...`) address for a stored key.
    ///
    /// Only works with mnemonic-based keys (not raw hex private keys).
    pub fn morpheum_address(&self, name: &str) -> Result<String, CliError> {
        use morpheum_signing_native::signer::Signer;
        let native = self.get_native_signer(name)?;
        let acct = native.account_id().0;
        Ok(morpheum_primitives::address::encode_address(&acct[acct.len() - 20..]))
    }

    /// Returns `true` if the stored secret is a raw hex private key rather
    /// than a BIP-39 mnemonic.
    pub fn is_hex_key(&self, name: &str) -> bool {
        self.load_secret(name)
            .map(|s| Self::looks_like_hex_key(s.expose_secret().trim()))
            .unwrap_or(false)
    }

    /// Returns the EVM (0x-prefixed) address for a stored key.
    pub fn evm_address(&self, name: &str) -> Result<String, CliError> {
        use morpheum_sdk_evm::alloy::signers::Signer;

        let signer = self.get_evm_signer(name)?;
        Ok(format!("{:#x}", signer.address()))
    }

    /// Derives a `SolanaSigner` from the stored BIP-39 mnemonic.
    ///
    /// Derivation path: `m/44'/501'/0'/0'` (standard Solana, SLIP-0010 Ed25519).
    /// The same mnemonic produces deterministic keys for Morpheum native, EVM,
    /// and Solana.
    pub fn get_solana_signer(&self, name: &str) -> Result<SolanaSigner, CliError> {
        let mnemonic = self.load_secret(name)?;
        SolanaSigner::from_mnemonic(mnemonic.expose_secret(), "")
            .map_err(CliError::Signing)
    }

    /// Returns the Base58-encoded Solana address for a stored key.
    pub fn solana_address(&self, name: &str) -> Result<String, CliError> {
        let signer = self.get_solana_signer(name)?;
        Ok(bs58::encode(signer.public_key_bytes()).into_string())
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
