use clap::{Args, Subcommand};

use morpheum_sdk_native::reputation::ReputationClient;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

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
    match cmd {
        ReputationQueryCommands::Score(args) => query_score(args, &dispatcher).await,
        ReputationQueryCommands::History(args) => query_history(args, &dispatcher).await,
        ReputationQueryCommands::MilestoneStatus(args) => {
            query_milestone_status(args, &dispatcher).await
        }
        ReputationQueryCommands::Params => query_params(&dispatcher).await,
    }
}

async fn query_score(args: ScoreArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = ReputationClient::new(dispatcher.sdk_config(), Box::new(transport));
    let result = client.query_score(args.agent_hash).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_history(args: HistoryArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = ReputationClient::new(dispatcher.sdk_config(), Box::new(transport));
    let result = client
        .query_history(args.agent_hash, args.limit, args.offset)
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_milestone_status(
    args: MilestoneStatusArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = ReputationClient::new(dispatcher.sdk_config(), Box::new(transport));
    let result = client.query_milestone_status(args.agent_hash).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = ReputationClient::new(dispatcher.sdk_config(), Box::new(transport));
    let result = client.query_params().await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}
