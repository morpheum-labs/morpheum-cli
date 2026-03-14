use clap::{Args, Subcommand};

use morpheum_sdk_native::MorpheumSdk;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

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
    let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);

    match cmd {
        X402QueryCommands::Balance(args) => {
            query_balance(args, &sdk, &dispatcher.output).await
        }
        X402QueryCommands::Balances(args) => {
            query_balances(args, &sdk, &dispatcher.output).await
        }
    }
}

async fn query_balance(
    args: BalanceArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.bank()
        .query_and_print_item(output, |c| async move {
            c.query_balance(&args.address, args.asset_index).await
        })
        .await
}

async fn query_balances(
    args: BalancesArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.bank()
        .query_and_print_list(output, |c| async move {
            c.query_balances(&args.address).await
        })
        .await
}
