use clap::{Args, Subcommand};

use morpheum_sdk_native::gmp::WarpRouteTransferBuilder;
use morpheum_signing_native::signer::Signer;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the GMP (General Message Passing) module.
#[derive(Subcommand)]
pub enum GmpCommands {
    /// Initiate a Warp Route transfer (burn on Morpheum, unlock on destination)
    WarpTransfer(WarpTransferArgs),
}

#[derive(Args)]
pub struct WarpTransferArgs {
    /// Hyperlane domain ID of the destination chain
    #[arg(long)]
    pub destination_domain: u32,

    /// 32-byte hex recipient address on the destination chain
    #[arg(long)]
    pub recipient: String,

    /// Bank asset index of the token to transfer
    #[arg(long)]
    pub asset_index: u64,

    /// Amount to transfer
    #[arg(long)]
    pub amount: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

pub async fn execute(cmd: GmpCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        GmpCommands::WarpTransfer(args) => warp_transfer(args, &dispatcher).await,
    }
}

async fn warp_transfer(args: WarpTransferArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let from_address = hex::encode(signer.account_id().0);

    let recipient_bytes = hex::decode(&args.recipient)
        .map_err(|e| CliError::invalid_input(format!("invalid recipient hex: {e}")))?;

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
        "Warp Route transfer submitted\nDestination domain: {}\nAmount: {} (asset {})\nTxHash: {}",
        args.destination_domain, args.amount, args.asset_index, txhash,
    ));

    Ok(())
}
