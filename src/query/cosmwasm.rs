use clap::{Args, Subcommand};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for CosmWasm contracts on Morpheum's embedded VM.
#[derive(Subcommand)]
pub enum CosmwasmQueryCommands {
    /// Query a contract's smart endpoint with a JSON message
    Smart(SmartArgs),

    /// Query raw contract storage by hex key
    Raw(RawArgs),

    /// Query contract metadata (code ID, admin, label)
    ContractInfo(ContractInfoArgs),
}

#[derive(Args)]
pub struct SmartArgs {
    /// Contract address (morm1...)
    #[arg(long)]
    pub contract: String,

    /// JSON-encoded query message
    #[arg(long)]
    pub query: String,
}

#[derive(Args)]
pub struct RawArgs {
    /// Contract address (morm1...)
    #[arg(long)]
    pub contract: String,

    /// Hex-encoded storage key
    #[arg(long)]
    pub key: String,
}

#[derive(Args)]
pub struct ContractInfoArgs {
    /// Contract address (morm1...)
    #[arg(long)]
    pub contract: String,
}

pub async fn execute(cmd: CosmwasmQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        CosmwasmQueryCommands::Smart(args) => smart(args, &dispatcher).await,
        CosmwasmQueryCommands::Raw(args) => raw(args, &dispatcher).await,
        CosmwasmQueryCommands::ContractInfo(args) => contract_info(args, &dispatcher).await,
    }
}

async fn smart(args: SmartArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    serde_json::from_str::<serde_json::Value>(&args.query)
        .map_err(|e| CliError::invalid_input(format!("--query is not valid JSON: {e}")))?;

    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::cosmwasm::CosmWasmClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );

    let response = client
        .query_smart(&morpheum_sdk_native::cosmwasm::QuerySmartRequest {
            contract: args.contract,
            query_data: args.query.into_bytes(),
        })
        .await?;

    match serde_json::from_slice::<serde_json::Value>(&response) {
        Ok(json) => {
            let pretty = serde_json::to_string_pretty(&json)
                .unwrap_or_else(|_| format!("{json:?}"));
            println!("{pretty}");
        }
        Err(_) => {
            println!("0x{}", hex::encode(&response));
        }
    }

    Ok(())
}

async fn raw(args: RawArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let key_bytes = hex::decode(args.key.strip_prefix("0x").unwrap_or(&args.key))
        .map_err(|e| CliError::invalid_input(format!("--key is not valid hex: {e}")))?;

    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::cosmwasm::CosmWasmClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );

    let response = client
        .query_raw(&morpheum_sdk_native::cosmwasm::QueryRawRequest {
            contract: args.contract,
            key: key_bytes,
        })
        .await?;

    if response.is_empty() {
        println!("(empty)");
    } else {
        match serde_json::from_slice::<serde_json::Value>(&response) {
            Ok(json) => {
                let pretty = serde_json::to_string_pretty(&json)
                    .unwrap_or_else(|_| format!("{json:?}"));
                println!("{pretty}");
            }
            Err(_) => {
                println!("0x{}", hex::encode(&response));
            }
        }
    }

    Ok(())
}

async fn contract_info(args: ContractInfoArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::cosmwasm::CosmWasmClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );

    let info = client.query_contract_info(&args.contract).await?;

    let admin_display = match &info.admin {
        None => "(none)".to_string(),
        Some(a) if a.is_empty() => "(none)".to_string(),
        Some(a) => a.clone(),
    };
    println!(
        "Contract: {}\n  Code ID: {}\n  Admin:   {}\n  Label:   {}",
        info.address, info.code_id, admin_display, info.label,
    );

    Ok(())
}
