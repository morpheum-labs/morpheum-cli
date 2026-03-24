use clap::{Args, Subcommand};

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
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::directory::DirectoryClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let agent_hash = args.agent_hash.clone();
    let result = client.query_profile(args.agent_hash).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    if result.is_some() {
        println!("{json}");
    } else {
        println!("No directory profile found for agent {agent_hash}");
    }
    Ok(())
}

async fn query_profiles(args: ProfilesArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::directory::DirectoryClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let filter = build_filter(&args);
    let (profiles, total_count) = client
        .query_profiles(args.limit, args.offset, filter)
        .await?;
    let result = serde_json::json!({
        "profiles": profiles,
        "total_count": total_count,
    });
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::directory::DirectoryClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_params().await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    if result.is_some() {
        println!("{json}");
    } else {
        println!("No directory parameters configured");
    }
    Ok(())
}

fn build_filter(args: &ProfilesArgs) -> Option<morpheum_sdk_native::directory::DirectoryFilter> {
    let has_filters = args.min_reputation.is_some() || args.tags.is_some();

    if !has_filters {
        return None;
    }

    Some(morpheum_sdk_native::directory::DirectoryFilter {
        min_reputation: args.min_reputation.unwrap_or(0),
        min_milestone_level: 0,
        tags: args.tags.clone().unwrap_or_default(),
        semantic_query: String::new(),
        limit: 0,
        offset: 0,
    })
}
