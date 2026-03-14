use clap::{Args, Subcommand};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for native x402 autonomous payments (Pillar 2).
///
/// Enables agents to pay and be paid autonomously using the x402
/// protocol (HTTP 402 + signed payment requests). Settlement is
/// native and finalises in one block.
///
/// Note: x402 does not yet have a dedicated SDK crate. These commands
/// construct raw payment messages directly against the bank module's
/// transfer primitives with x402-specific metadata attached.
#[derive(Subcommand)]
pub enum X402Commands {
    /// Execute an x402 payment to another agent or service
    Pay(PayArgs),

    /// Pre-authorise a spending allowance for a service endpoint
    Approve(ApproveArgs),
}

#[derive(Args)]
pub struct PayArgs {
    /// Destination address or agent hash
    pub destination: String,

    /// Amount to pay (in umorph or specified denom)
    pub amount: String,

    /// Payment memo / invoice reference
    #[arg(long)]
    pub invoice: Option<String>,

    /// Asset denomination (default: umorph)
    #[arg(long, default_value = "umorph")]
    pub denom: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct ApproveArgs {
    /// Service endpoint URL or agent hash to authorise
    pub spender: String,

    /// Maximum spend allowance (in umorph or specified denom)
    pub max_amount: String,

    /// Expiry timestamp for the approval (0 = no expiry)
    #[arg(long, default_value = "0")]
    pub expiry: u64,

    /// Asset denomination (default: umorph)
    #[arg(long, default_value = "umorph")]
    pub denom: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

pub async fn execute(cmd: X402Commands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        X402Commands::Pay(args) => pay(args, &dispatcher).await,
        X402Commands::Approve(args) => approve(args, &dispatcher).await,
    }
}

async fn pay(args: PayArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let from_address = signer.account_id().to_string();

    let memo = args
        .invoice
        .as_deref()
        .map_or_else(|| format!("x402-pay:{}", args.destination), |inv| format!("x402-pay:{}:{inv}", args.destination));

    let transfer = morpheum_sdk_native::bank::TransferBuilder::new()
        .from_address(&from_address)
        .to_address(&args.destination)
        .amount(&args.amount)
        .asset_index(0)
        .memo(&memo)
        .build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, transfer.to_any(), Some(memo),
    ).await?;

    dispatcher.output.success(format!(
        "x402 payment sent\nTo: {}\nAmount: {} {}\nTxHash: {}",
        args.destination, args.amount, args.denom, result.txhash,
    ));

    Ok(())
}

async fn approve(args: ApproveArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let from_address = signer.account_id().to_string();

    let memo = format!(
        "x402-approve:{}:{}:{}",
        args.spender, args.max_amount, args.expiry
    );

    let transfer = morpheum_sdk_native::bank::TransferBuilder::new()
        .from_address(&from_address)
        .to_address(&args.spender)
        .amount("0")
        .asset_index(0)
        .memo(&memo)
        .build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, transfer.to_any(), Some(memo),
    ).await?;

    dispatcher.output.success(format!(
        "x402 approval granted\nSpender: {}\nMax: {} {}\nExpiry: {}\nTxHash: {}",
        args.spender, args.max_amount, args.denom, args.expiry, result.txhash,
    ));

    Ok(())
}
