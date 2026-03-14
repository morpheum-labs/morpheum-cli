use clap::{Args, Subcommand};

use morpheum_sdk_native::MorpheumSdk;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

/// Query commands for the `marketplace` module.
///
/// Read-only access to agent marketplace listings, bids,
/// active listings, and module parameters.
#[derive(Subcommand)]
pub enum MarketplaceQueryCommands {
    /// Get a specific listing by ID
    Listing(ListingArgs),

    /// List all currently active listings (paginated)
    ActiveListings(ActiveListingsArgs),

    /// List bids for a specific listing (paginated)
    BidsByListing(BidsByListingArgs),

    /// Get the current marketplace module parameters
    Params,
}

#[derive(Args)]
pub struct ListingArgs {
    /// Listing ID
    #[arg(required = true)]
    pub listing_id: String,
}

#[derive(Args)]
pub struct ActiveListingsArgs {
    #[arg(long, default_value = "20")]
    pub limit: u32,

    #[arg(long, default_value = "0")]
    pub offset: u32,
}

#[derive(Args)]
pub struct BidsByListingArgs {
    /// Listing ID to query bids for
    #[arg(required = true)]
    pub listing_id: String,

    #[arg(long, default_value = "20")]
    pub limit: u32,

    #[arg(long, default_value = "0")]
    pub offset: u32,
}

pub async fn execute(
    cmd: MarketplaceQueryCommands,
    dispatcher: Dispatcher,
) -> Result<(), CliError> {
    let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);

    match cmd {
        MarketplaceQueryCommands::Listing(args) => {
            query_listing(args, &sdk, &dispatcher.output).await
        }
        MarketplaceQueryCommands::ActiveListings(args) => {
            query_active_listings(args, &sdk, &dispatcher.output).await
        }
        MarketplaceQueryCommands::BidsByListing(args) => {
            query_bids_by_listing(args, &sdk, &dispatcher.output).await
        }
        MarketplaceQueryCommands::Params => query_params(&sdk, &dispatcher.output).await,
    }
}

async fn query_listing(
    args: ListingArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.marketplace()
        .query_and_print_optional(
            output,
            &format!("No listing found with ID {}", args.listing_id),
            |c| async move { c.query_listing(&args.listing_id).await },
        )
        .await
}

async fn query_active_listings(
    args: ActiveListingsArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.marketplace()
        .query_and_print_paginated(output, |c| async move {
            c.query_active_listings(args.limit, args.offset).await
        })
        .await
}

async fn query_bids_by_listing(
    args: BidsByListingArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.marketplace()
        .query_and_print_paginated(output, |c| async move {
            c.query_bids_by_listing(&args.listing_id, args.limit, args.offset)
                .await
        })
        .await
}

async fn query_params(sdk: &MorpheumSdk, output: &Output) -> Result<(), CliError> {
    sdk.marketplace()
        .query_and_print_optional(
            output,
            "No marketplace parameters configured",
            |c| async move { c.query_params().await },
        )
        .await
}
