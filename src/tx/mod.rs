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

/// On-chain transaction commands across all Morpheum modules.
///
/// Each variant maps 1:1 to a Mormcore module. Gated behind the `modules`
/// feature until the SDK crates expose the required message types.
#[derive(Subcommand)]
pub enum TxCommands {
    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Identity(identity::IdentityCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Bank(bank::BankCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Reputation(reputation::ReputationCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Validation(validation::ValidationCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Memory(memory::MemoryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Vc(vc::VcCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Intent(intent::IntentCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Marketplace(marketplace::MarketplaceCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Job(job::JobCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    InferenceRegistry(inference_registry::InferenceRegistryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    AgentRegistry(agent_registry::AgentRegistryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Directory(directory::DirectoryCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    Interop(interop::InteropCommands),

    #[cfg(feature = "modules")]
    #[command(subcommand)]
    X402(x402::X402Commands),
}

#[allow(clippy::unused_async)]
pub async fn execute(cmd: TxCommands, _dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        #[cfg(feature = "modules")]
        TxCommands::Identity(sub) => identity::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        TxCommands::Bank(sub) => bank::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        TxCommands::Reputation(sub) => reputation::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        TxCommands::Validation(sub) => validation::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        TxCommands::Memory(sub) => memory::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        TxCommands::Vc(sub) => vc::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        TxCommands::Intent(sub) => intent::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        TxCommands::Marketplace(sub) => marketplace::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        TxCommands::Job(sub) => job::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        TxCommands::InferenceRegistry(sub) => inference_registry::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        TxCommands::AgentRegistry(sub) => agent_registry::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        TxCommands::Directory(sub) => directory::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        TxCommands::Interop(sub) => interop::execute(sub, dispatcher).await,
        #[cfg(feature = "modules")]
        TxCommands::X402(sub) => x402::execute(sub, dispatcher).await,
    }
}
