use clap::{Args, Subcommand};

use morpheum_sdk_native::intent::{
    SubmitIntentBuilder, CancelIntentBuilder, IntentType, IntentParams,
    ConditionalParams, TwapParams, DeclarativeParams,
};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `intent` module.
///
/// Supports declarative, TWAP, conditional, and multi-leg intent
/// submission and cancellation.
#[derive(Subcommand)]
pub enum IntentCommands {
    /// Submit a conditional intent (if condition → execute action)
    SubmitConditional(SubmitConditionalArgs),

    /// Submit a TWAP intent (time-weighted execution)
    SubmitTwap(SubmitTwapArgs),

    /// Submit a declarative intent (natural-language goal)
    SubmitDeclarative(SubmitDeclarativeArgs),

    /// Cancel a pending intent
    Cancel(CancelArgs),
}

#[derive(Args)]
pub struct SubmitConditionalArgs {
    /// Agent hash submitting the intent
    #[arg(long)]
    pub agent_hash: String,

    /// Condition expression (e.g. "BTC/USD > 100000")
    #[arg(long)]
    pub condition: String,

    /// Action to execute when condition is met (e.g. "sell 1 BTC")
    #[arg(long)]
    pub action: String,

    /// VC proof hash authorising this intent
    #[arg(long)]
    pub vc_proof: Option<String>,

    /// Expiry timestamp (0 = use module default)
    #[arg(long, default_value = "0")]
    pub expiry: u64,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct SubmitTwapArgs {
    /// Agent hash
    #[arg(long)]
    pub agent_hash: String,

    /// Direction (buy or sell)
    #[arg(long)]
    pub direction: String,

    /// Total order size
    #[arg(long)]
    pub total_size: u64,

    /// Duration in milliseconds
    #[arg(long)]
    pub duration_ms: u64,

    /// Number of slices
    #[arg(long)]
    pub num_slices: u32,

    /// Maximum slippage tolerance in basis points
    #[arg(long, default_value = "50")]
    pub slippage_bps: u32,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct SubmitDeclarativeArgs {
    /// Agent hash
    #[arg(long)]
    pub agent_hash: String,

    /// Natural-language goal (e.g. "Rebalance portfolio to 60/40 BTC/ETH")
    #[arg(long)]
    pub goal: String,

    /// Optional constraints as JSON
    #[arg(long)]
    pub constraints: Option<String>,

    /// Preferred execution style (aggressive, conservative, balanced)
    #[arg(long, default_value = "balanced")]
    pub style: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct CancelArgs {
    /// Intent ID to cancel
    #[arg(long)]
    pub intent_id: String,

    /// Reason for cancellation
    #[arg(long)]
    pub reason: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(cmd: IntentCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        IntentCommands::SubmitConditional(args) => submit_conditional(args, &dispatcher).await,
        IntentCommands::SubmitTwap(args) => submit_twap(args, &dispatcher).await,
        IntentCommands::SubmitDeclarative(args) => submit_declarative(args, &dispatcher).await,
        IntentCommands::Cancel(args) => cancel(args, &dispatcher).await,
    }
}

async fn submit_conditional(
    args: SubmitConditionalArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let agent_sig = signer.public_key().as_bytes().to_vec();

    let params = IntentParams::Conditional(ConditionalParams {
        condition: args.condition.clone(),
        action: args.action.clone(),
    });

    let mut builder = SubmitIntentBuilder::new()
        .agent_hash(&args.agent_hash)
        .intent_type(IntentType::Conditional)
        .params(params)
        .agent_signature(agent_sig);

    if let Some(ref vc) = args.vc_proof {
        builder = builder.vc_proof_hash(vc);
    }
    if args.expiry > 0 {
        builder = builder.expiry_timestamp(args.expiry);
    }

    let request = builder.build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Conditional intent submitted\nCondition: {}\nAction: {}\nTxHash: {}",
        args.condition, args.action, result.txhash,
    ));

    Ok(())
}

async fn submit_twap(args: SubmitTwapArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let agent_sig = signer.public_key().as_bytes().to_vec();

    let params = IntentParams::Twap(TwapParams {
        direction: args.direction.clone(),
        total_size: args.total_size,
        duration_ms: args.duration_ms,
        num_slices: args.num_slices,
        slice_curve: String::new(),
        slippage_tolerance_bps: args.slippage_bps,
        rebalance_trigger: String::new(),
    });

    let request = SubmitIntentBuilder::new()
        .agent_hash(&args.agent_hash)
        .intent_type(IntentType::Twap)
        .params(params)
        .agent_signature(agent_sig)
        .build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "TWAP intent submitted\n{} {} over {}ms in {} slices\nTxHash: {}",
        args.direction, args.total_size, args.duration_ms, args.num_slices, result.txhash,
    ));

    Ok(())
}

async fn submit_declarative(
    args: SubmitDeclarativeArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let agent_sig = signer.public_key().as_bytes().to_vec();

    let params = IntentParams::Declarative(DeclarativeParams {
        raw_goal: args.goal.clone(),
        goal_embedding: Vec::new(),
        constraints: args.constraints.clone().unwrap_or_default(),
        preferred_style: args.style.clone(),
    });

    let request = SubmitIntentBuilder::new()
        .agent_hash(&args.agent_hash)
        .intent_type(IntentType::Declarative)
        .params(params)
        .agent_signature(agent_sig)
        .build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Declarative intent submitted\nGoal: {}\nStyle: {}\nTxHash: {}",
        args.goal, args.style, result.txhash,
    ));

    Ok(())
}

async fn cancel(args: CancelArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let agent_sig = signer.public_key().as_bytes().to_vec();

    let request = CancelIntentBuilder::new()
        .intent_id(&args.intent_id)
        .agent_signature(agent_sig)
        .reason(&args.reason)
        .build()?;

    let result = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Intent {} cancelled\nReason: {}\nTxHash: {}",
        args.intent_id, args.reason, result.txhash,
    ));

    Ok(())
}
