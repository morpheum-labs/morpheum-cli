use clap::{Args, Subcommand};

use morpheum_sdk_native::MorpheumSdk;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

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
    let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);

    match cmd {
        MemoryQueryCommands::Entry(args) => {
            query_entry(args, &sdk, &dispatcher.output).await
        }
        MemoryQueryCommands::EntriesByAgent(args) => {
            query_entries_by_agent(args, &sdk, &dispatcher.output).await
        }
        MemoryQueryCommands::MemoryRoot(args) => {
            query_memory_root(args, &sdk, &dispatcher.output).await
        }
        MemoryQueryCommands::Params => {
            query_params(&sdk, &dispatcher.output).await
        }
    }
}

async fn query_entry(
    args: EntryArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.memory()
        .query_and_print_optional(
            output,
            &format!(
                "No memory entry found for agent {} key '{}'",
                args.agent_hash, args.key
            ),
            |c| async move { c.query_entry(&args.agent_hash, &args.key).await },
        )
        .await
}

async fn query_entries_by_agent(
    args: EntriesByAgentArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.memory()
        .query_and_print_paginated(output, |c| async move {
            c.query_entries_by_agent(&args.agent_hash, args.limit, args.offset)
                .await
        })
        .await
}

async fn query_memory_root(
    args: MemoryRootArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.memory()
        .query_and_print_optional(
            output,
            &format!("No memory root found for agent {}", args.agent_hash),
            |c| async move { c.query_memory_root(&args.agent_hash).await },
        )
        .await
}

async fn query_params(sdk: &MorpheumSdk, output: &Output) -> Result<(), CliError> {
    sdk.memory()
        .query_and_print_optional(
            output,
            "No memory parameters configured",
            |c| async move { c.query_params().await },
        )
        .await
}
