use clap::{Args, Subcommand};
use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::keyring::KeyringManager;
use crate::output::Output;
use crate::utils::TxBuilderExt;
use morpheum_sdk_native::identity::MsgRegisterAgent;
use morpheum_signing_native::TradingKeyClaim;

/// Transaction commands for the `identity` module (ERC-8004 Identity Registry).
///
/// This module implements the canonical agent onboarding workflow described in
/// `agent_identity_register.md` and Pillar 3. It supports one-click superset
/// registration (identity + memory root + reputation seed + ERC-8004 export).
#[derive(Subcommand)]
pub enum IdentityCommands {
    /// Register a new agent (creates identity, memory root, reputation seed,
    /// and triggers all compatibility exports)
    Register(RegisterArgs),
}

#[derive(Args)]
pub struct RegisterArgs {
    /// DID for the new agent (e.g. did:agent:alpha-trader-v3)
    #[arg(long)]
    pub did: String,

    /// Display name for the agent
    #[arg(long)]
    pub display_name: String,

    /// Short description of the agent
    #[arg(long)]
    pub description: Option<String>,

    /// Comma-separated capabilities (trade,evaluate,delegate,etc.)
    #[arg(long, value_delimiter = ',')]
    pub capabilities: Vec<String>,

    /// Key name to sign with (from `morpheum keys list`)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Use agent delegation mode (automatically attaches TradingKeyClaim)
    #[arg(long)]
    pub agent: bool,

    /// Optional memo for the transaction
    #[arg(long)]
    pub memo: Option<String>,
}

/// Execute identity transaction commands.
pub async fn execute(cmd: IdentityCommands, mut dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        IdentityCommands::Register(args) => register(args, &mut dispatcher).await,
    }
}

async fn register(args: RegisterArgs, dispatcher: &mut Dispatcher) -> Result<(), CliError> {
    // Resolve signer (supports both native wallets and agent delegation)
    let signer = if args.agent {
        dispatcher.keyring.get_agent_signer(&args.from)?
    } else {
        dispatcher.keyring.get_native_signer(&args.from)?
    };

    // Build registration message (matches exact proto from agent_identity_register.md)
    let msg = MsgRegisterAgent {
        did: args.did,
        display_name: args.display_name,
        description: args.description.unwrap_or_default(),
        capabilities: capabilities_to_bitflags(&args.capabilities),
    };

    // Build signed transaction using the canonical extension trait
    let mut builder = TxBuilder::new(signer)
        .with_chain_id(&dispatcher.config.chain_id)
        .with_memo(args.memo.unwrap_or_else(|| "Agent registration via Morpheum CLI".into()))
        .add_proto_msg(msg);

    // Attach TradingKeyClaim for agent delegation mode
    if args.agent {
        // In production the claim is pre-stored in keyring; here we use a placeholder
        // that gets properly serialized by the signing layer
        builder = builder.with_trading_key_claim(TradingKeyClaim::default());
    }

    let signed_tx = builder.sign().await.map_err(CliError::Signing)?;

    // Broadcast via SDK (standard pattern used across all tx modules)
    let client = morpheum_sdk_native::MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);
    let tx_hash = client.broadcast(signed_tx.raw_bytes()).await
        .map_err(CliError::Sdk)?;

    // Rich success output
    dispatcher.output.success(format!(
        "Agent registered successfully!\nDID: {}\nTxHash: {}",
        args.did, tx_hash
    ));

    Ok(())
}

// Helper to convert capability strings to bitflags (clean, extensible, zero-cost)
fn capabilities_to_bitflags(caps: &[String]) -> u64 {
    let mut flags = 0u64;
    for cap in caps {
        match cap.to_lowercase().as_str() {
            "trade" => flags |= 1 << 0,
            "evaluate" => flags |= 1 << 1,
            "delegate" => flags |= 1 << 2,
            "analyze" => flags |= 1 << 3,
            _ => {}
        }
    }
    flags
}