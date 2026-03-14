use clap::{Args, Subcommand};

use morpheum_sdk_native::MorpheumSdk;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

/// Query commands for the `reputation` module.
///
/// Read-only access to on-chain reputation scores, event history,
/// milestone/perk status, and module parameters (Pillar 3 — On-Chain Trust).
#[derive(Subcommand)]
pub enum ReputationQueryCommands {
    /// Get the current reputation score for an agent
    Score(ScoreArgs),

    /// List reputation event history for an agent (paginated)
    History(HistoryArgs),

    /// Get milestone and perk status for an agent
    MilestoneStatus(MilestoneStatusArgs),

    /// Get the current reputation module parameters
    Params,
}

#[derive(Args)]
pub struct ScoreArgs {
    /// Agent hash (hex-encoded SHA-256 of the agent DID)
    #[arg(required = true)]
    pub agent_hash: String,
}

#[derive(Args)]
pub struct HistoryArgs {
    /// Agent hash
    #[arg(required = true)]
    pub agent_hash: String,

    /// Maximum number of events to return
    #[arg(long, default_value = "20")]
    pub limit: u32,

    /// Pagination offset
    #[arg(long, default_value = "0")]
    pub offset: u32,
}

#[derive(Args)]
pub struct MilestoneStatusArgs {
    /// Agent hash
    #[arg(required = true)]
    pub agent_hash: String,
}

pub async fn execute(cmd: ReputationQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);

    match cmd {
        ReputationQueryCommands::Score(args) => {
            query_score(args, &sdk, &dispatcher.output).await
        }
        ReputationQueryCommands::History(args) => {
            query_history(args, &sdk, &dispatcher.output).await
        }
        ReputationQueryCommands::MilestoneStatus(args) => {
            query_milestone_status(args, &sdk, &dispatcher.output).await
        }
        ReputationQueryCommands::Params => {
            query_params(&sdk, &dispatcher.output).await
        }
    }
}

async fn query_score(
    args: ScoreArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.reputation()
        .query_and_print_optional(
            output,
            &format!("No reputation record found for agent {}", args.agent_hash),
            |c| async move { c.query_score(&args.agent_hash).await },
        )
        .await
}

async fn query_history(
    args: HistoryArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.reputation()
        .query_and_print_paginated(output, |c| async move {
            c.query_history(&args.agent_hash, args.limit, args.offset).await
        })
        .await
}

/// Milestone status returns a direct `MilestoneStatus` (non-Optional),
/// so `QueryClientExt::query_and_print_item` works cleanly.
async fn query_milestone_status(
    args: MilestoneStatusArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.reputation()
        .query_and_print_item(output, |c| async move {
            c.query_milestone_status(&args.agent_hash).await
        })
        .await
}

async fn query_params(sdk: &MorpheumSdk, output: &Output) -> Result<(), CliError> {
    sdk.reputation()
        .query_and_print_optional(
            output,
            "No reputation parameters configured",
            |c| async move { c.query_params().await },
        )
        .await
}
