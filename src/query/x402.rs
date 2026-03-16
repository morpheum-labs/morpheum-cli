use clap::{Args, Subcommand};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the x402 autonomous payment module.
///
/// Read-only access to payment receipts, spending policies,
/// agent capabilities, and module parameters.
#[derive(Subcommand)]
pub enum X402QueryCommands {
    /// Get a specific receipt by ID
    Receipt(ReceiptArgs),

    /// List receipts for an agent (paginated)
    Receipts(ReceiptsByAgentArgs),

    /// Get an agent's spending policy
    Policy(PolicyArgs),

    /// Get an agent's x402 payment capabilities
    Capabilities(CapabilitiesArgs),

    /// Get the current x402 module parameters
    Params,
}

#[derive(Args)]
pub struct ReceiptArgs {
    /// Receipt ID
    #[arg(required = true)]
    pub receipt_id: String,
}

#[derive(Args)]
pub struct ReceiptsByAgentArgs {
    /// Agent ID to query receipts for
    #[arg(required = true)]
    pub agent_id: String,

    /// Maximum number of results
    #[arg(long, default_value = "20")]
    pub limit: u32,

    /// Pagination key from a previous query
    #[arg(long)]
    pub pagination_key: Option<String>,
}

#[derive(Args)]
pub struct PolicyArgs {
    /// Agent ID
    #[arg(required = true)]
    pub agent_id: String,

    /// Policy ID
    #[arg(long, default_value = "")]
    pub policy_id: String,
}

#[derive(Args)]
pub struct CapabilitiesArgs {
    /// Agent ID
    #[arg(required = true)]
    pub agent_id: String,
}

pub async fn execute(
    cmd: X402QueryCommands,
    dispatcher: Dispatcher,
) -> Result<(), CliError> {
    match cmd {
        X402QueryCommands::Receipt(args) => query_receipt(args, &dispatcher).await,
        X402QueryCommands::Receipts(args) => query_receipts(args, &dispatcher).await,
        X402QueryCommands::Policy(args) => query_policy(args, &dispatcher).await,
        X402QueryCommands::Capabilities(args) => query_capabilities(args, &dispatcher).await,
        X402QueryCommands::Params => query_params(&dispatcher).await,
    }
}

async fn query_receipt(args: ReceiptArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::x402::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_receipt(tonic::Request::new(
            morpheum_proto::x402::v1::QueryReceiptRequest {
                receipt_id: args.receipt_id,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryReceipt failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_receipts(
    args: ReceiptsByAgentArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::x402::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_receipts_by_agent(tonic::Request::new(
            morpheum_proto::x402::v1::QueryReceiptsByAgentRequest {
                agent_id: args.agent_id,
                limit: args.limit,
                pagination_key: args.pagination_key.unwrap_or_default(),
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryReceiptsByAgent failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_policy(args: PolicyArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::x402::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_policy(tonic::Request::new(
            morpheum_proto::x402::v1::QueryPolicyRequest {
                agent_id: args.agent_id,
                policy_id: args.policy_id,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryPolicy failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_capabilities(
    args: CapabilitiesArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::x402::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_capabilities(tonic::Request::new(
            morpheum_proto::x402::v1::QueryCapabilitiesRequest {
                agent_id: args.agent_id,
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryCapabilities failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::x402::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_params(tonic::Request::new(
            morpheum_proto::x402::v1::QueryParamsRequest::default(),
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryParams failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}
