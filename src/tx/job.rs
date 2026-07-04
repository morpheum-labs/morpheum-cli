use clap::{Args, Subcommand};

use morpheum_signing_native::signer::Signer;
use morpheum_sdk_native::job::{
    CreateJobBuilder, FundJobBuilder, SubmitDeliverableBuilder, AttestBuilder,
    ClaimRefundBuilder, SetProviderBuilder, CancelJobBuilder,
    CompensationPolicy, Deliverable,
};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `job` module (ERC-8183 compliant).
///
/// Covers the full job lifecycle: create, fund, submit deliverable,
/// attest, refund, set provider, and cancel.
#[derive(Subcommand)]
pub enum JobCommands {
    /// Create a new job posting
    Create(CreateArgs),

    /// Fund an existing job with escrow
    Fund(FundArgs),

    /// Submit a deliverable for a job (provider)
    SubmitDeliverable(SubmitDeliverableArgs),

    /// Attest to job completion or rejection (evaluator)
    Attest(AttestArgs),

    /// Claim a refund for an expired or rejected job (client)
    ClaimRefund(ClaimRefundArgs),

    /// Set or change the provider for a job (client)
    SetProvider(SetProviderArgs),

    /// Cancel a job (client or provider)
    Cancel(CancelJobArgs),
}

#[derive(Args)]
pub struct CreateArgs {
    /// Evaluator agent hash
    #[arg(long)]
    pub evaluator_hash: String,

    /// Budget in USD
    #[arg(long)]
    pub budget_usd: u64,

    /// Expiry timestamp
    #[arg(long)]
    pub expiry: u64,

    /// Optional provider agent hash (can be set later)
    #[arg(long)]
    pub provider_hash: Option<String>,

    /// Job specification hash (off-chain document)
    #[arg(long)]
    pub spec_hash: Option<String>,

    /// VC proof hash authorising this job
    #[arg(long)]
    pub vc_proof: Option<String>,

    /// ARS v3 evaluation-fee track (USD) escrowed on top of the budget and paid
    /// to the evaluator on both completion and rejection. Zero (the default)
    /// inherits the governance `default_evaluation_fee_usd`.
    #[arg(long, default_value_t = 0)]
    pub evaluation_fee_usd: u64,

    /// ARS v6 self-funded coverage: the claim (USD) reimbursed to the client on
    /// a covered rejection. Non-zero requires governance to have coverage
    /// enabled; it selects the `CoverageReimbursed` policy and escrows a premium
    /// (`coverage * rate`) on top of the budget + fee. Zero (default) disables
    /// coverage.
    #[arg(long, default_value_t = 0)]
    pub coverage_amount_usd: u64,

    /// Key name to sign with (client key)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct FundArgs {
    /// Job ID to fund
    #[arg(long)]
    pub job_id: String,

    /// Amount in USD to deposit into escrow
    #[arg(long)]
    pub amount_usd: u64,

