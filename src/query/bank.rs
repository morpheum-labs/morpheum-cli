use clap::{Args, Subcommand};
use morpheum_primitives::constants::assets;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the `bank` module.
///
/// Read-only access to account balances, asset metadata, and supply info.
#[derive(Subcommand)]
pub enum BankQueryCommands {
    /// Get the balance of a specific asset for an address
    Balance(BalanceArgs),

    /// Get all balances for an address
    Balances(BalancesArgs),

    /// List all registered assets in the on-chain asset registry
    Assets(AssetsArgs),
}

#[derive(Args)]
pub struct BalanceArgs {
    /// Account address (hex)
    #[arg(long)]
    pub address: String,

    /// Asset index (numeric). Overridden by --asset if provided.
    #[arg(long, default_value = "0")]
    pub asset_index: u64,

    /// Asset name shorthand (e.g. "USDC" -> 1, "MORM" -> 0).
    /// Takes precedence over --asset-index.
    #[arg(long)]
    pub asset: Option<String>,
}

#[derive(Args)]
pub struct BalancesArgs {
    /// Account address (hex)
    #[arg(long)]
    pub address: String,
}

#[derive(Args)]
pub struct AssetsArgs {
    /// Filter by asset type (numeric proto enum value).
    /// 1=NATIVE_MORM, 2=TOKEN, 3=RWA, 4=STOCK, 5=COMMODITY, 6=CUSTOM, 7=CANONICAL_STABLECOIN
    #[arg(long)]
    pub type_filter: Option<i32>,
}

pub async fn execute(cmd: BankQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        BankQueryCommands::Balance(args) => balance(args, &dispatcher).await,
        BankQueryCommands::Balances(args) => balances(args, &dispatcher).await,
        BankQueryCommands::Assets(args) => query_assets(args, &dispatcher).await,
    }
}

fn resolve_asset_index(name: &str) -> Result<u64, CliError> {
    match name.to_uppercase().as_str() {
        "MORM" => Ok(assets::MORM_INDEX),
        "USDC" => Ok(assets::USDC_INDEX),
        "BTC" => Ok(assets::BTC_INDEX),
        "ETH" => Ok(assets::ETH_INDEX),
        "USDT" => Ok(assets::USDT_INDEX),
        "SOL" => Ok(assets::SOL_INDEX),
        _ => Err(CliError::invalid_input(format!(
            "unknown asset name '{name}' — known: MORM (0), USDC (1), BTC (2), ETH (3), USDT (4), SOL (5)"
        ))),
    }
}

async fn balance(args: BalanceArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let asset_index = match &args.asset {
        Some(name) => resolve_asset_index(name)?,
        None => args.asset_index,
    };

    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::bank::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_balance(tonic::Request::new(morpheum_proto::bank::v1::QueryBalanceRequest {
            address: args.address.clone(),
            asset_index,
            chain_type: None,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryBalance failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn balances(args: BalancesArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::bank::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_balances(tonic::Request::new(
            morpheum_proto::bank::v1::QueryBalancesRequest {
                address: args.address.clone(),
                chain_type: None,
                asset_type_filter: None,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryBalances failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_assets(args: AssetsArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::bank::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_assets(tonic::Request::new(
            morpheum_proto::bank::v1::QueryAssetsRequest {
                type_filter: args.type_filter,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryAssets failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}
