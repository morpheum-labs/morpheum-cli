//! Transaction commands for the SVM USDC native program on Morpheum.
//!
//! Submits `MsgExecute` targeting the USDC native program via the standard
//! `IngressService/SubmitTx` gRPC endpoint.

use clap::{Args, Subcommand};
use morpheum_signing_native::signer::Signer;
use morpheum_sdk_svm::usdc;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// SVM USDC native program transaction commands.
#[derive(Subcommand)]
pub enum SvmUsdcCommands {
    /// Transfer USDC via the SVM native program
    Transfer(TransferArgs),

    /// Approve a spender to use USDC via the SVM native program
    Approve(ApproveArgs),

    /// Transfer USDC from another account (requires prior approval)
    TransferFrom(TransferFromArgs),
}

#[derive(Args)]
pub struct TransferArgs {
    /// Recipient address (hex)
    #[arg(long)]
    pub to: String,

    /// Amount in smallest unit (e.g. 1000000 = 1 USDC)
    #[arg(long)]
    pub amount: u128,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from_key: String,
}

#[derive(Args)]
pub struct ApproveArgs {
    /// Spender address (hex)
    #[arg(long)]
    pub spender: String,

    /// Allowance amount in smallest unit
    #[arg(long)]
    pub amount: u128,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from_key: String,
}

#[derive(Args)]
pub struct TransferFromArgs {
    /// Source address to transfer from (hex)
    #[arg(long)]
    pub from: String,

    /// Destination address (hex)
    #[arg(long)]
    pub to: String,

    /// Amount in smallest unit
    #[arg(long)]
    pub amount: u128,

    /// Key name to sign with (must be the approved spender)
    #[arg(long, default_value = "default")]
    pub from_key: String,
}

pub async fn execute(cmd: SvmUsdcCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        SvmUsdcCommands::Transfer(args) => transfer(args, &dispatcher).await,
        SvmUsdcCommands::Approve(args) => approve(args, &dispatcher).await,
        SvmUsdcCommands::TransferFrom(args) => transfer_from(args, &dispatcher).await,
    }
}

async fn transfer(args: TransferArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from_key)?;
    let sender = hex::encode(signer.account_id().0);

    let msg = usdc::build_usdc_execute(
        &sender,
        usdc::encode_transfer(args.amount),
        vec![
            usdc::AccountMeta::writable(&sender),
            usdc::AccountMeta::writable(&args.to),
        ],
        usdc::DEFAULT_COMPUTE_LIMIT,
    )
    .map_err(|e| CliError::internal(format!("build MsgExecute: {e}")))?;

    let txhash = crate::utils::sign_and_broadcast(signer, dispatcher, msg, None).await?;

    dispatcher.output.success(format!(
        "SVM USDC Transfer\n To: {}\n Amount: {}\n TxHash: {txhash}",
        args.to, args.amount,
    ));
    Ok(())
}

async fn approve(args: ApproveArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from_key)?;
    let owner = hex::encode(signer.account_id().0);

    let msg = usdc::build_usdc_execute(
        &owner,
        usdc::encode_approve(args.amount),
        vec![
            usdc::AccountMeta::writable(&owner),
            usdc::AccountMeta::readonly(&args.spender),
        ],
        usdc::DEFAULT_COMPUTE_LIMIT,
    )
    .map_err(|e| CliError::internal(format!("build MsgExecute: {e}")))?;

    let txhash = crate::utils::sign_and_broadcast(signer, dispatcher, msg, None).await?;

    dispatcher.output.success(format!(
        "SVM USDC Approve\n Spender: {}\n Amount: {}\n TxHash: {txhash}",
        args.spender, args.amount,
    ));
    Ok(())
}

async fn transfer_from(args: TransferFromArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from_key)?;
    let spender = hex::encode(signer.account_id().0);

    let msg = usdc::build_usdc_execute(
        &spender,
        usdc::encode_transfer_from(args.amount),
        vec![
            usdc::AccountMeta::writable(&args.from),
            usdc::AccountMeta::writable(&args.to),
        ],
        usdc::DEFAULT_COMPUTE_LIMIT,
    )
    .map_err(|e| CliError::internal(format!("build MsgExecute: {e}")))?;

    let txhash = crate::utils::sign_and_broadcast(signer, dispatcher, msg, None).await?;

    dispatcher.output.success(format!(
        "SVM USDC TransferFrom\n From: {}\n To: {}\n Amount: {}\n TxHash: {txhash}",
        args.from, args.to, args.amount,
    ));
    Ok(())
}
