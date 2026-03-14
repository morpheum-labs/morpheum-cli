use clap::{Args, Subcommand};

use morpheum_sdk_native::MorpheumSdk;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

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
    let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);

    match cmd {
        InteropQueryCommands::BridgeRequest(args) => {
            query_bridge_request(args, &sdk, &dispatcher.output).await
        }
        InteropQueryCommands::IntentExport(args) => {
            query_intent_export(args, &sdk, &dispatcher.output).await
        }
        InteropQueryCommands::ProofExport(args) => {
            query_proof_export(args, &sdk, &dispatcher.output).await
        }
        InteropQueryCommands::Params => query_params(&sdk, &dispatcher.output).await,
    }
}

async fn query_bridge_request(
    args: BridgeRequestArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.interop()
        .query_and_print_optional(
            output,
            &format!("No bridge request found with ID {}", args.request_id),
            |c| async move { c.query_bridge_request(&args.request_id).await },
        )
        .await
}

async fn query_intent_export(
    args: IntentExportArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    let export = sdk
        .interop()
        .query_intent_export(&args.intent_id)
        .await?;

    match export {
        Some((packet, target_tx_hash)) => {
            output.print_item(&packet)?;
            output.info(format!("Target tx hash: {target_tx_hash}"));
        }
        None => output.warn(format!(
            "No intent export found for ID {}",
            args.intent_id
        )),
    }

    Ok(())
}

async fn query_proof_export(
    args: ProofExportArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.interop()
        .query_and_print_optional(
            output,
            &format!("No proof export found for ID {}", args.proof_id),
            |c| async move { c.query_proof_export(&args.proof_id).await },
        )
        .await
}

async fn query_params(sdk: &MorpheumSdk, output: &Output) -> Result<(), CliError> {
    sdk.interop()
        .query_and_print_optional(
            output,
            "No interop parameters configured",
            |c| async move { c.query_params().await },
        )
        .await
}
