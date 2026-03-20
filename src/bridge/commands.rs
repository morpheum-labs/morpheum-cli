//! Bridge CLI command definitions and handlers.

use clap::{Args, Subcommand, ValueEnum};

use morpheum_signing_native::signer::Signer;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Supported external chain types for bridge operations.
#[derive(Clone, Debug, ValueEnum)]
pub enum ChainType {
    /// Ethereum / EVM-compatible chain
    Evm,
    /// Solana / SVM-compatible chain
    Svm,
}

/// Bridge commands for cross-chain token transfers via Hyperlane Warp Routes.
#[derive(Subcommand)]
pub enum BridgeCommands {
    /// Deposit tokens from an external chain (EVM/SVM) to Morpheum
    Deposit(DepositArgs),

    /// Withdraw tokens from Morpheum to an external chain (EVM/SVM)
    Withdraw(WithdrawArgs),

    /// Check the status of a bridge transfer
    Status(StatusArgs),
}

#[derive(Args)]
pub struct DepositArgs {
    /// External chain type
    #[arg(long, value_enum)]
    pub chain: ChainType,

    /// Chain name (e.g. "ethereum", "solana", "arbitrum", "devnet")
    #[arg(long)]
    pub chain_name: Option<String>,

    /// Token symbol to deposit (e.g. "USDC", "WETH")
    #[arg(long)]
    pub token: String,

    /// Amount to deposit
    #[arg(long)]
    pub amount: String,

    /// 32-byte hex recipient address on Morpheum (defaults to sender's Morpheum address)
    #[arg(long)]
    pub recipient: Option<String>,

