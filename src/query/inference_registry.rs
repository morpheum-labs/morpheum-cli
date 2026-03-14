use clap::{Args, Subcommand};

use morpheum_proto::inference_registry::v1::query_client::QueryClient;
use morpheum_proto::inference_registry::v1::{
    QuantFormat, QueryModelRequest, QueryModelsByQuantRequest,
    QueryActiveModelsRequest, QueryParamsRequest,
};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the `inference_registry` module.
///
/// Read-only access to on-chain model commitments, quantization format views,
/// active model listings, and module parameters.
#[derive(Subcommand)]
pub enum InferenceRegistryQueryCommands {
    /// Get a specific model commitment by model ID
    Model(ModelArgs),

    /// List models filtered by quantization format
    ModelsByQuant(ModelsByQuantArgs),

    /// List all currently active models
    ActiveModels,

    /// Get the current inference registry module parameters
    Params,
}

#[derive(Args)]
pub struct ModelArgs {
    /// Model ID (hex-encoded)
    #[arg(required = true)]
    pub model_id: String,
}

#[derive(Args)]
pub struct ModelsByQuantArgs {
    /// Quantization format (fp16, q4km, q5km, q80)
    #[arg(required = true, value_parser = parse_quant_format)]
    pub quant_format: QuantFormat,
}

pub async fn execute(
    cmd: InferenceRegistryQueryCommands,
    dispatcher: Dispatcher,
) -> Result<(), CliError> {
    match cmd {
        InferenceRegistryQueryCommands::Model(args) => {
            query_model(args, &dispatcher).await
        }
        InferenceRegistryQueryCommands::ModelsByQuant(args) => {
            query_models_by_quant(args, &dispatcher).await
        }
        InferenceRegistryQueryCommands::ActiveModels => {
            query_active_models(&dispatcher).await
        }
        InferenceRegistryQueryCommands::Params => {
            query_params(&dispatcher).await
        }
    }
}

async fn query_model(args: ModelArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let model_id = args.model_id.clone();
    let response = client
        .query_model(tonic::Request::new(QueryModelRequest {
            model_id: args.model_id,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryModel failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    if response.model.is_some() {
        println!("{json}");
    } else {
        println!("No model found with ID {model_id}");
    }
    Ok(())
}

async fn query_models_by_quant(
    args: ModelsByQuantArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_models_by_quant(tonic::Request::new(QueryModelsByQuantRequest {
            quant_format: args.quant_format.into(),
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryModelsByQuant failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_active_models(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_active_models(tonic::Request::new(QueryActiveModelsRequest {}))
        .await
        .map_err(|e| CliError::Transport(format!("QueryActiveModels failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_params(tonic::Request::new(QueryParamsRequest {}))
        .await
        .map_err(|e| CliError::Transport(format!("QueryParams failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    if response.params.is_some() {
        println!("{json}");
    } else {
        println!("No inference registry parameters configured");
    }
    Ok(())
}

fn parse_quant_format(s: &str) -> Result<QuantFormat, String> {
    match s.to_lowercase().as_str() {
        "fp16" | "f16" => Ok(QuantFormat::Fp16),
        "q4km" | "q4_k_m" => Ok(QuantFormat::Q4KM),
        "q5km" | "q5_k_m" => Ok(QuantFormat::Q5KM),
        "q80" | "q8_0" => Ok(QuantFormat::Q80),
        other => Err(format!(
            "unknown quant format '{other}'; expected: fp16, q4km, q5km, q80"
        )),
    }
}
