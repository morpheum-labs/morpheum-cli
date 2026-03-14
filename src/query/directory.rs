use clap::{Args, Subcommand};

use morpheum_sdk_native::MorpheumSdk;
use morpheum_sdk_native::directory::types::DirectoryFilter;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

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
    let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);

    match cmd {
        DirectoryQueryCommands::Profile(args) => {
            query_profile(args, &sdk, &dispatcher.output).await
        }
        DirectoryQueryCommands::Profiles(args) => {
            query_profiles(args, &sdk, &dispatcher.output).await
        }
        DirectoryQueryCommands::Params => query_params(&sdk, &dispatcher.output).await,
    }
}

async fn query_profile(
    args: ProfileArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.directory()
        .query_and_print_optional(
            output,
            &format!("No directory profile found for agent {}", args.agent_hash),
            |c| async move { c.query_profile(&args.agent_hash).await },
        )
        .await
}

async fn query_profiles(
    args: ProfilesArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    let filter = build_filter(&args);
    sdk.directory()
        .query_and_print_paginated(output, |c| async move {
            c.query_profiles(args.limit, args.offset, filter).await
        })
        .await
}

async fn query_params(sdk: &MorpheumSdk, output: &Output) -> Result<(), CliError> {
    sdk.directory()
        .query_and_print_optional(
            output,
            "No directory parameters configured",
            |c| async move { c.query_params().await },
        )
        .await
}

fn build_filter(args: &ProfilesArgs) -> Option<DirectoryFilter> {
    let has_filters = args.min_reputation.is_some() || args.tags.is_some();

    if !has_filters {
        return None;
    }

    Some(DirectoryFilter {
        min_reputation: args.min_reputation.unwrap_or(0),
        tags: args.tags.clone().unwrap_or_default(),
        ..Default::default()
    })
}
