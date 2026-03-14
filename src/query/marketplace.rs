use clap::{Args, Subcommand};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

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
    match cmd {
        MarketplaceQueryCommands::Listing(args) => query_listing(args, &dispatcher).await,
        MarketplaceQueryCommands::ActiveListings(args) => {
            query_active_listings(args, &dispatcher).await
        }
        MarketplaceQueryCommands::BidsByListing(args) => {
            query_bids_by_listing(args, &dispatcher).await
        }
        MarketplaceQueryCommands::Params => query_params(&dispatcher).await,
    }
}

async fn query_listing(args: ListingArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::marketplace::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_listing(tonic::Request::new(
            morpheum_proto::marketplace::v1::QueryListingRequest {
                listing_id: args.listing_id,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryListing failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_active_listings(
    args: ActiveListingsArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::marketplace::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_active_listings(tonic::Request::new(
            morpheum_proto::marketplace::v1::QueryActiveListingsRequest {
                limit: args.limit,
                offset: args.offset,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryActiveListings failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_bids_by_listing(
    args: BidsByListingArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::marketplace::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_bids_by_listing(tonic::Request::new(
            morpheum_proto::marketplace::v1::QueryBidsByListingRequest {
                listing_id: args.listing_id,
                limit: args.limit,
                offset: args.offset,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryBidsByListing failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::marketplace::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_params(tonic::Request::new(
            morpheum_proto::marketplace::v1::QueryParamsRequest::default(),
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryParams failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}
