use clap::{Args, Subcommand};

use morpheum_signing_native::signer::Signer;
use morpheum_sdk_native::job::{
    CreateJobBuilder, FundJobBuilder, SubmitDeliverableBuilder, AttestBuilder,
    ClaimRefundBuilder, SetProviderBuilder, CancelJobBuilder,
    Deliverable,
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
    let client_sig = signer.public_key().to_proto_bytes();

    let mut builder = CreateJobBuilder::new()
        .client_agent_hash(&client_hash)
        .evaluator_agent_hash(&args.evaluator_hash)
        .budget_usd(args.budget_usd)
        .expiry_timestamp(args.expiry)
        .client_signature(client_sig);

    if let Some(ref provider) = args.provider_hash {
        builder = builder.provider_agent_hash(provider);
    }
    if let Some(ref spec) = args.spec_hash {
        builder = builder.job_spec_hash(spec);
    }
    if let Some(ref vc) = args.vc_proof {
        builder = builder.vc_proof_hash(vc);
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
    let client_sig = signer.public_key().to_proto_bytes();

    let request = FundJobBuilder::new()
        .job_id(&args.job_id)
        .amount_usd(args.amount_usd)
        .client_signature(client_sig)
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
    let provider_sig = signer.public_key().to_proto_bytes();

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
        .provider_signature(provider_sig)
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
    let evaluator_sig = signer.public_key().to_proto_bytes();

    let request = AttestBuilder::new()
        .job_id(&args.job_id)
        .completed(args.completed)
        .reason_hash(&args.reason_hash)
        .evaluator_signature(evaluator_sig)
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
    let caller_sig = signer.public_key().to_proto_bytes();

    let request = ClaimRefundBuilder::new()
        .job_id(&args.job_id)
        .caller_signature(caller_sig)
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
    let client_sig = signer.public_key().to_proto_bytes();

    let request = SetProviderBuilder::new()
        .job_id(&args.job_id)
        .new_provider_agent_hash(&args.provider_hash)
        .client_signature(client_sig)
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
    let signer_sig = signer.public_key().to_proto_bytes();

    let request = CancelJobBuilder::new()
        .job_id(&args.job_id)
        .signer_signature(signer_sig)
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
