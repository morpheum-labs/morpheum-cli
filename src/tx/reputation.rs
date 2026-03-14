use clap::{Args, Subcommand};

use morpheum_sdk_native::reputation::{
    ApplyPenaltyBuilder, ApplyRecoveryBuilder, ForceMilestoneBuilder, RecoveryActionType,
};
use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `reputation` module.
///
/// Covers penalty application, recovery boosts, and milestone forcing —
/// the full reputation lifecycle described in Pillar 3
/// (On-Chain Trust, Identity & Reputation).
#[derive(Subcommand)]
pub enum ReputationCommands {
    /// Apply a penalty to an agent's reputation score
    ApplyPenalty(ApplyPenaltyArgs),

    /// Apply a positive recovery boost to an agent's reputation
    ApplyRecovery(ApplyRecoveryArgs),

    /// Force a milestone level on an agent (governance only)
    ForceMilestone(ForceMilestoneArgs),
}

#[derive(Args)]
pub struct ApplyPenaltyArgs {
    /// Target agent hash (hex-encoded SHA-256 of the agent DID)
    #[arg(long)]
    pub agent_hash: String,

    /// Base penalty amount (before multiplier)
    #[arg(long)]
    pub base_amount: u64,

    /// Multiplier in basis-unit form (100 = 1.0x, 200 = 2.0x)
    #[arg(long, default_value = "100")]
    pub multiplier: u32,

    /// Human-readable reason for the penalty
    #[arg(long)]
    pub reason: String,

    /// Key name to sign with (governance or module authority)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo for the transaction
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct ApplyRecoveryArgs {
    /// Target agent hash
    #[arg(long)]
    pub agent_hash: String,

    /// Recovery action type (trade-fill, valid-proof, uptime-24h, marketplace-sale, milestone)
    #[arg(long, value_parser = parse_recovery_action)]
    pub action_type: RecoveryActionType,

    /// Amount of reputation to add
    #[arg(long)]
    pub amount: u64,

    /// Human-readable reason for the recovery
    #[arg(long)]
    pub reason: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo for the transaction
    #[arg(long)]
    pub memo: Option<String>,
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
        ReputationCommands::ApplyPenalty(args) => apply_penalty(args, &dispatcher).await,
        ReputationCommands::ApplyRecovery(args) => apply_recovery(args, &dispatcher).await,
        ReputationCommands::ForceMilestone(args) => force_milestone(args, &dispatcher).await,
    }
}

async fn apply_penalty(args: ApplyPenaltyArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let signer_bytes = signer.public_key().as_bytes().to_vec();

    let request = ApplyPenaltyBuilder::new()
        .agent_hash(&args.agent_hash)
        .base_amount(args.base_amount)
        .multiplier(args.multiplier)
        .reason(&args.reason)
        .signer(signer_bytes)
        .build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "Penalty applied to agent {}\nBase: {} (x{})\nReason: {}\nTxHash: {}",
        args.agent_hash, args.base_amount, args.multiplier, args.reason, result.txhash,
    ));

    Ok(())
}

async fn apply_recovery(
    args: ApplyRecoveryArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;

    let request = ApplyRecoveryBuilder::new()
        .agent_hash(&args.agent_hash)
        .action_type(args.action_type)
        .amount(args.amount)
        .reason(&args.reason)
        .build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "Recovery applied to agent {}\nAction: {}, Amount: {}\nReason: {}\nTxHash: {}",
        args.agent_hash, args.action_type, args.amount, args.reason, result.txhash,
    ));

    Ok(())
}

async fn force_milestone(
    args: ForceMilestoneArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let gov_sig = signer.public_key().as_bytes().to_vec();

    let request = ForceMilestoneBuilder::new()
        .agent_hash(&args.agent_hash)
        .milestone_level(args.level)
        .gov_signature(gov_sig)
        .build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "Milestone forced on agent {}\nLevel: {}\nTxHash: {}",
        args.agent_hash, args.level, result.txhash,
    ));

    Ok(())
}

fn parse_recovery_action(s: &str) -> Result<RecoveryActionType, String> {
    match s.to_lowercase().as_str() {
        "trade-fill" | "tradefill" => Ok(RecoveryActionType::TradeFill),
        "valid-proof" | "validproof" => Ok(RecoveryActionType::ValidProof),
        "uptime-24h" | "uptime24h" => Ok(RecoveryActionType::Uptime24h),
        "marketplace-sale" | "marketplacesale" => Ok(RecoveryActionType::MarketplaceSale),
        "milestone" => Ok(RecoveryActionType::Milestone),
        other => Err(format!(
            "unknown recovery action '{other}'; expected one of: \
             trade-fill, valid-proof, uptime-24h, marketplace-sale, milestone"
        )),
    }
}
