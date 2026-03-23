use clap::{Args, Subcommand};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the GMP module.
#[derive(Subcommand)]
pub enum GmpQueryCommands {
    /// Query GMP module parameters
    Params,

    /// Query registered GMP protocols
    Protocols,

    /// Check Hyperlane message delivery status
    Delivery(DeliveryArgs),
}

#[derive(Args)]
pub struct DeliveryArgs {
    /// Hyperlane message ID (hex, with or without 0x prefix)
    #[arg(long)]
    pub message_id: String,
}

pub async fn execute(cmd: GmpQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        GmpQueryCommands::Params => params(&dispatcher).await,
        GmpQueryCommands::Protocols => protocols(&dispatcher).await,
        GmpQueryCommands::Delivery(args) => delivery(args, &dispatcher).await,
    }
}

async fn params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::gmp::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_params(tonic::Request::new(
            morpheum_proto::gmp::v1::QueryParamsRequest {},
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryParams failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn protocols(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::gmp::v1::query_client::QueryClient::new(channel);
    let response = client
        .query_protocols(tonic::Request::new(
            morpheum_proto::gmp::v1::QueryProtocolsRequest {},
        ))
        .await
        .map_err(|e| CliError::Transport(format!("QueryProtocols failed: {e}")))?
        .into_inner();
    let json =
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn delivery(args: DeliveryArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let message_id = args
        .message_id
        .strip_prefix("0x")
        .unwrap_or(&args.message_id);

    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::gmp::v1::query_client::QueryClient::new(channel);

    let response = client
        .query_hyperlane_delivery(tonic::Request::new(
            morpheum_proto::gmp::v1::QueryHyperlaneDeliveryRequest {
                message_id: message_id.to_string(),
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("Hyperlane delivery query failed: {e}")))?
        .into_inner();

    let status = if response.delivered {
        "Delivered"
    } else {
        "Pending (not yet delivered)"
    };

    dispatcher.output.success(format!(
        "Hyperlane delivery status\n\
         MessageID: 0x{message_id}\n\
         Status: {status}",
    ));

    Ok(())
}
