use clap::{Args, Subcommand};

use morpheum_signing_native::signer::Signer;
use morpheum_sdk_native::directory::{
    UpdateProfileBuilder, UpdateVisibilityBuilder, VisibilityLevel,
};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `directory` module.
///
/// Manages agent profiles, visibility settings, and discovery
/// metadata in the on-chain agent directory.
#[derive(Subcommand)]
pub enum DirectoryCommands {
    /// Update an agent's public directory profile
    UpdateProfile(UpdateProfileArgs),

    /// Change an agent's visibility level
    UpdateVisibility(UpdateVisibilityArgs),
}

#[derive(Args)]
pub struct UpdateProfileArgs {
    /// Agent hash
    #[arg(long)]
    pub agent_hash: String,

    /// New display name
    #[arg(long)]
    pub display_name: String,

    /// New description
    #[arg(long)]
    pub description: String,

    /// Comma-separated tags for discovery (e.g. "trading,defi,arbitrage")
    #[arg(long)]
    pub tags: String,

    /// Key name to sign with (must be agent owner)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct UpdateVisibilityArgs {
    /// Agent hash
    #[arg(long)]
    pub agent_hash: String,

    /// New visibility level (public, owner-only, evaluator-only)
    #[arg(long, value_parser = parse_visibility)]
    pub visibility: VisibilityLevel,

    /// Key name to sign with (must be agent owner)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(cmd: DirectoryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        DirectoryCommands::UpdateProfile(args) => update_profile(args, &dispatcher).await,
        DirectoryCommands::UpdateVisibility(args) => update_visibility(args, &dispatcher).await,
    }
}

async fn update_profile(
    args: UpdateProfileArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let owner_sig = signer.public_key().to_proto_bytes();

    let request = UpdateProfileBuilder::new()
        .agent_hash(&args.agent_hash)
        .display_name(&args.display_name)
        .description(&args.description)
        .tags(&args.tags)
        .owner_signature(owner_sig)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Profile updated for agent {}\nName: {}\nTxHash: {}",
        args.agent_hash, args.display_name, txhash,
    ));

    Ok(())
}

async fn update_visibility(
    args: UpdateVisibilityArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let owner_sig = signer.public_key().to_proto_bytes();

    let request = UpdateVisibilityBuilder::new()
        .agent_hash(&args.agent_hash)
        .new_visibility(args.visibility)
        .owner_signature(owner_sig)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Visibility updated for agent {}\nNew level: {:?}\nTxHash: {}",
        args.agent_hash, args.visibility, txhash,
    ));

    Ok(())
}

fn parse_visibility(s: &str) -> Result<VisibilityLevel, String> {
    match s.to_lowercase().as_str() {
        "public" => Ok(VisibilityLevel::Public),
        "owner-only" | "owner" | "private" => Ok(VisibilityLevel::OwnerOnly),
        "evaluator-only" | "evaluator" => Ok(VisibilityLevel::EvaluatorOnly),
        other => Err(format!(
            "unknown visibility '{other}'; expected: public, owner-only, evaluator-only"
        )),
    }
}
