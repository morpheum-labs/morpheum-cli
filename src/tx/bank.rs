use clap::{Args, Subcommand};

use morpheum_sdk_native::bank::{
    TransferBuilder, CrossChainTransferBuilder, TransferToBucketBuilder,
    MintBuilder, OnboardAssetBuilder, WithdrawBuilder,
};
use morpheum_sdk_native::bank::types::{AssetIdentifier, ChainType};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `bank` module.
///
/// Covers native transfers, cross-chain transfers, bucket margin deposits,
/// minting, asset onboarding, and withdrawals.
#[derive(Subcommand)]
pub enum BankCommands {
    /// Transfer native assets between accounts
    Send(SendArgs),

    /// Transfer assets cross-chain (Ethereum, Solana, Bitcoin)
    CrossChainSend(CrossChainSendArgs),

    /// Transfer assets into a perpetuals margin bucket
    TransferToBucket(TransferToBucketArgs),

    /// Mint new assets (restricted to authorised module accounts)
    Mint(MintArgs),

    /// Onboard a new asset type to the chain
    OnboardAsset(OnboardAssetArgs),

    /// Withdraw assets to an external chain
    Withdraw(WithdrawArgs),
}

#[derive(Args)]
pub struct SendArgs {
    /// Recipient address (morpheum1...)
    pub to: String,

    /// Amount to send (e.g. "1000")
    pub amount: String,

    /// Asset index in the registry
    #[arg(long, default_value = "0")]
    pub asset_index: u64,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct CrossChainSendArgs {
    /// Target chain (ethereum, solana, bitcoin)
    #[arg(long, value_parser = parse_chain_type)]
    pub target_chain: ChainType,

    /// Destination address on the target chain
    pub to: String,

    /// Amount to send
    pub amount: String,

    /// Asset identifier — index or symbol (e.g. "0" or "MORM")
    #[arg(long)]
    pub asset: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct TransferToBucketArgs {
    /// Bucket ID for perpetuals margin
    #[arg(long)]
    pub bucket_id: String,

    /// Asset index
    #[arg(long, default_value = "0")]
    pub asset_index: u64,

    /// Amount to deposit into the bucket
    pub amount: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct MintArgs {
    /// Recipient address
    pub recipient: String,

    /// Asset index
    #[arg(long, default_value = "0")]
    pub asset_index: u64,

    /// Amount to mint
    pub amount: String,

    /// Module account authorising the mint
    #[arg(long)]
    pub module_account: Option<String>,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct OnboardAssetArgs {
    /// Display name for the asset
    #[arg(long)]
    pub name: String,

    /// Ticker symbol (e.g. "MORM")
    #[arg(long)]
    pub symbol: String,

    /// Asset type (0 = native, 1 = bridged, 2 = synthetic)
    #[arg(long)]
    pub asset_type: i32,

    /// Initial supply (optional, defaults to "0")
    #[arg(long, default_value = "0")]
    pub initial_supply: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct WithdrawArgs {
    /// Destination chain (ethereum, solana, bitcoin)
    #[arg(long, value_parser = parse_chain_type)]
    pub destination_chain: ChainType,

    /// Destination address on the external chain
    pub destination_address: String,

    /// Amount to withdraw
    pub amount: String,

    /// Asset identifier — index or symbol
    #[arg(long)]
    pub asset: String,

    /// Use fast withdrawal (higher fee, instant finality)
    #[arg(long)]
    pub fast: bool,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

pub async fn execute(cmd: BankCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        BankCommands::Send(args) => send(args, &dispatcher).await,
        BankCommands::CrossChainSend(args) => cross_chain_send(args, &dispatcher).await,
        BankCommands::TransferToBucket(args) => transfer_to_bucket(args, &dispatcher).await,
        BankCommands::Mint(args) => mint(args, &dispatcher).await,
        BankCommands::OnboardAsset(args) => onboard_asset(args, &dispatcher).await,
        BankCommands::Withdraw(args) => withdraw(args, &dispatcher).await,
    }
}

async fn send(args: SendArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let from_address = signer.account_id().to_string();

    let request = TransferBuilder::new()
        .from_address(&from_address)
        .to_address(&args.to)
        .amount(&args.amount)
        .asset_index(args.asset_index)
        .memo(args.memo.as_deref().unwrap_or_default())
        .build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Transfer complete\nTo: {}\nAmount: {} (asset {})\nTxHash: {}",
        args.to, args.amount, args.asset_index, result.txhash,
    ));

    Ok(())
}

async fn cross_chain_send(
    args: CrossChainSendArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let from_address = signer.account_id().to_string();

    let asset = parse_asset_identifier(&args.asset);

    let request = CrossChainTransferBuilder::new()
        .from_address(&from_address)
        .target_chain(args.target_chain)
        .to_address(&args.to)
        .amount(&args.amount)
        .asset(asset)
        .memo(args.memo.as_deref().unwrap_or_default())
        .build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Cross-chain transfer initiated\nTo: {} on {:?}\nAmount: {}\nTxHash: {}",
        args.to, args.target_chain, args.amount, result.txhash,
    ));

    Ok(())
}

async fn transfer_to_bucket(
    args: TransferToBucketArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let address = signer.account_id().to_string();

    let request = TransferToBucketBuilder::new()
        .address(&address)
        .bucket_id(&args.bucket_id)
        .asset_index(args.asset_index)
        .amount(&args.amount)
        .build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Deposited {} (asset {}) into bucket {}\nTxHash: {}",
        args.amount, args.asset_index, args.bucket_id, result.txhash,
    ));

    Ok(())
}

