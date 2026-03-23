//! Cross-chain execution infrastructure.
//!
//! Provides the composable primitives that any Morpheum module can use to
//! bridge tokens in and out. The [`CrossChainExecutor`] centralises all
//! external-chain interaction (EVM / SVM) so individual modules never touch
//! SDK bridge functions directly.
//!
//! # Design
//!
//! - [`ChainSpec`] parses a compact `"evm:sepolia"` identifier.
//! - [`CrossChainContext`] is a clap `Args` struct that modules `flatten` to
//!   opt into cross-chain funding.
//! - [`CrossChainExecutor`] handles chain-registry loading, signer selection,
//!   and SDK bridge calls.

use clap::{Args, ValueEnum};
use morpheum_sdk_core::ChainRegistryOps as _;
use morpheum_sdk_evm::alloy::primitives::{FixedBytes, U256};
use morpheum_sdk_evm::config::{ChainRegistry, TokenType};
use morpheum_sdk_svm::config::{SolanaChainRegistry, SvmTokenType};

use crate::error::CliError;
use crate::keyring::KeyringManager;
use crate::output::Output;

// ── Chain identification ────────────────────────────────────────────

/// Supported external chain VM types.
#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ChainType {
    /// Ethereum / EVM-compatible chains
    Evm,
    /// Solana / SVM-compatible chains
    Svm,
}

impl ChainType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Evm => "EVM",
            Self::Svm => "SVM",
        }
    }
}

/// Parsed chain specification from the compact `<type>:<network>` format.
///
/// Examples: `"evm:sepolia"`, `"svm:devnet"`, `"evm:arbitrum"`.
#[derive(Clone, Debug)]
pub struct ChainSpec {
    pub chain_type: ChainType,
    pub network: String,
}

impl ChainSpec {
    pub fn parse(s: &str) -> Result<Self, CliError> {
        let (type_str, network) = s.split_once(':').ok_or_else(|| {
            CliError::invalid_input(format!(
                "invalid chain format '{s}': expected '<type>:<name>' \
                 (e.g. 'evm:sepolia', 'svm:devnet')"
            ))
        })?;
        let chain_type = match type_str.to_lowercase().as_str() {
            "evm" => ChainType::Evm,
            "svm" => ChainType::Svm,
            other => {
                return Err(CliError::invalid_input(format!(
                    "unknown chain type '{other}': expected 'evm' or 'svm'"
                )))
            }
        };
        Ok(Self {
            chain_type,
            network: network.to_string(),
        })
    }
}

// ── Composable cross-chain args ─────────────────────────────────────

/// Composable cross-chain execution context.
///
/// Flatten into any module command's `Args` struct to enable cross-chain
/// funding. When `chain` is `Some`, the command routes through the
/// [`CrossChainExecutor`] instead of (or in addition to) a native tx.
///
/// # Example
///
/// ```ignore
/// #[derive(Args)]
/// pub struct PlaceOrderArgs {
///     #[arg(long)]
///     pub market: String,
///     #[command(flatten)]
///     pub xchain: CrossChainContext,
/// }
/// ```
#[derive(Args, Default, Clone, Debug)]
pub struct CrossChainContext {
    /// External chain for cross-chain execution (format: "evm:sepolia", "svm:devnet")
    #[arg(long = "xchain")]
    pub chain: Option<String>,

    /// Token to bridge for cross-chain funding (e.g. "USDC", "ETH", "SOL")
    #[arg(long = "xchain-token")]
    pub token: Option<String>,

    /// Recipient address on external chain (for outbound operations)
    #[arg(long = "xchain-recipient")]
    pub recipient: Option<String>,
}

impl CrossChainContext {
    pub fn is_active(&self) -> bool {
        self.chain.is_some()
    }

    pub fn require_chain_spec(&self) -> Result<ChainSpec, CliError> {
        let chain = self.chain.as_deref().ok_or_else(|| {
            CliError::invalid_input("--xchain is required for cross-chain operations")
        })?;
        ChainSpec::parse(chain)
    }

    pub fn require_token(&self) -> Result<&str, CliError> {
        self.token.as_deref().ok_or_else(|| {
            CliError::invalid_input("--xchain-token is required for cross-chain operations")
        })
    }
}

