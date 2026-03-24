use clap::{Args, Subcommand};

use morpheum_sdk_native::memory::MemoryClient;

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
    let transport = dispatcher.grpc_transport().await?;
    let client = MemoryClient::new(dispatcher.sdk_config(), Box::new(transport));
    let result = client.query_entry(args.agent_hash, args.key).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_entries_by_agent(
    args: EntriesByAgentArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = MemoryClient::new(dispatcher.sdk_config(), Box::new(transport));
    let result = client
        .query_entries_by_agent(args.agent_hash, args.limit, args.offset)
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_memory_root(
    args: MemoryRootArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = MemoryClient::new(dispatcher.sdk_config(), Box::new(transport));
    let result = client.query_memory_root(args.agent_hash).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = MemoryClient::new(dispatcher.sdk_config(), Box::new(transport));
    let result = client.query_params().await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}
