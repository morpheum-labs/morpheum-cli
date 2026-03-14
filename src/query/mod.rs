use clap::Subcommand;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

#[cfg(feature = "modules")]
pub mod identity;
#[cfg(feature = "modules")]
pub mod bank;
#[cfg(feature = "modules")]
pub mod reputation;
#[cfg(feature = "modules")]
pub mod validation;
#[cfg(feature = "modules")]
pub mod memory;
#[cfg(feature = "modules")]
pub mod vc;
#[cfg(feature = "modules")]
pub mod intent;
#[cfg(feature = "modules")]
pub mod marketplace;
#[cfg(feature = "modules")]
pub mod job;
#[cfg(feature = "modules")]
pub mod inference_registry;
#[cfg(feature = "modules")]
pub mod agent_registry;
#[cfg(feature = "modules")]
pub mod directory;
#[cfg(feature = "modules")]
pub mod interop;
#[cfg(feature = "modules")]
pub mod x402;

/// On-chain query commands across all Morpheum modules.
///
/// Symmetric counterpart to `TxCommands`. Gated behind the `modules`
/// feature until the SDK crates expose the required query types.
#[derive(Subcommand)]
pub enum QueryCommands {
    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Identity(identity::IdentityQueryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Bank(bank::BankQueryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Reputation(reputation::ReputationQueryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Validation(validation::ValidationQueryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Memory(memory::MemoryQueryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Vc(vc::VcQueryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Intent(intent::IntentQueryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Marketplace(marketplace::MarketplaceQueryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Job(job::JobQueryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    InferenceRegistry(inference_registry::InferenceRegistryQueryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    AgentRegistry(agent_registry::AgentRegistryQueryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Directory(directory::DirectoryQueryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Interop(interop::InteropQueryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    X402(x402::X402QueryCommands),
}

#[allow(clippy::unused_async)]
pub async fn execute(cmd: QueryCommands, _dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        #[cfg(feature = "modules")]
        QueryCommands::Identity(sub) => identity::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        QueryCommands::Bank(sub) => bank::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        QueryCommands::Reputation(sub) => reputation::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        QueryCommands::Validation(sub) => validation::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        QueryCommands::Memory(sub) => memory::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        QueryCommands::Vc(sub) => vc::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        QueryCommands::Intent(sub) => intent::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        QueryCommands::Marketplace(sub) => marketplace::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        QueryCommands::Job(sub) => job::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        QueryCommands::InferenceRegistry(sub) => inference_registry::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        QueryCommands::AgentRegistry(sub) => agent_registry::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        QueryCommands::Directory(sub) => directory::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        QueryCommands::Interop(sub) => interop::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        QueryCommands::X402(sub) => x402::execute(sub, dispatcher).await,
    }
}