// ── Deposit results ─────────────────────────────────────────────────

pub struct EvmDepositResult {
    pub tx_hash: String,
    pub message_id: String,
    pub amount_display: String,
    pub token: String,
    pub destination_domain: u32,
}

pub struct SvmDepositResult {
    pub signature: String,
    pub message_id: String,
    pub message_storage_pda: String,
    pub amount_display: String,
    pub token: String,
    pub destination_domain: u32,
}

// ── Executor ────────────────────────────────────────────────────────

/// Centralised cross-chain execution engine.
///
/// Wraps the SDK bridge functions so that individual module handlers never
/// deal with chain registries, signer selection, or raw bridge calls.
pub struct CrossChainExecutor<'a> {
    pub keyring: &'a KeyringManager,
    pub output: &'a Output,
}

impl<'a> CrossChainExecutor<'a> {
    pub fn from_dispatcher(dispatcher: &'a crate::dispatcher::Dispatcher) -> Self {
        Self {
            keyring: &dispatcher.keyring,
            output: &dispatcher.output,
        }
    }

    /// Deposit tokens from an EVM chain to Morpheum via Hyperlane Warp Route.
    pub async fn deposit_evm(
        &self,
        chain_name: &str,
        token_symbol: &str,
        amount_str: &str,
        recipient: Option<&str>,
        from_key: &str,
        destination_domain: u32,
        rpc_override: Option<&str>,
    ) -> Result<EvmDepositResult, CliError> {
        let registry = ChainRegistry::load_with_defaults(morpheum_sdk_evm::DEFAULT_CHAINS_TOML)
            .map_err(|e| CliError::chain("EVM", format!("chain registry: {e}")))?;

        let (mut chain, token) = registry
            .resolve(chain_name, token_symbol)
            .map_err(|e| {
                CliError::chain("EVM", format!("resolving chain '{chain_name}': {e}"))
            })?;

        if let Some(rpc) = rpc_override {
            chain.rpc_url = rpc.to_string();
        }

        let collateral = token.collateral_contract.ok_or_else(|| {
            CliError::chain(
                "EVM",
                format!(
                    "no collateral contract configured for {token_symbol} on {chain_name}"
                ),
            )
        })?;

        let amount = parse_token_amount(amount_str, token.decimals)?;

        let alloy_signer = self.keyring.get_evm_signer(from_key)?;
        let from_address = format!(
            "{:#x}",
            morpheum_sdk_evm::alloy::signers::Signer::address(&alloy_signer)
        );
        let recipient_bytes =
            resolve_recipient(recipient, from_key, self.keyring, true)?;

        let provider = morpheum_sdk_evm::build_provider(&chain.rpc_url, alloy_signer)
            .map_err(|e| CliError::chain("EVM", format!("provider: {e}")))?;

        match token.token_type {
            TokenType::Native => {
                self.output.info(format!(
                    "EVM deposit (native)\n\
                     From: {from_address}\n\
                     Chain: {chain_name} (RPC: {})\n\
                     Token: {token_symbol} (native)\n\
                     Warp Native: {:#x}\n\
                     Amount: {amount_str} ({amount} raw)\n\
                     Destination domain: {destination_domain}\n\
                     Recipient: 0x{}",
                    chain.rpc_url,
                    collateral,
                    hex::encode(recipient_bytes),
                ));

                self.output.info("Calling transferRemote (native)...");
                let result = morpheum_sdk_evm::transfer_remote_native(
                    &provider,
                    collateral,
                    destination_domain,
                    FixedBytes(recipient_bytes),
                    amount,
                )
                .await
                .map_err(|e| {
                    CliError::chain("EVM", format!("transferRemote native: {e}"))
                })?;

                Ok(EvmDepositResult {
                    tx_hash: format!("{:#x}", result.tx_hash),
                    message_id: format!("{:#x}", result.message_id),
                    amount_display: amount_str.to_string(),
                    token: token_symbol.to_string(),
                    destination_domain,
                })
            }
            TokenType::Erc20 => {
                self.output.info(format!(
                    "EVM deposit (ERC-20)\n\
                     From: {from_address}\n\
                     Chain: {chain_name} (RPC: {})\n\
                     Token: {token_symbol} ({:#x})\n\
                     Collateral: {:#x}\n\
                     Amount: {amount_str} ({amount} raw)\n\
                     Destination domain: {destination_domain}\n\
                     Recipient: 0x{}",
                    chain.rpc_url,
                    token.address,
                    collateral,
                    hex::encode(recipient_bytes),
                ));

                self.output.info("Approving ERC-20 spend...");
                let approve_hash = morpheum_sdk_evm::approve_erc20(
                    &provider,
                    token.address,
                    collateral,
                    amount,
                )
                .await
                .map_err(|e| CliError::chain("EVM", format!("approve: {e}")))?;
                self.output
                    .info(format!("Approval confirmed: {approve_hash:#x}"));

                self.output.info("Quoting Hyperlane dispatch fee...");
                let fee = morpheum_sdk_evm::quote_warp_fee(
                    &provider,
                    collateral,
                    destination_domain,
                    FixedBytes(recipient_bytes),
                    amount,
                )
                .await
                .map_err(|e| {
                    CliError::chain("EVM", format!("quoteDispatch: {e}"))
                })?;
                self.output.info(format!("Dispatch fee: {fee}"));

                self.output.info("Calling transferRemote...");
                let result = morpheum_sdk_evm::transfer_remote(
                    &provider,
                    collateral,
                    destination_domain,
                    FixedBytes(recipient_bytes),
                    amount,
                    fee,
                )
                .await
                .map_err(|e| {
                    CliError::chain("EVM", format!("transferRemote: {e}"))
                })?;

                Ok(EvmDepositResult {
                    tx_hash: format!("{:#x}", result.tx_hash),
                    message_id: format!("{:#x}", result.message_id),
                    amount_display: amount_str.to_string(),
                    token: token_symbol.to_string(),
                    destination_domain,
                })
            }
        }
    }

