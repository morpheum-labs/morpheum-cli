use clap::{Args, Subcommand};

use morpheum_signing_native::signer::Signer;
use morpheum_sdk_native::interop::{
    BridgeRequestBuilder, ExportIntentBuilder, ExportProofBuilder,
    CrossChainProofPacket,
};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `interop` module.
///
/// Cross-chain interoperability and GMP bridge commands —
/// the connectivity layer described in Pillar 4.
#[derive(Subcommand)]
pub enum InteropCommands {
    /// Submit a cross-chain bridge request
    Bridge(BridgeArgs),

    /// Export an intent to a target chain
    ExportIntent(ExportIntentArgs),

    /// Export a validation proof cross-chain
    ExportProof(ExportProofArgs),
}

#[derive(Args)]
pub struct BridgeArgs {
    /// Source chain identifier (e.g. "morpheum")
    #[arg(long)]
    pub source_chain: String,

    /// Target chain identifier (e.g. "ethereum", "base", "arbitrum")
    #[arg(long)]
    pub target_chain: String,

    /// Intent ID to bridge (bridges the intent payload)
    #[arg(long)]
    pub intent_id: Option<String>,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct ExportIntentArgs {
    /// Intent ID to export
    #[arg(long)]
    pub intent_id: String,

    /// Source agent hash
    #[arg(long)]
    pub source_agent_hash: String,

    /// Target chain
    #[arg(long)]
    pub target_chain: String,

    /// Intent data as hex-encoded bytes
    #[arg(long)]
    pub intent_data: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct ExportProofArgs {
    /// Source chain
    #[arg(long)]
    pub source_chain: String,

    /// Target chain
    #[arg(long)]
    pub target_chain: String,

    /// Agent hash whose proof is being exported
    #[arg(long)]
    pub agent_hash: String,

    /// Merkle proof string
    #[arg(long)]
    pub merkle_proof: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(cmd: InteropCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        InteropCommands::Bridge(args) => bridge(args, &dispatcher).await,
        InteropCommands::ExportIntent(args) => export_intent(args, &dispatcher).await,
        InteropCommands::ExportProof(args) => export_proof(args, &dispatcher).await,
    }
}

async fn bridge(args: BridgeArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let signer_bytes = signer.public_key().to_proto_bytes();

    let request = BridgeRequestBuilder::new()
        .source_chain(&args.source_chain)
        .target_chain(&args.target_chain)
        .signer(signer_bytes)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Bridge request submitted\n{} -> {}\nTxHash: {}",
        args.source_chain, args.target_chain, txhash,
    ));

    Ok(())
}

async fn export_intent(
    args: ExportIntentArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let sig = signer.public_key().to_proto_bytes();

    let intent_data = hex::decode(&args.intent_data)
        .map_err(|e| CliError::invalid_input(format!("invalid hex for intent_data: {e}")))?;

    let request = ExportIntentBuilder::new()
        .intent_id(&args.intent_id)
        .source_agent_hash(&args.source_agent_hash)
        .target_chain(&args.target_chain)
        .intent_data(intent_data)
        .signature(sig.clone())
        .signer(sig)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Intent {} exported to {}\nTxHash: {}",
        args.intent_id, args.target_chain, txhash,
    ));

    Ok(())
}

async fn export_proof(args: ExportProofArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let signer_bytes = signer.public_key().to_proto_bytes();

    let proof_packet = CrossChainProofPacket {
        source_chain: args.source_chain.clone(),
        target_chain: args.target_chain.clone(),
        agent_hash: args.agent_hash.clone(),
        proof: None,
        exported_at: 0,
        merkle_proof: args.merkle_proof.clone(),
    };

    let request = ExportProofBuilder::new()
        .proof_packet(proof_packet)
        .signer(signer_bytes)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Proof exported for agent {}\n{} -> {}\nTxHash: {}",
        args.agent_hash, args.source_chain, args.target_chain, txhash,
    ));

    Ok(())
}