async fn mint(args: MintArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;

    let mut builder = MintBuilder::new()
        .recipient_address(&args.recipient)
        .asset_index(args.asset_index)
        .amount(&args.amount);

    if let Some(ref module) = args.module_account {
        builder = builder.module_account(module);
    }

    let request = builder.build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Minted {} (asset {}) to {}\nTxHash: {}",
        args.amount, args.asset_index, args.recipient, result.txhash,
    ));

    Ok(())
}

async fn onboard_asset(args: OnboardAssetArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let from_address = signer.account_id().to_string();

    let request = OnboardAssetBuilder::new()
        .from_address(&from_address)
        .name(&args.name)
        .asset_symbol(&args.symbol)
        .asset_type(args.asset_type)
        .initial_supply(&args.initial_supply)
        .build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Asset onboarded: {} ({})\nType: {}, Supply: {}\nTxHash: {}",
        args.name, args.symbol, args.asset_type, args.initial_supply, result.txhash,
    ));

    Ok(())
}

async fn withdraw(args: WithdrawArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let from_address = signer.account_id().to_string();

    let asset = parse_asset_identifier(&args.asset);

    let request = WithdrawBuilder::new()
        .from_address(&from_address)
        .asset(asset)
        .amount(&args.amount)
        .destination_chain(args.destination_chain)
        .destination_address(&args.destination_address)
        .fast_withdrawal(args.fast)
        .build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Withdrawal initiated to {} on {:?}\nAmount: {}{}\nTxHash: {}",
        args.destination_address,
        args.destination_chain,
        args.amount,
        if args.fast { " (fast)" } else { "" },
        result.txhash,
    ));

    Ok(())
}

fn parse_chain_type(s: &str) -> Result<ChainType, String> {
    match s.to_lowercase().as_str() {
        "ethereum" | "eth" => Ok(ChainType::Ethereum),
        "solana" | "sol" => Ok(ChainType::Solana),
        "bitcoin" | "btc" => Ok(ChainType::Bitcoin),
        other => Err(format!(
            "unknown chain '{other}'; expected: ethereum, solana, bitcoin"
        )),
    }
}

fn parse_asset_identifier(s: &str) -> AssetIdentifier {
    match s.parse::<u64>() {
        Ok(idx) => AssetIdentifier::index(idx),
        Err(_) => AssetIdentifier::symbol(s),
    }
}
