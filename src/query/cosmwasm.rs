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

// ── Wire types for CosmWasm gRPC queries ────────────────────────────────────

#[derive(Clone, prost::Message)]
struct QuerySmartContractStateRequest {
    #[prost(string, tag = "1")]
    address: String,
    #[prost(bytes = "vec", tag = "2")]
    query_data: Vec<u8>,
}

#[derive(Clone, prost::Message)]
struct QuerySmartContractStateResponse {
    #[prost(bytes = "vec", tag = "1")]
    data: Vec<u8>,
}

#[derive(Clone, prost::Message)]
struct QueryRawContractStateRequest {
    #[prost(string, tag = "1")]
    address: String,
    #[prost(bytes = "vec", tag = "2")]
    query_data: Vec<u8>,
}

#[derive(Clone, prost::Message)]
struct QueryRawContractStateResponse {
    #[prost(bytes = "vec", tag = "1")]
    data: Vec<u8>,
}

#[derive(Clone, prost::Message)]
struct QueryContractInfoRequest {
    #[prost(string, tag = "1")]
    address: String,
}

#[derive(Clone, prost::Message)]
struct QueryContractInfoResponse {
    #[prost(string, tag = "1")]
    address: String,
    #[prost(message, optional, tag = "2")]
    contract_info: Option<ContractInfoProto>,
}

#[derive(Clone, prost::Message)]
struct ContractInfoProto {
    #[prost(uint64, tag = "1")]
    code_id: u64,
    #[prost(string, tag = "2")]
    creator: String,
    #[prost(string, tag = "3")]
    admin: String,
    #[prost(string, tag = "4")]
    label: String,
}

// ── gRPC helpers ────────────────────────────────────────────────────────────

async fn cosmwasm_unary<Req, Resp>(
    channel: tonic::transport::Channel,
    path: &'static str,
    request: Req,
) -> Result<Resp, CliError>
where
    Req: prost::Message + Clone + 'static,
    Resp: prost::Message + Default + Clone + 'static,
{
    let mut grpc = tonic::client::Grpc::new(channel);
    grpc.ready()
        .await
        .map_err(|e| CliError::Transport(format!("service not ready: {e}")))?;

    let path = path
        .parse::<http::uri::PathAndQuery>()
        .map_err(|e| CliError::internal(format!("invalid gRPC path: {e}")))?;

    let codec: tonic_prost::ProstCodec<Req, Resp> = tonic_prost::ProstCodec::default();

    grpc.unary(tonic::Request::new(request), path, codec)
        .await
        .map(tonic::Response::into_inner)
        .map_err(|e| CliError::Transport(format!("CosmWasm query failed: {e}")))
}

// ── Query handlers ──────────────────────────────────────────────────────────

async fn smart(args: SmartArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    serde_json::from_str::<serde_json::Value>(&args.query)
        .map_err(|e| CliError::invalid_input(format!("--query is not valid JSON: {e}")))?;

    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;

    let response: QuerySmartContractStateResponse = cosmwasm_unary(
        channel,
        "/cosmwasm.wasm.v1.Query/SmartContractState",
        QuerySmartContractStateRequest {
            address: args.contract.clone(),
            query_data: args.query.into_bytes(),
        },
    )
    .await?;

    match serde_json::from_slice::<serde_json::Value>(&response.data) {
        Ok(json) => {
            let pretty = serde_json::to_string_pretty(&json)
                .unwrap_or_else(|_| format!("{json:?}"));
            println!("{pretty}");
        }
        Err(_) => {
            println!("0x{}", hex::encode(&response.data));
        }
    }

    Ok(())
}

async fn raw(args: RawArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let key_bytes = hex::decode(args.key.strip_prefix("0x").unwrap_or(&args.key))
        .map_err(|e| CliError::invalid_input(format!("--key is not valid hex: {e}")))?;

    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;

    let response: QueryRawContractStateResponse = cosmwasm_unary(
        channel,
        "/cosmwasm.wasm.v1.Query/RawContractState",
        QueryRawContractStateRequest {
            address: args.contract.clone(),
            query_data: key_bytes,
        },
    )
    .await?;

    if response.data.is_empty() {
        println!("(empty)");
    } else {
        match serde_json::from_slice::<serde_json::Value>(&response.data) {
            Ok(json) => {
                let pretty = serde_json::to_string_pretty(&json)
                    .unwrap_or_else(|_| format!("{json:?}"));
                println!("{pretty}");
            }
            Err(_) => {
                println!("0x{}", hex::encode(&response.data));
            }
        }
    }

    Ok(())
}

async fn contract_info(args: ContractInfoArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;

    let response: QueryContractInfoResponse = cosmwasm_unary(
        channel,
        "/cosmwasm.wasm.v1.Query/ContractInfo",
        QueryContractInfoRequest {
            address: args.contract.clone(),
        },
    )
    .await?;

    match response.contract_info {
        Some(info) => {
            let admin_display = if info.admin.is_empty() {
                "(none)".to_string()
            } else {
                info.admin
            };
            println!(
                "Contract: {}\n  Code ID: {}\n  Creator: {}\n  Admin:   {}\n  Label:   {}",
                response.address, info.code_id, info.creator, admin_display, info.label,
            );
        }
        None => {
            println!("No contract info returned for {}", args.contract);
        }
    }

    Ok(())
}
