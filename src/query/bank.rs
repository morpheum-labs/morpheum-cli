use clap::{Args, Subcommand};

use morpheum_sdk_native::MorpheumSdk;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

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
    /// Bech32 address (e.g. morm1...)
    #[arg(required = true)]
    pub address: String,

    /// Asset index (0 = native MORM)
    #[arg(long, default_value = "0")]
    pub asset_index: u64,
}

#[derive(Args)]
pub struct BalancesArgs {
    /// Bech32 address
    #[arg(required = true)]
    pub address: String,
}

pub async fn execute(cmd: BankQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);

    match cmd {
        BankQueryCommands::Balance(args) => {
            query_balance(args, &sdk, &dispatcher.output).await
        }
        BankQueryCommands::Balances(args) => {
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
