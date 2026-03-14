use clap::{Args, Subcommand};
use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

/// Query commands for the `identity` module (ERC-8004 Identity Registry).
///
/// These commands provide rich, permissioned read access to agent identities,
/// metadata, capabilities, ownership, and CAIP-10 portability information.
///
/// Fully consistent with the tx side and Pillar 3 trust layer.
#[derive(Subcommand)]
pub enum IdentityQueryCommands {
    /// Get a complete agent identity record
    Get(GetArgs),

    /// Query an agent by its CAIP-10 identifier (cross-chain portable)
    ByCaip(ByCaipArgs),
}

#[derive(Args)]
pub struct GetArgs {
    /// Agent DID or native AgentId (e.g. did:agent:alpha-trader-v3)
    #[arg(required = true)]
    pub id: String,
}

#[derive(Args)]
pub struct ByCaipArgs {
    /// CAIP-10 identifier (e.g. morpheum:1:agent-0x...)
    #[arg(required = true)]
    pub caip: String,
}

/// Execute identity query commands.
pub async fn execute(cmd: IdentityQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    let sdk = morpheum_sdk_native::MorpheumSdk::new(
        &dispatcher.config.rpc_url,
        &dispatcher.config.chain_id,
    );

    match cmd {
        IdentityQueryCommands::Get(args) => get_agent(args, &sdk, &dispatcher.output).await,
        IdentityQueryCommands::ByCaip(args) => get_by_caip(args, &sdk, &dispatcher.output).await,
    }
}

async fn get_agent(
    args: GetArgs,
    sdk: &morpheum_sdk_native::MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.identity()
        .query_and_print_item(output, |c| async move {
            c.query_agent_record(&args.id).await
        })
        .await
}

async fn get_by_caip(
    args: ByCaipArgs,
    sdk: &morpheum_sdk_native::MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.identity()
        .query_and_print_item(output, |c| async move {
            c.query_agent_by_caip(&args.caip).await
        })
        .await
}