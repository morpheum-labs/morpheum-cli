use clap::Subcommand;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

pub mod registry;

#[cfg(feature = "identity")]
pub mod identity;
#[cfg(feature = "bank")]
pub mod bank;
#[cfg(feature = "reputation")]
pub mod reputation;
#[cfg(feature = "validation")]
pub mod validation;
#[cfg(feature = "memory")]
pub mod memory;
#[cfg(feature = "vc")]
pub mod vc;
#[cfg(feature = "intent")]
pub mod intent;
#[cfg(feature = "marketplace")]
pub mod marketplace;
#[cfg(feature = "job")]
pub mod job;
#[cfg(feature = "inferreg")]
pub mod inferreg;
#[cfg(feature = "agentreg")]
pub mod agentreg;
#[cfg(feature = "directory")]
pub mod directory;
#[cfg(feature = "interop")]
pub mod interop;
#[cfg(feature = "x402")]
pub mod x402;
#[cfg(all(feature = "gmp", feature = "interop"))]
pub mod gmp;
#[cfg(feature = "gov")]
pub mod gov;
#[cfg(feature = "cosmwasm")]
pub mod cosmwasm;
#[cfg(feature = "cctp")]
pub mod cctp;
#[cfg(feature = "svm")]
pub mod svm_usdc;

/// On-chain query commands across all Morpheum modules.
///
/// Symmetric counterpart to `TxCommands`, individually gated per module.
#[derive(Subcommand)]
pub enum QueryCommands {
    /// Query the SDK chain/token registries (supported chains, tokens, routes)
    #[command(subcommand)]
    Registry(registry::RegistryQueryCommands),

    #[cfg(feature = "identity")]
    #[command(subcommand)]
    Identity(identity::IdentityQueryCommands),

    #[cfg(feature = "bank")]
    #[command(subcommand)]
    Bank(bank::BankQueryCommands),

    #[cfg(feature = "reputation")]
    #[command(subcommand)]
    Reputation(reputation::ReputationQueryCommands),

    #[cfg(feature = "validation")]
    #[command(subcommand)]
    Validation(validation::ValidationQueryCommands),

    #[cfg(feature = "memory")]
    #[command(subcommand)]
    Memory(memory::MemoryQueryCommands),

    #[cfg(feature = "vc")]
    #[command(subcommand)]
    Vc(vc::VcQueryCommands),

    #[cfg(feature = "intent")]
    #[command(subcommand)]
    Intent(intent::IntentQueryCommands),

    #[cfg(feature = "marketplace")]
    #[command(subcommand)]
    Marketplace(marketplace::MarketplaceQueryCommands),

    #[cfg(feature = "job")]
    #[command(subcommand)]
    Job(job::JobQueryCommands),

    #[cfg(feature = "inferreg")]
    #[command(subcommand)]
    InferenceRegistry(inferreg::InferenceRegistryQueryCommands),

    #[cfg(feature = "agentreg")]
    #[command(subcommand)]
    AgentRegistry(agentreg::AgentRegistryQueryCommands),

    #[cfg(feature = "directory")]
    #[command(subcommand)]
    Directory(directory::DirectoryQueryCommands),

    #[cfg(feature = "interop")]
    #[command(subcommand)]
    Interop(interop::InteropQueryCommands),

    #[cfg(feature = "x402")]
    #[command(subcommand)]
    X402(x402::X402QueryCommands),

    #[cfg(all(feature = "gmp", feature = "interop"))]
    #[command(subcommand)]
    Gmp(gmp::GmpQueryCommands),

    #[cfg(feature = "gov")]
    #[command(subcommand)]
    Gov(gov::GovQueryCommands),

    /// CosmWasm contract queries (smart, raw, contract-info)
    #[cfg(feature = "cosmwasm")]
    #[command(subcommand)]
    Cosmwasm(cosmwasm::CosmwasmQueryCommands),

    /// CCTP handler queries (config, pending, routes)
    #[cfg(feature = "cctp")]
    #[command(subcommand)]
    Cctp(cctp::CctpQueryCommands),

    /// SVM USDC native program queries (program-id, balance, allowance)
    #[cfg(feature = "svm")]
    #[command(subcommand)]
    SvmUsdc(svm_usdc::SvmUsdcQueryCommands),
}

#[allow(clippy::unused_async, unused_variables)]
pub async fn execute(cmd: QueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        QueryCommands::Registry(sub) => registry::execute(sub, dispatcher).await,
        #[cfg(feature = "identity")]
        QueryCommands::Identity(sub) => identity::execute(sub, dispatcher).await,
        #[cfg(feature = "bank")]
        QueryCommands::Bank(sub) => bank::execute(sub, dispatcher).await,
        #[cfg(feature = "reputation")]
        QueryCommands::Reputation(sub) => reputation::execute(sub, dispatcher).await,
        #[cfg(feature = "validation")]
        QueryCommands::Validation(sub) => validation::execute(sub, dispatcher).await,
        #[cfg(feature = "memory")]
        QueryCommands::Memory(sub) => memory::execute(sub, dispatcher).await,
        #[cfg(feature = "vc")]
        QueryCommands::Vc(sub) => vc::execute(sub, dispatcher).await,
        #[cfg(feature = "intent")]
        QueryCommands::Intent(sub) => intent::execute(sub, dispatcher).await,
        #[cfg(feature = "marketplace")]
        QueryCommands::Marketplace(sub) => marketplace::execute(sub, dispatcher).await,
        #[cfg(feature = "job")]
        QueryCommands::Job(sub) => job::execute(sub, dispatcher).await,
        #[cfg(feature = "inferreg")]
        QueryCommands::InferenceRegistry(sub) => inferreg::execute(sub, dispatcher).await,
        #[cfg(feature = "agentreg")]
        QueryCommands::AgentRegistry(sub) => agentreg::execute(sub, dispatcher).await,
        #[cfg(feature = "directory")]
        QueryCommands::Directory(sub) => directory::execute(sub, dispatcher).await,
        #[cfg(feature = "interop")]
        QueryCommands::Interop(sub) => interop::execute(sub, dispatcher).await,
        #[cfg(feature = "x402")]
        QueryCommands::X402(sub) => x402::execute(sub, dispatcher).await,
        #[cfg(all(feature = "gmp", feature = "interop"))]
        QueryCommands::Gmp(sub) => gmp::execute(sub, dispatcher).await,
        #[cfg(feature = "gov")]
        QueryCommands::Gov(sub) => gov::execute(sub, dispatcher).await,
        #[cfg(feature = "cosmwasm")]
        QueryCommands::Cosmwasm(sub) => cosmwasm::execute(sub, dispatcher).await,
        #[cfg(feature = "cctp")]
        QueryCommands::Cctp(sub) => cctp::execute(sub, dispatcher).await,
        #[cfg(feature = "svm")]
        QueryCommands::SvmUsdc(sub) => svm_usdc::execute(sub, dispatcher).await,
    }
}