    /// Deposit tokens from an SVM chain to Morpheum via Hyperlane Warp Route.
    pub fn deposit_svm(
        &self,
        chain_name: &str,
        token_symbol: &str,
        amount_str: &str,
        recipient: Option<&str>,
        from_key: &str,
        destination_domain: u32,
        rpc_override: Option<&str>,
    ) -> Result<SvmDepositResult, CliError> {
        use morpheum_sdk_svm::solana_sdk::signer::keypair::Keypair;

        let registry =
            SolanaChainRegistry::load_with_defaults(morpheum_sdk_svm::DEFAULT_CHAINS_TOML)
                .map_err(|e| CliError::chain("SVM", format!("chain registry: {e}")))?;

        let (mut chain, token) = registry
            .resolve(chain_name, token_symbol)
            .map_err(|e| {
                CliError::chain("SVM", format!("resolving chain '{chain_name}': {e}"))
            })?;

        if let Some(rpc) = rpc_override {
            chain.rpc_url = rpc.to_string();
        }

        let mailbox = chain.hyperlane_mailbox_program.ok_or_else(|| {
            CliError::chain(
                "SVM",
                format!("no hyperlane_mailbox_program configured for {chain_name}"),
            )
        })?;

        let amount = parse_svm_amount(amount_str, token.decimals)?;

        let solana_signer = self.keyring.get_solana_signer(from_key)?;
        let from_address =
            bs58::encode(solana_signer.public_key_bytes()).into_string();
        let recipient_bytes =
            resolve_recipient(recipient, from_key, self.keyring, false)?;

        let mut keypair_bytes = [0u8; 64];
        keypair_bytes[..32].copy_from_slice(&solana_signer.private_key_bytes());
        keypair_bytes[32..].copy_from_slice(&solana_signer.public_key_bytes());
        let keypair = Keypair::try_from(keypair_bytes.as_slice())
            .map_err(|e| CliError::chain("SVM", format!("keypair: {e}")))?;

        let provider =
            morpheum_sdk_svm::provider::build_provider(&chain.rpc_url, keypair)
                .map_err(|e| CliError::chain("SVM", format!("provider: {e}")))?;

        match token.token_type {
            SvmTokenType::Native => {
                let warp_route =
                    chain.native_warp_route_program.ok_or_else(|| {
                        CliError::chain(
                            "SVM",
                            format!(
                                "no native_warp_route_program configured for \
                                 {chain_name}"
                            ),
                        )
                    })?;

                self.output.info(format!(
                    "SVM deposit (native)\n\
                     From: {from_address}\n\
                     Chain: {chain_name} (RPC: {})\n\
                     Token: {token_symbol} (native)\n\
                     Native Warp Route: {warp_route}\n\
                     Amount: {amount} lamports\n\
                     Destination domain: {destination_domain}\n\
                     Recipient: 0x{}",
                    chain.rpc_url,
                    hex::encode(recipient_bytes),
                ));

                self.output.info("Calling transfer_remote_native...");
                let result = morpheum_sdk_svm::bridge::transfer_remote_native(
                    &provider,
                    &warp_route,
                    &mailbox,
                    destination_domain,
                    recipient_bytes,
                    amount,
                )
                .map_err(|e| {
                    CliError::chain("SVM", format!("transfer_remote_native: {e}"))
                })?;

                Ok(SvmDepositResult {
                    signature: result.signature.to_string(),
                    message_id: hex::encode(result.message_id),
                    message_storage_pda: result.message_storage_pda.to_string(),
                    amount_display: amount_str.to_string(),
                    token: token_symbol.to_string(),
                    destination_domain,
                })
            }
            SvmTokenType::Spl => {
                let warp_route = chain.warp_route_program.ok_or_else(|| {
                    CliError::chain(
                        "SVM",
                        format!(
                            "no warp_route_program configured for {chain_name}"
                        ),
                    )
                })?;

                self.output.info(format!(
                    "SVM deposit (SPL)\n\
                     From: {from_address}\n\
                     Chain: {chain_name} (RPC: {})\n\
                     Token: {token_symbol} (mint: {})\n\
                     Warp Route: {warp_route}\n\
                     Amount: {amount}\n\
                     Destination domain: {destination_domain}\n\
                     Recipient: 0x{}",
                    chain.rpc_url,
                    token.mint,
                    hex::encode(recipient_bytes),
                ));

                self.output.info("Calling transfer_remote...");
                let result = morpheum_sdk_svm::bridge::transfer_remote(
                    &provider,
                    &warp_route,
                    &mailbox,
                    &token.mint,
                    destination_domain,
                    recipient_bytes,
                    amount,
                )
                .map_err(|e| {
                    CliError::chain("SVM", format!("transfer_remote: {e}"))
                })?;

                Ok(SvmDepositResult {
                    signature: result.signature.to_string(),
                    message_id: hex::encode(result.message_id),
                    message_storage_pda: result.message_storage_pda.to_string(),
                    amount_display: amount_str.to_string(),
                    token: token_symbol.to_string(),
                    destination_domain,
                })
            }
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

/// Resolves a 32-byte recipient from either an explicit hex string or the
/// keyring-derived Morpheum address.
pub fn resolve_recipient(
    explicit: Option<&str>,
    key_name: &str,
    keyring: &KeyringManager,
    allow_20_byte: bool,
) -> Result<[u8; 32], CliError> {
    let raw = match explicit {
        Some(hex_str) => {
            let s = hex_str.strip_prefix("0x").unwrap_or(hex_str);
            hex::decode(s).map_err(|e| {
                CliError::invalid_input(format!("invalid recipient hex: {e}"))
            })?
        }
        None => {
            use morpheum_signing_native::signer::Signer;
            let native = keyring.get_native_signer(key_name)?;
            native.account_id().0.to_vec()
        }
    };

    let mut fixed = [0u8; 32];
    if raw.len() == 32 {
        fixed.copy_from_slice(&raw);
    } else if raw.len() == 20 && allow_20_byte {
        fixed[12..].copy_from_slice(&raw);
    } else if allow_20_byte {
        return Err(CliError::invalid_input(
            "recipient must be 20 or 32 bytes",
        ));
    } else {
        return Err(CliError::invalid_input(
            "recipient must be exactly 32 bytes",
        ));
    }
    Ok(fixed)
}

/// Parses a human-readable amount to an on-chain `U256` using the token's
/// decimal precision (e.g. 6 decimals: `"100"` -> `100_000_000`).
pub fn parse_token_amount(
    amount_str: &str,
    decimals: u8,
) -> Result<U256, CliError> {
    let parts: Vec<&str> = amount_str.split('.').collect();
    let (whole, frac) = match parts.len() {
        1 => (parts[0], ""),
        2 => (parts[0], parts[1]),
        _ => return Err(CliError::invalid_input("invalid amount format")),
    };

    let whole_val: u128 = whole
        .parse()
        .map_err(|e| CliError::invalid_input(format!("invalid amount: {e}")))?;

    let frac_len = frac.len();
    if frac_len > decimals as usize {
        return Err(CliError::invalid_input(format!(
            "amount has {frac_len} fractional digits but token only supports \
             {decimals}"
        )));
    }

    let frac_val: u128 = if frac.is_empty() {
        0
    } else {
        frac.parse().map_err(|e| {
            CliError::invalid_input(format!("invalid fractional part: {e}"))
        })?
    };

    let scale = 10u128.pow(decimals as u32);
    let frac_scale = 10u128.pow((decimals as u32) - (frac_len as u32));
    let raw = whole_val * scale + frac_val * frac_scale;

    Ok(U256::from(raw))
}

/// Parses a human-readable amount to a raw `u64` for SVM chains.
pub fn parse_svm_amount(
    amount_str: &str,
    decimals: u8,
) -> Result<u64, CliError> {
    let parts: Vec<&str> = amount_str.split('.').collect();
    let (whole, frac) = match parts.len() {
        1 => (parts[0], ""),
        2 => (parts[0], parts[1]),
        _ => return Err(CliError::invalid_input("invalid amount format")),
    };

    let whole_val: u64 = whole
        .parse()
        .map_err(|e| CliError::invalid_input(format!("invalid amount: {e}")))?;

    let frac_len = frac.len();
    if frac_len > decimals as usize {
        return Err(CliError::invalid_input(format!(
            "amount has {frac_len} fractional digits but token only supports \
             {decimals}"
        )));
    }

    let frac_val: u64 = if frac.is_empty() {
        0
    } else {
        frac.parse().map_err(|e| {
            CliError::invalid_input(format!("invalid fractional part: {e}"))
        })?
    };

    let scale = 10u64.pow(decimals as u32);
    let frac_scale = 10u64.pow((decimals as u32) - (frac_len as u32));
    let raw = whole_val
        .checked_mul(scale)
        .and_then(|v| v.checked_add(frac_val.checked_mul(frac_scale)?))
        .ok_or_else(|| CliError::invalid_input("amount overflow"))?;

    Ok(raw)
}

/// Resolves a Morpheum-side warp route contract and Hyperlane domain from
/// chain registries.
pub fn resolve_warp_target(
    chain_type: &ChainType,
    chain_name: &str,
    token_symbol: &str,
    explicit_domain: Option<u32>,
) -> Result<(String, u32), CliError> {
    match chain_type {
        ChainType::Evm => {
            let registry = ChainRegistry::load_with_defaults(
                morpheum_sdk_evm::DEFAULT_CHAINS_TOML,
            )
            .map_err(|e| CliError::chain("EVM", format!("chain registry: {e}")))?;
            let (chain, token) = registry
                .resolve(chain_name, token_symbol)
                .map_err(|e| CliError::chain("EVM", format!("{e}")))?;
            let warp = token.morpheum_warp_route.ok_or_else(|| {
                CliError::chain(
                    "EVM",
                    format!(
                        "no morpheum_warp_route for {token_symbol} on \
                         {chain_name}"
                    ),
                )
            })?;
            Ok((warp, explicit_domain.unwrap_or(chain.hyperlane_domain)))
        }
        ChainType::Svm => {
            let registry = SolanaChainRegistry::load_with_defaults(
                morpheum_sdk_svm::DEFAULT_CHAINS_TOML,
            )
            .map_err(|e| CliError::chain("SVM", format!("chain registry: {e}")))?;
            let (chain, token) = registry
                .resolve(chain_name, token_symbol)
                .map_err(|e| CliError::chain("SVM", format!("{e}")))?;
            let warp = token.morpheum_warp_route.ok_or_else(|| {
                CliError::chain(
                    "SVM",
                    format!(
                        "no morpheum_warp_route for {token_symbol} on \
                         {chain_name}"
                    ),
                )
            })?;
            Ok((warp, explicit_domain.unwrap_or(chain.hyperlane_domain)))
        }
    }
}
