use clap::{Args, Subcommand, ValueEnum};

use morpheum_sdk_native::intent::{
    SubmitIntentBuilder, CancelIntentBuilder,
    Comparator, ConditionalParams, DeclarativeParams, OrderAction, Side, SliceCurve, Tif,
    TriggerCondition, TwapParams,
};
use morpheum_signing_native::signer::Signer;

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

/// Order side for an execution-engine order or trigger action.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CliSide {
    Buy,
    Sell,
}

impl From<CliSide> for Side {
    fn from(s: CliSide) -> Self {
        match s {
            CliSide::Buy => Side::Buy,
            CliSide::Sell => Side::Sell,
        }
    }
}

/// Time-in-force for an execution-engine order.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CliTif {
    Gtc,
    Ioc,
    Fok,
}

impl From<CliTif> for Tif {
    fn from(t: CliTif) -> Self {
        match t {
            CliTif::Gtc => Tif::Gtc,
            CliTif::Ioc => Tif::Ioc,
            CliTif::Fok => Tif::Fok,
        }
    }
}

/// Comparator for a conditional trigger evaluated against the committed mark.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CliComparator {
    Above,
    Below,
}

impl From<CliComparator> for Comparator {
    fn from(c: CliComparator) -> Self {
        match c {
            CliComparator::Above => Comparator::Above,
            CliComparator::Below => Comparator::Below,
        }
    }
}

#[derive(Args)]
pub struct SubmitConditionalArgs {
    /// Agent hash submitting the intent
    #[arg(long)]
    pub agent_hash: String,

    /// Market the committed-mark trigger watches
    #[arg(long)]
    pub market_index: u64,

    /// Comparator applied to the committed mark against `trigger_price_e8`
    #[arg(long, value_enum)]
    pub cmp: CliComparator,

    /// Committed-mark trigger price (1e8 fixed-point decimal string)
    #[arg(long)]
    pub trigger_price_e8: String,

    /// Bucket the action order trades against
    #[arg(long)]
    pub bucket_id: u64,

    /// Side of the order to place when the trigger fires
    #[arg(long, value_enum)]
    pub side: CliSide,

    /// Order quantity (1e8 satoshi-scale)
    #[arg(long)]
    pub quantity: u64,

    /// Limit price for the action order (1e8 fixed-point decimal string)
    #[arg(long)]
    pub price_e8: String,

    /// Time-in-force for the action order
    #[arg(long, value_enum, default_value = "gtc")]
    pub tif: CliTif,

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

    /// Market to execute the TWAP against
    #[arg(long)]
    pub market_index: u64,

    /// Bucket the TWAP slices trade against
    #[arg(long)]
    pub bucket_id: u64,

    /// Direction (buy or sell)
    #[arg(long, value_enum)]
    pub side: CliSide,

    /// Total order size
    #[arg(long)]
    pub total_size: u64,

    /// Duration in milliseconds
    #[arg(long)]
    pub duration_ms: u64,

    /// Number of slices
    #[arg(long)]
    pub num_slices: u32,

    /// Time-in-force for each slice
    #[arg(long, value_enum, default_value = "gtc")]
    pub tif: CliTif,

    /// Per-slice limit price (1e8 fixed-point decimal string)
    #[arg(long)]
    pub limit_price_e8: String,

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
    let agent_sig = signer.public_key().to_proto_bytes();

    let params = ConditionalParams {
        condition: TriggerCondition {
            market_index: args.market_index,
            cmp: args.cmp.into(),
            trigger_price_e8: args.trigger_price_e8.clone(),
        },
        action: OrderAction {
            market_index: args.market_index,
            bucket_id: args.bucket_id,
            side: args.side.into(),
            quantity: args.quantity,
            price_e8: args.price_e8.clone(),
            tif: args.tif.into(),
        },
    };

    let mut builder = SubmitIntentBuilder::new()
        .agent_hash(&args.agent_hash)
        .conditional(params)
        .agent_signature(agent_sig);

    if let Some(ref vc) = args.vc_proof {
        builder = builder.vc_proof_hash(vc);
    }
    if args.expiry > 0 {
        builder = builder.expiry_timestamp(args.expiry);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Conditional intent submitted\nTrigger: market={} {:?} {}\nAction: {:?} qty={} @ {}\nTxHash: {}",
        args.market_index, args.cmp, args.trigger_price_e8,
        args.side, args.quantity, args.price_e8, txhash,
    ));

    Ok(())
}

async fn submit_twap(args: SubmitTwapArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let agent_sig = signer.public_key().to_proto_bytes();

    let params = TwapParams {
        market_index: args.market_index,
        bucket_id: args.bucket_id,
        side: args.side.into(),
        total_size: args.total_size,
        num_slices: args.num_slices,
        duration_ms: args.duration_ms,
        curve: SliceCurve::Uniform,
        tif: args.tif.into(),
        limit_price_e8: args.limit_price_e8.clone(),
    };

    let request = SubmitIntentBuilder::new()
        .agent_hash(&args.agent_hash)
        .twap(params)
        .agent_signature(agent_sig)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "TWAP intent submitted\n{:?} {} over {}ms in {} slices @ {}\nTxHash: {}",
        args.side, args.total_size, args.duration_ms, args.num_slices, args.limit_price_e8, txhash,
    ));

    Ok(())
}

async fn submit_declarative(
    args: SubmitDeclarativeArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let agent_sig = signer.public_key().to_proto_bytes();

    let params = DeclarativeParams {
        raw_goal: args.goal.clone(),
        goal_embedding: Vec::new(),
        constraints: args.constraints.clone().unwrap_or_default(),
        preferred_style: args.style.clone(),
    };

    let request = SubmitIntentBuilder::new()
        .agent_hash(&args.agent_hash)
        .declarative(params)
        .agent_signature(agent_sig)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Declarative intent submitted\nGoal: {}\nStyle: {}\nTxHash: {}",
        args.goal, args.style, txhash,
    ));

    Ok(())
}

async fn cancel(args: CancelArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let agent_sig = signer.public_key().to_proto_bytes();

    let request = CancelIntentBuilder::new()
        .intent_id(&args.intent_id)
        .agent_signature(agent_sig)
        .reason(&args.reason)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Intent {} cancelled\nReason: {}\nTxHash: {}",
        args.intent_id, args.reason, txhash,
    ));

    Ok(())
}
