use clap::{Args, Subcommand};

use morpheum_proto::agent_registry::v1::query_client::QueryClient;
use morpheum_proto::agent_registry::v1::{
    QueryAgentRecordRequest, QueryAgentByCaipRequest, QueryExportStatusRequest,
    QueryParamsRequest,
};

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
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let agent_hash = args.agent_hash.clone();
    let response = client
        .query_agent_record(tonic::Request::new(QueryAgentRecordRequest {
            agent_hash: args.agent_hash,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryAgentRecord failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    if response.record.is_some() {
        println!("{json}");
    } else {
        println!("No agent record found for hash {agent_hash}");
    }
    Ok(())
}

async fn query_by_caip(args: ByCaipArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let caip_id = args.caip_id.clone();
    let response = client
        .query_agent_by_caip(tonic::Request::new(QueryAgentByCaipRequest {
            caip_id: args.caip_id,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryAgentByCAIP failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    if response.record.is_some() {
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
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_export_status(tonic::Request::new(QueryExportStatusRequest {
            agent_hash: args.agent_hash,
            protocols: args.protocols,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryExportStatus failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
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
        println!("No agent registry parameters configured");
    }
    Ok(())
}
