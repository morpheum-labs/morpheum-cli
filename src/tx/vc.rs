use clap::{Args, Subcommand};

use morpheum_sdk_native::vc::{VcIssueBuilder, VcRevokeBuilder, VcSelfRevokeBuilder, VcClaims};
use morpheum_sdk_native::AccountId;
use morpheum_signing_native::signer::Signer;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `vc` (Verifiable Credentials) module.
///
/// Covers issuance, revocation, and self-revocation of on-chain
/// Verifiable Credentials used for agent delegation and permissions.
#[derive(Subcommand)]
pub enum VcCommands {
    /// Issue a new Verifiable Credential to an agent
    Issue(IssueArgs),

    /// Revoke a VC (issuer-initiated)
    Revoke(RevokeArgs),

    /// Self-revoke a VC (subject agent surrenders the credential)
    SelfRevoke(SelfRevokeArgs),
}

#[derive(Args)]
pub struct IssueArgs {
    /// Subject agent hash (hex, 32 bytes) receiving the credential
    #[arg(long)]
    pub subject: String,

    /// Maximum daily USD spend limit for the credential
    #[arg(long, default_value = "0")]
    pub max_daily_usd: u64,

    /// Allowed trading pairs bitflags
    #[arg(long, default_value = "0")]
    pub allowed_pairs: u64,

    /// Maximum slippage in basis points
    #[arg(long, default_value = "0")]
    pub max_slippage_bps: u32,

    /// Maximum position size in USD
    #[arg(long, default_value = "0")]
    pub max_position_usd: u64,

    /// Optional custom constraints as JSON string
    #[arg(long)]
    pub custom_constraints: Option<String>,

    /// Custom expiry timestamp (0 = use module default)
    #[arg(long, default_value = "0")]
    pub expiry: u64,

    /// Key name to sign with (issuer key)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct RevokeArgs {
    /// VC ID to revoke
    #[arg(long)]
    pub vc_id: String,

    /// Reason for revocation
    #[arg(long)]
    pub reason: Option<String>,

    /// Key name to sign with (must be the original issuer)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct SelfRevokeArgs {
    /// VC ID to self-revoke
    #[arg(long)]
    pub vc_id: String,

    /// Reason for self-revocation
    #[arg(long)]
    pub reason: Option<String>,

    /// Key name to sign with (must be the VC subject)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(cmd: VcCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        VcCommands::Issue(args) => issue(args, &dispatcher).await,
        VcCommands::Revoke(args) => revoke(args, &dispatcher).await,
        VcCommands::SelfRevoke(args) => self_revoke(args, &dispatcher).await,
    }
}

async fn issue(args: IssueArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let issuer_id = signer.account_id();
    let issuer_sig = signer.public_key().to_proto_bytes();

    let subject_id = parse_account_id(&args.subject)?;

    let claims = VcClaims {
        max_daily_usd: args.max_daily_usd,
        allowed_pairs_bitflags: args.allowed_pairs,
        max_slippage_bps: args.max_slippage_bps,
        max_position_usd: args.max_position_usd,
        custom_constraints: args.custom_constraints.clone(),
    };

    let mut builder = VcIssueBuilder::new()
        .issuer(issuer_id)
        .subject(subject_id)
        .claims(claims)
        .issuer_signature(issuer_sig);

    if args.expiry > 0 {
        builder = builder.expiry(args.expiry);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "VC issued to {}\nMax daily: ${}, Max position: ${}\nTxHash: {}",
        args.subject, args.max_daily_usd, args.max_position_usd, txhash,
    ));

    Ok(())
}

async fn revoke(args: RevokeArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let issuer_id = signer.account_id();
    let issuer_sig = signer.public_key().to_proto_bytes();

    let mut builder = VcRevokeBuilder::new()
        .vc_id(&args.vc_id)
        .issuer(issuer_id)
        .issuer_signature(issuer_sig);

    if let Some(ref reason) = args.reason {
        builder = builder.reason(reason);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "VC {} revoked\nTxHash: {}", args.vc_id, txhash,
    ));

    Ok(())
}

async fn self_revoke(args: SelfRevokeArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let agent_sig = signer.public_key().to_proto_bytes();

    let mut builder = VcSelfRevokeBuilder::new()
        .vc_id(&args.vc_id)
        .agent_signature(agent_sig);

    if let Some(ref reason) = args.reason {
        builder = builder.reason(reason);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "VC {} self-revoked\nTxHash: {}", args.vc_id, txhash,
    ));

    Ok(())
}

fn parse_account_id(hex_str: &str) -> Result<AccountId, CliError> {
    let bytes = hex::decode(hex_str).map_err(|e| {
        CliError::invalid_input(format!("invalid hex for account ID: {e}"))
    })?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| {
        CliError::invalid_input("account ID must be exactly 32 bytes (64 hex chars)")
    })?;
    Ok(AccountId::new(arr))
}
