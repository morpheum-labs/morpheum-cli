use clap::{Args, Subcommand};

use morpheum_signing_native::signer::Signer;
use morpheum_sdk_native::agentreg::TriggerProtocolSyncBuilder;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `agentreg` module.
///
/// The central interoperability sync engine that keeps the unified
/// agent registry in sync with ERC-8004, A2A, MCP, DID, and x402
/// views (Pillar 2 adapters + Pillar 3 trust layer).
#[derive(Subcommand)]
pub enum AgentRegistryCommands {
    /// Trigger a protocol sync for an agent (re-exports to ERC-8004, A2A, MCP, DID)
    TriggerSync(TriggerSyncArgs),
}

#[derive(Args)]
pub struct TriggerSyncArgs {
    /// Agent hash (hex) to sync
    #[arg(long)]
    pub agent_hash: String,

    /// Comma-separated protocols to sync (erc8004, a2a, mcp, did, x402)
    #[arg(long, value_delimiter = ',')]
    pub protocols: Vec<String>,

    /// Key name to sign with (authority key)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(
    cmd: AgentRegistryCommands,
    dispatcher: Dispatcher,
) -> Result<(), CliError> {
    match cmd {
        AgentRegistryCommands::TriggerSync(args) => trigger_sync(args, &dispatcher).await,
    }
}

async fn trigger_sync(args: TriggerSyncArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let authority = hex::encode(signer.account_id().0);

    let agent_hash_bytes = hex::decode(&args.agent_hash)
        .map_err(|e| CliError::invalid_input(format!("invalid hex for agent_hash: {e}")))?;

    let request = TriggerProtocolSyncBuilder::new()
        .authority(&authority)
        .agent_hash(agent_hash_bytes)
        .protocols(args.protocols.clone())
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Protocol sync triggered for agent {}\nProtocols: {}\nTxHash: {}",
        args.agent_hash,
        args.protocols.join(", "),
        txhash,
    ));

    Ok(())
}
