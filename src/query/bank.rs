use clap::{Args, Subcommand};

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

pub async fn execute(cmd: BankQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        BankQueryCommands::Balance(args) => balance(args, &dispatcher).await,
        BankQueryCommands::Balances(args) => balances(args, &dispatcher).await,
    }
}

/// Resolves an asset name shorthand to a numeric asset index.
fn resolve_asset_index(name: &str) -> Result<u64, CliError> {
    match name.to_uppercase().as_str() {
        "MORM" => Ok(0),
        "USDC" => Ok(1),
        _ => Err(CliError::invalid_input(format!(
            "unknown asset name '{name}' — known: MORM (0), USDC (1)"
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
