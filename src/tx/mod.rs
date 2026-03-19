use clap::Subcommand;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

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
#[cfg(feature = "inference_registry")]
pub mod inference_registry;
#[cfg(feature = "agent_registry")]
pub mod agent_registry;
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

/// On-chain transaction commands across all Morpheum modules.
///
/// Each variant maps 1:1 to a Mormcore module, individually gated behind
/// its own feature flag until the corresponding SDK crate is ready.
#[derive(Subcommand)]
pub enum TxCommands {
    #[cfg(feature = "identity")]
    #[command(subcommand)]
    Identity(identity::IdentityCommands),

    #[cfg(feature = "bank")]
    #[command(subcommand)]
    Bank(bank::BankCommands),

    #[cfg(feature = "reputation")]
    #[command(subcommand)]
    Reputation(reputation::ReputationCommands),

    #[cfg(feature = "validation")]
    #[command(subcommand)]
    Validation(validation::ValidationCommands),

    #[cfg(feature = "memory")]
    #[command(subcommand)]
    Memory(memory::MemoryCommands),

    #[cfg(feature = "vc")]
    #[command(subcommand)]
    Vc(vc::VcCommands),

    #[cfg(feature = "intent")]
    #[command(subcommand)]
    Intent(intent::IntentCommands),

    #[cfg(feature = "marketplace")]
    #[command(subcommand)]
    Marketplace(marketplace::MarketplaceCommands),

    #[cfg(feature = "job")]
    #[command(subcommand)]
    Job(job::JobCommands),

    #[cfg(feature = "inference_registry")]
    #[command(subcommand)]
    InferenceRegistry(inference_registry::InferenceRegistryCommands),

    #[cfg(feature = "agent_registry")]
    #[command(subcommand)]
    AgentRegistry(agent_registry::AgentRegistryCommands),

    #[cfg(feature = "directory")]
    #[command(subcommand)]
    Directory(directory::DirectoryCommands),

    #[cfg(feature = "interop")]
    #[command(subcommand)]
    Interop(interop::InteropCommands),

    #[cfg(feature = "x402")]
    #[command(subcommand)]
    X402(x402::X402Commands),

    #[cfg(all(feature = "gmp", feature = "interop"))]
    #[command(subcommand)]
    Gmp(gmp::GmpCommands),

    #[cfg(feature = "gov")]
    #[command(subcommand)]
    Gov(gov::GovCommands),
}

#[allow(clippy::unused_async, unused_variables)]
pub async fn execute(cmd: TxCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        #[cfg(feature = "identity")]
        TxCommands::Identity(sub) => identity::execute(sub, dispatcher).await,
        #[cfg(feature = "bank")]
        TxCommands::Bank(sub) => bank::execute(sub, dispatcher).await,
        #[cfg(feature = "reputation")]
        TxCommands::Reputation(sub) => reputation::execute(sub, dispatcher).await,
        #[cfg(feature = "validation")]
        TxCommands::Validation(sub) => validation::execute(sub, dispatcher).await,
        #[cfg(feature = "memory")]
        TxCommands::Memory(sub) => memory::execute(sub, dispatcher).await,
        #[cfg(feature = "vc")]
        TxCommands::Vc(sub) => vc::execute(sub, dispatcher).await,
        #[cfg(feature = "intent")]
        TxCommands::Intent(sub) => intent::execute(sub, dispatcher).await,
        #[cfg(feature = "marketplace")]
        TxCommands::Marketplace(sub) => marketplace::execute(sub, dispatcher).await,
        #[cfg(feature = "job")]
        TxCommands::Job(sub) => job::execute(sub, dispatcher).await,
        #[cfg(feature = "inference_registry")]
        TxCommands::InferenceRegistry(sub) => inference_registry::execute(sub, dispatcher).await,
        #[cfg(feature = "agent_registry")]
        TxCommands::AgentRegistry(sub) => agent_registry::execute(sub, dispatcher).await,
        #[cfg(feature = "directory")]
        TxCommands::Directory(sub) => directory::execute(sub, dispatcher).await,
        #[cfg(feature = "interop")]
        TxCommands::Interop(sub) => interop::execute(sub, dispatcher).await,
        #[cfg(feature = "x402")]
        TxCommands::X402(sub) => x402::execute(sub, dispatcher).await,
        #[cfg(all(feature = "gmp", feature = "interop"))]
        TxCommands::Gmp(sub) => gmp::execute(sub, dispatcher).await,
        #[cfg(feature = "gov")]
        TxCommands::Gov(sub) => gov::execute(sub, dispatcher).await,
    }
}
