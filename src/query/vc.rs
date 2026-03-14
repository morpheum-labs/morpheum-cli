use clap::{Args, Subcommand};

use morpheum_sdk_native::MorpheumSdk;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

/// Query commands for the `vc` (Verifiable Credentials) module.
///
/// Read-only access to VCs, credential status, issuer/subject views,
/// revocation bitmaps, and module parameters (W3C VC on-chain).
#[derive(Subcommand)]
pub enum VcQueryCommands {
    /// Get a Verifiable Credential by ID
    Get(GetArgs),

    /// Get the current status of a VC (valid, revoked, expired)
    Status(StatusArgs),

    /// List VCs issued by a specific issuer (paginated)
    ByIssuer(ByIssuerArgs),

    /// List VCs issued to a specific subject (paginated)
    BySubject(BySubjectArgs),

    /// Get the revocation bitmap for an issuer
    RevocationBitmap(RevocationBitmapArgs),

    /// Get the current VC module parameters
    Params,
}

#[derive(Args)]
pub struct GetArgs {
    /// Verifiable Credential ID
    #[arg(required = true)]
    pub vc_id: String,
}

#[derive(Args)]
pub struct StatusArgs {
    /// Verifiable Credential ID
    #[arg(required = true)]
    pub vc_id: String,
}

#[derive(Args)]
pub struct ByIssuerArgs {
    /// Issuer account address (bech32)
    #[arg(required = true)]
    pub issuer: String,

    #[arg(long, default_value = "20")]
    pub limit: u32,

    #[arg(long, default_value = "0")]
    pub offset: u32,
}

#[derive(Args)]
pub struct BySubjectArgs {
    /// Subject account address (bech32)
    #[arg(required = true)]
    pub subject: String,

    #[arg(long, default_value = "20")]
    pub limit: u32,

    #[arg(long, default_value = "0")]
    pub offset: u32,
}

#[derive(Args)]
pub struct RevocationBitmapArgs {
    /// Issuer account address (bech32)
    #[arg(required = true)]
    pub issuer: String,
}

pub async fn execute(cmd: VcQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);

    match cmd {
        VcQueryCommands::Get(args) => query_vc(args, &sdk, &dispatcher.output).await,
        VcQueryCommands::Status(args) => query_status(args, &sdk, &dispatcher.output).await,
        VcQueryCommands::ByIssuer(args) => query_by_issuer(args, &sdk, &dispatcher.output).await,
        VcQueryCommands::BySubject(args) => {
            query_by_subject(args, &sdk, &dispatcher.output).await
        }
        VcQueryCommands::RevocationBitmap(args) => {
            query_revocation_bitmap(args, &sdk, &dispatcher.output).await
        }
        VcQueryCommands::Params => query_params(&sdk, &dispatcher.output).await,
    }
}

/// `query_vc` returns `Result<Vc, SdkError>` (non-Optional — 404 becomes SdkError),
/// so `QueryClientExt::query_and_print_item` applies directly.
async fn query_vc(
    args: GetArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.vc()
        .query_and_print_item(output, |c| async move {
            c.query_vc(&args.vc_id).await
        })
        .await
}

/// `query_vc_status` returns `Result<VcStatus, SdkError>` (non-Optional).
async fn query_status(
    args: StatusArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.vc()
        .query_and_print_item(output, |c| async move {
            c.query_vc_status(&args.vc_id).await
        })
        .await
}

async fn query_by_issuer(
    args: ByIssuerArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.vc()
        .query_and_print_list(output, |c| async move {
            c.query_vcs_by_issuer(&args.issuer, args.limit, args.offset)
                .await
        })
        .await
}

async fn query_by_subject(
    args: BySubjectArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.vc()
        .query_and_print_list(output, |c| async move {
            c.query_vcs_by_subject(&args.subject, args.limit, args.offset)
                .await
        })
        .await
}

async fn query_revocation_bitmap(
    args: RevocationBitmapArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    let bitmap = sdk.vc().query_revocation_bitmap(&args.issuer).await?;

    output.info(format!(
        "Revocation bitmap for issuer {} ({} bytes)",
        args.issuer,
        bitmap.len()
    ));
    output.success(hex::encode(&bitmap));

    Ok(())
}

/// `query_params` returns `Result<Params, SdkError>` (non-Optional).
async fn query_params(sdk: &MorpheumSdk, output: &Output) -> Result<(), CliError> {
    sdk.vc()
        .query_and_print_item(output, |c| async move { c.query_params().await })
        .await
}
