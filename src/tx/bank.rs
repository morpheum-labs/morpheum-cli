use clap::{Args, Subcommand};

use morpheum_signing_native::signer::Signer;
use morpheum_sdk_native::bank::{
    TransferBuilder, TransferToBucketBuilder,
    MintBuilder, OnboardAssetBuilder,
};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::xchain::{ChainSpec, ChainType, CrossChainExecutor, resolve_warp_target};

/// Transaction commands for the `bank` module.
///
/// Covers local transfers, cross-chain deposits/withdrawals via Hyperlane
/// Warp Routes, bucket margin deposits, minting, and asset onboarding.
#[derive(Subcommand)]
pub enum BankCommands {
    /// Transfer native assets between accounts on Morpheum
    Send(SendArgs),

    /// Deposit tokens from an external chain (EVM/SVM) to Morpheum via Hyperlane
    Deposit(DepositArgs),

    /// Withdraw tokens from Morpheum to an external chain via Hyperlane Warp Route
    Withdraw(WithdrawArgs),

    /// Transfer assets into a perpetuals margin bucket
    TransferToBucket(TransferToBucketArgs),

    /// Mint new assets (restricted to authorised module accounts)
    Mint(MintArgs),

    /// Onboard a new asset type to the chain
    OnboardAsset(OnboardAssetArgs),
}

// ── Send (local) ────────────────────────────────────────────────────

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

// ── Deposit (external chain -> Morpheum) ────────────────────────────

#[derive(Args)]
pub struct DepositArgs {
    /// External chain (format: "evm:sepolia", "svm:devnet")
    #[arg(long)]
    pub chain: String,

    /// Token symbol (e.g. "USDC", "ETH", "SOL")
    #[arg(long)]
    pub token: String,

    /// Amount (human-readable, e.g. "100" or "0.05")
    #[arg(long)]
    pub amount: String,

    /// 32-byte hex recipient address on Morpheum (defaults to sender's address)
    #[arg(long)]
    pub recipient: Option<String>,

    /// Morpheum Hyperlane domain ID
    #[arg(long, default_value_t = morpheum_primitives::constants::hyperlane::MORPHEUM_DOMAIN)]
    pub destination_domain: u32,

    /// Override the external chain's RPC URL (instead of the SDK registry default)
    #[arg(long)]
    pub chain_rpc: Option<String>,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

// ── Withdraw (Morpheum -> external chain) ───────────────────────────

#[derive(Args)]
pub struct WithdrawArgs {
    /// External chain (format: "evm:sepolia", "svm:devnet")
    #[arg(long)]
    pub chain: String,

    /// Token symbol (e.g. "USDC", "ETH", "SOL")
    #[arg(long)]
    pub token: String,

    /// Amount (human-readable, e.g. "100" or "0.05")
    #[arg(long)]
    pub amount: String,

    /// Recipient address on the destination chain (hex)
    #[arg(long)]
    pub recipient: String,

    /// Explicit warp route contract (overrides registry lookup)
    #[arg(long)]
    pub warp_route_contract: Option<String>,

    /// Explicit destination domain (overrides registry lookup)
    #[arg(long)]
    pub destination_domain: Option<u32>,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

// ── TransferToBucket ────────────────────────────────────────────────

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

// ── Mint ────────────────────────────────────────────────────────────

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

// ── OnboardAsset ────────────────────────────────────────────────────

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

// ── Execute ─────────────────────────────────────────────────────────

pub async fn execute(cmd: BankCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        BankCommands::Send(args) => send(args, &dispatcher).await,
        BankCommands::Deposit(args) => deposit(args, &dispatcher).await,
        BankCommands::Withdraw(args) => withdraw(args, &dispatcher).await,
        BankCommands::TransferToBucket(args) => {
            transfer_to_bucket(args, &dispatcher).await
        }
        BankCommands::Mint(args) => mint(args, &dispatcher).await,
        BankCommands::OnboardAsset(args) => {
            onboard_asset(args, &dispatcher).await
        }
    }
}

// ── Send ────────────────────────────────────────────────────────────

async fn send(args: SendArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let from_address = hex::encode(signer.account_id().0);

    let request = TransferBuilder::new()
        .from_address(&from_address)
        .to_address(&args.to)
        .amount(&args.amount)
        .asset_index(args.asset_index)
        .memo(args.memo.as_deref().unwrap_or_default())
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
        "Transfer complete\nTo: {}\nAmount: {} (asset {})\nTxHash: {}",
        args.to, args.amount, args.asset_index, txhash,
    ));

    Ok(())
}

// ── Deposit (external -> Morpheum) ──────────────────────────────────

