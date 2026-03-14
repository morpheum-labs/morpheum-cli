use clap::{Args, Subcommand};

use morpheum_sdk_native::identity::{
    AgentMetadataCardInput, Capability, RegisterAgentBuilder,
};
use morpheum_signing_native::signer::Signer;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::utils::sign_and_broadcast;

/// Transaction commands for the `identity` module.
#[derive(Subcommand)]
pub enum IdentityCommands {
    /// Register a new agent identity on-chain
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

    /// Comma-separated capabilities (trade, evaluate, delegate, analyze)
    #[arg(long, value_delimiter = ',')]
    pub capabilities: Vec<String>,

    /// Key name to sign with (from `morpheum keys list`)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Mark as self-owned (autonomous) agent
    #[arg(long)]
    pub self_owned: bool,

    /// Optional memo for the transaction
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(cmd: IdentityCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        IdentityCommands::Register(args) => register(args, dispatcher).await,
    }
}

async fn register(args: RegisterArgs, dispatcher: Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let owner_hash = hex::encode(signer.account_id().0);

    let caps = capabilities_to_bitflags(&args.capabilities);
    let metadata = AgentMetadataCardInput {
        display_name: args.display_name,
        description: args.description.unwrap_or_default(),
        tags: String::new(),
        version: "1.0.0".into(),
        capabilities: caps,
    };

    let request = RegisterAgentBuilder::new()
        .did(&args.did)
        .owner_agent_hash(&owner_hash)
        .metadata(metadata)
        .owner_signature(vec![0u8; 64])
        .capabilities(caps)
        .self_owned(args.self_owned)
        .build()
        .map_err(CliError::Sdk)?;

    let txhash = sign_and_broadcast(
        signer,
        &dispatcher,
        request.to_any(),
        args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "Agent registered!\nDID: {}\nTxHash: {txhash}",
        args.did
    ));

    Ok(())
}

fn capabilities_to_bitflags(caps: &[String]) -> u64 {
    let mut flags = 0u64;
    for cap in caps {
        flags |= match cap.to_lowercase().as_str() {
            "trade" => Capability::TRADE,
            "evaluate" => Capability::EVALUATE,
            "manage" => Capability::MANAGE,
            "memory" => Capability::MEMORY,
            _ => 0,
        };
    }
    flags
}
