use clap::{Args, Subcommand};

use morpheum_sdk_native::marketplace::{
    ListAgentBuilder, PlaceBidBuilder, AcceptBidBuilder, RequestEvaluationBuilder,
    ListingType, RevenueShareConfig,
};
use morpheum_signing_native::signer::Signer;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `marketplace` module.
///
/// Covers agent listings, bidding, bid acceptance, and evaluation
/// requests in the native agent marketplace.
#[derive(Subcommand)]
pub enum MarketplaceCommands {
    /// List an agent for sale, co-ownership, or rental
    List(ListArgs),

    /// Place a bid on an active listing
    PlaceBid(PlaceBidArgs),

    /// Accept a bid on your listing
    AcceptBid(AcceptBidArgs),

    /// Request an independent evaluation of an agent
    RequestEvaluation(RequestEvaluationArgs),
}

#[derive(Args)]
pub struct ListArgs {
    /// Agent hash being listed
    #[arg(long)]
    pub agent_hash: String,

    /// Listing type (full-ownership, co-ownership, rental, evaluation-only)
    #[arg(long, value_parser = parse_listing_type)]
    pub listing_type: ListingType,

    /// Price in USD (integer, micro-precision handled on-chain)
    #[arg(long)]
    pub price_usd: u64,

    /// Hash of the listing metadata (off-chain description, screenshots, etc.)
    #[arg(long)]
    pub metadata_hash: Option<String>,

    /// Duration in seconds (for rental listings; 0 = permanent sale)
    #[arg(long, default_value = "0")]
    pub duration: u64,

    /// Creator revenue share in basis points
    #[arg(long, default_value = "5000")]
    pub creator_cut_bps: u32,

    /// Expiry timestamp for the listing (0 = no expiry)
    #[arg(long, default_value = "0")]
    pub expires_at: u64,

    /// Key name to sign with (must be agent owner)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct PlaceBidArgs {
    /// Listing ID to bid on
    #[arg(long)]
    pub listing_id: String,

    /// Bid amount in USD
    #[arg(long)]
    pub amount_usd: u64,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct AcceptBidArgs {
    /// Listing ID
    #[arg(long)]
    pub listing_id: String,

    /// Bid ID to accept
    #[arg(long)]
    pub bid_id: String,

    /// Key name to sign with (must be seller)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct RequestEvaluationArgs {
    /// Agent hash to evaluate
    #[arg(long)]
    pub agent_hash: String,

    /// Evaluator agent hash
    #[arg(long)]
    pub evaluator_hash: String,

    /// Evaluation fee in USD
    #[arg(long)]
    pub fee_usd: u64,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(cmd: MarketplaceCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        MarketplaceCommands::List(args) => list(args, &dispatcher).await,
        MarketplaceCommands::PlaceBid(args) => place_bid(args, &dispatcher).await,
        MarketplaceCommands::AcceptBid(args) => accept_bid(args, &dispatcher).await,
        MarketplaceCommands::RequestEvaluation(args) => {
            request_evaluation(args, &dispatcher).await
        }
    }
}

async fn list(args: ListArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let seller_hash = hex::encode(signer.account_id().0);
    let seller_sig = signer.public_key().to_proto_bytes();

    let revenue_share = RevenueShareConfig {
        creator_cut_bps: args.creator_cut_bps,
        seller_cut_bps: 10_000 - args.creator_cut_bps,
        evaluator_cut_bps: 0,
        platform_cut_bps: 0,
    };

    let mut builder = ListAgentBuilder::new()
        .agent_hash(&args.agent_hash)
        .seller_agent_hash(&seller_hash)
        .listing_type(args.listing_type)
        .price_usd(args.price_usd)
        .revenue_share_config(revenue_share)
        .seller_signature(seller_sig);

    if let Some(ref hash) = args.metadata_hash {
        builder = builder.metadata_hash(hash);
    }
    if args.duration > 0 {
        builder = builder.duration_seconds(args.duration);
    }
    if args.expires_at > 0 {
        builder = builder.expires_at(args.expires_at);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Agent {} listed for ${} ({:?})\nTxHash: {}",
        args.agent_hash, args.price_usd, args.listing_type, txhash,
    ));

    Ok(())
}

async fn place_bid(args: PlaceBidArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let bidder_sig = signer.public_key().to_proto_bytes();

    let request = PlaceBidBuilder::new()
        .listing_id(&args.listing_id)
        .amount_usd(args.amount_usd)
        .bidder_signature(bidder_sig)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Bid of ${} placed on listing {}\nTxHash: {}",
        args.amount_usd, args.listing_id, txhash,
    ));

    Ok(())
}

async fn accept_bid(args: AcceptBidArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let seller_sig = signer.public_key().to_proto_bytes();

    let request = AcceptBidBuilder::new()
        .listing_id(&args.listing_id)
        .bid_id(&args.bid_id)
        .seller_signature(seller_sig)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Bid {} accepted on listing {}\nTxHash: {}",
        args.bid_id, args.listing_id, txhash,
    ));

    Ok(())
}

async fn request_evaluation(
    args: RequestEvaluationArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let requester_sig = signer.public_key().to_proto_bytes();

    let request = RequestEvaluationBuilder::new()
        .agent_hash(&args.agent_hash)
        .evaluator_agent_hash(&args.evaluator_hash)
        .fee_usd(args.fee_usd)
        .requester_signature(requester_sig)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Evaluation requested for agent {}\nEvaluator: {}, Fee: ${}\nTxHash: {}",
        args.agent_hash, args.evaluator_hash, args.fee_usd, txhash,
    ));

    Ok(())
}

fn parse_listing_type(s: &str) -> Result<ListingType, String> {
    match s.to_lowercase().as_str() {
        "full-ownership" | "full" => Ok(ListingType::FullOwnership),
        "co-ownership" | "co" => Ok(ListingType::CoOwnership),
        "rental" | "rent" => Ok(ListingType::Rental),
        "evaluation-only" | "eval" => Ok(ListingType::EvaluationOnly),
        other => Err(format!(
            "unknown listing type '{other}'; expected: full-ownership, co-ownership, rental, evaluation-only"
        )),
    }
}
