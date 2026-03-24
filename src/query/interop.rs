use clap::{Args, Subcommand};

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
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::interop::InteropClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let request_id = args.request_id.clone();
    let result = client.query_bridge_request(args.request_id).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    if result.is_some() {
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
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::interop::InteropClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let intent_id = args.intent_id.clone();
    let result = client.query_intent_export(args.intent_id).await?;
    match result {
        Some((packet, target_tx_hash)) => {
            let json =
                serde_json::to_string_pretty(&packet).unwrap_or_else(|_| format!("{packet:?}"));
            println!("{json}");
            if !target_tx_hash.is_empty() {
                println!("Target tx hash: {target_tx_hash}");
            }
        }
        None => {
            println!("No intent export found for ID {intent_id}");
        }
    }
    Ok(())
}

async fn query_proof_export(
    args: ProofExportArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::interop::InteropClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let proof_id = args.proof_id.clone();
    let result = client.query_proof_export(args.proof_id).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    if result.is_some() {
        println!("{json}");
    } else {
        println!("No proof export found for ID {proof_id}");
    }
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::interop::InteropClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_params().await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    if result.is_some() {
        println!("{json}");
    } else {
        println!("No interop parameters configured");
    }
    Ok(())
}
