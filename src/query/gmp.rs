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
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::gmp::GmpClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_params().await?;
    let json =
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn protocols(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::gmp::GmpClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_protocols().await?;
    let json =
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn delivery(args: DeliveryArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let message_id = args
        .message_id
        .strip_prefix("0x")
        .unwrap_or(&args.message_id)
        .to_string();

    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::gmp::GmpClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_hyperlane_delivery(&message_id).await?;

    let status = if result.delivered {
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
