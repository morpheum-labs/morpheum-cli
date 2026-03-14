use clap::{Args, Subcommand};

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
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client =
        morpheum_proto::reputation::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_reputation_score(tonic::Request::new(
            morpheum_proto::reputation::v1::QueryReputationScoreRequest {
                agent_hash: args.agent_hash,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("query_reputation_score failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_history(args: HistoryArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client =
        morpheum_proto::reputation::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_reputation_history(tonic::Request::new(
            morpheum_proto::reputation::v1::QueryReputationHistoryRequest {
                agent_hash: args.agent_hash,
                limit: args.limit,
                offset: args.offset,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("query_reputation_history failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_milestone_status(
    args: MilestoneStatusArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client =
        morpheum_proto::reputation::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_milestone_status(tonic::Request::new(
            morpheum_proto::reputation::v1::QueryMilestoneStatusRequest {
                agent_hash: args.agent_hash,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("query_milestone_status failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client =
        morpheum_proto::reputation::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_params(tonic::Request::new(
            morpheum_proto::reputation::v1::QueryParamsRequest {},
        ))
        .await
        .map_err(|e| CliError::Transport(format!("query_params failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}
