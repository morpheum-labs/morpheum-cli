use clap::{Args, Subcommand};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

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
    match cmd {
        VcQueryCommands::Get(args) => query_vc(args, &dispatcher).await,
        VcQueryCommands::Status(args) => query_status(args, &dispatcher).await,
        VcQueryCommands::ByIssuer(args) => query_by_issuer(args, &dispatcher).await,
        VcQueryCommands::BySubject(args) => query_by_subject(args, &dispatcher).await,
        VcQueryCommands::RevocationBitmap(args) => {
            query_revocation_bitmap(args, &dispatcher).await
        }
        VcQueryCommands::Params => query_params(&dispatcher).await,
    }
}

async fn query_vc(args: GetArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::vc::VcClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_vc(args.vc_id).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_status(args: StatusArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::vc::VcClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_vc_status(args.vc_id).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_by_issuer(args: ByIssuerArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::vc::VcClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client
        .query_vcs_by_issuer(args.issuer, args.limit, args.offset)
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_by_subject(args: BySubjectArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::vc::VcClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client
        .query_vcs_by_subject(args.subject, args.limit, args.offset)
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_revocation_bitmap(
    args: RevocationBitmapArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::vc::VcClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_revocation_bitmap(args.issuer).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::vc::VcClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_params().await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}
