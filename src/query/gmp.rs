use clap::Subcommand;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the GMP module.
#[derive(Subcommand)]
pub enum GmpQueryCommands {
    /// Query GMP module parameters
    Params,

    /// Query registered GMP protocols
    Protocols,
}

pub async fn execute(cmd: GmpQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        GmpQueryCommands::Params => params(&dispatcher).await,
        GmpQueryCommands::Protocols => protocols(&dispatcher).await,
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
