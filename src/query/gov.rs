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
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_gov::GovClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );

    let result = client.query_params().await?;

    let json =
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn proposal(args: ProposalArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_gov::GovClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );

    let result = client.query_proposal(args.proposal_id).await?;

    let json =
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn proposals(args: ProposalsArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_gov::GovClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );

    let result = client
        .query_proposals(args.limit, args.offset, None, None, None)
        .await?;

    let json =
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}
