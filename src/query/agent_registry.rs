use clap::{Args, Subcommand};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

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
    match cmd {
        AgentRegistryQueryCommands::Record(args) => {
            query_record(args, &dispatcher).await
        }
        AgentRegistryQueryCommands::ByCaip(args) => {
            query_by_caip(args, &dispatcher).await
        }
        AgentRegistryQueryCommands::ExportStatus(args) => {
            query_export_status(args, &dispatcher).await
        }
        AgentRegistryQueryCommands::Params => query_params(&dispatcher).await,
    }
}

async fn query_record(args: RecordArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::agent_registry::AgentRegistryClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let agent_hash = args.agent_hash.clone();
    let result = client.query_agent_record(args.agent_hash).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    if result.is_some() {
        println!("{json}");
    } else {
        println!("No agent record found for hash {agent_hash}");
    }
    Ok(())
}

async fn query_by_caip(args: ByCaipArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::agent_registry::AgentRegistryClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let caip_id = args.caip_id.clone();
    let result = client.query_agent_by_caip(args.caip_id).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    if result.is_some() {
        println!("{json}");
    } else {
        println!("No agent found for CAIP-10 identifier {caip_id}");
    }
    Ok(())
}

async fn query_export_status(
    args: ExportStatusArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::agent_registry::AgentRegistryClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client
        .query_export_status(args.agent_hash, args.protocols)
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::agent_registry::AgentRegistryClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_params().await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    if result.is_some() {
        println!("{json}");
    } else {
        println!("No agent registry parameters configured");
    }
    Ok(())
}
