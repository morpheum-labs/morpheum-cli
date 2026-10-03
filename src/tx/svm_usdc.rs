//! Transaction commands for the SVM USDC native program on Morpheum.
//!
//! Submits `MsgExecute` targeting the USDC native program via the standard
//! `IngressService/SubmitTx` gRPC endpoint.

use clap::{Args, Subcommand};
use morpheum_sdk_svm::usdc;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::utils::TxMessage;
use morpheum_signing_native::TxGasLimit;

/// The compute limit every USDC program call the CLI builds states for
/// itself: the SDK's default. Its transaction declares at least this much
/// (see [`TxMessage::declared_gas_limit`]). A constant, so an SDK default no
/// transaction could declare fails the build rather than a submission.
const COMPUTE_LIMIT: TxGasLimit = match TxGasLimit::new(usdc::DEFAULT_COMPUTE_LIMIT) {
    Ok(limit) => limit,
    Err(_) => panic!("the SDK's USDC compute limit must be a declarable gas limit"),
};

/// A USDC program call from `sender`, stating [`COMPUTE_LIMIT`] both in the
/// message and as the gas its transaction must declare at least. The query
/// commands that read through a transaction build theirs here too.
pub fn usdc_call(
    sender: &str,
    instruction_data: Vec<u8>,
    accounts: Vec<usdc::AccountMeta>,
) -> Result<TxMessage, CliError> {
    usdc::build_usdc_execute(sender, instruction_data, accounts, COMPUTE_LIMIT.get())
        .map(|any| TxMessage::with_own_gas_limit(any, COMPUTE_LIMIT))
        .map_err(|e| CliError::internal(format!("build MsgExecute: {e}")))
}

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
    let sender = crate::account::id_hex(&signer);

    let msg = usdc_call(
        &sender,
        usdc::encode_transfer(args.amount),
        vec![
            usdc::AccountMeta::writable(&sender),
            usdc::AccountMeta::writable(&args.to),
        ],
    )?;

    let txhash = crate::utils::sign_and_broadcast(signer, dispatcher, msg, None).await?;

    dispatcher.output.success(format!(
        "SVM USDC Transfer\n To: {}\n Amount: {}\n TxHash: {txhash}",
        args.to, args.amount,
    ));
    Ok(())
}

async fn approve(args: ApproveArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from_key)?;
    let owner = crate::account::id_hex(&signer);

    let msg = usdc_call(
        &owner,
        usdc::encode_approve(args.amount),
        vec![
            usdc::AccountMeta::writable(&owner),
            usdc::AccountMeta::readonly(&args.spender),
        ],
    )?;

    let txhash = crate::utils::sign_and_broadcast(signer, dispatcher, msg, None).await?;

    dispatcher.output.success(format!(
        "SVM USDC Approve\n Spender: {}\n Amount: {}\n TxHash: {txhash}",
        args.spender, args.amount,
    ));
    Ok(())
}

async fn transfer_from(args: TransferFromArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from_key)?;
    let spender = crate::account::id_hex(&signer);

    let msg = usdc_call(
        &spender,
        usdc::encode_transfer_from(args.amount),
        vec![
            usdc::AccountMeta::writable(&args.from),
            usdc::AccountMeta::writable(&args.to),
        ],
    )?;

    let txhash = crate::utils::sign_and_broadcast(signer, dispatcher, msg, None).await?;

    dispatcher.output.success(format!(
        "SVM USDC TransferFrom\n From: {}\n To: {}\n Amount: {}\n TxHash: {txhash}",
        args.from, args.to, args.amount,
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A USDC program call states [`COMPUTE_LIMIT`] in the message the chain
    /// runs, and its transaction declares at least that limit: unset, never
    /// less; `--gas-limit` at the limit is declared as given, and one below
    /// it is refused.
    #[test]
    fn a_usdc_call_declares_at_least_the_compute_limit_it_states() {
        let below = TxGasLimit::new(COMPUTE_LIMIT.get() - 1).expect("in range");
        let call = usdc_call("00", usdc::encode_transfer(1), Vec::new()).expect("builds");

        assert!(call.declared_gas_limit(None).expect("unset never refuses") >= COMPUTE_LIMIT);
        assert_eq!(
            call.declared_gas_limit(Some(COMPUTE_LIMIT))
                .expect("at the limit"),
            COMPUTE_LIMIT,
        );
        assert!(call.declared_gas_limit(Some(below)).is_err());

        let stated: serde_json::Value =
            serde_json::from_slice(&call.into_any().value).expect("MsgExecute is JSON");
        assert_eq!(stated["compute_limit"], COMPUTE_LIMIT.get());
    }
}
