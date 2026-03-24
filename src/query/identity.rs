use clap::{Args, Subcommand};

use morpheum_sdk_native::identity::IdentityClient;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the `identity` module.
#[derive(Subcommand)]
pub enum IdentityQueryCommands {
    /// Get a complete agent identity record by DID or hash
    Get(GetArgs),
}

#[derive(Args)]
pub struct GetArgs {
    /// Agent DID or hash (e.g. did:agent:alpha-trader-v3)
    #[arg(required = true)]
    pub id: String,
}

pub async fn execute(cmd: IdentityQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        IdentityQueryCommands::Get(args) => get_agent(args, &dispatcher).await,
    }
}

async fn get_agent(args: GetArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = IdentityClient::new(dispatcher.sdk_config(), Box::new(transport));

    let result = if args.id.starts_with("did:") {
        client.query_agent_by_did(&args.id).await?
    } else {
        client.query_agent(&args.id).await?
    };

    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");

    Ok(())
}
