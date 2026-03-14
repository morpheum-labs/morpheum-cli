use clap::{Args, Subcommand};

use morpheum_proto::interop::v1::query_client::QueryClient;
use morpheum_proto::interop::v1::{
    QueryBridgeRequestRequest, QueryIntentExportRequest, QueryProofExportRequest,
    QueryParamsRequest,
};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the `interop` module.
///
/// Read-only access to cross-chain bridge requests, intent exports,
/// proof exports, and module parameters (Pillar 4 — Bridges & Gateways).
#[derive(Subcommand)]
pub enum InteropQueryCommands {
    /// Get the status of a bridge request
    BridgeRequest(BridgeRequestArgs),

    /// Get the status of an exported intent
    IntentExport(IntentExportArgs),

    /// Get the status of an exported proof
    ProofExport(ProofExportArgs),

    /// Get the current interop module parameters
    Params,
}

#[derive(Args)]
pub struct BridgeRequestArgs {
    /// Bridge request ID
    #[arg(required = true)]
    pub request_id: String,
}

#[derive(Args)]
pub struct IntentExportArgs {
    /// Intent ID
    #[arg(required = true)]
    pub intent_id: String,
}

#[derive(Args)]
pub struct ProofExportArgs {
    /// Proof ID
    #[arg(required = true)]
    pub proof_id: String,
}

pub async fn execute(
    cmd: InteropQueryCommands,
    dispatcher: Dispatcher,
) -> Result<(), CliError> {
    match cmd {
        InteropQueryCommands::BridgeRequest(args) => {
            query_bridge_request(args, &dispatcher).await
        }
        InteropQueryCommands::IntentExport(args) => {
            query_intent_export(args, &dispatcher).await
        }
        InteropQueryCommands::ProofExport(args) => {
            query_proof_export(args, &dispatcher).await
        }
        InteropQueryCommands::Params => query_params(&dispatcher).await,
    }
}

async fn query_bridge_request(
    args: BridgeRequestArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let request_id = args.request_id.clone();
    let response = client
        .query_bridge_request(tonic::Request::new(QueryBridgeRequestRequest {
            request_id: args.request_id,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryBridgeRequest failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    if response.found {
        println!("{json}");
    } else {
        println!("No bridge request found with ID {request_id}");
    }
    Ok(())
}

async fn query_intent_export(
    args: IntentExportArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let intent_id = args.intent_id.clone();
    let response = client
        .query_intent_export(tonic::Request::new(QueryIntentExportRequest {
            intent_id: args.intent_id,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryIntentExport failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    if response.found {
        println!("{json}");
        if !response.target_tx_hash.is_empty() {
            println!("Target tx hash: {}", response.target_tx_hash);
        }
    } else {
        println!("No intent export found for ID {intent_id}");
    }
    Ok(())
}

async fn query_proof_export(
    args: ProofExportArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let proof_id = args.proof_id.clone();
    let response = client
        .query_proof_export(tonic::Request::new(QueryProofExportRequest {
            proof_id: args.proof_id,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryProofExport failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    if response.found {
        println!("{json}");
    } else {
        println!("No proof export found for ID {proof_id}");
    }
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_params(tonic::Request::new(QueryParamsRequest {}))
        .await
        .map_err(|e| CliError::Transport(format!("QueryParams failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    if response.params.is_some() {
        println!("{json}");
    } else {
        println!("No interop parameters configured");
    }
    Ok(())
}
