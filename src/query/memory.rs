use clap::{Args, Subcommand};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the `memory` module.
///
/// Read-only access to persistent agent memory entries, Merkle roots,
/// and module parameters.
#[derive(Subcommand)]
pub enum MemoryQueryCommands {
    /// Get a specific memory entry by agent hash and key
    Entry(EntryArgs),

    /// List memory entries for an agent (paginated)
    EntriesByAgent(EntriesByAgentArgs),

    /// Get the memory Merkle root for an agent
    MemoryRoot(MemoryRootArgs),

    /// Get the current memory module parameters
    Params,
}

#[derive(Args)]
pub struct EntryArgs {
    /// Agent hash
    #[arg(required = true)]
    pub agent_hash: String,

    /// Memory entry key (e.g. "strategy/v1")
    #[arg(required = true)]
    pub key: String,
}

#[derive(Args)]
pub struct EntriesByAgentArgs {
    /// Agent hash
    #[arg(required = true)]
    pub agent_hash: String,

    #[arg(long, default_value = "20")]
    pub limit: u32,

    #[arg(long, default_value = "0")]
    pub offset: u32,
}

#[derive(Args)]
pub struct MemoryRootArgs {
    /// Agent hash
    #[arg(required = true)]
    pub agent_hash: String,
}

pub async fn execute(cmd: MemoryQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        MemoryQueryCommands::Entry(args) => query_entry(args, &dispatcher).await,
        MemoryQueryCommands::EntriesByAgent(args) => {
            query_entries_by_agent(args, &dispatcher).await
        }
        MemoryQueryCommands::MemoryRoot(args) => query_memory_root(args, &dispatcher).await,
        MemoryQueryCommands::Params => query_params(&dispatcher).await,
    }
}

async fn query_entry(args: EntryArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client =
        morpheum_proto::memory::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_memory_entry(tonic::Request::new(
            morpheum_proto::memory::v1::QueryMemoryEntryRequest {
                agent_hash: args.agent_hash,
                key: args.key,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("query_memory_entry failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_entries_by_agent(
    args: EntriesByAgentArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client =
        morpheum_proto::memory::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_memory_entries_by_agent(tonic::Request::new(
            morpheum_proto::memory::v1::QueryMemoryEntriesByAgentRequest {
                agent_hash: args.agent_hash,
                limit: args.limit,
                offset: args.offset,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("query_memory_entries_by_agent failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_memory_root(
    args: MemoryRootArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client =
        morpheum_proto::memory::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_memory_root(tonic::Request::new(
            morpheum_proto::memory::v1::QueryMemoryRootRequest {
                agent_hash: args.agent_hash,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("query_memory_root failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client =
        morpheum_proto::memory::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_params(tonic::Request::new(
            morpheum_proto::memory::v1::QueryParamsRequest {},
        ))
        .await
        .map_err(|e| CliError::Transport(format!("query_params failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}
