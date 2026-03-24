//! Query commands for the CCTP handler contract on Morpheum.

use clap::{Args, Subcommand};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the `cctp` handler contract.
///
/// Reads state from the `hpl-cctp-handler` `CosmWasm` contract via gRPC smart
/// query, delegating JSON construction and deserialization to the SDK.
#[derive(Subcommand)]
pub enum CctpQueryCommands {
    /// Query the CCTP handler configuration
    Config(CctpConfigArgs),

    /// List all pending CCTP transfers
    Pending(CctpPendingArgs),

    /// Query a specific pending transfer by source domain and nonce
    PendingByNonce(CctpPendingByNonceArgs),

    /// List enrolled remote routes
    Routes(CctpRoutesArgs),
}

#[derive(Args)]
pub struct CctpConfigArgs {
    /// CCTP handler contract address (morm1...)
    #[arg(long)]
    pub contract: String,
}

#[derive(Args)]
pub struct CctpPendingArgs {
    /// CCTP handler contract address (morm1...)
    #[arg(long)]
    pub contract: String,
}

#[derive(Args)]
pub struct CctpPendingByNonceArgs {
    /// CCTP handler contract address (morm1...)
    #[arg(long)]
    pub contract: String,

    /// CCTP source domain (e.g. 0 for Ethereum)
    #[arg(long)]
    pub source_domain: u32,

    /// CCTP nonce from the source chain burn
    #[arg(long)]
    pub nonce: u64,
}

#[derive(Args)]
pub struct CctpRoutesArgs {
    /// CCTP handler contract address (morm1...)
    #[arg(long)]
    pub contract: String,
}

pub async fn execute(cmd: CctpQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        CctpQueryCommands::Config(args) => config(args, &dispatcher).await,
        CctpQueryCommands::Pending(args) => pending(args, &dispatcher).await,
        CctpQueryCommands::PendingByNonce(args) => pending_by_nonce(args, &dispatcher).await,
        CctpQueryCommands::Routes(args) => routes(args, &dispatcher).await,
    }
}

async fn cctp_client(dispatcher: &Dispatcher) -> Result<morpheum_sdk_cosmwasm::CosmWasmClient, CliError> {
    let transport = dispatcher.grpc_transport().await?;
    Ok(morpheum_sdk_cosmwasm::CosmWasmClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    ))
}

async fn config(args: CctpConfigArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let client = cctp_client(dispatcher).await?;
    let resp = morpheum_sdk_cctp::query_config(&client, &args.contract)
        .await
        .map_err(|e| CliError::Sdk(e.into()))?;

    let json = serde_json::to_string_pretty(&resp)
        .unwrap_or_else(|_| format!("{resp:?}"));
    println!("{json}");
    Ok(())
}

async fn pending(args: CctpPendingArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let client = cctp_client(dispatcher).await?;
    let transfers = morpheum_sdk_cctp::query_pending_transfers(&client, &args.contract)
        .await
        .map_err(|e| CliError::Sdk(e.into()))?;

    if transfers.is_empty() {
        println!("No pending transfers.");
    } else {
        let json = serde_json::to_string_pretty(&transfers)
            .unwrap_or_else(|_| format!("{:?}", transfers));
        println!("{json}");
    }
    Ok(())
}

async fn pending_by_nonce(
    args: CctpPendingByNonceArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let client = cctp_client(dispatcher).await?;
    let transfer = morpheum_sdk_cctp::query_pending_by_nonce(
        &client,
        &args.contract,
        args.source_domain,
        args.nonce,
    )
    .await
    .map_err(|e| CliError::Sdk(e.into()))?;

    match transfer {
        Some(transfer) => {
            let json = serde_json::to_string_pretty(&transfer)
                .unwrap_or_else(|_| format!("{transfer:?}"));
            println!("{json}");
        }
        None => {
            println!(
                "No pending transfer found for domain {} nonce {}",
                args.source_domain, args.nonce
            );
        }
    }
    Ok(())
}

async fn routes(args: CctpRoutesArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let client = cctp_client(dispatcher).await?;
    let routes = morpheum_sdk_cctp::query_routes(&client, &args.contract)
        .await
        .map_err(|e| CliError::Sdk(e.into()))?;

    if routes.is_empty() {
        println!("No routes enrolled.");
    } else {
        let json = serde_json::to_string_pretty(&routes)
            .unwrap_or_else(|_| format!("{:?}", routes));
        println!("{json}");
    }
    Ok(())
}