    /// Key name to sign with (client key)
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct SubmitDeliverableArgs {
    /// Job ID
    #[arg(long)]
    pub job_id: String,

    /// Memory root hash of the deliverable data
    #[arg(long)]
    pub memory_root_hash: String,

    /// Optional payload as hex-encoded bytes
    #[arg(long)]
    pub payload: Option<String>,

    /// Key name to sign with (provider key)
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct AttestArgs {
    /// Job ID
    #[arg(long)]
    pub job_id: String,

    /// Whether the job was completed successfully
    #[arg(long)]
    pub completed: bool,

    /// Hash of the reason/report (required if not completed)
    #[arg(long, default_value = "")]
    pub reason_hash: String,

    /// ARS v2 agreement commitment the evaluator judged against. Must equal the
    /// job's stored `job_spec_hash` (leave empty for jobs with no agreement).
    #[arg(long, default_value = "")]
    pub agreement_hash: String,

    /// Key name to sign with (evaluator key)
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct ClaimRefundArgs {
    /// Job ID
    #[arg(long)]
    pub job_id: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct SetProviderArgs {
    /// Job ID
    #[arg(long)]
    pub job_id: String,

    /// New provider agent hash
    #[arg(long)]
    pub provider_hash: String,

    /// Key name to sign with (client key)
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct CancelJobArgs {
    /// Job ID to cancel
    #[arg(long)]
    pub job_id: String,

    /// Key name to sign with (client or provider)
    #[arg(long, default_value = "default")]
    pub from: String,
}

pub async fn execute(cmd: JobCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        JobCommands::Create(args) => create(args, &dispatcher).await,
        JobCommands::Fund(args) => fund(args, &dispatcher).await,
        JobCommands::SubmitDeliverable(args) => submit_deliverable(args, &dispatcher).await,
        JobCommands::Attest(args) => attest(args, &dispatcher).await,
        JobCommands::ClaimRefund(args) => claim_refund(args, &dispatcher).await,
        JobCommands::SetProvider(args) => set_provider(args, &dispatcher).await,
        JobCommands::Cancel(args) => cancel(args, &dispatcher).await,
    }
}

async fn create(args: CreateArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let client_hash = hex::encode(signer.account_id().0);

    let mut builder = CreateJobBuilder::new()
        .client_agent_hash(&client_hash)
        .evaluator_agent_hash(&args.evaluator_hash)
        .budget_usd(args.budget_usd)
        .evaluation_fee_usd(args.evaluation_fee_usd)
        .expiry_timestamp(args.expiry);

    if let Some(ref provider) = args.provider_hash {
        builder = builder.provider_agent_hash(provider);
    }
    if let Some(ref spec) = args.spec_hash {
        builder = builder.job_spec_hash(spec);
    }
    if let Some(ref vc) = args.vc_proof {
        builder = builder.vc_proof_hash(vc);
    }
    // ARS v6: requesting coverage selects the CoverageReimbursed policy (the
    // only policy under which the escrowed premium can ever be claimed).
    if args.coverage_amount_usd > 0 {
        builder = builder
            .coverage_amount_usd(args.coverage_amount_usd)
            .compensation_policy(CompensationPolicy::CoverageReimbursed);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Job created\nBudget: ${}, Evaluator: {}\nTxHash: {}",
        args.budget_usd, args.evaluator_hash, txhash,
    ));

    Ok(())
}

async fn fund(args: FundArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;

    let request = FundJobBuilder::new()
        .job_id(&args.job_id)
        .amount_usd(args.amount_usd)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    ).await?;

    dispatcher.output.success(format!(
        "Job {} funded with ${}\nTxHash: {}",
        args.job_id, args.amount_usd, txhash,
    ));

    Ok(())
}

async fn submit_deliverable(
    args: SubmitDeliverableArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let provider_hash = hex::encode(signer.account_id().0);

    let payload = args
        .payload
        .as_deref()
        .map(hex::decode)
        .transpose()
        .map_err(|e| CliError::invalid_input(format!("invalid hex payload: {e}")))?
        .unwrap_or_default();

    let deliverable = Deliverable {
        job_id: args.job_id.clone(),
        provider_agent_hash: provider_hash,
        memory_root_hash: args.memory_root_hash.clone(),
        payload,
        blob_merkle_root: Vec::new(),
        submitted_at: 0,
    };

    let request = SubmitDeliverableBuilder::new()
        .job_id(&args.job_id)
        .deliverable(deliverable)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    ).await?;

    dispatcher.output.success(format!(
        "Deliverable submitted for job {}\nTxHash: {}",
        args.job_id, txhash,
    ));

    Ok(())
}

async fn attest(args: AttestArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;

    let request = AttestBuilder::new()
        .job_id(&args.job_id)
        .completed(args.completed)
        .reason_hash(&args.reason_hash)
        .agreement_hash(&args.agreement_hash)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    ).await?;

    let status = if args.completed { "completed" } else { "rejected" };
    dispatcher.output.success(format!(
        "Job {} attested as {}\nTxHash: {}",
        args.job_id, status, txhash,
    ));

    Ok(())
}

async fn claim_refund(args: ClaimRefundArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;

    let request = ClaimRefundBuilder::new()
        .job_id(&args.job_id)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    ).await?;

    dispatcher.output.success(format!(
        "Refund claimed for job {}\nTxHash: {}",
        args.job_id, txhash,
    ));

    Ok(())
}

async fn set_provider(args: SetProviderArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;

    let request = SetProviderBuilder::new()
        .job_id(&args.job_id)
        .new_provider_agent_hash(&args.provider_hash)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    ).await?;

    dispatcher.output.success(format!(
        "Provider for job {} set to {}\nTxHash: {}",
        args.job_id, args.provider_hash, txhash,
    ));

    Ok(())
}

async fn cancel(args: CancelJobArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;

    let request = CancelJobBuilder::new()
        .job_id(&args.job_id)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    ).await?;

    dispatcher.output.success(format!(
        "Job {} cancelled\nTxHash: {}",
        args.job_id, txhash,
    ));

    Ok(())
}