async fn deposit(
    args: DepositArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let spec = ChainSpec::parse(&args.chain)?;
    let executor = CrossChainExecutor::from_dispatcher(dispatcher);

    match spec.chain_type {
        ChainType::Evm => {
            let result = executor
                .deposit_evm(
                    &spec.network,
                    &args.token,
                    &args.amount,
                    args.recipient.as_deref(),
                    &args.from,
                    args.destination_domain,
                    args.chain_rpc.as_deref(),
                )
                .await?;

            dispatcher.output.success(format!(
                "Deposit submitted (EVM)\n\
                 TxHash: {}\n\
                 MessageID: {}\n\
                 Amount: {} {} -> Morpheum domain {}",
                result.tx_hash,
                result.message_id,
                result.amount_display,
                result.token,
                result.destination_domain,
            ));
        }
        ChainType::Svm => {
            let result = executor.deposit_svm(
                &spec.network,
                &args.token,
                &args.amount,
                args.recipient.as_deref(),
                &args.from,
                args.destination_domain,
                args.chain_rpc.as_deref(),
            )?;

            dispatcher.output.success(format!(
                "Deposit submitted (SVM)\n\
                 Signature: {}\n\
                 MessageID: 0x{}\n\
                 MessageStoragePDA: {}\n\
                 Amount: {} {} -> Morpheum domain {}",
                result.signature,
                result.message_id,
                result.message_storage_pda,
                result.amount_display,
                result.token,
                result.destination_domain,
            ));
        }
    }

    Ok(())
}

// ── Withdraw (Morpheum -> external) ─────────────────────────────────

#[cfg(feature = "_transport")]
async fn withdraw(
    args: WithdrawArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    use morpheum_sdk_cosmwasm::WarpRouteTransferBuilder;

    let spec = ChainSpec::parse(&args.chain)?;

    let (warp_route_contract, destination_domain) =
        if let Some(ref explicit_contract) = args.warp_route_contract {
            let domain = args.destination_domain.ok_or_else(|| {
                CliError::invalid_input(
                    "--destination-domain is required when \
                     --warp-route-contract is used",
                )
            })?;
            (explicit_contract.clone(), domain)
        } else {
            resolve_warp_target(
                &spec.chain_type,
                &spec.network,
                &args.token,
                args.destination_domain,
            )?
        };

    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let acct = signer.account_id().0;
    let from_address = morpheum_primitives::address::encode_address(&acct[acct.len() - 20..]);

    let recipient_bytes = {
        let s = args
            .recipient
            .strip_prefix("0x")
            .unwrap_or(&args.recipient);
        hex::decode(s).map_err(|e| {
            CliError::invalid_input(format!("invalid recipient hex: {e}"))
        })?
    };

    let chain_label = spec.chain_type.label();

    let request = WarpRouteTransferBuilder::new()
        .sender(&from_address)
        .warp_route_contract(&warp_route_contract)
        .destination_domain(destination_domain)
        .recipient(recipient_bytes)
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
        "Withdrawal submitted ({chain_label})\n\
         Contract: {warp_route_contract}\n\
         Destination domain: {destination_domain}\n\
         Amount: {}\n\
         TxHash: {txhash}",
        args.amount,
    ));

    Ok(())
}

#[cfg(not(feature = "_transport"))]
async fn withdraw(
    _args: WithdrawArgs,
    _dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    Err(CliError::invalid_input(
        "withdraw requires transport support -- enable the bank feature",
    ))
}

// ── TransferToBucket ────────────────────────────────────────────────

async fn transfer_to_bucket(
    args: TransferToBucketArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let address = hex::encode(signer.account_id().0);

    let request = TransferToBucketBuilder::new()
        .address(&address)
        .bucket_id(&args.bucket_id)
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
        "Deposited {} (asset {}) into bucket {}\nTxHash: {}",
        args.amount, args.asset_index, args.bucket_id, txhash,
    ));

    Ok(())
}

// ── Mint ────────────────────────────────────────────────────────────

async fn mint(
    args: MintArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;

    let mut builder = MintBuilder::new()
        .recipient_address(&args.recipient)
        .asset_index(args.asset_index)
        .amount(&args.amount);

    if let Some(ref module) = args.module_account {
        builder = builder.module_account(module);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer,
        dispatcher,
        request.to_any(),
        None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Minted {} (asset {}) to {}\nTxHash: {}",
        args.amount, args.asset_index, args.recipient, txhash,
    ));

    Ok(())
}

// ── OnboardAsset ────────────────────────────────────────────────────

async fn onboard_asset(
    args: OnboardAssetArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let from_address = hex::encode(signer.account_id().0);

    let request = OnboardAssetBuilder::new()
        .from_address(&from_address)
        .name(&args.name)
        .asset_symbol(&args.symbol)
        .asset_type(args.asset_type)
        .initial_supply(&args.initial_supply)
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
        "Asset onboarded: {} ({})\nType: {}, Supply: {}\nTxHash: {}",
        args.name, args.symbol, args.asset_type, args.initial_supply, txhash,
    ));

    Ok(())
}
