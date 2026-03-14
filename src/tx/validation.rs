use clap::{Args, Subcommand};

use morpheum_signing_native::signer::Signer;
use morpheum_sdk_native::validation::{
    SubmitProofBuilder, RevokeProofBuilder, ProofType,
};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `validation` module.
///
/// Handles proof submission, revocation, and parameter governance —
/// the attestation layer of Pillar 3 (ERC-8004 Validation Registry).
#[derive(Subcommand)]
pub enum ValidationCommands {
    /// Submit a new validation proof for an agent
    SubmitProof(SubmitProofArgs),

    /// Revoke an existing validation proof
    RevokeProof(RevokeProofArgs),
}

#[derive(Args)]
pub struct SubmitProofArgs {
    /// Agent hash of the agent being validated
    #[arg(long)]
    pub agent_hash: String,

    /// Proof type (backtest, inference, human-eval, tee, external-validator, marketplace-eval, custom)
    #[arg(long, value_parser = parse_proof_type)]
    pub proof_type: ProofType,

    /// Score contribution (0–10000)
    #[arg(long)]
    pub score: u32,

    /// Hash of the full proof payload stored in Persistent Memory
    #[arg(long)]
    pub data_hash: String,

    /// Key name to sign with (verifier key)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct RevokeProofArgs {
    /// ID of the proof to revoke
    #[arg(long)]
    pub proof_id: String,

    /// Verifier agent hash performing the revocation
    #[arg(long)]
    pub verifier_hash: String,

    /// Reason for revocation
    #[arg(long)]
    pub reason: String,

    /// Key name to sign with (verifier or governance key)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(cmd: ValidationCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        ValidationCommands::SubmitProof(args) => submit_proof(args, &dispatcher).await,
        ValidationCommands::RevokeProof(args) => revoke_proof(args, &dispatcher).await,
    }
}

async fn submit_proof(args: SubmitProofArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let verifier_sig = signer.public_key().to_proto_bytes();

    let request = SubmitProofBuilder::new()
        .agent_hash(&args.agent_hash)
        .proof_type(args.proof_type)
        .score_contribution(args.score)
        .data_hash(&args.data_hash)
        .verifier_signature(verifier_sig)
        .build()
        .map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "Proof submitted for agent {}\nType: {}, Score: {}\nTxHash: {}",
        args.agent_hash, args.proof_type, args.score, txhash,
    ));

    Ok(())
}

async fn revoke_proof(args: RevokeProofArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let verifier_sig = signer.public_key().to_proto_bytes();

    let request = RevokeProofBuilder::new()
        .proof_id(&args.proof_id)
        .verifier_agent_hash(&args.verifier_hash)
        .verifier_signature(verifier_sig)
        .reason(&args.reason)
        .build()
        .map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "Proof {} revoked\nReason: {}\nTxHash: {}",
        args.proof_id, args.reason, txhash,
    ));

    Ok(())
}

fn parse_proof_type(s: &str) -> Result<ProofType, String> {
    match s.to_lowercase().as_str() {
        "backtest" => Ok(ProofType::Backtest),
        "inference" => Ok(ProofType::Inference),
        "human-eval" | "humaneval" => Ok(ProofType::HumanEval),
        "tee" | "tee-attestation" => Ok(ProofType::TeeAttestation),
        "external-validator" | "external" => Ok(ProofType::ExternalValidator),
        "marketplace-eval" | "marketplace" => Ok(ProofType::MarketplaceEval),
        "custom" => Ok(ProofType::Custom),
        other => Err(format!(
            "unknown proof type '{other}'; expected: backtest, inference, \
             human-eval, tee, external-validator, marketplace-eval, custom"
        )),
    }
}
