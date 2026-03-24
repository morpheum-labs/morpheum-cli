use clap::{Args, Subcommand};

use morpheum_proto::validation::v1::ProofType as ProtoProofType;
use morpheum_sdk_native::validation::{ProofType, ValidationClient};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the `validation` module.
///
/// Read-only access to validation proofs (TEE, zkML, optimistic, etc.),
/// filtered views, and module parameters.
#[derive(Subcommand)]
pub enum ValidationQueryCommands {
    /// Get a specific validation proof by ID
    Proof(ProofArgs),

    /// List proofs submitted by a specific agent (paginated)
    ProofsByAgent(ProofsByAgentArgs),

    /// List proofs filtered by type (paginated)
    ProofsByType(ProofsByTypeArgs),

    /// Get the current validation module parameters
    Params,
}

#[derive(Args)]
pub struct ProofArgs {
    /// Proof ID
    #[arg(required = true)]
    pub proof_id: String,
}

#[derive(Args)]
pub struct ProofsByAgentArgs {
    /// Agent hash
    #[arg(required = true)]
    pub agent_hash: String,

    #[arg(long, default_value = "20")]
    pub limit: u32,

    #[arg(long, default_value = "0")]
    pub offset: u32,
}

#[derive(Args)]
pub struct ProofsByTypeArgs {
    /// Proof type (backtest, inference, human-eval, tee, external-validator, marketplace-eval, custom)
    #[arg(long, value_parser = parse_proof_type)]
    pub proof_type: ProtoProofType,

    #[arg(long, default_value = "20")]
    pub limit: u32,

    #[arg(long, default_value = "0")]
    pub offset: u32,
}

pub async fn execute(
    cmd: ValidationQueryCommands,
    dispatcher: Dispatcher,
) -> Result<(), CliError> {
    match cmd {
        ValidationQueryCommands::Proof(args) => query_proof(args, &dispatcher).await,
        ValidationQueryCommands::ProofsByAgent(args) => {
            query_proofs_by_agent(args, &dispatcher).await
        }
        ValidationQueryCommands::ProofsByType(args) => {
            query_proofs_by_type(args, &dispatcher).await
        }
        ValidationQueryCommands::Params => query_params(&dispatcher).await,
    }
}

async fn query_proof(args: ProofArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = ValidationClient::new(dispatcher.sdk_config(), Box::new(transport));
    let result = client.query_proof(args.proof_id).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_proofs_by_agent(
    args: ProofsByAgentArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = ValidationClient::new(dispatcher.sdk_config(), Box::new(transport));
    let result = client
        .query_proofs_by_agent(args.agent_hash, args.limit, args.offset)
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_proofs_by_type(
    args: ProofsByTypeArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = ValidationClient::new(dispatcher.sdk_config(), Box::new(transport));
    let result = client
        .query_proofs_by_type(
            ProofType::from_proto(args.proof_type.into()),
            args.limit,
            args.offset,
        )
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = ValidationClient::new(dispatcher.sdk_config(), Box::new(transport));
    let result = client.query_params().await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

fn parse_proof_type(s: &str) -> Result<ProtoProofType, String> {
    match s.to_lowercase().as_str() {
        "backtest" => Ok(ProtoProofType::Backtest),
        "inference" => Ok(ProtoProofType::Inference),
        "human-eval" | "humaneval" => Ok(ProtoProofType::HumanEval),
        "tee" | "tee-attestation" => Ok(ProtoProofType::TeeAttestation),
        "external-validator" | "external" => Ok(ProtoProofType::ExternalValidator),
        "marketplace-eval" | "marketplace" => Ok(ProtoProofType::MarketplaceEval),
        "custom" => Ok(ProtoProofType::Custom),
        other => Err(format!(
            "unknown proof type '{other}'; expected: backtest, inference, \
             human-eval, tee, external-validator, marketplace-eval, custom"
        )),
    }
}
