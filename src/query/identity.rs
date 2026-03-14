use clap::{Args, Subcommand};

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
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;

    let mut client =
        morpheum_proto::identity::v1::query_client::QueryClient::new(channel);

    let (agent_hash, did) = if args.id.starts_with("did:") {
        (String::new(), args.id.clone())
    } else {
        (args.id.clone(), String::new())
    };

    let request = morpheum_proto::identity::v1::QueryAgentRequest {
        agent_hash,
        did,
    };

    let response = client
        .query_agent(tonic::Request::new(request))
        .await
        .map_err(|e| CliError::Transport(format!("QueryAgent failed: {e}")))?
        .into_inner();

    if !response.found {
        return Err(CliError::Transport(format!(
            "agent '{}' not found",
            args.id
        )));
    }

    let json = serde_json::to_string_pretty(&response)
        .unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");

    Ok(())
}