    /// Morpheum Hyperlane domain ID
    #[arg(long, default_value = "1836016741")]
    pub destination_domain: u32,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct WithdrawArgs {
    /// External chain type for the destination
    #[arg(long, value_enum)]
    pub chain: ChainType,

    /// Hyperlane domain ID of the destination chain
    #[arg(long)]
    pub destination_domain: u32,

    /// 32-byte hex recipient address on the destination chain
    #[arg(long)]
    pub recipient: String,

    /// Bank asset index of the token to transfer
    #[arg(long)]
    pub asset_index: u64,

    /// Amount to withdraw
    #[arg(long)]
    pub amount: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct StatusArgs {
    /// Chain type to query
    #[arg(long, value_enum)]
    pub chain: ChainType,

    /// Transaction hash to look up
    #[arg(long)]
    pub tx_hash: String,

    /// Chain name or RPC URL (for external chain queries)
    #[arg(long)]
    pub chain_name: Option<String>,
}

pub async fn execute(cmd: BridgeCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        BridgeCommands::Deposit(args) => deposit(args, &dispatcher).await,
        BridgeCommands::Withdraw(args) => withdraw(args, &dispatcher).await,
        BridgeCommands::Status(args) => status(args, &dispatcher).await,
    }
}

// ── Deposit (External -> Morpheum) ──────────────────────────────────

async fn deposit(args: DepositArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    match args.chain {
        ChainType::Evm => deposit_evm(args, dispatcher).await,
        ChainType::Svm => deposit_svm(args, dispatcher).await,
    }
}

async fn deposit_evm(args: DepositArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let chain_name = args.chain_name.as_deref().unwrap_or("ethereum");

    let registry =
        morpheum_sdk_evm::config::ChainRegistry::load_with_defaults(
            morpheum_sdk_evm::DEFAULT_CHAINS_TOML,
        )
        .map_err(|e| CliError::Evm(format!("chain registry: {e}")))?;

    let (chain, _token) = registry
        .resolve(chain_name, &args.token)
        .map_err(|e| CliError::Evm(format!("resolving chain '{chain_name}': {e}")))?;

    let alloy_signer = dispatcher.keyring.get_evm_signer(&args.from)?;
    let from_address = format!("{:#x}", morpheum_sdk_evm::alloy::signers::Signer::address(&alloy_signer));

    let recipient_bytes = match &args.recipient {
        Some(hex_str) => {
            let s = hex_str.strip_prefix("0x").unwrap_or(hex_str);
            hex::decode(s)
                .map_err(|e| CliError::invalid_input(format!("invalid recipient hex: {e}")))?
        }
        None => {
            let native = dispatcher.keyring.get_native_signer(&args.from)?;
            native.account_id().0.to_vec()
        }
    };

    let mut recipient_fixed = [0u8; 32];
    if recipient_bytes.len() == 32 {
        recipient_fixed.copy_from_slice(&recipient_bytes);
    } else if recipient_bytes.len() == 20 {
        recipient_fixed[12..].copy_from_slice(&recipient_bytes);
    } else {
        return Err(CliError::invalid_input("recipient must be 20 or 32 bytes"));
    }

    dispatcher.output.success(format!(
        "EVM bridge deposit prepared\n\
         From: {from_address}\n\
         Chain: {chain_name}\n\
         RPC: {}\n\
         Destination domain: {}\n\
         Recipient: 0x{}\n\
         Token: {}\n\
         Amount: {}\n\n\
         To complete, call the Warp Route contract's transferRemote() with the above parameters.",
        chain.rpc_url,
        args.destination_domain,
        hex::encode(recipient_fixed),
        args.token,
        args.amount,
    ));

    Ok(())
}

async fn deposit_svm(args: DepositArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let chain_name = args.chain_name.as_deref().unwrap_or("solana");

    let registry =
        morpheum_sdk_svm::config::SolanaChainRegistry::load_with_defaults(
            morpheum_sdk_svm::DEFAULT_CHAINS_TOML,
        )
        .map_err(|e| CliError::Svm(format!("chain registry: {e}")))?;

    let (chain, _token) = registry
        .resolve(chain_name, &args.token)
        .map_err(|e| CliError::Svm(format!("resolving chain '{chain_name}': {e}")))?;

    let solana_signer = dispatcher.keyring.get_solana_signer(&args.from)?;
    let from_address = bs58::encode(solana_signer.public_key_bytes()).into_string();

    let recipient_bytes = match &args.recipient {
        Some(hex_str) => {
            let s = hex_str.strip_prefix("0x").unwrap_or(hex_str);
            hex::decode(s)
                .map_err(|e| CliError::invalid_input(format!("invalid recipient hex: {e}")))?
        }
        None => {
            let native = dispatcher.keyring.get_native_signer(&args.from)?;
            native.account_id().0.to_vec()
        }
    };

    if recipient_bytes.len() != 32 {
        return Err(CliError::invalid_input("recipient must be exactly 32 bytes"));
    }

    dispatcher.output.success(format!(
        "SVM bridge deposit prepared\n\
         From: {from_address}\n\
         Chain: {chain_name}\n\
         RPC: {}\n\
         Destination domain: {}\n\
         Recipient: 0x{}\n\
         Token: {}\n\
         Amount: {}\n\n\
         To complete, submit a Warp Route transfer_remote instruction to Solana.",
        chain.rpc_url,
        args.destination_domain,
        hex::encode(&recipient_bytes),
        args.token,
        args.amount,
    ));

    Ok(())
}

// ── Withdraw (Morpheum -> External) ─────────────────────────────────

#[cfg(feature = "_transport")]
async fn withdraw(args: WithdrawArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    use morpheum_sdk_native::gmp::WarpRouteTransferBuilder;

    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let from_address = hex::encode(signer.account_id().0);

    let recipient_bytes = {
        let s = args.recipient.strip_prefix("0x").unwrap_or(&args.recipient);
        hex::decode(s)
            .map_err(|e| CliError::invalid_input(format!("invalid recipient hex: {e}")))?
    };

    let chain_label = match args.chain {
        ChainType::Evm => "EVM",
        ChainType::Svm => "SVM",
    };

    let request = WarpRouteTransferBuilder::new()
        .sender(from_address)
        .destination_domain(args.destination_domain)
        .recipient(recipient_bytes)
        .asset_index(args.asset_index)
        .amount(&args.amount)
        .build()
        .map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer,
        dispatcher,
        request.to_any(),
        None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Warp Route withdrawal submitted ({chain_label})\n\
         Destination domain: {}\n\
         Amount: {} (asset {})\n\
         TxHash: {txhash}",
        args.destination_domain, args.amount, args.asset_index,
    ));

    Ok(())
}

#[cfg(not(feature = "_transport"))]
async fn withdraw(_args: WithdrawArgs, _dispatcher: &Dispatcher) -> Result<(), CliError> {
    Err(CliError::invalid_input(
        "withdraw requires transport support — enable the gmp or interop feature",
    ))
}

// ── Status ──────────────────────────────────────────────────────────

async fn status(args: StatusArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let chain_label = match args.chain {
        ChainType::Evm => "EVM",
        ChainType::Svm => "SVM",
    };

    dispatcher.output.success(format!(
        "Bridge status lookup ({chain_label})\n\
         TxHash: {}\n\
         Chain: {}\n\n\
         Status: pending (detailed lookup requires indexer integration — coming soon)",
        args.tx_hash,
        args.chain_name.as_deref().unwrap_or("default"),
    ));

    Ok(())
}
