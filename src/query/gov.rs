use clap::{Args, Subcommand};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the `gov` module.
///
/// Read-only access to governance parameters, proposals, deposits, votes,
/// and tally results.
#[derive(Subcommand)]
pub enum GovQueryCommands {
    /// Get current governance parameters
    Params,

    /// Get a single proposal by ID
    Proposal(ProposalArgs),

    /// List proposals with optional filters and pagination
    Proposals(ProposalsArgs),
}

#[derive(Args)]
pub struct ProposalArgs {
    /// Proposal ID
    #[arg(long)]
    pub proposal_id: u64,
}

#[derive(Args)]
pub struct ProposalsArgs {
    /// Maximum number of proposals to return
    #[arg(long, default_value = "100")]
    pub limit: i32,

    /// Offset for pagination
    #[arg(long, default_value = "0")]
    pub offset: i32,
}

pub async fn execute(cmd: GovQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        GovQueryCommands::Params => params(&dispatcher).await,
        GovQueryCommands::Proposal(args) => proposal(args, &dispatcher).await,
        GovQueryCommands::Proposals(args) => proposals(args, &dispatcher).await,
    }
}

async fn params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::gov::v1::query_client::QueryClient::new(channel);

    let response = client
        .query_params(tonic::Request::new(morpheum_proto::gov::v1::QueryParamsRequest {}))
        .await
        .map_err(|e| CliError::Transport(format!("QueryParams failed: {e}")))?
        .into_inner();

    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn proposal(args: ProposalArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::gov::v1::query_client::QueryClient::new(channel);

    let response = client
        .query_proposal(tonic::Request::new(
            morpheum_proto::gov::v1::QueryProposalRequest {
                proposal_id: args.proposal_id,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryProposal failed: {e}")))?
        .into_inner();

    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn proposals(args: ProposalsArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::gov::v1::query_client::QueryClient::new(channel);

    let response = client
        .query_proposals(tonic::Request::new(
            morpheum_proto::gov::v1::QueryProposalsRequest {
                limit: args.limit,
                offset: args.offset,
                status_filter: 0,
                class_filter: 0,
                proposer_filter: String::new(),
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryProposals failed: {e}")))?
        .into_inner();

    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}
