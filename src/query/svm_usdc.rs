//! Query commands for the SVM USDC native program on Morpheum.
//!
//! `BalanceOf` and `Allowance` are read operations that go through the
//! SVM native program. Since `MsgExecute` goes through consensus, these
//! queries submit a transaction and decode the return data. For
//! lightweight balance checks, prefer `morpheum query bank balance`.

use clap::{Args, Subcommand};
use morpheum_sdk_svm::usdc;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// SVM USDC native program query commands.
#[derive(Subcommand)]
pub enum SvmUsdcQueryCommands {
    /// Display the USDC native program ID
    ProgramId,

    /// Query USDC balance via the SVM native program (submits a tx)
    Balance(BalanceArgs),

    /// Query USDC allowance via the SVM native program (submits a tx)
    Allowance(AllowanceArgs),
}

#[derive(Args)]
pub struct BalanceArgs {
    /// Address to query balance for (hex)
    #[arg(long)]
    pub address: String,

    /// Key name to sign the query tx with
    #[arg(long, default_value = "default")]
    pub from_key: String,
}

#[derive(Args)]
pub struct AllowanceArgs {
    /// Owner address (hex)
    #[arg(long)]
    pub owner: String,

    /// Spender address (hex)
    #[arg(long)]
    pub spender: String,

    /// Key name to sign the query tx with
    #[arg(long, default_value = "default")]
    pub from_key: String,
}

pub async fn execute(cmd: SvmUsdcQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        SvmUsdcQueryCommands::ProgramId => program_id(&dispatcher),
        SvmUsdcQueryCommands::Balance(args) => balance(args, &dispatcher).await,
        SvmUsdcQueryCommands::Allowance(args) => allowance(args, &dispatcher).await,
    }
}

#[allow(clippy::unnecessary_wraps)]
fn program_id(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let id = usdc::usdc_program_id();
    dispatcher.output.success(format!(
        "SVM USDC Native Program ID: {id}\n Asset Index: {}",
        usdc::USDC_ASSET_INDEX,
    ));
    Ok(())
}

async fn balance(args: BalanceArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    use morpheum_signing_native::signer::Signer;

    let signer = dispatcher.keyring.get_native_signer(&args.from_key)?;
    let sender = hex::encode(signer.account_id().0);

    let msg = usdc::build_usdc_execute(
        &sender,
        usdc::encode_balance_of(),
        vec![usdc::AccountMeta::readonly(&args.address)],
        usdc::DEFAULT_COMPUTE_LIMIT,
    )
    .map_err(|e| CliError::internal(format!("build MsgExecute: {e}")))?;

    let txhash = crate::utils::sign_and_broadcast(signer, dispatcher, msg, None).await?;

    dispatcher.output.success(format!(
        "SVM USDC BalanceOf submitted (reads go through consensus)\n Address: {}\n TxHash: {txhash}\n\
         Tip: For lightweight balance queries, use: morpheum query bank balance --address {} --asset USDC",
        args.address, args.address,
    ));
    Ok(())
}

async fn allowance(args: AllowanceArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    use morpheum_signing_native::signer::Signer;

    let signer = dispatcher.keyring.get_native_signer(&args.from_key)?;
    let sender = hex::encode(signer.account_id().0);

    let msg = usdc::build_usdc_execute(
        &sender,
        usdc::encode_allowance(),
        vec![
            usdc::AccountMeta::readonly(&args.owner),
            usdc::AccountMeta::readonly(&args.spender),
        ],
        usdc::DEFAULT_COMPUTE_LIMIT,
    )
    .map_err(|e| CliError::internal(format!("build MsgExecute: {e}")))?;

    let txhash = crate::utils::sign_and_broadcast(signer, dispatcher, msg, None).await?;

    dispatcher.output.success(format!(
        "SVM USDC Allowance submitted (reads go through consensus)\n Owner: {}\n Spender: {}\n TxHash: {txhash}",
        args.owner, args.spender,
    ));
    Ok(())
}
