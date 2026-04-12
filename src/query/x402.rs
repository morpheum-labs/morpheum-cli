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

    /// List pending Upto pre-authorizations for an agent
    PendingUpto(PendingUptoArgs),
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

#[derive(Args)]
pub struct PendingUptoArgs {
    /// Agent ID to query pending pre-authorizations for
    #[arg(required = true)]
    pub agent_id: String,

    /// Filter by seller address
    #[arg(long)]
    pub seller_address: Option<String>,

    /// Query a specific pre-authorization by ID
    #[arg(long)]
    pub pre_auth_id: Option<String>,
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
        X402QueryCommands::PendingUpto(args) => query_pending_upto(args, &dispatcher).await,
    }
}

async fn query_receipt(args: ReceiptArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::x402::X402Client::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_receipt(args.receipt_id).await?;
    let json =
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_receipts(
    args: ReceiptsByAgentArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::x402::X402Client::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client
        .query_receipts_by_agent(args.agent_id, args.limit, args.pagination_key)
        .await?;
    let json =
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_policy(args: PolicyArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::x402::X402Client::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_policy(args.agent_id, args.policy_id).await?;
    let json =
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_capabilities(
    args: CapabilitiesArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::x402::X402Client::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_capabilities(args.agent_id).await?;
    let json =
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::x402::X402Client::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_params().await?;
    let json =
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_pending_upto(
    args: PendingUptoArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::x402::X402Client::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );

    let mut req = morpheum_sdk_native::x402::QueryPendingUptoRequest::new(&args.agent_id);
    if let Some(ref seller) = args.seller_address {
        req = req.seller_address(seller);
    }
    if let Some(ref id) = args.pre_auth_id {
        req = req.pre_auth_id(id);
    }

    let result = client.query_pending_upto(req).await?;
    let json =
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}
