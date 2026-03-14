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
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::vc::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_vc(tonic::Request::new(morpheum_proto::vc::v1::QueryVcRequest {
            vc_id: args.vc_id,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryVc failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_status(args: StatusArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::vc::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_vc_status(tonic::Request::new(
            morpheum_proto::vc::v1::QueryVcStatusRequest {
                vc_id: args.vc_id,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryVcStatus failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_by_issuer(args: ByIssuerArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::vc::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_vcs_by_issuer(tonic::Request::new(
            morpheum_proto::vc::v1::QueryVcsByIssuerRequest {
                issuer_agent_hash: args.issuer,
                limit: args.limit,
                offset: args.offset,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryVcsByIssuer failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_by_subject(args: BySubjectArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::vc::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_vcs_by_subject(tonic::Request::new(
            morpheum_proto::vc::v1::QueryVcsBySubjectRequest {
                subject_agent_hash: args.subject,
                limit: args.limit,
                offset: args.offset,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryVcsBySubject failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_revocation_bitmap(
    args: RevocationBitmapArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let issuer = args.issuer.clone();
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::vc::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_revocation_bitmap(tonic::Request::new(
            morpheum_proto::vc::v1::QueryRevocationBitmapRequest {
                issuer_agent_hash: args.issuer,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryRevocationBitmap failed: {e}")))?
        .into_inner();
    dispatcher.output.info(format!(
        "Revocation bitmap for issuer {} ({} bytes)",
        issuer,
        response.bitmap.len()
    ));
    dispatcher.output.success(hex::encode(&response.bitmap));
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::vc::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_params(tonic::Request::new(
            morpheum_proto::vc::v1::QueryParamsRequest::default(),
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryParams failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}
