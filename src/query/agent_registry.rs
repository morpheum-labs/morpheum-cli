use clap::{Args, Subcommand};

use morpheum_sdk_native::MorpheumSdk;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

/// Query commands for the `agent_registry` module.
///
/// Read-only access to agent records, CAIP-10 resolution,
/// protocol export status, and module parameters.
#[derive(Subcommand)]
pub enum AgentRegistryQueryCommands {
    /// Get an agent record by its hash
    Record(RecordArgs),

    /// Resolve an agent by CAIP-10 identifier
    ByCaip(ByCaipArgs),

    /// Get protocol export/sync status for an agent
    ExportStatus(ExportStatusArgs),

    /// Get the current agent registry module parameters
    Params,
}

#[derive(Args)]
pub struct RecordArgs {
    /// Agent hash (hex-encoded)
    #[arg(required = true)]
    pub agent_hash: String,
}

#[derive(Args)]
pub struct ByCaipArgs {
    /// CAIP-10 identifier (e.g. morpheum:1:agent-0x...)
    #[arg(required = true)]
    pub caip_id: String,
}

#[derive(Args)]
pub struct ExportStatusArgs {
    /// Agent hash
    #[arg(required = true)]
    pub agent_hash: String,

    /// Filter by protocol names (e.g. erc8004, mcp, a2a). Omit for all.
    #[arg(long, value_delimiter = ',')]
    pub protocols: Vec<String>,
}

pub async fn execute(
    cmd: AgentRegistryQueryCommands,
    dispatcher: Dispatcher,
) -> Result<(), CliError> {
    let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);

    match cmd {
        AgentRegistryQueryCommands::Record(args) => {
            query_record(args, &sdk, &dispatcher.output).await
        }
        AgentRegistryQueryCommands::ByCaip(args) => {
            query_by_caip(args, &sdk, &dispatcher.output).await
        }
        AgentRegistryQueryCommands::ExportStatus(args) => {
            query_export_status(args, &sdk, &dispatcher.output).await
        }
        AgentRegistryQueryCommands::Params => {
            query_params(&sdk, &dispatcher.output).await
        }
    }
}

async fn query_record(
    args: RecordArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.agent_registry()
        .query_and_print_optional(
            output,
            &format!("No agent record found for hash {}", args.agent_hash),
            |c| async move { c.query_agent_record(&args.agent_hash).await },
        )
        .await
}

async fn query_by_caip(
    args: ByCaipArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.agent_registry()
        .query_and_print_optional(
            output,
            &format!("No agent found for CAIP-10 identifier {}", args.caip_id),
            |c| async move { c.query_agent_by_caip(&args.caip_id).await },
        )
        .await
}

async fn query_export_status(
    args: ExportStatusArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.agent_registry()
        .query_and_print_list(output, |c| async move {
            c.query_export_status(&args.agent_hash, args.protocols).await
        })
        .await
}

async fn query_params(sdk: &MorpheumSdk, output: &Output) -> Result<(), CliError> {
    sdk.agent_registry()
        .query_and_print_optional(
            output,
            "No agent registry parameters configured",
            |c| async move { c.query_params().await },
        )
        .await
}
