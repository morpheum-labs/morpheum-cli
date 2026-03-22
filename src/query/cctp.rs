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

    /// Query a specific pending transfer by message hash
    PendingByHash(CctpPendingByHashArgs),

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
pub struct CctpPendingByHashArgs {
    /// CCTP handler contract address (morm1...)
    #[arg(long)]
    pub contract: String,

    /// Message hash (hex string)
    #[arg(long)]
    pub hash: String,
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
        CctpQueryCommands::PendingByHash(args) => pending_by_hash(args, &dispatcher).await,
        CctpQueryCommands::Routes(args) => routes(args, &dispatcher).await,
    }
}

// Re-use the same CosmWasm smart-query gRPC wire types and helper from the
// cosmwasm module — kept local to avoid cross-module coupling.

#[derive(Clone, prost::Message)]
struct QuerySmartContractStateRequest {
    #[prost(string, tag = "1")]
    address: String,
    #[prost(bytes = "vec", tag = "2")]
    query_data: Vec<u8>,
}

#[derive(Clone, prost::Message)]
struct QuerySmartContractStateResponse {
    #[prost(bytes = "vec", tag = "1")]
    data: Vec<u8>,
}

async fn smart_query<R: serde::de::DeserializeOwned>(
    dispatcher: &Dispatcher,
    contract: &str,
    query_msg: &morpheum_sdk_cctp::QueryMsg,
) -> Result<R, CliError> {
    let query_data = serde_json::to_vec(query_msg)
        .map_err(|e| CliError::internal(format!("CCTP query serialization: {e}")))?;

    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;

    let mut grpc = tonic::client::Grpc::new(channel);
    grpc.ready()
        .await
        .map_err(|e| CliError::Transport(format!("service not ready: {e}")))?;

    let path = "/cosmwasm.wasm.v1.Query/SmartContractState"
        .parse::<http::uri::PathAndQuery>()
        .map_err(|e| CliError::internal(format!("invalid gRPC path: {e}")))?;

    let codec: tonic_prost::ProstCodec<
        QuerySmartContractStateRequest,
        QuerySmartContractStateResponse,
    > = tonic_prost::ProstCodec::default();

    let response: QuerySmartContractStateResponse = grpc
        .unary(
            tonic::Request::new(QuerySmartContractStateRequest {
                address: contract.to_string(),
                query_data,
            }),
            path,
            codec,
        )
        .await
        .map(tonic::Response::into_inner)
        .map_err(|e| CliError::Transport(format!("CCTP query failed: {e}")))?;

    serde_json::from_slice(&response.data)
        .map_err(|e| CliError::internal(format!("CCTP response deserialization: {e}")))
}

async fn config(args: CctpConfigArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let resp: morpheum_sdk_cctp::ConfigResponse =
        smart_query(dispatcher, &args.contract, &morpheum_sdk_cctp::QueryMsg::Config {}).await?;

    let json = serde_json::to_string_pretty(&resp)
        .unwrap_or_else(|_| format!("{resp:?}"));
    println!("{json}");
    Ok(())
}

async fn pending(args: CctpPendingArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let resp: morpheum_sdk_cctp::PendingTransfersResponse = smart_query(
        dispatcher,
        &args.contract,
        &morpheum_sdk_cctp::QueryMsg::PendingTransfers {},
    )
    .await?;

    if resp.transfers.is_empty() {
        println!("No pending transfers.");
    } else {
        let json = serde_json::to_string_pretty(&resp.transfers)
            .unwrap_or_else(|_| format!("{:?}", resp.transfers));
        println!("{json}");
    }
    Ok(())
}

async fn pending_by_hash(
    args: CctpPendingByHashArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let resp: morpheum_sdk_cctp::PendingTransferResponse = smart_query(
        dispatcher,
        &args.contract,
        &morpheum_sdk_cctp::QueryMsg::PendingByHash {
            hash: args.hash.clone(),
        },
    )
    .await?;

    match resp.transfer {
        Some(transfer) => {
            let json = serde_json::to_string_pretty(&transfer)
                .unwrap_or_else(|_| format!("{transfer:?}"));
            println!("{json}");
        }
        None => {
            println!("No pending transfer found for hash: {}", args.hash);
        }
    }
    Ok(())
}

async fn routes(args: CctpRoutesArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let resp: morpheum_sdk_cctp::RoutesResponse = smart_query(
        dispatcher,
        &args.contract,
        &morpheum_sdk_cctp::QueryMsg::Routes {},
    )
    .await?;

    if resp.routes.is_empty() {
        println!("No routes enrolled.");
    } else {
        let json = serde_json::to_string_pretty(&resp.routes)
            .unwrap_or_else(|_| format!("{:?}", resp.routes));
        println!("{json}");
    }
    Ok(())
}
