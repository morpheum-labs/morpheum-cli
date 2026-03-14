use clap::{Args, Subcommand};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the `intent` module.
///
/// Read-only access to agent intents (conditional, TWAP, declarative),
/// active intent listings, and module parameters.
#[derive(Subcommand)]
pub enum IntentQueryCommands {
    /// Get a specific intent by ID
    Get(GetArgs),

    /// List intents submitted by a specific agent (paginated)
    ByAgent(ByAgentArgs),

    /// List active (pending/executing) intents for an agent
    Active(ActiveArgs),

    /// Get the current intent module parameters
    Params,
}

#[derive(Args)]
pub struct GetArgs {
    /// Intent ID
    #[arg(required = true)]
    pub intent_id: String,
}

#[derive(Args)]
pub struct ByAgentArgs {
    /// Agent hash
    #[arg(required = true)]
    pub agent_hash: String,

    #[arg(long, default_value = "20")]
    pub limit: u32,

    #[arg(long, default_value = "0")]
    pub offset: u32,
}

#[derive(Args)]
pub struct ActiveArgs {
    /// Agent hash
    #[arg(required = true)]
    pub agent_hash: String,

    #[arg(long, default_value = "20")]
    pub limit: u32,
}

pub async fn execute(cmd: IntentQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        IntentQueryCommands::Get(args) => query_intent(args, &dispatcher).await,
        IntentQueryCommands::ByAgent(args) => query_by_agent(args, &dispatcher).await,
        IntentQueryCommands::Active(args) => query_active(args, &dispatcher).await,
        IntentQueryCommands::Params => query_params(&dispatcher).await,
    }
}

async fn query_intent(args: GetArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::intent::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_intent(tonic::Request::new(
            morpheum_proto::intent::v1::QueryIntentRequest {
                intent_id: args.intent_id,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryIntent failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_by_agent(args: ByAgentArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::intent::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_intents_by_agent(tonic::Request::new(
            morpheum_proto::intent::v1::QueryIntentsByAgentRequest {
                agent_hash: args.agent_hash,
                limit: args.limit,
                offset: args.offset,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryIntentsByAgent failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_active(args: ActiveArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::intent::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_active_intents(tonic::Request::new(
            morpheum_proto::intent::v1::QueryActiveIntentsRequest {
                agent_hash: args.agent_hash,
                limit: args.limit,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryActiveIntents failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::intent::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_params(tonic::Request::new(
            morpheum_proto::intent::v1::QueryParamsRequest::default(),
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryParams failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}
