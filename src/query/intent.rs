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
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::intent::IntentClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_intent(args.intent_id).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_by_agent(args: ByAgentArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::intent::IntentClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client
        .query_intents_by_agent(args.agent_hash, args.limit, args.offset)
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_active(args: ActiveArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::intent::IntentClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client
        .query_active_intents(args.agent_hash, args.limit)
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::intent::IntentClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_params().await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}
