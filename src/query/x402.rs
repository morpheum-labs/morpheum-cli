use clap::{Args, Subcommand};

use morpheum_proto::bank::v1::query_client::QueryClient;
use morpheum_proto::bank::v1::{QueryBalanceRequest, QueryBalancesRequest};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for native x402 payment protocol queries.
///
/// Since x402 settlements are executed as bank transfers with protocol-specific
/// memo metadata, balance and transaction queries are routed through the bank
/// module client. This provides native, non-mocked query access using existing
/// primitives.
#[derive(Subcommand)]
pub enum X402QueryCommands {
    /// Get the x402 escrow/payment balance for an address
    Balance(BalanceArgs),

    /// List all balances for an x402 participant
    Balances(BalancesArgs),
}

#[derive(Args)]
pub struct BalanceArgs {
    /// Bech32 address of the x402 participant
    #[arg(required = true)]
    pub address: String,

    /// Asset index (0 = native MORM, used for x402 settlements)
    #[arg(long, default_value = "0")]
    pub asset_index: u64,
}

#[derive(Args)]
pub struct BalancesArgs {
    /// Bech32 address of the x402 participant
    #[arg(required = true)]
    pub address: String,
}

pub async fn execute(cmd: X402QueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        X402QueryCommands::Balance(args) => query_balance(args, &dispatcher).await,
        X402QueryCommands::Balances(args) => query_balances(args, &dispatcher).await,
    }
}

async fn query_balance(args: BalanceArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_balance(tonic::Request::new(QueryBalanceRequest {
            address: args.address,
            asset_index: args.asset_index,
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

async fn query_balances(args: BalancesArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_balances(tonic::Request::new(QueryBalancesRequest {
            address: args.address,
            chain_type: None,
            asset_type_filter: None,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryBalances failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}
