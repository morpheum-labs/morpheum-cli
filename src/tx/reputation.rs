use clap::{Args, Subcommand};

use morpheum_signing_native::signer::Signer;
use morpheum_sdk_native::reputation::ForceMilestoneBuilder;
use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `reputation` module.
///
/// Penalty/recovery are not client-submittable: reputation deltas are driven
/// consensus-side (system events), so the only governance transaction exposed
/// here is milestone forcing.
#[derive(Subcommand)]
pub enum ReputationCommands {
    /// Force a milestone level on an agent (governance only)
    ForceMilestone(ForceMilestoneArgs),
}

#[derive(Args)]
pub struct ForceMilestoneArgs {
    /// Target agent hash
    #[arg(long)]
    pub agent_hash: String,

    /// Milestone level to force (0-indexed, max 7)
    #[arg(long)]
    pub level: u32,

    /// Key name to sign with (must be governance key)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo for the transaction
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(cmd: ReputationCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        ReputationCommands::ForceMilestone(args) => force_milestone(args, &dispatcher).await,
    }
}

async fn force_milestone(
    args: ForceMilestoneArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let gov_sig = signer.public_key().to_proto_bytes();

    let request = ForceMilestoneBuilder::new()
        .agent_hash(&args.agent_hash)
        .milestone_level(args.level)
        .gov_signature(gov_sig)
        .build()
        .map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "Milestone forced on agent {}\nLevel: {}\nTxHash: {}",
        args.agent_hash, args.level, txhash,
    ));

    Ok(())
}
