use clap::{Args, Subcommand};

use morpheum_sdk_native::MorpheumSdk;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

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
    let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);

    match cmd {
        IntentQueryCommands::Get(args) => query_intent(args, &sdk, &dispatcher.output).await,
        IntentQueryCommands::ByAgent(args) => {
            query_by_agent(args, &sdk, &dispatcher.output).await
        }
        IntentQueryCommands::Active(args) => {
            query_active(args, &sdk, &dispatcher.output).await
        }
        IntentQueryCommands::Params => query_params(&sdk, &dispatcher.output).await,
    }
}

async fn query_intent(
    args: GetArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.intent()
        .query_and_print_optional(
            output,
            &format!("No intent found with ID {}", args.intent_id),
            |c| async move { c.query_intent(&args.intent_id).await },
        )
        .await
}

async fn query_by_agent(
    args: ByAgentArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.intent()
        .query_and_print_paginated(output, |c| async move {
            c.query_intents_by_agent(&args.agent_hash, args.limit, args.offset)
                .await
        })
        .await
}

async fn query_active(
    args: ActiveArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.intent()
        .query_and_print_paginated(output, |c| async move {
            c.query_active_intents(&args.agent_hash, args.limit).await
        })
        .await
}

async fn query_params(sdk: &MorpheumSdk, output: &Output) -> Result<(), CliError> {
    sdk.intent()
        .query_and_print_optional(
            output,
            "No intent parameters configured",
            |c| async move { c.query_params().await },
        )
        .await
}
