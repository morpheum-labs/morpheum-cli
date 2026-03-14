use clap::{Args, Subcommand};

use morpheum_sdk_native::MorpheumSdk;
use morpheum_sdk_native::validation::types::ProofType;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

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
    /// Proof type (optimistic, zkml, tee-attestation, cross-validated)
    #[arg(long, value_parser = parse_proof_type)]
    pub proof_type: ProofType,

    #[arg(long, default_value = "20")]
    pub limit: u32,

    #[arg(long, default_value = "0")]
    pub offset: u32,
}

pub async fn execute(
    cmd: ValidationQueryCommands,
    dispatcher: Dispatcher,
) -> Result<(), CliError> {
    let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);

    match cmd {
        ValidationQueryCommands::Proof(args) => {
            query_proof(args, &sdk, &dispatcher.output).await
        }
        ValidationQueryCommands::ProofsByAgent(args) => {
            query_proofs_by_agent(args, &sdk, &dispatcher.output).await
        }
        ValidationQueryCommands::ProofsByType(args) => {
            query_proofs_by_type(args, &sdk, &dispatcher.output).await
        }
        ValidationQueryCommands::Params => {
            query_params(&sdk, &dispatcher.output).await
        }
    }
}

async fn query_proof(
    args: ProofArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.validation()
        .query_and_print_optional(
            output,
            &format!("No proof found with ID {}", args.proof_id),
            |c| async move { c.query_proof(&args.proof_id).await },
        )
        .await
}

async fn query_proofs_by_agent(
    args: ProofsByAgentArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.validation()
        .query_and_print_paginated(output, |c| async move {
            c.query_proofs_by_agent(&args.agent_hash, args.limit, args.offset)
                .await
        })
        .await
}

async fn query_proofs_by_type(
    args: ProofsByTypeArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.validation()
        .query_and_print_paginated(output, |c| async move {
            c.query_proofs_by_type(args.proof_type, args.limit, args.offset)
                .await
        })
        .await
}

async fn query_params(sdk: &MorpheumSdk, output: &Output) -> Result<(), CliError> {
    sdk.validation()
        .query_and_print_optional(
            output,
            "No validation parameters configured",
            |c| async move { c.query_params().await },
        )
        .await
}

fn parse_proof_type(s: &str) -> Result<ProofType, String> {
    match s.to_lowercase().as_str() {
        "optimistic" => Ok(ProofType::Optimistic),
        "zkml" => Ok(ProofType::ZkMl),
        "tee-attestation" | "tee" => Ok(ProofType::TeeAttestation),
        "cross-validated" | "crossvalidated" => Ok(ProofType::CrossValidated),
        other => Err(format!(
            "unknown proof type '{other}'; expected: optimistic, zkml, tee-attestation, cross-validated"
        )),
    }
}
