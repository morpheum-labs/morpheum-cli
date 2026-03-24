use clap::{Args, Subcommand};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the `bank` module.
///
/// Read-only access to account balances, asset metadata, and supply info.
/// All queries are delegated to `morpheum_sdk_bank::BankClient`.
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

async fn balance(args: BalanceArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let asset_index = match &args.asset {
        Some(name) => morpheum_sdk_native::bank::resolve_asset_index(name)?,
        None => args.asset_index,
    };

    let client = dispatcher.bank_client().await?;
    let result = client.query_balance(&args.address, asset_index).await?;
    let json = serde_json::to_string_pretty(&result)
        .unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn balances(args: BalancesArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let client = dispatcher.bank_client().await?;
    let result = client.query_balances(&args.address).await?;
    let json = serde_json::to_string_pretty(&result)
        .unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_assets(args: AssetsArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let client = dispatcher.bank_client().await?;
    let result = client.query_assets(args.type_filter).await?;
    let json = serde_json::to_string_pretty(&result)
        .unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}
