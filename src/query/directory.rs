use clap::{Args, Subcommand};

use morpheum_proto::directory::v1::query_client::QueryClient;
use morpheum_proto::directory::v1::{
    DirectoryFilter, QueryDirectoryProfileRequest, QueryDirectoryProfilesRequest,
    QueryParamsRequest,
};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the `directory` module.
///
/// Read-only access to agent directory profiles, filtered discovery,
/// and module parameters.
#[derive(Subcommand)]
pub enum DirectoryQueryCommands {
    /// Get a specific agent's directory profile
    Profile(ProfileArgs),

    /// List directory profiles with optional filters (paginated)
    Profiles(ProfilesArgs),

    /// Get the current directory module parameters
    Params,
}

#[derive(Args)]
pub struct ProfileArgs {
    /// Agent hash
    #[arg(required = true)]
    pub agent_hash: String,
}

#[derive(Args)]
pub struct ProfilesArgs {
    #[arg(long, default_value = "20")]
    pub limit: u32,

    #[arg(long, default_value = "0")]
    pub offset: u32,

    /// Minimum reputation score filter
    #[arg(long)]
    pub min_reputation: Option<u64>,

    /// Comma-separated tag filter (e.g. "hft,btc")
    #[arg(long)]
    pub tags: Option<String>,
}

pub async fn execute(
    cmd: DirectoryQueryCommands,
    dispatcher: Dispatcher,
) -> Result<(), CliError> {
    match cmd {
        DirectoryQueryCommands::Profile(args) => {
            query_profile(args, &dispatcher).await
        }
        DirectoryQueryCommands::Profiles(args) => {
            query_profiles(args, &dispatcher).await
        }
        DirectoryQueryCommands::Params => query_params(&dispatcher).await,
    }
}

async fn query_profile(args: ProfileArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let agent_hash = args.agent_hash.clone();
    let response = client
        .query_directory_profile(tonic::Request::new(QueryDirectoryProfileRequest {
            agent_hash: args.agent_hash,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryDirectoryProfile failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    if response.found {
        println!("{json}");
    } else {
        println!("No directory profile found for agent {agent_hash}");
    }
    Ok(())
}

async fn query_profiles(args: ProfilesArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let filter = build_filter(&args);
    let response = client
        .query_directory_profiles(tonic::Request::new(QueryDirectoryProfilesRequest {
            filter,
            limit: args.limit,
            offset: args.offset,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryDirectoryProfiles failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_params(tonic::Request::new(QueryParamsRequest {}))
        .await
        .map_err(|e| CliError::Transport(format!("QueryParams failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    if response.params.is_some() {
        println!("{json}");
    } else {
        println!("No directory parameters configured");
    }
    Ok(())
}

fn build_filter(args: &ProfilesArgs) -> Option<DirectoryFilter> {
    let has_filters = args.min_reputation.is_some() || args.tags.is_some();

    if !has_filters {
        return None;
    }

    Some(DirectoryFilter {
        min_reputation: args.min_reputation.unwrap_or(0),
        min_milestone_level: 0,
        tags: args.tags.clone().unwrap_or_default(),
        semantic_query: String::new(),
        limit: 0,
        offset: 0,
        x402_only: false,
    })
}
